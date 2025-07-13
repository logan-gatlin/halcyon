use super::*;

pub fn unification(
  constraints: &[TypeConstraint],
) -> Result<Vec<Substitution>> {
  let mut cons = constraints.to_vec();
  let mut solution = vec![];
  while let Some(con) = cons.pop() {
    if con.0.ambiguous() || con.1.ambiguous() {
      continue;
    }
    let span = con.2;
    match (con.0, con.1) {
      (t1, t2) if t1 == t2 => {},
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
          panic!();
        }
        p1.into_iter()
          .zip(p2.into_iter())
          .for_each(|(t1, t2)| cons.push(TypeConstraint(t1, t2, span)));
        cons.push(TypeConstraint(*r1, *r2, span));
      },
      (Type::TypeVariable(tv), t) | (t, Type::TypeVariable(tv))
        if !t.contains_type_var(tv) =>
      {
        cons.iter_mut().for_each(|TypeConstraint(t1, t2, _)| {
          t1.substitute(tv, &t);
          t2.substitute(tv, &t);
        });
        solution.push(Substitution(tv, t));
      },
      (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => cons
        .extend_from_slice(
          &p1
            .into_iter()
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
        &t1
          .into_iter()
          .zip(t2.into_iter())
          .map(|(t1, t2)| TypeConstraint(t1, t2, span))
          .collect::<Vec<_>>(),
      ),
      (t1, t2) => {
        return Err(lint_nospan(TypeLint::TypeMismatch))
          .context(format!("{t1}"))
          .context(format!("{t2}"))
          .span(span);
      },
    }
  }
  Ok(solution)
}

pub fn apply_solution(
  nodes: &mut HlIrModule,
  mut node_ptr: IrPtr,
  solution: Vec<Substitution>,
) {
  let mut visited = HashSet::new();
  let mut to_visit = vec![];
  'outer: loop {
    visited.insert(node_ptr);
    let node = &mut nodes[node_ptr];
    match node.kind.clone() {
      HlIrKind::Declaration { value, in_, .. } => {
        to_visit.push(value);
        if let Some(in_) = in_ {
          to_visit.push(in_);
        }
      },
      HlIrKind::Immediate(_) => {},
      HlIrKind::Block(items) => to_visit.extend_from_slice(&items),
      HlIrKind::Identifier(_) => {},
      HlIrKind::Tuple(items) => to_visit.extend_from_slice(&items),
      HlIrKind::StructDef { field_types, .. } => {
        to_visit.extend_from_slice(&field_types)
      },
      HlIrKind::StructLiteral { field_values, .. } => {
        to_visit.extend_from_slice(&field_values);
      },
      HlIrKind::Field { of, .. } => to_visit.push(of),
      HlIrKind::Binary { left, right, .. } => {
        to_visit.push(left);
        to_visit.push(right);
      },
      HlIrKind::Unary { child, .. } => {
        to_visit.push(child);
      },
      HlIrKind::FunctionDef { body, .. } => {
        to_visit.push(body);
      },
      HlIrKind::FunctionCall {
        callee, arguments, ..
      } => {
        to_visit.push(callee);
        to_visit.extend_from_slice(&arguments);
      },
      HlIrKind::If {
        predicate,
        then,
        else_,
      } => {
        to_visit.push(predicate);
        to_visit.push(then);
        if let Some(else_) = else_ {
          to_visit.push(else_);
        }
      },
    }
    while let Some(next) = to_visit.pop() {
      if !visited.contains(&next) {
        node_ptr = next;
        continue 'outer;
      }
    }
    break;
  }
  visited.into_iter().for_each(|n| {
    let nt = &mut nodes[n].type_;
    solution
      .iter()
      .for_each(|Substitution(tv, t)| nt.substitute(*tv, t));
  });
}
