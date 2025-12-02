use crate::{
    IntoLog,
    WithContext,
};

use super::*;

#[derive(Debug, Clone, derive_new::new)]
pub struct EqualityConstraint {
    pub left: Type,
    pub right: Type,
    pub span: Span,
}

#[derive(Debug, Clone, derive_new::new)]
pub struct StructConstraint {
    pub base: Type,
    pub field: Typed<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    pub equality: Vec<EqualityConstraint>,
    pub struct_: Vec<StructConstraint>,
}

#[derive(Debug, Clone, derive_new::new)]
pub struct Solution {
    pub old: TypeVariable,
    pub new: Type,
}

pub(super) fn solve_constraints(
    env: &mut Environment,
    logger: &mut Logger,
) -> Vec<Solution> {
    let eq_cons = &mut env.constraints.equality;
    let struct_cons = &mut env.constraints.struct_;
    let mut solutions = vec![];
    while let Some(EqualityConstraint { left, right, span }) = eq_cons.pop() {
        match (left, right) {
            (Type::Variable(t1), Type::Variable(t2)) if t1 != t2 => {
                let t2 = Type::Variable(t2);
                let new_solution = Solution::new(t1, t2.clone());
                let t2 = [t2];
                substitute_type_variables(eq_cons, &[t1], &t2);
                substitute_type_variables(struct_cons, &[t1], &t2);
                solutions.push(new_solution);
            }
            (t1, t2) if t1 == t2 => {}
            (Type::Variable(tv), t) | (t, Type::Variable(tv)) if !t.contains_type_variable(tv) => {
                let new_solution = Solution::new(tv, t.clone());
                let t = [t];
                substitute_type_variables(eq_cons, &[tv], &t);
                substitute_type_variables(struct_cons, &[tv], &t);
                solutions.push(new_solution);
            }
            (Type::Function(a1, b1), Type::Function(a2, b2)) => {
                eq_cons.push(EqualityConstraint::new(*a1, *a2, span));
                eq_cons.push(EqualityConstraint::new(*b1, *b2, span));
            }
            (Type::Array(t1), Type::Array(t2)) => {
                eq_cons.push(EqualityConstraint::new(*t1, *t2, span));
            }
            (Type::Tuple(p1), Type::Tuple(p2)) if p1.len() == p2.len() => {
                for (t1, t2) in p1.into_iter().zip(p2) {
                    eq_cons.push(EqualityConstraint::new(t1, t2, span));
                }
            }
            (
                Type::Sum {
                    variant_names: names1,
                    variant_types: types1,
                    ..
                },
                Type::Sum {
                    variant_names: names2,
                    variant_types: types2,
                    ..
                },
            ) if names1 == names2 => {
                for (t1, t2) in types1.into_iter().zip(types2) {
                    eq_cons.push(EqualityConstraint::new(t1, t2, span));
                }
            }
            (
                Type::Struct {
                    name: name1,
                    fields: f1,
                },
                Type::Struct {
                    name: name2,
                    fields: f2,
                },
            ) if name1 == name2 => {
                for (t1, t2) in f1.into_values().zip(f2.into_values()) {
                    eq_cons.push(EqualityConstraint::new(t1, t2, span));
                }
            }
            (Type::Instantiation(_, types1), Type::Instantiation(_, types2))
                if types1.len() == types2.len() =>
            {
                for (t1, t2) in types1.into_iter().zip(types2) {
                    eq_cons.push(EqualityConstraint::new(t1, t2, span));
                }
            }
            (Type::Instantiation(path, types), t2) | (t2, Type::Instantiation(path, types)) => {
                if let Some(t1) = env
                    .symbols
                    .get_type(&path)
                    .clone()
                    .instantiate(&types)
                    .into_log(logger, span)
                {
                    eq_cons.push(EqualityConstraint::new(t1, t2, span));
                }
            }
            (t1, t2) => {
                logger
                    .error("Type error")
                    .primary("This expression is not well typed", span)
                    .note(format!("Impossible constraint: {t1} = {t2}"))
                    .done();
            }
        }
    }
    solutions
}

impl Visit<Type> for EqualityConstraint {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        self.left._visit(f);
        self.right._visit(f);
    }
}

impl Visit<Type> for StructConstraint {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        self.base._visit(f);
        self.field.type_.visit(f);
    }
}

impl Visit<Type> for ConstraintSet {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        self.equality._visit(f);
        self.struct_._visit(f);
    }
}
