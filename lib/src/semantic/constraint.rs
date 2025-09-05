use super::*;

#[derive(Debug, Clone, PartialEq, Eq, sx::SXRepr)]
pub struct TypeConstraint(pub Type, pub Type, pub Span);

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
    pub fn solve_constraints(self) -> Result<Vec<Solution>> {
        let mut constraints = self.constraints;
        let mut struct_constraints = self.struct_constraints;
        let mut solution: Vec<Solution> = vec![];
        // Solve type constraints
        while let Some(TypeConstraint(a, b, span)) = constraints.pop() {
            match (a, b) {
                (Type::Variable(t1), Type::Variable(t2)) if t1 != t2 => {
                    let new_solution = Solution(t1, Type::Variable(t2));
                    unify(&mut constraints, &new_solution);
                    unify(&mut struct_constraints, &new_solution);
                    solution.push(new_solution);
                }
                (t1, t2) if t1 == t2 => {}
                (Type::Variable(tv), t) | (t, Type::Variable(tv))
                    if !t.contains_type_variable(tv) =>
                {
                    let new_solution = Solution(tv, t);
                    unify(&mut constraints, &new_solution);
                    unify(&mut struct_constraints, &new_solution);
                    solution.push(new_solution);
                }
                (Type::Function(a1, b1), Type::Function(a2, b2)) => {
                    constraints.push(TypeConstraint(*a1, *a2, span));
                    constraints.push(TypeConstraint(*b1, *b2, span));
                }
                (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => {
                    for (t1, t2) in p1.into_iter().zip(p2) {
                        constraints.push(TypeConstraint(t1, t2, span));
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
                        constraints.push(TypeConstraint(t1, t2, span));
                    }
                }
                (
                    Type::Struct {
                        name: name1,
                        member_types: mt1,
                        ..
                    },
                    Type::Struct {
                        name: name2,
                        member_types: mt2,
                        ..
                    },
                ) if name1 == name2 => {
                    for (t1, t2) in mt1.into_iter().zip(mt2) {
                        constraints.push(TypeConstraint(t1, t2, span));
                    }
                }
                (Type::Instantiation(_, types1), Type::Instantiation(_, types2))
                    if types1.len() == types2.len() =>
                {
                    for (t1, t2) in types1.into_iter().zip(types2) {
                        constraints.push(TypeConstraint(t1, t2, span));
                    }
                }
                (Type::Instantiation(path, types), t2) | (t2, Type::Instantiation(path, types)) => {
                    let t1 = Universe::get()
                        .get_named_type(&path)
                        .instantiate(&types)
                        .span(span)?;
                    constraints.push(TypeConstraint(t1, t2, span));
                }
                (t1, t2) => {
                    return Err(lint_nospan(TypeLint::TypeMismatch))
                        .context(format!("{t1}"))
                        .context(format!("{t2}"))
                        .span(span);
                }
            }
        }
        // Solve struct constraints
        let mut map: HashMap<Type, (HashSet<(String, Type)>, Span)> = HashMap::new();
        for StructConstraint {
            of_t,
            field_t,
            name,
            span,
        } in struct_constraints
        {
            if let Some((set, _)) = map.get_mut(&of_t) {
                set.insert((name, field_t));
            } else {
                let mut set = HashSet::new();
                set.insert((name, field_t));
                map.insert(of_t, (set, span));
            }
        }
        // TODO this does not really work with polymorphic structs
        for (type_, (fieldset, span)) in map {
            println!("FIELDS: {type_} . {fieldset:#?}");
            let not_exist = lint(TypeLint::NonExistantField, span, [format!("{type_}")]);
            match &type_ {
                Type::Variable(tv) => {
                    let e = lint(
                        TypeLint::NoStructWithFields,
                        span,
                        [fieldset
                            .clone()
                            .into_iter()
                            .map(|(field, _)| field)
                            .collect::<Vec<_>>()
                            .join(", ")],
                    );
                    let possibilities = Universe::get().find_struct_with_names(
                        &fieldset.clone().into_iter().map(|(name, _)| name).collect(),
                    );
                    if possibilities.len() != 1 {
                        return Err(e);
                    }
                    let mut current_tv = self.current_tv;
                    let struct_t = possibilities
                        .first()
                        .unwrap()
                        .clone()
                        .instantiate_with(|| {
                            current_tv += 1;
                            current_tv
                        })
                        .span(span)?;
                    solution.push(Solution(*tv, struct_t.clone()));
                    for (name, type_) in fieldset {
                        let field_t = struct_t.field_type(&name).unwrap();
                        if let Type::Variable(tv) = type_ {
                            solution.push(Solution(tv, field_t))
                        } else if type_ != field_t {
                            return Err(lint(
                                TypeLint::TypeMismatch,
                                span,
                                [format!("{type_}"), format!("{field_t}")],
                            ));
                        }
                    }
                }
                Type::Struct { member_names, .. } => {
                    for (name, _type_) in fieldset {
                        if !member_names.contains(&name) {
                            return Err(not_exist).context(name.clone());
                        }
                    }
                }
                _ if let Some((name, _type_)) = fieldset.iter().next() => {
                    return Err(not_exist).context(name.clone());
                }
                _ => unreachable!("Struct constraint with no fields"),
            }
        }
        Ok(solution)
    }
}
