use crate::{
  BinaryOp, Expression, ExpressionKind, Immediate, Parameter, Statement,
  StatementKind, UnaryOp,
  semantic::{Symbol, SymbolTable},
};

use super::{primitives::*, uid};
use crate::err::*;

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Nothing,
  Prim(Primitive),
  Struct(Vec<Parameter>),
  StructDef(Vec<Parameter>),
  FunctionRef {
    params: Vec<Type>,
    returns: Box<Type>,
  },
  FunctionDef {
    params: Vec<Type>,
    returns: Box<Type>,
    id: uid,
  },
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Nothing => write!(f, "nothing"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Struct(vec) => write!(f, "struct {vec:?}"),
      Type::StructDef(vec) => write!(f, "struct definition"),
      Type::FunctionRef { params, returns } => {
        write!(f, "({params:?}) -> {returns}")
      },
      Type::FunctionDef {
        params,
        returns,
        id,
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
      (Struct(p1), Struct(p2)) => p1
        .iter()
        .map(|p| p.type_actual.clone())
        .eq(p2.iter().map(|p| p.type_actual.clone())),
      (
        FunctionRef {
          params: p1,
          returns: r1,
        },
        FunctionRef {
          params: p2,
          returns: r2,
        },
      ) => p1.iter().eq(p2.iter()) && r1 == r2,
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

  pub fn coerce(expect: &Type, actual: &Type) -> Result<Type> {
    use Primitive as p;
    use Type::*;
    let e = || {
      error().reason(format!(
        "Could not coerce type '{actual:?}' into '{expect:?}'"
      ))
    };
    match (expect, actual) {
      (Ambiguous, Ambiguous) => e(),
      (Ambiguous, Prim(p::integer_ambiguous)) => Ok(Prim(p::integer)),
      (Ambiguous, Prim(p::real_ambiguous)) => Ok(Prim(p::real)),
      (Ambiguous, t) => Ok(t.clone()),
      (Prim(p1), Prim(p2)) => {
        let (p1, p2) = Primitive::coerce_ambiguous(*p1, *p2);
        if p1 != p2 { e() } else { Ok(Type::Prim(p1)) }
      },
      (t1, t2) if t1 == t2 => Ok(t1.clone()),
      _ => e(),
    }
  }
}
