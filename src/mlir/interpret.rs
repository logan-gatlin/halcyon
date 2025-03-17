use crate::{compile::*, compiler_print, hlir::*, lint::*, mlir::*, operator::*};

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
      if self.control_stack.len() > RECURSION_LIMIT {
        return Err(lint_nospan(EvalLint::RecursionLimit));
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
              return Err(lint_nospan(TypeLint::TypeMismatch))
                .context(format!("{expected_type}"))
                .context(format!("{return_type}"));
            }
            self.retrieve_state();
            self.block = new_block;
            self.ip = new_ip;
            self.push(return_value);
            continue;
          } else {
            break;
          }
        }
        Block::Unreachable => {
          return Err(lint_nospan(EvalLint::Unreachable));
        }
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
            v => Err(lint(TypeLint::TypeMismatch, span, &[
              "boolean".to_string(),
              format!("{v}"),
            ])),
          }
        }
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
            match &instr.kind {
              MlIrKind::Const(const_value) => self.push(const_value.clone()),
              MlIrKind::Set(mangle) => {
                let value = self.pop();
                if self.module.constants.contains_key(mangle) {
                  if self.const_value_map.contains_key(mangle) {
                    panic!("Duplicate initializations of {mangle} during const-eval")
                  }
                  let set_type = self.type_of_const(&value);
                  let old_type = self.type_map.insert(mangle.clone(), set_type.clone());
                  if let Some(old_type) = old_type
                    && set_type != old_type
                  {
                    return Err(lint(TypeLint::TypeMismatch, instr.span, &[
                      format!("{old_type}"),
                      format!("{set_type}"),
                    ]));
                  }
                  self.const_value_map.insert(mangle.clone(), value);
                } else {
                  self.rt_value_map.insert(mangle.clone(), value);
                };
              }
              MlIrKind::Get(mangle) => {
                let map = if self.const_value_map.contains_key(mangle) {
                  &mut self.const_value_map
                } else {
                  &mut self.rt_value_map
                };
                let Some(value) = map.get(mangle).cloned() else {
                  return Err(lint(EvalLint::Circular, instr.span, &[]));
                };
                self.push(value.clone());
              }
              MlIrKind::BinaryOp { kind } => {
                let right = self.pop();
                let right_t = self.type_of_const(&right);
                let left = self.pop();
                let left_t = self.type_of_const(&left);
                let opdef = optable
                  .try_binary(*kind, &left_t, &right_t)
                  .span(instr.span)?;
                let result = VirtualMachine::run(vec![left, right], opdef.asm, opdef.produces)
                  .span(instr.span)?;
                self.push(result);
              }
              MlIrKind::UnaryOp { kind } => {
                let on = self.pop();
                let on_t = self.type_of_const(&on);
                let opdef = optable.try_unary(*kind, &on_t).span(instr.span)?;
                let result =
                  VirtualMachine::run(vec![on], opdef.asm, opdef.produces).span(instr.span)?;
                self.push(result);
              }
              MlIrKind::Field(field_name) => {
                let top = self.pop();
                let ConstValue::StructLiteral {
                  member_names,
                  member_values,
                } = top
                else {
                  return Err(lint(TypeLint::NoFieldOnType, instr.span, &[format!(
                    "{}",
                    self.type_of_const(&top)
                  )]));
                };
                let pos = member_names
                  .iter()
                  .position(|n| n == field_name)
                  .lint(TypeLint::FieldMissing)
                  .context(format!("{field_name}"))
                  .span(instr.span)?;
                self.push(member_values[pos].clone());
              }
              MlIrKind::StructLiteral { param_names } => {
                let member_values: Vec<_> =
                  (0..param_names.len()).map(|_| self.pop()).rev().collect();
                self.push(ConstValue::StructLiteral {
                  member_names: param_names.clone(),
                  member_values,
                })
              }
              MlIrKind::StructDef {
                fields: param_names,
              } => {
                let mut member_types: Vec<_> = (0..param_names.len())
                  .rev()
                  .map(|i| {
                    let top = self.pop();
                    if let ConstValue::Type(t) = top {
                      Ok(t)
                    } else {
                      Err(lint(TypeLint::TypeMismatch, instr.span, &[
                        "type".to_string(),
                        format!("{}", self.type_of_const(&top)),
                      ]))
                    }
                  })
                  .try_collect()?;
                member_types.reverse();
                self.push(ConstValue::Type(Type::Struct {
                  member_names: param_names.clone(),
                  member_types,
                }))
              }
              MlIrKind::TypeAssert(mangle) => {
                let assert_type = self.pop();
                let ConstValue::Type(assert) = assert_type else {
                  return Err(lint(TypeLint::TypeMismatch, instr.span, &[
                    "type".to_string(),
                    format!("{assert_type}"),
                  ]));
                };
                let actual_val = self.pop();
                let actual_t = self.type_of_const(&actual_val);
                if actual_t != assert {
                  return Err(lint(TypeLint::TypeMismatch, instr.span, &[
                    format!("{assert}"),
                    format!("{actual_t}"),
                  ]));
                }
                self.push(actual_val);
                if let Some(mangle) = mangle {
                  self.type_map.insert(mangle.clone(), assert);
                }
              }
              MlIrKind::Call { arity } => {
                let func = self.pop();
                let func_type = self.type_of_const(&func);
                let Type::Function {
                  param_types,
                  return_type,
                } = func_type
                else {
                  return Err(lint(TypeLint::NonFunctionCall, instr.span, &[format!(
                    "{func_type}"
                  )]));
                };
                if param_types.len() != *arity {
                  return Err(lint(
                    if param_types.len() > *arity {
                      TypeLint::TooManyArgs
                    } else {
                      TypeLint::TooFewArgs
                    },
                    instr.span,
                    &[format!("{}", param_types.len())],
                  ));
                }
                let values: Vec<_> = param_types
                  .into_iter()
                  .enumerate()
                  .rev()
                  .map(|(id, expected)| {
                    let param = self.pop();
                    let param_t = self.type_of_const(&param);
                    if param_t != expected {
                      Err(lint(TypeLint::TypeMismatch, instr.span, &[
                        format!("{expected}"),
                        format!("{param_t}"),
                      ]))
                    } else {
                      Ok(param)
                    }
                  })
                  .try_collect()?;
                let ConstValue::Function(mangle) = func else {
                  unreachable!()
                };
                if let Some(builtin) = Builtin::from_mangle(&mangle) {
                  values.into_iter().rev().for_each(|v| self.push(v));
                  self.execute_builtin(builtin).span(instr.span)?;
                } else {
                  self.save_state();
                  self.control_stack.push(ReturnAddress {
                    block: self.block,
                    ip: self.ip + 1,
                    expected_type: *return_type,
                  });
                  let fun = self.module.functions.get(&mangle).unwrap();
                  values
                    .into_iter()
                    .rev()
                    .zip(fun.parameter_mangles.iter())
                    .for_each(|(value, mangle)| {
                      self.rt_value_map.insert(mangle.clone(), value);
                    });
                  self.block = fun.block;
                  self.ip = 0;
                  continue; // Prevent IP from incrementing
                }
              }
              MlIrKind::Drop => {
                self.pop();
              }
              MlIrKind::StartScope => self.start_scope_v(),
              MlIrKind::EndScope => self.end_scope_v(),
            }
            self.ip += 1;
            Ok(self.block)
          }
        }
      }?;
    }
    Ok(())
  }

  fn execute_builtin(&mut self, builtin: Builtin) -> Result<()> {
    match builtin {
      Builtin::Type => panic!(),
      Builtin::PrintString => {
        let ConstValue::String {
          virtual_address, ..
        } = self.pop()
        else {
          panic!();
        };
        let s = String::from_utf8_lossy(&self.module.heap[virtual_address]);
        compiler_print(s);
      }
      Builtin::PrintReal => {
        let ConstValue::Real(r) = self.pop() else {
          panic!();
        };
        compiler_print(r.to_string());
      }
      Builtin::PrintGlyph => {
        let ConstValue::Glyph(g) = self.pop() else {
          panic!();
        };
        compiler_print(g);
      }
      Builtin::PrintInteger => {
        let ConstValue::Integer(i) = self.pop() else {
          panic!();
        };
        compiler_print(i.to_string());
      }
      Builtin::PrintBoolean => {
        let ConstValue::Boolean(b) = self.pop() else {
          panic!();
        };
        compiler_print(b.to_string());
      }
      Builtin::PrintType => {
        let ConstValue::Type(t) = self.pop() else {
          panic!();
        };
        compiler_print(format!("{t}"));
      }
    };
    Ok(())
  }
}
