use sx::SXRepr;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, sx::SXRepr)]
pub struct TypeConstraint(pub Type, pub Type);

#[derive(Debug, Clone, PartialEq, Eq, sx::SXRepr)]
pub struct StructConstraint {
    pub of_t: Type,
    pub field_t: Type,
    pub name: String,
    pub span: Span,
}

impl Visit<Type> for TypeConstraint {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        self.0._visit(f);
        self.1._visit(f);
    }
}

impl Visit<Type> for StructConstraint {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        self.of_t._visit(f);
        self.field_t._visit(f);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sx::SXRepr)]
pub struct Solution(TypeVariable, Type);

fn unify(t: &mut impl Visit<Type>, Solution(tv, new_t): &Solution) {
    t.visit(|t| {
        if let Type::Variable(old_tv) = t
            && old_tv == tv
        {
            *t = new_t.clone();
        }
    })
}

pub fn unify_all(t: &mut impl Visit<Type>, solution: &[Solution]) {
    for s in solution {
        unify(t, s);
    }
}

impl Environment {
    pub fn solve_constraints(self) -> Vec<Solution> {
        let mut constraints = self.constraints;
        let mut solution: Vec<Solution> = vec![];
        while let Some(TypeConstraint(a, b)) = constraints.pop() {
            match (a, b) {
                (Type::Variable(t1), Type::Variable(t2)) if t1 != t2 => {
                    let new_solution = Solution(t1, Type::Variable(t2));
                    unify(&mut constraints, &new_solution);
                    solution.push(new_solution);
                }
                (t1, t2) if t1 == t2 => {}
                (Type::Variable(tv), t) | (t, Type::Variable(tv))
                    if !t.contains_type_variable(tv) =>
                {
                    let new_solution = Solution(tv, t);
                    unify(&mut constraints, &new_solution);
                    solution.push(new_solution);
                }
                (Type::Function(a1, b1), Type::Function(a2, b2)) => {
                    constraints.push(TypeConstraint(*a1, *a2));
                    constraints.push(TypeConstraint(*b1, *b2));
                }
                (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => {
                    for (t1, t2) in p1.into_iter().zip(p2) {
                        constraints.push(TypeConstraint(t1, t2));
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
                        constraints.push(TypeConstraint(t1, t2));
                    }
                }
                (Type::Instantiation(_, types1), Type::Instantiation(_, types2))
                    if types1.len() == types2.len() =>
                {
                    for (t1, t2) in types1.into_iter().zip(types2) {
                        constraints.push(TypeConstraint(t1, t2));
                    }
                }
                (Type::Instantiation(path, types), t2) | (t2, Type::Instantiation(path, types)) => {
                    let t1 = Universe::get().get_named_type(&path).instantiate(&types);
                    constraints.push(TypeConstraint(t1, t2));
                }
                c => panic!("{:#?}", c),
            }
        }
        solution
    }
}
