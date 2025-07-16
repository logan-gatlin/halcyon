use std::collections::HashMap;

use crate::hlir::{Mangle, Type, mangle_builtin};

#[derive(Debug, Clone, Copy)]
pub enum Builtin {
  Println,
}

impl std::fmt::Display for Builtin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl Builtin {
  pub const ALL: [Self; 1] = [Self::Println];

  pub fn name_to_mangle() -> HashMap<String, Mangle> {
    Self::ALL
      .into_iter()
      .map(|bt| (format!("{bt}"), mangle_builtin(bt)))
      .collect()
  }

  pub fn get_type(&self) -> Type {
    match self {
      Self::Println => Type::func(Type::String, Type::Unit),
    }
  }

  pub fn mangle_to_type() -> HashMap<Mangle, Type> {
    Self::ALL
      .into_iter()
      .map(|bt| (mangle_builtin(bt), bt.get_type()))
      .collect()
  }
}
