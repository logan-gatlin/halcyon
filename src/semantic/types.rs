use crate::{BinaryOp, UnaryOp};

use super::{UID, primitives::*};
use crate::err::*;

/// Struct ID
pub type SID = usize;

#[derive(Debug, Clone)]
pub struct StructureDef(pub Vec<(String, UID)>);

#[derive(Debug, Clone)]
pub struct FunctionDef {
  pub params: Vec<UID>,
  pub returns: UID,
}

pub fn nothing_mangle() -> UID {
  "$$nothing".into()
}

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Nothing,
  Alias(Box<Type>),
  Prim(Primitive),
  Struct(SID),
  Function(UID),
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Nothing => write!(f, "nothing"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Struct(vec) => write!(f, "struct {vec:?}"),
      Type::Alias(t) => write!(f, "type alias ({t})"),
      Type::Function(fid) => write!(f, "func {fid:?}"),
    }
  }
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type::*;
    match (self, other) {
      (Ambiguous, Ambiguous) => true,
      (Prim(p1), Prim(p2)) if p1 == p2 => true,
      (Struct(p1), Struct(p2)) => p1 == p2,
      (Function(id1), Function(id2)) => id1 == id2,
      (Alias(t1), Alias(t2)) => t1 == t2,
      (Nothing, Nothing) => true,
      _ => false,
    }
  }
}

impl Type {
  pub fn is_alias(&self) -> Result<Type> {
    if let Type::Alias(t) = self {
      Ok(*t.clone())
    } else {
      error().reason(format!("Expected type, found value with type {}", self))
    }
  }

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

  pub fn deduce(self, hint: &Type) -> Result<Self> {
    use Type::*;
    match (self, hint) {
      (Ambiguous, t) => Ok(t.clone()),
      (t, Ambiguous) => Ok(t),
      (Prim(mut p1), Prim(p2)) => {
        p1 = p1.coerce(*p2)?;
        Ok(Prim(p1))
      },
      (t1, t2) if &t1 == t2 => Ok(t1.clone()),
      (t1, t2) => {
        error().reason(format!("Cannot coerce type '{t2}' into '{t1}'"))
      },
    }
  }

  pub fn coerce(self, expect: &Type) -> Result<Self> {
    use Type::*;
    match (self, expect) {
      (Prim(mut p1), Prim(p2)) => {
        p1 = p1.coerce(*p2)?;
        Ok(Prim(p1))
      },
      (t1, t2) if &t1 == t2 => Ok(t1.clone()),
      (t1, t2) => {
        error().reason(format!("Cannot coerce type '{t2}' into '{t1}'"))
      },
    }
  }

  pub fn promote(self) -> Self {
    use Type::*;
    match self {
      Prim(mut p) => {
        p.promote();
        Prim(p)
      },
      t => t,
    }
  }
}
