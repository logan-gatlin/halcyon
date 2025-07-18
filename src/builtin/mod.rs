use std::collections::HashMap;

use crate::hlir::{Mangle, Type, mangle_builtin};

#[derive(Debug, Clone, Copy)]
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

  pub fn get_mangle(&self) -> String {
    mangle_builtin(self)
  }

  pub fn get_type(&self) -> Type {
    match self {
      Self::Assert => Type::func(Type::Boolean, Type::Unit),
    }
  }
}
