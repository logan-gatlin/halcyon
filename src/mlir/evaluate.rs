use crate::{compile::VirtualMachine, compiler_print};

use super::*;

#[derive(Clone)]
struct FunctionReturn {
  previous_values: HashMap<Mangle, ConstValue>,
  return_block: Mangle,
  return_ip: usize,
}

impl MlIrModule {
  pub fn get_const(&self, mangle: &Mangle) -> Option<ConstValue> {
    if let Some(v) = Builtin::from_mangle(mangle) {
      return Some(v.value().clone());
    }
    let block = self.blocks.get(mangle)?;
    match &block.kind {
      BlockKind::Constant(evaluation) => evaluation.clone(),
      BlockKind::Function { value, .. } => value.clone(),
      BlockKind::TypeAssert(_) => None,
      BlockKind::Parameter(_) => None,
      BlockKind::GlobalScope(value) => value.clone(),
    }
  }

  pub fn evaluate(&mut self, mangle: &Mangle) {
    match &self.blocks.get(mangle).unwrap().kind {
      BlockKind::Constant(_) => {
        let new_value = Some(self.evaluate_constant(mangle).unwrap());
        self.blocks.get_mut(mangle).unwrap().kind = BlockKind::Constant(new_value);
      }
      BlockKind::Function {
        parameters,
        return_type,
        ..
      } => {
        let new_value = Some(self.evaluate_function(mangle).unwrap());
        self.blocks.get_mut(mangle).unwrap().kind = BlockKind::Function {
          parameters: parameters.clone(),
          return_type: return_type.clone(),
          value: new_value,
        };
      }
      BlockKind::TypeAssert(_) => {}
      BlockKind::Parameter(_) => {}
      BlockKind::GlobalScope(_) => {
        let new_value = Some(self.evaluate_constant(mangle).unwrap());
        self.blocks.get_mut(mangle).unwrap().kind = BlockKind::GlobalScope(new_value);
      }
    }
  }

  pub fn evaluate_function(&self, mangle: &Mangle) -> Result<ConstValue> {
    let Some(Block {
      kind: BlockKind::Function {
        parameters,
        return_type,
        value,
      },
      ..
    }) = self.blocks.get(mangle)
    else {
      panic!()
    };
    if let Some(value) = value {
      return Ok(value.clone());
    }
    let parameters = parameters
      .iter()
      .map(|p| self.evaluate_constant(p))
      .try_collect::<Vec<_>>()?
      .into_iter()
      .map(|v| {
        if let ConstValue::Type(t) = v {
          Some(t)
        } else {
          None
        }
      })
      .try_collect::<Vec<_>>()
      .unwrap();
    let ConstValue::Type(returns) = return_type
      .clone()
      .map(|r| self.evaluate_constant(&r))
      .unwrap_or(Ok(ConstValue::Type(Primitive::nothing.promote())))?
    else {
      panic!();
    };
    Ok(ConstValue::Function {
      name: mangle.clone(),
      parameters,
      returns,
    })
  }

