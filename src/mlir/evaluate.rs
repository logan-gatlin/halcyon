use crate::{compile::*, compiler_print};

use super::*;

impl MlIrModule {
  fn prelude() -> HashMap<Mangle, ConstValue> {
    Builtin::ALL
      .into_iter()
      .map(|bt| (bt.to_mangle(), bt.value()))
      .collect()
  }

  pub fn evaluate(&self, ptr: IrPtr) -> Result<ConstValue> {
    let span = self.source_map.get(&ptr).unwrap();
    let mut return_stack: Vec<IrPtr> = vec![];
    let mut stack: Vec<ConstValue> = vec![];
    let mut ip = span.0;
    let mut name_map = Self::prelude();
    let optable = OpTable::new();
    loop {
      if ip >= span.0 + span.1 && return_stack.len() == 0 {
        break;
      }
      let instr = self.ir.get(ip).unwrap();
      use MlIrKind::*;
      let span = instr.span;
      println!("{stack:?}");
      println!("{instr}");
      match instr.kind.clone() {
        Const(const_value) => stack.push(const_value),
        Set(mangle) => {
          name_map.insert(mangle, stack.pop().unwrap());
        },
        Get(mangle) => stack.push(name_map.get(&mangle).unwrap().clone()),
        BinaryOp(kind) => {
          let first = stack.pop().unwrap();
          let second = stack.pop().unwrap();
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
        UnaryOp(kind) => {
          let value = stack.pop().unwrap();
          let opkind = optable.try_unary(kind, &value.type_of()).span(span)?;
          let result =
            VirtualMachine::run(vec![value], opkind.asm, opkind.produces)
              .unwrap();
          stack.push(result);
        },
        Field(field) => {
          let top = stack.pop().unwrap();
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
        StructLiteral(member_names) => {
          let mut member_values = vec![];
          for _ in 0..member_names.len() {
            member_values.push(stack.pop().unwrap());
          }
          member_values.reverse();
          stack.push(ConstValue::StructLiteral {
            member_names,
            member_values,
          });
        },
        StructDef(fields) => {
          let mut types = vec![];
          for _ in 0..fields.len() {
            let top = stack.pop().unwrap();
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
        Tuple(count) => {
          let mut items = vec![];
          for _ in 0..count {
            items.push(stack.pop().unwrap());
          }
          items.reverse();
          stack.push(ConstValue::Tuple(items))
        },
        TypeAssert => {
          let top = stack.pop().unwrap();
          let ConstValue::Type(expect) = top else {
            return Err(lint(
              TypeLint::TypeMismatch,
              span,
              &[format!("{}", Type::Type), format!("{}", top.type_of())],
            ));
          };
          let value = stack.pop().unwrap();
          if expect != value.type_of() {
            return Err(lint(
              TypeLint::TypeMismatch,
              span,
              &[format!("{}", expect), format!("{}", value.type_of())],
            ));
          }
          stack.push(value);
        },
        Call(_) => todo!(),
        Drop => {
          stack.pop();
        },
        If => {
          let top = stack.pop().unwrap();
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
              if ip >= self.ir.len() {
                panic!()
              }
              match &self.ir.get(ip).unwrap().kind {
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
            if ip >= self.ir.len() {
              panic!()
            }
            match &self.ir.get(ip).unwrap().kind {
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
        Function(_) => todo!(),
        Return => todo!(),
        Nop => {},
      }
      ip += 1
    }
    Ok(stack.pop().unwrap())
  }
}
