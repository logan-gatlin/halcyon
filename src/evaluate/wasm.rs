#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum WasmType {
  funcref,
  f32,
  f64,
  i32,
  i64,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Wasm {
  import {
    ns1: String,
    ns2: String,
    object: Box<Wasm>,
  },
  reg {
    type_: WasmType,
    ident: String,
    global: bool,
    initial: Option<Box<Wasm>>,
  },
  regset {
    ident: String,
    global: bool,
  },
  regget {
    ident: String,
    global: bool,
  },
  function {
    ident: String,
    params: Vec<(String, WasmType)>,
    results: Vec<WasmType>,
    body: Vec<Wasm>,
  },
  if_,
  else_,
  loop_(String),
  block(String),
  branch(String),
  call(String),
  constant(WasmType, String),
  add(WasmType),
  subtract(WasmType),
  multiply(WasmType),
  divide(WasmType),
  remainder(WasmType),
  and(WasmType),
  or(WasmType),
  xor(WasmType),
  equal(WasmType),
  unequal(WasmType),
  greater_s(WasmType),
  greater_u(WasmType),
  lesser_s(WasmType),
  lesser_u(WasmType),
  greaterequal_s(WasmType),
  greaterequal_u(WasmType),
  lesserequal_s(WasmType),
  lesserequal_u(WasmType),
  negate(WasmType),
  nop,
  trap,
  custom(String),
  drop,
  memory {
    min: usize,
    max: usize,
  },
  data {
    offset: usize,
    content: Vec<u8>,
  },
  return_,
  end,
  comment(String),
  /// IDK what this does
  start(String),
}
