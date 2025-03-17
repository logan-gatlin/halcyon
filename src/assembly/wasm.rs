use std::fmt::Display;

use crate::naming::Mangle;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum WasmType {
  FuncRef,
  F32,
  F64,
  I32,
  I64,
}

impl Display for WasmType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      WasmType::FuncRef => "funcref",
      WasmType::F32 => "f32",
      WasmType::F64 => "f64",
      WasmType::I32 => "i32",
      WasmType::I64 => "i64",
    };
    write!(f, "{}", s)
  }
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, Clone)]
pub enum WasmValue {
  FuncRef(Mangle),
  F32(f32),
  F64(f64),
  I32(i32),
  I64(i64),
}

impl WasmValue {
  pub fn type_of(&self) -> WasmType {
    match self {
      WasmValue::FuncRef(_) => WasmType::FuncRef,
      WasmValue::F32(_) => WasmType::F32,
      WasmValue::F64(_) => WasmType::F64,
      WasmValue::I32(_) => WasmType::I32,
      WasmValue::I64(_) => WasmType::I64,
    }
  }
}

impl Display for WasmValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      WasmValue::FuncRef(_) => format!(""),
      WasmValue::F32(val) => format!("{val}"),
      WasmValue::F64(val) => format!("{val}"),
      WasmValue::I32(val) => format!("{val}"),
      WasmValue::I64(val) => format!("{val}"),
    };
    write!(f, "{}", s)
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Wasm {
  Import {
    ns1: String,
    ns2: String,
    object: Box<Wasm>,
  },
  Local(WasmType, String),
  LocalSet(String),
  LocalGet(String),
  Function {
    ident: String,
    params: Vec<(String, WasmType)>,
    results: Vec<WasmType>,
    body: Vec<Wasm>,
  },
  If,
  Else,
  Loop(String),
  Block(String),
  Branch(String),
  Call(String),
  Constant(WasmValue),
  Add(WasmType),
  Subtract(WasmType),
  Multiply(WasmType),
  Divide(WasmType),
  Remainder(WasmType),
  And(WasmType),
  Or(WasmType),
  Xor(WasmType),
  Equal(WasmType),
  Unequal(WasmType),
  GreaterSigned(WasmType),
  GreaterUnsigned(WasmType),
  LesserSigned(WasmType),
  LesserUnsigned(WasmType),
  GreaterEqualSigned(WasmType),
  GreaterEqualUnsigned(WasmType),
  LesserEqualSigned(WasmType),
  LesserEqualUnsigned(WasmType),
  Negate(WasmType),
  Nop,
  Unreachable,
  Custom(String),
  Drop,
  Memory {
    min: usize,
    max: usize,
  },
  Data {
    offset: usize,
    content: Vec<u8>,
  },
  Return,
  End,
  Comment(String),
  Start(String),
}
