use crate::semantic::Type;
use crate::semantic::primitives::Primitive;
use crate::{BinaryOp, err::*};
use crate::{
  Immediate,
  semantic::ir::{Node, NodeKind},
};

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
      Type::Struct { member_types, .. } => {
        member_types.iter().map(|t| t.count_registers()).sum()
      },
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
      Type::Struct { member_types, .. } => member_types
        .iter()
        .rev()
        .flat_map(|t| t.asm_types())
        .collect(),
      Type::Function { .. } => vec![a::funcref],
      Type::Type(_) => vec![],
      _ => panic!("Splatted ambiguous type"),
    }
  }
}

pub struct Compiler {}

impl Compiler {
  fn flatten_functions(&self, mut node: Node, global: &mut Vec<Node>) -> Node {
    use NodeKind::*;
    node.kind = match node.kind {
      Immediate(_) => node.kind,
      Identifier { .. } => node.kind,
      StructLiteral { names, values } => StructLiteral {
        names,
        values: values
          .into_iter()
          .map(|v| self.flatten_functions(v, global))
          .collect(),
      },
      BinaryOp {
        op,
        opdef,
        left,
        right,
      } => BinaryOp {
        op,
        opdef,
        left: self.flatten_functions(*left, global).into(),
        right: self.flatten_functions(*right, global).into(),
      },
      UnaryOp { op, opdef, child } => UnaryOp {
        op,
        opdef,
        child: self.flatten_functions(*child, global).into(),
      },
      Field { namespace, index } => Field {
        namespace: self.flatten_functions(*namespace, global).into(),
        index,
      },
      If {
        predicate,
        then,
        else_,
      } => If {
        predicate: self.flatten_functions(*predicate, global).into(),
        then: self.flatten_functions(*then, global).into(),
        else_: if let Some(else_) = else_ {
          Some(self.flatten_functions(*else_, global).into())
        } else {
          None
        },
      },
      Call {
        mangle,
        callee,
        params,
      } => Call {
        mangle,
        callee: self.flatten_functions(*callee, global).into(),
        params: params
          .into_iter()
          .map(|p| self.flatten_functions(p, global))
          .collect(),
      },
      Function {
        mangle,
        arguments,
        nodes,
      } => todo!(),
      Declaration {
        name,
        mangle,
        is_constant,
        type_assert,
        value,
      } => todo!(),
      Block { nodes } => todo!(),
      Remainder { node } => todo!(),
      Loop {
        names,
        initials,
        body,
      } => todo!(),
      Break { expr } => todo!(),
    };
    node
  }
}
