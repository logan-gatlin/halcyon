pub mod analyzer;
//pub mod bottom_up;
pub mod consteval;
pub mod expression;
pub mod ir;
pub mod operators;
pub mod primitives;
pub mod typecheck;
//pub mod top_down;

pub use consteval::*;
pub use ir::*;
use operators::OpTable;
pub use primitives::*;

use std::collections::HashMap;

use analyzer::*;
//use operators::OpTable;

use crate::err::*;
use crate::error;
use crate::semantic::Primitive;

pub type Mangle = String;

impl Primitive {
  pub fn promote(self) -> Type {
    Type::Prim(self)
  }
}

/// Convert parse tree to AST
pub struct Analyzer {
  scope_depth: usize,
  salt: usize,
  path: Vec<String>,
  _name_to_symbol: HashMap<String, Symbol>,
  event_stack: Vec<Event>,
  pub op_table: OpTable,
  data_segment: Vec<u8>,
  data_offset: usize,
  constants: HashMap<Mangle, Node>,
  main: Option<Mangle>,
}

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
  /// Function type
  Function {
    param_types: Vec<Type>,
    return_type: Box<Type>,
  },
  /// Alias type
  Reference(Box<Type>),
  /// Higher level type
  Type,
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type as t;
    match (self, other) {
      (t::Ambiguous, t::Ambiguous) => {
        panic!("Tried to compare ambiguous types")
      }
      (t::Type, t::Type) => true,
      (t::Prim(p1), t::Prim(p2)) => p1 == p2,
      (
        t::Struct {
          member_names: names1,
          member_types: types1,
        },
        t::Struct {
          member_names: names2,
          member_types: types2,
        },
      ) => names1 == names2 && types1 == types2,
      (
        t::Function {
          param_types: p1,
          return_type: r1,
        },
        t::Function {
          param_types: p2,
          return_type: r2,
        },
      ) => p1.len() == p2.len() && p1 == p2 && r1 == r2,
      _ => false,
    }
  }
}

impl Eq for Type {}

impl std::hash::Hash for Type {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Type::Prim(primitive) => primitive.hash(state),
      Type::Struct {
        member_names,
        member_types,
      } => {
        member_names.hash(state);
        member_types.hash(state);
      }
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types.hash(state);
        return_type.hash(state);
      }
      Type::Type => "type".hash(state),
      Type::Ambiguous => panic!("Tried to hash ambiguous type"),
      Type::Reference(t) => {
        "ref".hash(state);
        t.hash(state);
      }
    }
  }
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "?"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Struct {
        member_names,
        member_types,
      } => {
        let fields = member_names
          .into_iter()
          .zip(member_types.into_iter())
          .map(|(name, type_)| format!("{name}: {type_}"))
          .collect::<Vec<_>>()
          .join(", ");
        write!(f, "struct {{ {fields} }}")
      }
      Type::Type => write!(f, "type"),
      Type::Function {
        param_types,
        return_type,
      } => write!(
        f,
        "({}) -> {}",
        param_types
          .iter()
          .map(|t| format!("{t}"))
          .collect::<Vec<_>>()
          .join(", "),
        return_type
      ),
      Type::Reference(t) => write!(f, "{t}&"),
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
    let puny = punycode::encode(&p).unwrap();
    let bytes = format!("{}{puny}", puny.len());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}

/// Builtin mangle syntax:
/// "$" {ident}
pub fn mangle_builtin(name: impl std::fmt::Display) -> Mangle {
  format!("{name}")
}
