use super::*;

pub type TypeVariable = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint(pub TypeRef, pub TypeRef, pub Span);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution(pub TypeVariable, pub TypeRef);

pub fn solve_constraints(constraints: &[Constraint]) -> Result<Vec<Substitution>> {
    let mut cons = constraints.to_vec();
    cons.reverse();
    let mut solution = vec![];
    while let Some(con) = cons.pop() {
        let span = con.2;
        let t1 = con.0;
        let t2 = con.1;
        match (t1, t2) {
            (t1, t2) if t1 == t2 => {}
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                cons.push(Constraint(*p1, *p2, span));
                cons.push(Constraint(*r1, *r2, span));
            }
            (t, Type::TypeVariable(tv)) | (Type::TypeVariable(tv), t)
                if !t.contains_type_var(tv) =>
            {
                cons.iter_mut().for_each(|Constraint(t1, t2, _)| {
                    t1.unify(tv, &t);
                    t2.unify(tv, &t);
                });
                solution.push(Substitution(tv, t.into()));
            }
            (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => cons
                .extend_from_slice(
                    &p1.into_iter()
                        .zip(p2.into_iter())
                        .map(|(t1, t2)| Constraint(t1, t2, span))
                        .collect::<Vec<_>>(),
                ),
            (
                Type::Struct {
                    member_names: n1,
                    member_types: t1,
                },
                Type::Struct {
                    member_names: n2,
                    member_types: t2,
                },
            ) if n1 == n2 => cons.extend_from_slice(
                &t1.into_iter()
                    .zip(t2.into_iter())
                    .map(|(t1, t2)| Constraint(t1, t2, span))
                    .collect::<Vec<_>>(),
            ),
            (
                Type::Sum {
                    variant_names: n1,
                    variant_types: t1,
                },
                Type::Sum {
                    variant_names: n2,
                    variant_types: t2,
                },
            ) if n1 == n2 => cons.extend_from_slice(
                &t1.into_iter()
                    .zip(t2.into_iter())
                    .map(|(t1, t2)| Constraint(t1, t2, span))
                    .collect::<Vec<_>>(),
            ),
            (t1, t2) => {
                return Err(lint_nospan(TypeLint::TypeMismatch))
                    .context(format!("{t1}"))
                    .context(format!("{t2}"))
                    .span(span);
            }
        }
    }
    Ok(solution)
}

pub trait Unify {
    fn unify(&mut self, tv: TypeVariable, type_: &Type);
    fn unify_all(&mut self, subs: &[Substitution]) {
        for Substitution(tv, t) in subs {
            self.unify(*tv, t);
        }
    }
}

impl<T> Unify for Vec<T>
where
    T: Unify,
{
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.iter_mut().for_each(|u| u.unify(tv, type_));
    }
}

impl<H, T> Unify for HashMap<H, T>
where
    H: std::hash::Hash,
    T: Unify,
{
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.iter_mut().for_each(|(_, v)| v.unify(tv, type_))
    }
}

impl Unify for Constraint {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.0.unify(tv, type_);
        self.1.unify(tv, type_);
    }
}

impl Unify for Type {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        match self {
            Type::TypeVariable(t) => {
                if *t == tv {
                    *self = type_.clone();
                }
            }
            Type::Struct { member_types, .. } => {
                member_types.iter_mut().for_each(|t| {
                    t.unify(tv, type_);
                });
            }
            Type::Product(items) => items.iter_mut().for_each(|i| {
                i.unify(tv, type_);
            }),
            Type::Sum { variant_types, .. } => {
                variant_types.iter_mut().for_each(|t| t.unify(tv, type_))
            }
            Type::Function(a, b) => {
                a.unify(tv, type_);
                b.unify(tv, type_);
            }
            Type::Named(_)
            | Type::Any
            | Type::_ClosureCapture
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => {}
        }
    }
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} == {}", self.0, self.1)
    }
}

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{} <- {}", self.0, self.1)
    }
}