  pub fn evaluate_constant(&self, mangle: &Mangle) -> Result<ConstValue> {
    if let Some(value) = self.get_const(mangle) {
      return Ok(value);
    }
    let mut block_name = mangle.clone();
    let mut block = &self.blocks.get(mangle).unwrap().body;
    let optable = OpTable::new();
    let mut return_stack: Vec<FunctionReturn> = vec![];
    let mut ip = 0;
    let stack = &mut vec![];
    let pop = |stack: &mut Vec<ConstValue>| stack.pop().unwrap_or(ConstValue::Nothing);
    let mut name_map = HashMap::new();
    loop {
      if ip == block.len() {
        if let Some(state) = return_stack.pop() {
          ip = state.return_ip;
          block_name = state.return_block;
          block = &self.blocks.get(&block_name).unwrap().body;
          name_map = state.previous_values;
        }
        // Catch case when last function call was at the end of the previous function
        if ip == block.len() {
          return Ok(pop(stack));
        }
      }
      let instr = block.get(ip).unwrap();
      use MlIrKind::*;
      match instr.kind.clone() {
        Const(const_value) => {
          stack.push(const_value);
        }
        Set(mangle) => {
          let value = pop(stack);
          name_map.insert(mangle, value);
        }
        Get(mangle) => {
          let value = name_map
            .get(&mangle)
            .cloned()
            .or(self.get_const(&mangle))
            .unwrap();
          stack.push(value.clone());
        }
        BinaryOp { kind } => {
          let first = pop(stack);
          let second = pop(stack);
          let opkind = optable
            .try_binary(kind, &first.type_of(), &second.type_of())
            .unwrap();
          let result =
            VirtualMachine::run(vec![second, first], opkind.asm, opkind.produces).unwrap();
          stack.push(result);
        }
        UnaryOp { kind } => {
          let value = pop(stack);
          let opkind = optable.try_unary(kind, &value.type_of()).unwrap();
          let result = VirtualMachine::run(vec![value], opkind.asm, opkind.produces).unwrap();
          stack.push(result);
        }
        Field(field) => {
          let ConstValue::StructLiteral {
            member_names,
            member_values,
          } = pop(stack)
          else {
            panic!()
          };
          let position = member_names.iter().position(|s| s == &field).unwrap();
          stack.push(member_values[position].clone());
        }
        StructLiteral { param_names } => {
          let mut param_values = vec![];
          for _ in 0..param_names.len() {
            param_values.push(pop(stack));
          }
          param_values.reverse();
          stack.push(ConstValue::StructLiteral {
            member_names: param_names,
            member_values: param_values,
          });
        }
        StructDef { fields } => {
          let mut types = vec![];
          for _ in 0..fields.len() {
            let ConstValue::Type(t) = pop(stack) else {
              panic!()
            };
            types.push(t);
          }
          types.reverse();
          stack.push(ConstValue::Type(Type::Struct {
            member_names: fields,
            member_types: types,
          }));
        }
        TypeAssert => {
          let ConstValue::Type(t) = pop(stack) else {
            panic!()
          };
          let value = pop(stack);
          if t != value.type_of() {
            panic!();
          }
          stack.push(value);
        }
        Call { arity } => {
          let mut arguments = vec![];
          for _ in 0..arity {
            arguments.push(pop(stack));
          }
          let ConstValue::Function {
            name: func_name,
            parameters: param_types,
            returns: return_type,
          } = pop(stack)
          else {
            panic!()
          };
          if let Some(bt) = Builtin::from_mangle(&func_name) {
            self.execute_builtin(bt, &mut arguments);
            ip += 1;
            continue;
          }
          let return_state = FunctionReturn {
            previous_values: name_map.clone(),
            return_block: block_name.clone(),
            return_ip: ip + 1,
          };
          return_stack.push(return_state);
          name_map.clear();
          let BlockKind::Function {
            parameters: parameter_names,
            ..
          } = &self.blocks.get(&func_name).unwrap().kind
          else {
            panic!()
          };
          parameter_names.into_iter().for_each(|n| {
            name_map.insert(n.clone(), arguments.pop().unwrap());
          });
          block = &self.blocks.get(&func_name).unwrap().body;
          block_name = func_name;
          ip = 0;
          continue;
        }
        Drop => {
          pop(stack);
        }
        If => {
          let ConstValue::Boolean(predicate) = pop(stack) else {
            panic!()
          };
          if !predicate {
            let mut nesting = 0;
            loop {
              ip += 1;
              if ip >= block.len() {
                panic!()
              }
              match &block.get(ip).unwrap().kind {
                If => nesting += 1,
                End => {
                  if nesting == 0 {
                    break;
                  }
                  nesting -= 1;
                }
                Else if nesting == 0 => break,
                _ => {}
              }
            }
          }
        }
        Else => {
          let mut nesting = 0;
          loop {
            ip += 1;
            if ip >= block.len() {
              panic!()
            }
            match &block.get(ip).unwrap().kind {
              If => nesting += 1,
              End => {
                if nesting == 0 {
                  break;
                }
                nesting -= 1;
              }
              _ => {}
            }
          }
        }
        End => {}
        Loop => {}
        Repeat => {
          let mut nesting = 0;
          loop {
            if ip == 0 {
              panic!();
            }
            ip -= 1;
            match &block.get(ip).unwrap().kind {
              Repeat => nesting += 1,
              Loop => {
                if nesting == 0 {
                  break;
                }
                nesting -= 1;
              }
              _ => {}
            }
          }
        }
        Break => {
          let mut nesting = 0;
          loop {
            ip += 1;
            if ip >= block.len() {
              panic!()
            }
            match &block.get(ip).unwrap().kind {
              Loop => nesting += 1,
              Repeat => {
                if nesting == 0 {
                  break;
                }
                nesting -= 1;
              }
              _ => {}
            }
          }
        }
      };
      ip += 1;
    }
  }

  fn execute_builtin(&self, builtin: Builtin, stack: &mut Vec<ConstValue>) -> Result<()> {
    match builtin {
      Builtin::PrintString => {
        let ConstValue::String {
          virtual_address, ..
        } = stack.pop().unwrap()
        else {
          panic!();
        };
        let s = String::from_utf8_lossy(&self.virtual_memory[virtual_address]);
        compiler_print(s);
      }
      Builtin::PrintReal => {
        let ConstValue::Real(r) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(r.to_string());
      }
      Builtin::PrintGlyph => {
        let ConstValue::Glyph(g) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(g);
      }
      Builtin::PrintInteger => {
        let ConstValue::Integer(i) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(i.to_string());
      }
      Builtin::PrintBoolean => {
        let ConstValue::Boolean(b) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(b.to_string());
      }
      Builtin::PrintType => {
        let ConstValue::Type(t) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(format!("{t}"));
      }
      _ => panic!(),
    };
    Ok(())
  }
}
