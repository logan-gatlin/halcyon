use crate::semantic::{primitives::Primitive, Type};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum AsmType {
  funcref,
  f32,
  f64,
  i32,
  i64,
}

impl std::fmt::Display for AsmType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      AsmType::funcref => "funcref",
      AsmType::f32 => "f32",
      AsmType::f64 => "f64",
      AsmType::i32 => "i32",
      AsmType::i64 => "i64",
    };
    write!(f, "{s}")
  }
}

impl AsmType {
  const PTR_T: Self = Self::i32;
}

#[derive(Debug, Clone)]
pub struct WasmModule(pub Vec<Wasm>);

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Wasm {
  reg {
    type_: AsmType,
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
    params: Vec<(String, AsmType)>,
    results: Vec<AsmType>,
    body: Vec<Wasm>,
  },
  ifelse {
    then: Vec<Wasm>,
    else_: Vec<Wasm>,
  },
  block {
    name: String,
    body: Vec<Wasm>,
  },
  loop_ {
    name: String,
    body: Vec<Wasm>,
  },
  branch(String),
  call(String),
  constant(AsmType, String),
  add(AsmType),
  subtract(AsmType),
  multiply(AsmType),
  divide(AsmType),
  remainder(AsmType),
  and(AsmType),
  or(AsmType),
  xor(AsmType),
  equal(AsmType),
  unequal(AsmType),
  greater_s(AsmType),
  greater_u(AsmType),
  lesser_s(AsmType),
  lesser_u(AsmType),
  greaterequal_s(AsmType),
  greaterequal_u(AsmType),
  lesserequal_s(AsmType),
  lesserequal_u(AsmType),
  negate(AsmType),
  nop,
  trap,
  drop,
  memory {
    min: usize,
    max: usize,
  },
  data {
    offset: usize,
    content: String,
  },
  comment(String),
  /// IDK what this does
  start(String),
}

impl Type {
  pub fn count_registers(&self) -> usize {
    use Primitive as p;
    match self {
      Type::Prim(primitive) => match primitive {
        p::nothing | p::never => 0,
        p::glyph | p::integer | p::real | p::boolean => 1,
        p::string => 2,
        _ => panic!("Counted registers of literal type"),
      },
      Type::Struct { member_types, .. } => member_types
        .iter()
        .map(|t| t.clone().unwrap_type_name().unwrap().count_registers())
        .sum(),
      Type::Function { .. } => 1,
      Type::Type(_) => 0,
      _ => panic!("Counted registers of ambiguous type"),
    }
  }

  pub fn register_types(&self) -> Vec<AsmType> {
    use AsmType as a;
    use Primitive as p;
    match self {
      Type::Prim(primitive) => match primitive {
        p::nothing | p::never => vec![],
        p::integer => vec![a::i64],
        p::real => vec![a::f64],
        p::boolean => vec![a::i32],
        p::string => vec![a::PTR_T, a::PTR_T],
        p::glyph => vec![a::i32],
        p::integer_literal | p::real_literal => panic!("Splatted literal type"),
      },
      Type::Struct { member_types, .. } => member_types
        .iter()
        .flat_map(|t| t.clone().unwrap_type_name().unwrap().register_types())
        .collect(),
      Type::Function { .. } => vec![a::funcref],
      Type::Type(_) => vec![],
      _ => panic!("Splatted ambiguous type"),
    }
  }
}
