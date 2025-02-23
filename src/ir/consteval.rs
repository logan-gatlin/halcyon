use crate::assembly::vm::VirtualMachine;
use crate::{assembly::operators::OpTable, err::*};
use crate::{error, ir::types::Primitive};

use super::solver::{Solver, StackValue};
use super::{Block, ConstValue, IrKind, IrPtr, types::Type};

impl Solver {
  fn pop(&mut self) -> ConstValue {
    match self.value_stack.pop() {
      Some(StackValue::Value(v)) => v,
      Some(StackValue::OldValue(v)) => {
        panic!()
      },
      Some(StackValue::Guard) => {
        self.value_stack.push(StackValue::Guard);
        ConstValue::Nothing
      },
      None => ConstValue::Nothing,
    }
  }

  fn push(&mut self, val: ConstValue) {
    self.value_stack.push(StackValue::Value(val));
  }

  fn save_state(&mut self) {
    self.value_stack.extend(
      self
        .rt_value_map
        .clone()
        .into_iter()
        .map(|v| StackValue::OldValue(v)),
    );
    self.rt_value_map.clear();
  }

  fn retrieve_state(&mut self) {
    self.rt_value_map.clear();
    while let Some(StackValue::OldValue((mangle, val))) =
      self.value_stack.last()
    {
      self.rt_value_map.insert(mangle.clone(), val.clone());
      self.value_stack.pop();
    }
  }

  fn start_scope_v(&mut self) {
    self.value_stack.push(StackValue::Guard);
  }

  fn end_scope_v(&mut self) {
    let mut last = ConstValue::Nothing;
    while let Some(StackValue::Value(v)) = self.value_stack.last() {
      last = v.clone();
      self.value_stack.pop();
    }
    if let Some(StackValue::Guard) = self.value_stack.last() {
      self.value_stack.pop();
    } else {
      panic!("Unguard without corresponding guard");
    }
    self.push(last);
  }

  pub fn conseval_module(&mut self) -> Result<()> {
    let mut deps = self
      .dependency_graph
      .clone()
      .into_iter()
      .collect::<Vec<_>>();
    deps
      .sort_unstable_by(|(_, deps1), (_, deps2)| deps1.len().cmp(&deps2.len()));
    println!("{deps:#?}");
    for (mangle, deps) in deps {
      self.rt_value_map.clear();
      self.value_stack.clear();
      if deps.contains(&mangle) {
        return error!("Encountered circular dependency");
      }
      // Type function
      if let Some(func) = self.module.functions.get(&mangle).cloned() {
        let mut param_types = vec![];
        for (id, m) in func.parameter_mangles.into_iter().enumerate() {
          let block = self.module.parameters.get(&m).unwrap();
          self.evaluate_block(*block, false)?;
          let ConstValue::Type(t) = self.pop().clone() else {
            return error!(
              "The type assertion for parameter {} is a term, not a type",
              id + 1
            );
          };
          param_types.push(t);
        }
        let return_type = if let Some(r) = func.returns_mangle {
          let block = self.module.parameters.get(&r).unwrap();
          self.evaluate_block(*block, false)?;
          let ConstValue::Type(t) = self.pop().clone() else {
            return error!(
              "The type assertion for this function's return type is a term, \
               not a type",
            );
          };
          t
        } else {
          Primitive::nothing.promote()
        };
        self.type_map.insert(
          func.mangle,
          Type::Function {
            param_types,
            return_type: return_type.into(),
          },
        );
      }
      // Resolve constants
      else if let Some(const_block) =
        self.module.constants.get(&mangle).cloned()
      {
        self.evaluate_block(const_block, false)?;
        let value = self.pop();
        self
          .type_map
          .insert(mangle.clone(), self.type_of_const(&value));
      } else {
        panic!("Unhandled dependent {mangle}");
      }
    }
    println!(
      "{:#?}",
      self
        .const_value_map
        .iter()
        .filter(|n| !n.0.starts_with("_"))
        .collect::<Vec<_>>()
    );
    Ok(())
  }

