mod constraints;

use std::collections::HashMap;

use crate::{hlir::*, operator::*, span::*};
pub use constraints::*;

/*
pub fn simplify_constraints(&mut self) {
  while let Some(TypeConstraint(t1, t2)) = self.constraints.pop() {
    match (t1, t2) {
      (t1, t2) if t1 == t2 => {},
      // Function decomposition
      (
        Type::Function {
          param_types: p1,
          return_type: r1,
        },
        Type::Function {
          param_types: p2,
          return_type: r2,
        },
      ) => {
        if p1.len() != p2.len() {
          // Type (arity) error
          panic!("Wrong function arity");
        }
        p1.into_iter()
          .zip(p2.into_iter())
          .for_each(|(p1, p2)| self.constraints.push(TypeConstraint(p1, p2)));
        self.constraints.push(TypeConstraint(*r1, *r2));
      },
      // Polymorphic value
      (Type::Undetermined(tv), t) | (t, Type::Undetermined(tv)) => {
        for node in &mut self.hlir.nodes {
          if let Type::Undetermined(this_tv) = node.type_.clone()
            && tv == this_tv
          {
            node.type_ = t.clone();
          }
        }

        self
          .constraints
          .iter_mut()
          .for_each(|c| c.substitute(tv, t.clone()));
      },
      _ => todo!(),
    }
  }
}
*/
