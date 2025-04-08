use std::collections::HashSet;

use crate::{hlir::*, operator::*, span::*};

#[derive(Debug, Clone)]
pub enum TypeConstraint {
  // For direct type inference
  Equals(Type, Type),
  UnaryResult(Type, UnaryOp, Type),
}

impl PartialEq for TypeConstraint {
  fn eq(&self, other: &Self) -> bool {
    use TypeConstraint as t;
    match (self, other) {
      (t::Equals(t1, t2), t::Equals(t3, t4)) => t1 == t3 && t2 == t4,
      (t::UnaryResult(p1, o1, c1), t::UnaryResult(p2, o2, c2)) => {
        p1 == p2 && o1 == o2 && c1 == c2
      },
      _ => false,
    }
  }
}

impl TypeConstraint {
  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    match self {
      TypeConstraint::Equals(t1, t2) => {
        t1.contains_type_var(tv) || t2.contains_type_var(tv)
      },
      TypeConstraint::UnaryResult(t1, unary_op, t2) => [t1, t2]
        .into_iter()
        .fold(false, |contains, t| contains || t.contains_type_var(tv)),
    }
  }

  pub fn substitute(&mut self, tv: TypeVariable, type_: Type) {
    match self {
      TypeConstraint::Equals(t1, t2) => [t1, t2]
        .into_iter()
        .for_each(|t| t.substitute(tv, type_.clone())),
      TypeConstraint::UnaryResult(prod, unary_op, t) => {
        [prod, t]
          .into_iter()
          .for_each(|t| t.substitute(tv, type_.clone()));
      },
    }
  }
}

impl Type {
  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    match self {
      Type::Polymorphic(t) => tv == *t,
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
      Type::Ambiguous => false,
      Type::Primitive(primitive) => false,
    }
  }

  pub fn substitute(&mut self, tv: TypeVariable, type_: Type) {
    match self {
      Type::Ambiguous => {},
      Type::Polymorphic(t) => {
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
    }
  }
}
