pub mod wasm;

use wasm::{Wasm, WasmType};

use crate::semantic::primitives::Primitive;
use crate::semantic::{ConstValue, Mangle, Type};
use crate::{err::*, error};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum ExWasmType {
  basic(WasmType),
  type_,
  string,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum ExValue {
  funcref(Mangle),
  f32(f32),
  f64(f64),
  i32(i32),
  i64(i64),
  type_(Type),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum ExWasm {
  basic(Wasm),
  // Push arbitrary types
  push(ConstValue),
  // Pop arbitrary types
  pop(Type),
  /// Pop 1 type `t`, push Type::Reference(`t`)
  reference,
  /// Pop N (string, type) pairs, push 1 struct type with
  /// fields corresponding to each (name : type) pair.
  define_struct(usize),
}

impl ConstValue {
  pub fn to_exvalue(&self) -> Vec<ExValue> {
    use ExValue as e;
    match self.clone() {
      ConstValue::Nothing => vec![],
      ConstValue::Integer(val) => vec![e::i64(val)],
      ConstValue::Real(val) => vec![e::f64(val)],
      ConstValue::Boolean(val) => vec![e::i32(val as i32)],
      ConstValue::String { address, length } => {
        vec![e::i32(address as i32), e::i32(length as i32)]
      },
      ConstValue::Glyph(val) => vec![e::i32(val as i32)],
      ConstValue::Function(val) => vec![e::funcref(val)],
      ConstValue::StructLiteral { member_values, .. } => member_values
        .into_iter()
        .rev()
        .flat_map(|v| v.to_exvalue())
        .collect(),
      ConstValue::Type(t) => vec![e::type_(t)],
    }
  }

  pub fn from_exvalue(
    expects: Type,
    stack: &mut Vec<ExValue>,
  ) -> Result<ConstValue> {
    let irretrievable = error!("Cannot retrieve '{expects}' from exvalues");
    let unexpected = error!(
      "Found unexpected value on stack when retrieving '{expects}' type"
    );
    let mut pop = || stack.pop().reason("Not enough values on the stack");
    match expects {
      Type::Ambiguous => irretrievable,
      Type::Reference(_) => irretrievable,
      Type::Prim(primitive) => match primitive {
        Primitive::nothing => Ok(ConstValue::Nothing),
        Primitive::never => irretrievable,
        Primitive::integer => {
          if let ExValue::i64(val) = pop()? {
            Ok(ConstValue::Integer(val))
          } else {
            unexpected
          }
        },
        Primitive::real => {
          if let ExValue::f64(val) = pop()? {
            Ok(ConstValue::Real(val))
          } else {
            unexpected
          }
        },
        Primitive::boolean => {
          if let ExValue::i32(val) = pop()? {
            Ok(ConstValue::Boolean(val == 1))
          } else {
            unexpected
          }
        },
        Primitive::string => {
          let ExValue::i32(length) = pop()? else {
            return unexpected;
          };
          let ExValue::i32(address) = pop()? else {
            return unexpected;
          };
          Ok(ConstValue::String {
            length: length as usize,
            address: address as usize,
          })
        },
        Primitive::glyph => {
          if let ExValue::i32(val) = pop()? {
            Ok(ConstValue::Glyph(
              char::from_u32(val as u32)
                .reason("Failed to decode glyph from top of stack")?,
            ))
          } else {
            unexpected
          }
        },
      },
      Type::Struct {
        member_types,
        member_names,
      } => Ok(ConstValue::StructLiteral {
        member_names,
        member_values: member_types
          .into_iter()
          .map(|t| ConstValue::from_exvalue(t, stack))
          .try_collect()?,
      }),
      // TODO revisit this
      Type::Function { .. } => {
        if let ExValue::funcref(val) = pop()? {
          Ok(ConstValue::Function(val))
        } else {
          unexpected
        }
      },
      Type::Type => {
        if let ExValue::type_(val) = pop()? {
          Ok(ConstValue::Type(val))
        } else {
          unexpected
        }
      },
    }
  }
}

pub struct VM {
  stack: Vec<ExValue>,
}
