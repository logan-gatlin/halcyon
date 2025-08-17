use std::collections::HashSet;

use super::*;

pub type TypeVariable = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintKind {
    Type(Type, Type),
    StructField { of: Type, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub kind: ConstraintKind,
    pub span: Span,
}

fn print_constraints(cons: &[Constraint]) {
    cons.iter().for_each(|c| print!("{c} ;; "));
    println!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution(pub TypeVariable, pub TypeRef);

pub fn solve_constraints(constraints: &[Constraint]) -> Result<Vec<Substitution>> {
    let (type_constraints, struct_constraints): (Vec<_>, Vec<_>) = constraints
        .into_iter()
        .map(|c| match c.kind.clone() {
            ConstraintKind::Type(t1, t2) => (Some((t1, t2, c.span)), None),
            ConstraintKind::StructField { of, name } => (None, Some((of, name, c.span))),
        })
        .unzip();
    let type_constraints = type_constraints.into_iter().flatten().collect::<Vec<_>>();
    let mut struct_constraints = struct_constraints.into_iter().flatten().collect::<Vec<_>>();
    let mut type_solution = solve_type_constraints(&type_constraints)?;
    struct_constraints.iter_mut().for_each(|(of, _, _)| {
        of.unify_all(&type_solution);
    });
    type_solution.extend_from_slice(&solve_struct_constraints(&struct_constraints)?);
    Ok(type_solution)
}

fn solve_struct_constraints(constraints: &[(Type, String, Span)]) -> Result<Vec<Substitution>> {
    let mut solution = vec![];
    let constraints = constraints.to_vec();
    let mut map: HashMap<Type, (HashSet<String>, Span)> = HashMap::new();
    for (of, name, span) in constraints {
        if let Some((set, _)) = map.get_mut(&of) {
            set.insert(name);
        } else {
            let mut set = HashSet::new();
            set.insert(name);
            map.insert(of, (set, span));
        }
    }
    for (type_, (fieldset, span)) in map {
        let not_exist = lint(TypeLint::NonExistantField, span, [format!("{type_}")]);
        match &type_ {
            Type::TypeVariable(tv) => {
                let e = lint(
                    TypeLint::NoStructWithFields,
                    span,
                    [fieldset.clone().into_iter().collect::<Vec<_>>().join(", ")],
                );
                let possibilities = Type::find_structs_with_fields(&fieldset);
                if possibilities.len() != 1 {
                    return Err(e);
                }
                solution.push(Substitution(*tv, possibilities.get(0).unwrap().clone()));
            }
            Type::Struct { member_names, .. } => {
                for name in fieldset {
                    if !member_names.contains(&name) {
                        return Err(not_exist).context(name.clone());
                    }
                }
            }
            _ if let Some(field) = fieldset.iter().next() => {
                return Err(not_exist).context(field.clone());
            }
            _ => unreachable!("Struct constraint with no fields"),
        }
    }
    Ok(solution)
}

fn solve_type_constraints(constraints: &[(Type, Type, Span)]) -> Result<Vec<Substitution>> {
    let mut cons = constraints.to_vec();
    cons.reverse();
    let mut solution = vec![];
    while let Some(con) = cons.pop() {
        let span = con.2;
        let t1 = con.0;
        let t2 = con.1;
        match (t1, t2) {
            (t1, t2) if t1.strict_eq(&t2) => {}
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                cons.push((*p1, *p2, span));
                cons.push((*r1, *r2, span));
            }
            (t, Type::TypeVariable(tv)) | (Type::TypeVariable(tv), t)
                if !t.contains_type_var(tv) =>
            {
                cons.iter_mut().for_each(|(t1, t2, _)| {
                    t1.unify(tv, &t);
                    t2.unify(tv, &t);
                });
                solution.push(Substitution(tv, t));
            }
            (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => cons
                .extend_from_slice(
                    &p1.into_iter()
                        .zip(p2.into_iter())
                        .map(|(t1, t2)| (t1, t2, span))
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
                    .map(|(t1, t2)| (t1, t2, span))
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
                    .map(|(t1, t2)| (t1, t2, span))
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
        match &mut self.kind {
            ConstraintKind::Type(t1, t2) => {
                t1.unify(tv, type_);
                t2.unify(tv, type_);
            }
            ConstraintKind::StructField { of, .. } => {
                of.unify(tv, type_);
            }
        }
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
        match &self.kind {
            ConstraintKind::Type(t1, t2) => write!(f, "{t1} == {t2}"),
            ConstraintKind::StructField { of, name } => write!(f, "({of}.{name})"),
        }
    }
}

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{} <- {}", self.0, self.1)
    }
}
