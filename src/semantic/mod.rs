pub mod analyzer;
mod ir;
pub mod primitives;

pub use analyzer::Analyzer;
pub use primitives::*;

use crate::semantic::Primitive;

pub type Mangle = String;

#[derive(Debug, Clone)]
pub enum Type {
  /// Indeterminate type
  Ambiguous,
  /// A primitive type
  Prim(Primitive),
  /// User defined type
  Struct {
    member_names: Vec<String>,
    member_types: Vec<Type>,
  },
  /// Unique function type
  Function {
    // param names are part of the type to allow kwargs in the future
    param_names: Vec<String>,
    param_types: Vec<Type>,
    return_type: Box<Type>,
  },
  Type(Box<Type>),
  Unresolved(Mangle),
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Struct { .. } => write!(f, "struct"),
      Type::Type(tid) => write!(f, "alias ({tid})"),
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
  format!("$${name}")
}

pub const AMBIGUOUS_MANGLE: &str = "$$ambiguous";
