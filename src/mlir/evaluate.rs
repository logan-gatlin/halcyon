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

  pub fn evaluate(
    &mut self,
    mangle: &Mangle,
    memory: &mut Memory,
  ) -> Result<()> {
    match &self.blocks.get(mangle).unwrap().kind {
      BlockKind::Constant(_) => {
        let new_value = Some(self.evaluate_constant(mangle, memory)?);
        self.blocks.get_mut(mangle).unwrap().kind =
          BlockKind::Constant(new_value);
      },
      BlockKind::Function {
        parameters,
        parameter_spans,
        return_type,
        return_span,
        ..
      } => {
        let new_value = Some(self.evaluate_function(mangle, memory)?);
        self.blocks.get_mut(mangle).unwrap().kind = BlockKind::Function {
          parameters: parameters.clone(),
          parameter_spans: parameter_spans.clone(),
          return_type: return_type.clone(),
          return_span: return_span.clone(),
          value: new_value,
        };
      },
      BlockKind::TypeAssert(_) => {},
      BlockKind::Parameter(_) => {},
      BlockKind::GlobalScope(_) => {
        let new_value = Some(self.evaluate_constant(mangle, memory)?);
        self.blocks.get_mut(mangle).unwrap().kind =
          BlockKind::GlobalScope(new_value);
      },
    }
    Ok(())
  }

  pub fn evaluate_function(
    &self,
    mangle: &Mangle,
    memory: &mut Memory,
  ) -> Result<ConstValue> {
    let Some(Block {
      kind:
        BlockKind::Function {
          parameters,
          parameter_spans,
          return_type,
          return_span,
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
      .map(|p| self.evaluate_constant(p, memory))
      .try_collect::<Vec<_>>()?
      .into_iter()
      .zip(parameter_spans.into_iter())
      .map(|(v, s)| {
        if let ConstValue::Type(t) = v {
          Ok(t)
        } else {
          Err(lint(
            TypeLint::TypeMismatch,
            *s,
            &[format!("{}", Type::Type), format!("{}", v.type_of())],
          ))
        }
      })
      .try_collect::<Vec<_>>()?;
    let return_type = return_type
      .clone()
      .map(|r| self.evaluate_constant(&r, memory))
      .unwrap_or(Ok(ConstValue::Type(Primitive::nothing.promote())))?;
    let ConstValue::Type(returns) = return_type else {
      let e = Err(lint_nospan(TypeLint::TypeMismatch))
        .context(format!("{}", Type::Type))
        .context(format!("{}", return_type.type_of()));
      if let Some(span) = return_span {
        return e.span(*span);
      } else {
        return e;
      }
    };
    Ok(ConstValue::Function {
      name: mangle.clone(),
      parameters,
      returns,
    })
  }

  pub fn evaluate_constant(
    &self,
    mangle: &Mangle,
    memory: &mut Memory,
  ) -> Result<ConstValue> {
    if let Some(value) = self.get_const(mangle) {
      return Ok(value);
    }
    let mut block_name = mangle.clone();
    let mut block = &self.blocks.get(mangle).unwrap().body;
    let optable = OpTable::new();
    let mut return_stack: Vec<FunctionReturn> = vec![];
    let mut ip = 0;
    let stack = &mut vec![];
    let pop =
      |stack: &mut Vec<ConstValue>| stack.pop().unwrap_or(ConstValue::Nothing);
    let mut name_map = HashMap::new();
    loop {
      if ip == block.len() {
        if let Some(state) = return_stack.pop() {
          ip = state.return_ip;
          block_name = state.return_block;
          block = &self.blocks.get(&block_name).unwrap().body;
          name_map = state.previous_values;
        }
        // Catch case when last function call was at the end of the
        // previous function
        if ip == block.len() {
          return Ok(pop(stack));
        }
      }
      let instr = block.get(ip).expect("Invalid jump during compeval");
      let span = instr.span;
      use MlIrKind::*;
      match instr.kind.clone() {
        Const(const_value) => {
          stack.push(const_value);
        },
        Set(mangle) => {
          let value = pop(stack);
          name_map.insert(mangle, value);
        },
        Get(mangle) => {
          let value = name_map
            .get(&mangle)
            .cloned()
            .or(self.get_const(&mangle))
            .ok_or(lint(EvalLint::Circular, span, &[]))?;
          stack.push(value.clone());
        },
        BinaryOp { kind } => {
          let first = pop(stack);
          let second = pop(stack);
          let opkind = optable
            .try_binary(kind, &first.type_of(), &second.type_of())
            .span(span)?;
          let result = VirtualMachine::run(
            vec![second, first],
            opkind.asm,
            opkind.produces,
          )
          .unwrap();
          stack.push(result);
        },
        UnaryOp { kind } => {
          let value = pop(stack);
          let opkind = optable.try_unary(kind, &value.type_of()).span(span)?;
          let result =
            VirtualMachine::run(vec![value], opkind.asm, opkind.produces)
              .unwrap();
          stack.push(result);
        },
        Field(field) => {
          let top = pop(stack);
          let ConstValue::StructLiteral {
            member_names,
            member_values,
          } = top
          else {
            return Err(lint(
              TypeLint::NoFieldOnType,
              span,
              &[format!("{}", top.type_of())],
            ));
          };
          let position = member_names.iter().position(|s| s == &field).ok_or(
            lint(TypeLint::NoFieldOnType, span, &[format!("{field}")]),
          )?;
          stack.push(member_values[position].clone());
        },
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
        },
        StructDef { fields } => {
          let mut types = vec![];
          for _ in 0..fields.len() {
            let top = pop(stack);
            let ConstValue::Type(t) = top else {
              return Err(lint(
                TypeLint::TypeMismatch,
                span,
                &[format!("{}", Type::Type), format!("{}", top.type_of())],
              ));
            };
            types.push(t);
          }
          types.reverse();
          stack.push(ConstValue::Type(Type::Struct {
            member_names: fields,
            member_types: types,
          }));
        },
        TypeAssert => {
          let top = pop(stack);
          let ConstValue::Type(expect) = top else {
            return Err(lint(
              TypeLint::TypeMismatch,
              span,
              &[format!("{}", Type::Type), format!("{}", top.type_of())],
            ));
          };
          let value = pop(stack);
          if expect != value.type_of() {
            return Err(lint(
              TypeLint::TypeMismatch,
              span,
              &[format!("{}", expect), format!("{}", value.type_of())],
            ));
          }
          stack.push(value);
        },
        Call { arity, spans } => {
          let mut arguments = vec![];
          for _ in 0..arity {
            arguments.push(pop(stack));
          }
          let top = pop(stack);
          let ConstValue::Function {
            name: func_name,
            parameters,
            returns,
          } = top
          else {
            return Err(lint(
              TypeLint::NonFunctionCall,
              span,
              &[format!("{}", top.type_of())],
            ));
          };
          arguments
            .iter()
            .zip(parameters.iter())
            .zip(spans.iter())
            .map(|((argument, expect), span)| {
              if &argument.type_of() != expect {
                Ok(())
              } else {
                Err(lint(
                  TypeLint::TypeMismatch,
                  *span,
                  &[format!("{expect}"), format!("{}", argument.type_of())],
                ))
              }
            })
            .try_collect::<Vec<_>>()?;
          if let Some(bt) = Builtin::from_mangle(&func_name) {
            self.execute_builtin(bt, &mut arguments, memory)?;
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
            panic!("Expected function block type")
          };
          parameter_names.into_iter().for_each(|n| {
            name_map.insert(n.clone(), arguments.pop().unwrap());
          });
          block = &self.blocks.get(&func_name).unwrap().body;
          block_name = func_name;
          ip = 0;
          continue;
        },
        Drop => {
          pop(stack);
        },
        If => {
          let top = pop(stack);
          let ConstValue::Boolean(predicate) = top else {
            return Err(lint(
              TypeLint::TypeMismatch,
              span,
              &[
                format!("{}", Primitive::boolean.promote()),
                format!("{}", top.type_of()),
              ],
            ));
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
                },
                Else if nesting == 0 => break,
                _ => {},
              }
            }
          }
        },
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
              },
              _ => {},
            }
          }
        },
        End => {},
        Loop => {},
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
              },
              _ => {},
            }
          }
        },
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
              },
              _ => {},
            }
          }
        },
      };
      ip += 1;
    }
  }

  fn execute_builtin(
    &self,
    builtin: Builtin,
    stack: &mut Vec<ConstValue>,
    memory: &mut Memory,
  ) -> Result<()> {
    match builtin {
      Builtin::PrintString => {
        let ConstValue::String { address, length } = stack.pop().unwrap()
        else {
          panic!();
        };
        let s = String::from_utf8_lossy(&memory.bytes_at(address, length));
        compiler_print(s);
      },
      Builtin::PrintReal => {
        let ConstValue::Real(r) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(r.to_string());
      },
      Builtin::PrintGlyph => {
        let ConstValue::Glyph(g) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(g);
      },
      Builtin::PrintInteger => {
        let ConstValue::Integer(i) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(i.to_string());
      },
      Builtin::PrintBoolean => {
        let ConstValue::Boolean(b) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(b.to_string());
      },
      Builtin::PrintType => {
        let ConstValue::Type(t) = stack.pop().unwrap() else {
          panic!();
        };
        compiler_print(format!("{t}"));
      },
      _ => panic!(),
    };
    Ok(())
  }
}
