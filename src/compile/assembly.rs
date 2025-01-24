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
  greater(AsmType),
  lesser(AsmType),
  greaterequal(AsmType),
  lesserequal(AsmType),
  negate(AsmType),
  nop,
  trap,
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
      Type::Struct { member_types, .. } => member_types.iter().map(|t| t.count_registers()).sum(),
      Type::Function { .. } => 1,
      Type::Type(_) => 0,
      _ => panic!("Counted registers of ambiguous type"),
    }
  }

  pub fn asm_types(&self) -> Vec<AsmType> {
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
      Type::Struct { member_types, .. } => {
        member_types.iter().flat_map(|t| t.asm_types()).collect()
      }
      Type::Function { .. } => vec![a::funcref],
      Type::Type(_) => vec![],
      _ => panic!("Splatted ambiguous type"),
    }
  }
}
