use std::{cell::OnceCell, collections::HashMap};

use crate::ir::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Builtin {
  Assert,
}

impl std::fmt::Display for Builtin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Builtin::Assert => "assert",
      }
    )
  }
}

impl Builtin {
  pub const ALL: [Self; 1] = [Self::Assert];
  const MANGLE_MAP: OnceCell<HashMap<String, Self>> = OnceCell::new();

  pub fn from_mangle(mangle: &String) -> Option<Self> {
    Self::MANGLE_MAP
      .get_or_init(|| {
        Self::ALL
          .into_iter()
          .map(|bt| (bt.get_mangle(), bt))
          .collect()
      })
      .get(mangle)
      .cloned()
  }

  pub fn get_mangle(&self) -> String {
    mangle_builtin(self)
  }

  pub fn get_type(&self) -> TypeRef {
    match self {
      Self::Assert => Type::func(Type::Boolean.to_ref(), Type::Unit.to_ref()),
    }
  }
}
