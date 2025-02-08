use crate::semantic::Mangle;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum WasmType {
  FuncRef,
  F32,
  F64,
  I32,
  I64,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum WasmValue {
  FuncRef(Mangle),
  F32(f32),
  F64(f64),
  I32(i32),
  I64(i64),
}

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
