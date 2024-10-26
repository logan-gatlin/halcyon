use crate::{
  BinaryOp, Expression, ExpressionKind, Immediate, Parameter, Statement,
  StatementKind, UnaryOp,
};

use super::{UID, primitives::*};
use crate::err::*;

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Nothing,
  Prim(Primitive),
  Struct(Vec<Type>),
  StructDef(Vec<Parameter>),
  FunctionRef {
    params: Vec<Type>,
    returns: Box<Type>,
    id: UID,
  },
  FunctionDef {
    params: Vec<Type>,
    returns: Box<Type>,
    id: UID,
  },
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Nothing => write!(f, "nothing"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Struct(vec) => write!(f, "struct {vec:?}"),
      Type::StructDef(_) => write!(f, "struct definition"),
      Type::FunctionRef {
        params, returns, ..
      } => {
        write!(f, "({params:?}) -> {returns}")
      },
      Type::FunctionDef {
        params, returns, ..
      } => write!(f, "({params:?}) -> {returns}"),
    }
  }
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type::*;
    match (self, other) {
      (Ambiguous, Ambiguous) => true,
      (Prim(p1), Prim(p2)) if p1 == p2 => true,
      (Struct(p1), Struct(p2)) => p1.iter().eq(p2.iter()),
      (FunctionRef { id: id1, .. }, FunctionRef { id: id2, .. }) => id1 == id2,
      (FunctionDef { id: id1, .. }, FunctionDef { id: id2, .. }) => id1 == id2,
      (Nothing, Nothing) => true,
      _ => false,
    }
  }
}

impl Type {
  pub fn binary_op(lhs: &Type, op: BinaryOp, rhs: &Type) -> Result<Type> {
    use Type as t;
    let e = error().reason(format!(
      "Binary {op} is not defined for {lhs:?} and {rhs:?}",
    ));
    match (lhs, rhs) {
      (t::Prim(a), t::Prim(b)) => {
        let p = Primitive::binary_op(*a, op, *b)?;
        Ok(t::Prim(p))
      },
      _ => e,
    }
  }

  pub fn unary_op(op: UnaryOp, child: &Type) -> Result<Type> {
    use Type as t;
    if let t::Prim(p) = child {
      let p = Primitive::unary_op(op, *p)?;
      Ok(t::Prim(p))
    } else {
      error().reason(format!("Unary {op} is not defined for {child:?}"))
    }
  }

  pub fn coerce(&mut self, expect: &Type) {
    use Type::*;
    *self = match (expect, self.clone()) {
      (Ambiguous, Ambiguous) => Ambiguous,
      (Ambiguous, t) => t.clone(),
      (Prim(mut p1), Prim(p2)) => {
        p1.coerce(p2);
        Prim(p1)
      },
      (t1, t2) if t1 == &t2 => t1.clone(),
      _ => Ambiguous,
    };
  }
}
