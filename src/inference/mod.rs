mod constraints;

use std::collections::{HashMap, HashSet};

use crate::{hlir::*, operator::*, span::*};
pub use constraints::*;

#[derive(Debug, Clone)]
pub struct TypeConstraint(pub Type, pub Type);

impl PartialEq for TypeConstraint {
  fn eq(&self, other: &Self) -> bool {
    self.0 == self.1
  }
}

impl TypeConstraint {
  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    self.0.contains_type_var(tv) || self.1.contains_type_var(tv)
  }

  pub fn substitute(&mut self, tv: TypeVariable, type_: Type) {
    self.0.substitute(tv, type_.clone());
    self.1.substitute(tv, type_);
  }
}

impl std::fmt::Display for TypeConstraint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} = {}", self.0, self.1)
  }
}

impl Type {
  fn contains_type_var(&self, tv: TypeVariable) -> bool {
    match self {
      Type::Undetermined(t) => tv == *t,
      Type::Struct {
        member_names,
        member_types,
      } => member_types
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Tuple(items) => items
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Variant(hash_set) => hash_set
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .into_iter()
          .fold(false, |accum, x| accum || x.contains_type_var(tv))
          || return_type.contains_type_var(tv)
      },
      Type::Reference(t) => t.contains_type_var(tv),
      Type::Type => false,
      Type::Dependent(_) => false,
      Type::Ambiguous => false,
      Type::Primitive(primitive) => false,
    }
  }

  fn substitute(&mut self, tv: TypeVariable, type_: Type) {
    match self {
      Type::Ambiguous => {},
      Type::Undetermined(t) => {
        if *t == tv {
          *self = type_;
        }
      },
      Type::Primitive(primitive) => {},
      Type::Struct {
        member_names,
        member_types,
      } => {
        member_types
          .iter_mut()
          .for_each(|t| t.substitute(tv, type_.clone()));
      },
      Type::Tuple(items) => items
        .iter_mut()
        .for_each(|i| i.substitute(tv, type_.clone())),
      Type::Variant(hash_set) => {
        *self = Type::Variant(
          hash_set
            .clone()
            .into_iter()
            .map(|mut t| {
              t.substitute(tv, type_.clone());
              t
            })
            .collect::<HashSet<_>>(),
        );
      },
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .iter_mut()
          .for_each(|t| t.substitute(tv, type_.clone()));
        return_type.substitute(tv, type_);
      },
      Type::Reference(r) => r.substitute(tv, type_),
      Type::Type => {},
      Type::Dependent(_) => {},
    }
  }
}
