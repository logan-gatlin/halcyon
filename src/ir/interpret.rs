use crate::{
  assembly::{operators::OpTable, vm::VirtualMachine},
  err::*,
  error,
  ir::{Block, ConstValue, IrKind, solver::RECURSION_LIMIT, types::Type},
};

use super::{
  IrPtr,
  solver::{ReturnAddress, Solver},
};

impl Solver {
  pub fn evaluate_block(&mut self, block: IrPtr) -> Result<()> {
    self.block = block;
    self._evaluate_block()
  }

  pub fn _evaluate_block(&mut self) -> Result<()> {
    self.value_stack.clear();
    self.control_stack.clear();
    self.ip = 0;
    let optable = OpTable::new();
    loop {
      /*
      println!("");
      println!("block: {block}, ip: {ip}");
      println!("{:?}", self.stack);
      */
      if self.control_stack.len() > RECURSION_LIMIT {
        return error!(
          "Reached recursion limit ({RECURSION_LIMIT}) during constant \
           evaluation"
        );
      }
      self.block = match self.module.blocks[self.block].clone() {
        Block::Terminal => {
          if let Some(ReturnAddress {
            block: new_block,
            ip: new_ip,
            expected_type,
          }) = self.control_stack.pop()
          {
            let return_value = self.pop();
            let return_type = self.type_of_const(&return_value);
            if expected_type != return_type {
              return error!(
                "Function returned '{return_type}' when '{expected_type}' was \
                 expected"
              );
            }
            self.retrieve_state();
            self.block = new_block;
            self.ip = new_ip;
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
          self.ip = 0;
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
        Block::Basic { body, next } => {
          if self.ip == body.len() {
            self.ip = 0;
            Ok(next)
          } else if self.ip > body.len() {
            panic!(
              "Instruction pointer overshot body: index {} with length {} in \
               block {}",
              self.ip,
              self.block,
              body.len()
            )
          } else {
            let instr = &body[self.ip];
            //println!("{instr:?}");
            match &instr.kind {
              IrKind::Const(const_value) => self.push(const_value.clone()),
              IrKind::Set(mangle) => {
                let value = self.pop();
                if self.module.constants.contains_key(mangle) {
                  if self.const_value_map.contains_key(mangle) {
                    panic!(
                      "Duplicate initializations of {mangle} during const-eval"
                    )
                  }
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
                  return error!(
                    "Failed to evaluate recursive expression. Recursion \
                     during compile-time evaluation is experimental. It is \
                     possible what you are trying to do is syntactically \
                     valid, but currently it is not supported"
                  )
                  .span(&instr.span);
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
              IrKind::StructDef {
                fields: param_names,
              } => {
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
                let assert_type = self.pop();
                let ConstValue::Type(assert) = assert_type else {
                  return error!(
                    "Type assertion expects a type, but recieved the value \
                     '{assert_type}'"
                  )
                  .span(&instr.span);
                };
                let actual_val = self.pop();
                let actual_t = self.type_of_const(&actual_val);
                if actual_t != assert {
                  return error!(
                    "The asserted type is '{assert}', but expression has type \
                     '{actual_t}'"
                  )
                  .span(&instr.span);
                }
                self.push(actual_val);
                if let Some(mangle) = mangle {
                  self.type_map.insert(mangle.clone(), assert);
                }
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
                //cflow_stack.push((block, ip + 1));
                self.control_stack.push(ReturnAddress {
                  block: self.block,
                  ip: self.ip + 1,
                  expected_type: *return_type,
                });
                let fun = self.module.functions.get(&mangle).unwrap();
                self.block = fun.block;
                self.ip = 0;
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
            self.ip += 1;
            Ok(self.block)
          }
        },
      }?;
    }
    Ok(())
  }
}
