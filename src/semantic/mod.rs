pub mod analyzer;
pub mod builtin;
pub mod ir;
pub mod names;
pub mod primitives;

pub use primitives::*;

use crate::semantic::Primitive;

/// Type ID
pub type TID = usize;
/// Name mangle
pub type SID = String;

#[derive(Debug, Clone, Copy)]
pub enum Lifetime {
  /// Exists for lifetime of program
  Static,
  /// Exists for lifetime of contained scope
  Dynamic,
}

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Prim(Primitive),
  Nothing,
  Never,
  Struct {
    size: usize,
    name: String,
    member_names: Vec<String>,
    member_types: Vec<TID>,
  },
  Alias(TID),
  Function {
    name: String,
    arg_names: Vec<String>,
    arg_types: Vec<TID>,
  },
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Nothing => write!(f, "nothing"),
      Type::Never => write!(f, "never"),
      Type::Struct { name, .. } => write!(f, "struct {name}"),
      Type::Alias(tid) => write!(f, "alias ({tid})"),
      Type::Function { name, .. } => write!(f, "func ({name})"),
    }
  }
}
