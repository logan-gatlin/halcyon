use super::*;

pub fn type_inference(
  nodes: &mut HlIrModule,
  ptr: IrPtr,
  environment: &mut Environment,
  constraints: &mut Vec<TypeConstraint>,
) -> Result<Type> {
  use HlIrKind as h;
  use TypeConstraint as tc;
  let span = nodes[ptr].span;
  let type_ = match nodes[ptr].kind.clone() {
    // Type declarations
    h::Declaration {
      assignee,
      is_type: true,
      is_recursive,
      value,
      in_,
    } => {
      if is_recursive {
        panic!("Recursive types are not yet supported");
      }
      let type_ = parse_type(nodes, value, environment)?;
      environment.insert_type(assignee.clone(), Type::Type, false);
      environment.insert_value(assignee, ConstValue::Type(type_));
      if let Some(in_) = in_ {
        type_inference(nodes, in_, environment, constraints)?
      } else {
        Type::Unit
      }
    },
    // Let declarations
    h::Declaration {
      assignee,
      is_type: false,
      is_recursive,
      value,
      in_,
    } => {
      let mut new_constraints = constraints.clone();
      let _old_t = if is_recursive {
        let tv = environment.fresh_type_var();
        // TODO revisit false here
        environment.insert_type(assignee.clone(), tv.clone(), false);
        let t =
          type_inference(nodes, value, environment, &mut new_constraints)?;
        new_constraints.push(tc(t.clone(), tv, nodes[value].span));
        t
      } else {
        type_inference(nodes, value, environment, &mut new_constraints)?
      };
      let solution = unification(&new_constraints)?;
      apply_solution(nodes, ptr, solution);
      let new_t = nodes[value].type_.clone();
      environment.insert_type(assignee, new_t, true);
      // TODO replace all usages of type variable in recursive
      // decl

      if let Some(in_) = in_ {
        type_inference(nodes, in_, environment, constraints)?
      } else {
        Type::Unit
      }
    },
    h::Immediate(c) => match c {
      ConstValue::Unit => Type::Unit,
      ConstValue::Integer(_) => Type::Integer,
      ConstValue::Real(_) => Type::Real,
      ConstValue::Boolean(_) => Type::Boolean,
      ConstValue::String { .. } => Type::String,
      ConstValue::Glyph(_) => Type::Glyph,
      _ => unreachable!(),
    },
    h::Identifier(i) => environment.get_type(&i).clone(),
    h::Tuple(items) => Type::Product(
      items
        .into_iter()
        .map(|i| type_inference(nodes, i, environment, constraints))
        .try_collect()?,
    ),
    h::StructDef { .. } => todo!(),
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
    },
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
            tc(Type::Integer, left_t, nodes[left].span),
            tc(Type::Integer, right_t, nodes[right].span),
          ]);
          Type::Integer
        },
        StarDot | SlashDot | PlusDot | MinusDot => {
          constraints.extend_from_slice(&[
            tc(left_t.clone(), right_t.clone(), span),
            tc(Type::Real, left_t, nodes[left].span),
            tc(Type::Real, right_t, nodes[right].span),
          ]);
          Type::Real
        },
        And | Or | Xor => {
          constraints.extend_from_slice(&[
            tc(left_t.clone(), right_t.clone(), span),
            tc(Type::Boolean, left_t, nodes[left].span),
            tc(Type::Boolean, right_t, nodes[right].span),
          ]);
          Type::Boolean
        },
        DoubleEqual | BangEqual | Less | LessEqual | Greater | GreaterEqual => {
          constraints.push(tc(left_t.clone(), right_t.clone(), span));
          constraints.push(tc(left_t, right_t, nodes[right].span));
          Type::Boolean
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
      };
      constraints.push(tc(expect_t.clone(), child_t, nodes[child].span));
      expect_t
    },
    h::FunctionDef {
      parameter_name,
      parameter_type: parameter_types,
      body,
      ..
    } => {
      let parameter_type = match parameter_types {
        Some(n) => parse_type(nodes, n, environment)?,
        None => environment.fresh_type_var(),
      };
      environment.insert_type(parameter_name, parameter_type.clone(), false);
      let return_type = type_inference(nodes, body, environment, constraints)?;
      Type::Function(parameter_type.into(), return_type.into())
    },
    h::FunctionCall { callee, argument } => {
      let tv = environment.fresh_type_var();
      let callee_t = type_inference(nodes, callee, environment, constraints)?;
      let arg_t = type_inference(nodes, argument, environment, constraints)?;
      constraints.push(tc(
        callee_t,
        Type::Function(arg_t.into(), tv.clone().into()),
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
        Type::Unit
      };
      constraints.extend_from_slice(&[
        tc(pred_t, Type::Boolean, nodes[predicate].span),
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
        *ty = environment.get_type(cap);
      });
  }
  nodes[ptr].type_ = type_.clone();
  Ok(type_)
}
