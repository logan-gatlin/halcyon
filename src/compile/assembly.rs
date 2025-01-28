use crate::semantic::{Type, primitives::Primitive};

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
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Asm {
  module(Vec<Asm>),
  reg {
    type_: AsmType,
    ident: String,
    global: bool,
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
    body: Vec<Asm>,
  },
  ifelse {
    then: Vec<Asm>,
    else_: Vec<Asm>,
  },
  block {
    name: String,
    body: Vec<Asm>,
  },
  loop_ {
    name: String,
    body: Vec<Asm>,
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
  comment(String),
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
      Type::Struct { member_types, .. } => {
        member_types.iter().map(|t| t.count_registers()).sum()
      },
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
        .flat_map(|t| t.register_types())
        .collect(),
      Type::Function { .. } => vec![a::funcref],
      Type::Type(_) => vec![],
      _ => panic!("Splatted ambiguous type"),
    }
  }
}
