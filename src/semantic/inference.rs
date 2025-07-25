use super::*;

pub fn type_inference(
  nodes: &mut IrModule,
  ptr: IrPtr,
  environment: &mut Environment,
  constraints: &mut Vec<TypeConstraint>,
) -> Result<TypeRef> {
  use IrKind as h;
  use TypeConstraint as tc;
  let span = nodes[ptr].span;
  let type_ = match nodes[ptr].kind.clone() {
    h::ImportedSymbol(name, type_) => {
      environment.insert_value_type(name.clone(), type_.clone(), true);
      environment.get_value_type(&name)
    },
    // Let declarations
    h::Declaration {
      assignee,
      value,
      in_,
    } => {
      let mut new_constraints = constraints.clone();
      type_inference(nodes, value, environment, &mut new_constraints)?;
      let solution = unification(&new_constraints)?;
      apply_solution(nodes, ptr, solution);
      let second_solution =
        unification(&check_structs(environment, nodes, ptr)?)?;
      apply_solution(nodes, ptr, second_solution);
      let new_t = nodes[value].type_.clone();
      environment.insert_value_type(assignee, new_t, true);

      if let Some(in_) = in_ {
        type_inference(nodes, in_, environment, constraints)?
      } else {
        Type::Unit.to_ref()
      }
    },
    h::Immediate(c) => match c {
      ConstValue::Unit => Type::Unit,
      ConstValue::Integer(_) => Type::Integer,
      ConstValue::Real(_) => Type::Real,
      ConstValue::Boolean(_) => Type::Boolean,
      ConstValue::String { .. } => Type::String,
      ConstValue::Glyph(_) => Type::Glyph,
    }
    .to_ref(),
    h::Identifier(i) => environment.get_value_type(&i).clone(),
    h::Tuple(items) => Type::Product(
      items
        .into_iter()
        .map(|i| type_inference(nodes, i, environment, constraints))
        .try_collect()?,
    )
    .to_ref(),
    h::StructLiteral {
      field_names,
      field_values,
      ..
    } => Type::Struct {
      member_names: field_names,
      member_types: field_values
        .into_iter()
        .map(|v| type_inference(nodes, v, environment, constraints))
        .try_collect()?,
    }
    .to_ref(),
    h::Field { of, .. } => {
      type_inference(nodes, of, environment, constraints)?;
      environment.fresh_type_var()
    },
    h::Binary { op, left, right } => {
      let left_t = type_inference(nodes, left, environment, constraints)?;
      let right_t = type_inference(nodes, right, environment, constraints)?;
      use BinaryOp::*;
      match op {
        Semicolon => right_t,
        Star | Slash | Percent | Plus | Minus => {
          constraints.extend_from_slice(&[
            tc(left_t.clone(), right_t.clone(), span),
            tc(Type::Integer.to_ref(), left_t, nodes[left].span),
            tc(Type::Integer.to_ref(), right_t, nodes[right].span),
          ]);
          Type::Integer.to_ref()
        },
        StarDot | SlashDot | PlusDot | MinusDot => {
          constraints.extend_from_slice(&[
            tc(left_t.clone(), right_t.clone(), span),
            tc(Type::Real.to_ref(), left_t, nodes[left].span),
            tc(Type::Real.to_ref(), right_t, nodes[right].span),
          ]);
          Type::Real.to_ref()
        },
        And | Or | Xor => {
          constraints.extend_from_slice(&[
            tc(left_t.clone(), right_t.clone(), span),
            tc(Type::Boolean.to_ref(), left_t, nodes[left].span),
            tc(Type::Boolean.to_ref(), right_t, nodes[right].span),
          ]);
          Type::Boolean.to_ref()
        },
        DoubleEqual | BangEqual | Less | LessEqual | Greater | GreaterEqual => {
          constraints.push(tc(left_t.clone(), right_t.clone(), span));
          constraints.push(tc(left_t, right_t, nodes[right].span));
          Type::Boolean.to_ref()
        },
        _ => todo!(),
      }
    },
    h::Unary { op, child } => {
      let child_t = type_inference(nodes, child, environment, constraints)?;
      use UnaryOp::*;
      let expect_t = match op {
        Not => Type::Boolean,
        MinusDot => Type::Real,
        Minus => Type::Integer,
      }
      .to_ref();
      constraints.push(tc(expect_t.clone(), child_t, nodes[child].span));
      expect_t
    },
    h::FunctionDef {
      parameter_name,
      parameter_type,
      body,
      ..
    } => {
      let parameter_type = match (&parameter_name, parameter_type) {
        (Some(_), Some(type_)) => type_,
        (None, None) => Type::Unit.to_ref(),
        (Some(_), None) => environment.fresh_type_var(),
        (None, Some(_)) => panic!(),
      };
      environment.insert_value_type(
        parameter_name.unwrap_or("()".into()),
        parameter_type.clone(),
        false,
      );
      let return_type = type_inference(nodes, body, environment, constraints)?;
      Type::func(parameter_type, return_type)
    },
    h::RecursiveDeclaration {
      assignee,
      parameter_name,
      parameter_type,
      body,
      in_,
      function_type,
      ..
    } => {
      let mut new_constraints = constraints.clone();
      let recursive_type_var = environment.fresh_type_var();
      environment.insert_value_type(
        assignee.clone(),
        recursive_type_var.clone(),
        false,
      );
      // <Copied from FunctionDef>
      let parameter_type = match (&parameter_name, parameter_type) {
        (Some(_), Some(type_)) => type_,
        (None, None) => Type::Unit.to_ref(),
        (Some(_), None) => environment.fresh_type_var(),
        (None, Some(_)) => panic!(),
      };
      environment.insert_value_type(
        parameter_name.unwrap_or("()".into()),
        parameter_type.clone(),
        false,
      );
      let return_type =
        type_inference(nodes, body, environment, &mut new_constraints)?;
      let inferred_type = Type::func(parameter_type, return_type);
      // </Copied from FunctionDef>
      let solution = unification(&new_constraints)?;
      solution.iter().for_each(|Substitution(tv, type_)| {
        inferred_type.borrow_mut().substitute(*tv, &type_.borrow());
      });
      environment.insert_value_type(assignee, inferred_type.clone(), true);
      *function_type.borrow_mut() = (*inferred_type).borrow().clone();
      apply_solution(nodes, ptr, solution);
      if let Some(in_) = in_ {
        type_inference(nodes, in_, environment, constraints)?
      } else {
        Type::Unit.to_ref()
      }
    },
    h::FunctionCall { callee, argument } => {
      let tv = environment.fresh_type_var();
      let callee_t = type_inference(nodes, callee, environment, constraints)?;
      let arg_t = type_inference(nodes, argument, environment, constraints)?;
      constraints.push(tc(
        callee_t,
        Type::func(arg_t, tv.clone()),
        nodes[argument].span,
      ));
      tv
    },
    h::If {
      predicate,
      then,
      else_,
    } => {
      let tv = environment.fresh_type_var();
      let pred_t = type_inference(nodes, predicate, environment, constraints)?;
      let then_t = type_inference(nodes, then, environment, constraints)?;
      let else_t = if let Some(else_) = else_ {
        type_inference(nodes, else_, environment, constraints)?
      } else {
        Type::Unit.to_ref()
      };
      constraints.extend_from_slice(&[
        tc(pred_t, Type::Boolean.to_ref(), nodes[predicate].span),
        tc(then_t, tv.clone(), nodes[then].span),
        tc(
          else_t,
          tv.clone(),
          if let Some(else_) = else_ {
            nodes[else_].span
          } else {
            nodes[then].span
          },
        ),
      ]);
      tv
    },
  };
  if let h::FunctionDef {
    captures,
    capture_types,
    ..
  } = &mut nodes[ptr].kind
  {
    captures
      .into_iter()
      .zip(capture_types.into_iter())
      .for_each(|(cap, ty)| {
        *ty = environment.get_value_type(cap);
      });
  }
  nodes[ptr].type_ = type_.clone();
  Ok(type_)
}