  pub fn type_of_const(&self, val: &ConstValue) -> Type {
    use Primitive as p;
    match val {
      ConstValue::Nothing => p::nothing.promote(),
      ConstValue::Integer(_) => p::integer.promote(),
      ConstValue::Real(_) => p::real.promote(),
      ConstValue::Boolean(_) => p::boolean.promote(),
      ConstValue::String { address, length } => p::string.promote(),
      ConstValue::Glyph(_) => p::glyph.promote(),
      ConstValue::Function(mangle) => {
        self.type_map.get(mangle).unwrap().clone()
      },
      ConstValue::StructLiteral {
        member_names,
        member_values,
      } => {
        let member_types = member_values
          .into_iter()
          .map(|v| self.type_of_const(v))
          .collect::<Vec<_>>();
        Type::Struct {
          member_names: member_names.clone(),
          member_types,
        }
      },
      ConstValue::Type(_) => Type::Type,
    }
  }

  fn evaluate_block(
    &mut self,
    mut block: IrPtr,
    typecheck_only: bool,
  ) -> Result<()> {
    self.value_stack.clear();
    let optable = OpTable::new();
    let mut cflow_stack = vec![];
    let mut ip = 0;
    loop {
      /*
      println!("");
      println!("block: {block}, ip: {ip}");
      println!("{:?}", self.stack);
      */
      block = match self.module.blocks[block].clone() {
        Block::Terminal => {
          if let Some((new_block, new_ip)) = cflow_stack.pop() {
            let return_value = self.pop();
            self.retrieve_state();
            block = new_block;
            ip = new_ip;
            self.push(return_value);
            continue;
          } else {
            break;
          }
        },
        Block::Unreachable => {
          return error!("Encountered unreachable during constant evaluation");
        },
        Block::Branch {
          span,
          when_true,
          when_false,
        } => {
          //println!("BRANCH");
          ip = 0;
          let val = self.pop();
          match val {
            ConstValue::Boolean(true) => Ok(when_true),
            ConstValue::Boolean(false) => Ok(when_false),
            v => error!(
              "The predicate of an 'if' expression must be a boolean, found \
               '{v}'"
            )
            .span(&span),
          }
        },
        Block::Basic { body, next, typed } => {
          if ip == body.len() {
            ip = 0;
            Ok(next)
          } else if ip > body.len() {
            panic!(
              "Instruction pointer overshot body: index {ip} with length {} \
               in block {block}",
              body.len()
            )
          } else {
            let instr = &body[ip];
            if !instr.typecheck_only && typecheck_only {
              ip += 1;
              continue;
            }
            //println!("{instr:?}");
            match &instr.kind {
              IrKind::Const(const_value) => self.push(const_value.clone()),
              IrKind::Set(mangle) => {
                let value = self.pop();
                let set_type = self.type_of_const(&value);
                let old_type =
                  self.type_map.insert(mangle.clone(), set_type.clone());
                if let Some(old_type) = old_type
                  && set_type != old_type
                {
                  return error!(
                    "This binding expects type '{old_type}', but recieved \
                     '{set_type}'"
                  )
                  .span(&instr.span);
                }
                if self.module.constants.contains_key(mangle) {
                  if self.const_value_map.contains_key(mangle) {
                    panic!(
                      "Duplicate initializations of {mangle} during const-eval"
                    )
                  }
                  self.const_value_map.insert(mangle.clone(), value);
                } else {
                  self.rt_value_map.insert(mangle.clone(), value);
                };
              },
              IrKind::Get(mangle) => {
                let map = if self.const_value_map.contains_key(mangle) {
                  &mut self.const_value_map
                } else {
                  &mut self.rt_value_map
                };
                let Some(value) = map.get(mangle).cloned() else {
                  panic!("Value map is missing {mangle}");
                };
                self.push(value.clone());
              },
              IrKind::BinaryOp { kind } => {
                let right = self.pop();
                let right_t = self.type_of_const(&right);
                let left = self.pop();
                let left_t = self.type_of_const(&left);
                let opdef = optable
                  .try_binary(*kind, &left_t, &right_t)
                  .span(&instr.span)?;
                let result = VirtualMachine::run(
                  vec![left, right],
                  opdef.asm,
                  opdef.produces,
                )
                .span(&instr.span)?;
                self.push(result);
              },
              IrKind::UnaryOp { kind } => {
                let on = self.pop();
                let on_t = self.type_of_const(&on);
                let opdef =
                  optable.try_unary(*kind, &on_t).span(&instr.span)?;
                let result =
                  VirtualMachine::run(vec![on], opdef.asm, opdef.produces)
                    .span(&instr.span)?;
                self.push(result);
              },
              IrKind::Field(field_name) => {
                let ConstValue::StructLiteral {
                  member_names,
                  member_values,
                } = self.pop()
                else {
                  return error!("Only struct literals can have fields")
                    .span(&instr.span);
                };
                let pos = member_names
                  .iter()
                  .position(|n| n == field_name)
                  .reason("Struct does not contain field '{field}'")
                  .span(&instr.span)?;
                self.push(member_values[pos].clone());
              },
              IrKind::StructLiteral { param_names } => {
                let member_values: Vec<_> =
                  (0..param_names.len()).map(|_| self.pop()).rev().collect();
                self.push(ConstValue::StructLiteral {
                  member_names: param_names.clone(),
                  member_values,
                })
              },
              IrKind::StructDef { param_names } => {
                let mut member_types: Vec<_> = (0..param_names.len())
                  .rev()
                  .map(|i| {
                    if let ConstValue::Type(t) = self.pop() {
                      Ok(t)
                    } else {
                      error!(
                        "Structure definition must contain only type names, \
                         found that field {} contains a term",
                        param_names[i]
                      )
                      .span(&instr.span)
                    }
                  })
                  .try_collect()?;
                member_types.reverse();
                self.push(ConstValue::Type(Type::Struct {
                  member_names: param_names.clone(),
                  member_types,
                }))
              },
              IrKind::TypeAssert(mangle) => {
                let assert_val = self.pop();
                let ConstValue::Type(assert_t) = assert_val else {
                  return error!(
                    "Type assertion expects a type, but recieved a term"
                  )
                  .span(&instr.span);
                };
                let actual_val = self.pop();
                let actual_t = self.type_of_const(&actual_val);
                if actual_t != assert_t {
                  return error!(
                    "The asserted type is '{assert_t}', but expression has \
                     type '{actual_t}'"
                  )
                  .span(&instr.span);
                }
                if let Some(mangle) = mangle {
                  self.type_map.insert(mangle.clone(), assert_t);
                }
                self.push(actual_val);
              },
              IrKind::Call { arity } => {
                let func = self.pop();
                let func_type = self.type_of_const(&func);
                let Type::Function {
                  param_types,
                  return_type,
                } = func_type
                else {
                  return error!("Cannot call type '{func_type}'")
                    .span(&instr.span);
                };
                if param_types.len() != *arity {
                  return error!(
                    "Function expects {} arguments, but recieved {arity}",
                    param_types.len()
                  )
                  .span(&instr.span);
                }
                let values: Vec<_> = param_types
                  .into_iter()
                  .enumerate()
                  .rev()
                  .map(|(id, expected)| {
                    let param = self.pop();
                    let param_t = self.type_of_const(&param);
                    if param_t != expected {
                      error!(
                        "Function expected type '{expected}' for argument {} \
                         but recieved type '{param_t}'",
                        id + 1
                      )
                      .span(&instr.span)
                    } else {
                      Ok(param)
                    }
                  })
                  .try_collect()?;
                let ConstValue::Function(mangle) = func else {
                  unreachable!()
                };
                self.save_state();
                cflow_stack.push((block, ip + 1));
                let fun = self.module.functions.get(&mangle).unwrap();
                block = fun.block;
                ip = 0;
                values
                  .into_iter()
                  .rev()
                  .zip(fun.parameter_mangles.iter())
                  .for_each(|(value, mangle)| {
                    self.rt_value_map.insert(mangle.clone(), value);
                  });
                continue; // Prevent IP from incrementing
              },
              IrKind::Drop => {
                self.pop();
              },
              IrKind::StartScope => self.start_scope_v(),
              IrKind::EndScope => self.end_scope_v(),
            }
            ip += 1;
            Ok(block)
          }
        },
      }?;
    }
    Ok(())
  }
}
