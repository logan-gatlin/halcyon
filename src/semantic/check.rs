use super::*;

pub fn check_structs(
  env: &Environment,
  ir: &IrModule,
  ptr: IrPtr,
) -> Result<Vec<TypeConstraint>> {
  let range = ir.ir_range(ptr);
  let structs = env
    .type_map
    .iter()
    .flat_map(|(mangle, t)| {
      if let Type::Struct {
        member_names,
        member_types,
      } = &*t.borrow()
      {
        Some((mangle, member_names.clone(), member_types.clone()))
      } else {
        None
      }
    })
    .collect::<Vec<_>>();
  let mut constraints = vec![];

  for node in &ir.nodes[range] {
    if let IrKind::Field { of, index } = &node.kind {
      let node_type = node.type_.clone();
      let of_type = ir[*of].type_.borrow().clone();
      // LHS inferred to be struct
      if let Type::Struct {
        member_names,
        member_types,
      } = of_type
      {
        // Field name is correct
        if let Some(index) = member_names.iter().position(|n| n == index) {
          // Field type is too broad
          if member_types[index] < node_type {
            constraints.push(TypeConstraint(
              member_types[index].clone(),
              node_type,
              node.span,
            ));
          }
          // Field type is incorrect
          else if member_types[index] != node_type {
            return Err(lint(
              TypeLint::TypeMismatch,
              node.span,
              [
                format!("{}", member_types[index].borrow()),
                format!("{}", node_type.borrow()),
              ],
            ));
          }
          // Otherwise, field and type are both correct
        }
        // Field name is incorrect
        else {
          return Err(lint(
            TypeLint::NonExistantField,
            node.span,
            [format!("{index}"), format!("{}", node_type.borrow())],
          ));
        }
      }
      // LHS failed to infer, try to use context clues
      else if let Type::TypeVariable(tv) = of_type {
        match &structs
          .iter()
          // Filter to only candidate types
          .filter(|(_, names, _)| {
            names.contains(index)
          })
          .collect::<Vec<_>>()[..]
        {
          // No candidates found
          [] => {
            return Err(lint(
              TypeLint::NonExistantField,
              node.span,
              [format!("{index}"), format!("{}", node_type.borrow())],
            ));
          },
          // Exactly one candidate found
          [(mangle, names, types)] => {
            let new_this_t =
              types[names.iter().position(|i| i == index).unwrap()].clone();
            let new_of_t = env.get_type(mangle);
            constraints.extend_from_slice(&[
              TypeConstraint(new_this_t, node_type, node.span),
              TypeConstraint(
                new_of_t,
                Type::TypeVariable(tv).to_ref(),
                node.span,
              ),
            ]);
          },
          // Multiple candidates found
          [..] => {
            return Err(lint(TypeLint::AmbiguousExpression, node.span, []));
          },
        };
      }
    }
  }
  Ok(constraints)
}
