use super::*;

pub fn unification(constraints: &[TypeConstraint]) -> Result<Vec<Substitution>> {
    let mut cons = constraints.to_vec();
    cons.reverse();
    let mut solution = vec![];
    while let Some(con) = cons.pop() {
        let span = con.2;
        // This is gross
        let t1 = (*con.0.borrow()).clone();
        let t2 = (*con.1.borrow()).clone();
        match (t1, t2) {
            (t1, t2) if t1 == t2 => {}
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                cons.push(TypeConstraint(p1, p2, span));
                cons.push(TypeConstraint(r1, r2, span));
            }
            (t, Type::TypeVariable(tv)) | (Type::TypeVariable(tv), t)
                if !t.contains_type_var(tv) =>
            {
                cons.iter_mut().for_each(|TypeConstraint(t1, t2, _)| {
                    t1.borrow_mut().unify(tv, &t);
                    t2.borrow_mut().unify(tv, &t);
                });
                solution.push(Substitution(tv, t.into()));
            }
            (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => cons
                .extend_from_slice(
                    &p1.into_iter()
                        .zip(p2.into_iter())
                        .map(|(t1, t2)| TypeConstraint(t1, t2, span))
                        .collect::<Vec<_>>(),
                ),
            (
                Type::Struct {
                    member_types: t1, ..
                },
                Type::Struct {
                    member_types: t2, ..
                },
            ) if t1.len() == t2.len() => cons.extend_from_slice(
                &t1.into_iter()
                    .zip(t2.into_iter())
                    .map(|(t1, t2)| TypeConstraint(t1, t2, span))
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

pub fn apply_solution(nodes: &mut IrModule, mut node_ptr: IrPtr, solution: Vec<Substitution>) {
    let mut visited = HashSet::new();
    let mut to_visit = vec![];
    'outer: loop {
        visited.insert(node_ptr);
        let node = &mut nodes[node_ptr];
        match node.kind.clone() {
            IrKind::Declaration { value, in_, .. } => {
                to_visit.push(value);
                if let Some(in_) = in_ {
                    to_visit.push(in_);
                }
            }
            IrKind::ImportedSymbol(..) | IrKind::Immediate(_) | IrKind::Identifier(_) => {}
            IrKind::Tuple(items) => to_visit.extend_from_slice(&items),
            IrKind::StructLiteral { field_values, .. } => {
                to_visit.extend_from_slice(&field_values);
            }
            IrKind::Field { of, .. } => to_visit.push(of),
            IrKind::Binary { left, right, .. } => {
                to_visit.push(left);
                to_visit.push(right);
            }
            IrKind::Unary { child, .. } => {
                to_visit.push(child);
            }
            IrKind::FunctionDef { body, .. } => {
                to_visit.push(body);
            }
            IrKind::FunctionCall {
                callee,
                argument: arguments,
                ..
            } => {
                to_visit.push(callee);
                to_visit.push(arguments);
            }
            IrKind::If {
                predicate,
                then,
                else_,
            } => {
                to_visit.push(predicate);
                to_visit.push(then);
                if let Some(else_) = else_ {
                    to_visit.push(else_);
                }
            }
            IrKind::Match {
                scrutinee,
                branches,
                ..
            } => {
                to_visit.push(scrutinee);
                to_visit.extend_from_slice(&branches);
            }
        }
        while let Some(next) = to_visit.pop() {
            if !visited.contains(&next) {
                node_ptr = next;
                continue 'outer;
            }
        }
        break;
    }
    visited
        .into_iter()
        .for_each(|n| nodes[n].unify_all(&solution));
}
