pub mod analyzer;
mod ir;
mod operators;
pub mod primitives;

pub use analyzer::Analyzer;
pub use primitives::*;

use crate::err::*;
use crate::error;
use crate::semantic::Primitive;

pub type Mangle = String;

impl Primitive {
  pub fn promote(self) -> Type {
    Type::Prim(self)
  }
}

#[derive(Debug, Clone)]
pub enum Type {
  /// Indeterminate type
  Ambiguous,
  /// A primitive type
  Prim(Primitive),
  /// User defined type
  Struct {
    mangle: Mangle,
    member_names: Vec<String>,
    member_types: Vec<Type>,
  },
  /// Unique function type
  Function {
    // param names are part of the type to allow kwargs in the future
    mangle: Mangle,
    param_names: Vec<String>,
    param_types: Vec<Type>,
    return_type: Box<Type>,
  },
  Type(Box<Type>),
  Unresolved(Mangle),
}

impl Type {
  pub fn expect_type_name(self) -> Result<Self> {
    if let Type::Type(_) = self {
      Ok(self)
    } else {
      error!("Expected the name of a type here, found '{self:?}'")
    }
  }

  pub fn unwrap_type_name(self) -> Result<Self> {
    if let Type::Type(t) = self {
      Ok(*t)
    } else {
      error!("Expected the name of a type here, found '{self:?}'")
    }
  }
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type as t;
    match (self, other) {
      (t::Ambiguous, t::Ambiguous) => true,
      (t::Prim(p1), t::Prim(p2)) => p1 == p2,
      (t::Struct { mangle: m1, .. }, t::Struct { mangle: m2, .. }) => m1 == m2,
      (t::Function { mangle: m1, .. }, t::Function { mangle: m2, .. }) => {
        m1 == m2
      },
      (t::Type(t1), t::Type(t2)) => t1 == t2,
      (t::Unresolved(t1), t::Unresolved(t2)) => {
        panic!("Tried to compare unresolved types '{t1}' and '{t2}'")
      },
      _ => false,
    }
  }
}

impl Eq for Type {
}

impl std::hash::Hash for Type {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Type::Prim(primitive) => primitive.hash(state),
      Type::Struct { mangle, .. } | Type::Function { mangle, .. } => {
        mangle.hash(state)
      },
      Type::Type(t) => t.hash(state),
      Type::Unresolved(t) => panic!("Tried to hash unresolved type '{t}'"),
      Type::Ambiguous => panic!("Tried to hash ambiguous type"),
    }
  }
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Struct { .. } => write!(f, "struct"),
      Type::Type(tid) => write!(f, "{tid}"),
      Type::Function { .. } => write!(f, "func"),
      Type::Unresolved(m) => write!(f, "? ({m})"),
    }
  }
}

/// Name mangle syntax:
/// mangle ::= "$" path salt
/// path ::= {path-element}*
/// path-element ::= length ident
/// ident ::= _a-zA-Z {_a-zA-Z0-9}*
/// length ::= {0-9}+
/// salt ::= {a-zA-Z}*
pub fn mangle_name(path: Vec<String>, salt: &str) -> Mangle {
  let mut buf: Vec<u8> = vec![];
  for p in path {
    let bytes = format!("{}{}", p.len(), punycode::encode(&p).unwrap());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}

/// Builtin mangle syntax:
/// $${ident}
pub fn mangle_builtin(name: impl std::fmt::Display) -> Mangle {
  format!("${name}")
}

pub const AMBIGUOUS_MANGLE: &str = "$ambiguous";
