use super::*;

pub fn type_inference(
  nodes: &mut HlIrModule,
  ptr: IrPtr,
  environment: &mut Environment,
  fresh_type_var: &mut impl FnMut() -> Type,
  constraints: &mut Vec<TypeConstraint>,
) -> Result<Type> {
  use HlIrKind as h;
  let span = nodes[ptr].span;
  let type_ = match nodes[ptr].kind.clone() {
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
      println!("{type_}");
      environment.insert_type(assignee.clone(), Type::Type);
      environment.insert_value(assignee, ConstValue::Type(type_));
      if let Some(in_) = in_ {
        type_inference(nodes, in_, environment, fresh_type_var, constraints)?
      } else {
        Type::Primitive(Primitive::nothing)
      }
    },
    h::Declaration {
      assignee,
      is_type: false,
      is_recursive,
      value,
      in_,
    } => {
      if is_recursive {
        let tv = fresh_type_var();
        environment.insert_type(assignee, tv.clone());
        let t = type_inference(
          nodes,
          value,
          environment,
          fresh_type_var,
          constraints,
        )?;
        constraints.push(TypeConstraint(t, tv, nodes[value].span));
      } else {
        let t = type_inference(
          nodes,
          value,
          environment,
          fresh_type_var,
          constraints,
        )?;
        environment.insert_type(assignee, t);
      }

      if let Some(in_) = in_ {
        type_inference(nodes, in_, environment, fresh_type_var, constraints)?
      } else {
        Type::Primitive(Primitive::nothing)
      }
    },
    h::Immediate(c) => Type::Primitive(match c {
      ConstValue::Nothing => Primitive::nothing,
      ConstValue::Integer(_) => Primitive::integer,
      ConstValue::Real(_) => Primitive::real,
      ConstValue::Boolean(_) => Primitive::boolean,
      ConstValue::String { .. } => Primitive::string,
      ConstValue::Glyph(_) => Primitive::glyph,
      _ => unreachable!(),
    }),
    h::Block(items) => items
      .into_iter()
      .try_fold(Type::Primitive(Primitive::nothing), |_, i| {
        type_inference(nodes, i, environment, fresh_type_var, constraints)
      })?,
    h::Identifier(i) => environment.get_type(&i).clone(),
    h::Tuple(items) => Type::Product(
      items
        .into_iter()
        .map(|i| {
          type_inference(nodes, i, environment, fresh_type_var, constraints)
        })
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
        .map(|v| {
          type_inference(nodes, v, environment, fresh_type_var, constraints)
        })
        .try_collect()?,
    },
    h::Field { of, .. } => {
      type_inference(nodes, of, environment, fresh_type_var, constraints)?;
      fresh_type_var()
    },
    h::Binary { op, left, right } => {
      let tv = fresh_type_var();
      let left_t =
        type_inference(nodes, left, environment, fresh_type_var, constraints)?;
      let right_t =
        type_inference(nodes, right, environment, fresh_type_var, constraints)?;
      constraints.push(TypeConstraint(left_t.clone(), right_t.clone(), span));
      use BinaryOp::*;
      match op {
        Star | Slash | Percent | Plus | Minus => {
          constraints.push(TypeConstraint(
            Primitive::integer.into(),
            left_t.clone(),
            nodes[left].span,
          ));
          constraints.push(TypeConstraint(
            Primitive::integer.into(),
            right_t.clone(),
            nodes[right].span,
          ));
          constraints.push(TypeConstraint(
            Primitive::integer.into(),
            tv.clone(),
            span,
          ));
        },
        And | Nand | Or | Xor | Xnor => {
          constraints.push(TypeConstraint(
            Type::Primitive(Primitive::boolean),
            left_t.clone(),
            nodes[left].span,
          ));
          constraints.push(TypeConstraint(
            Type::Primitive(Primitive::boolean),
            right_t.clone(),
            nodes[right].span,
          ));
          constraints.push(TypeConstraint(
            Type::Primitive(Primitive::boolean),
            tv.clone(),
            span,
          ));
        },

        DoubleEqual | Less | LessEqual | Greater | GreaterEqual | BangEqual => {
          constraints.push(TypeConstraint(
            left_t.clone(),
            tv.clone(),
            nodes[left].span,
          ));
          constraints.push(TypeConstraint(
            right_t.clone(),
            tv.clone(),
            nodes[right].span,
          ));
        },
        Arrow => {
          constraints.push(TypeConstraint(
            Type::Type,
            left_t,
            nodes[left].span,
          ));
          constraints.push(TypeConstraint(
            Type::Type,
            right_t,
            nodes[right].span,
          ));
          constraints.push(TypeConstraint(Type::Type, tv.clone(), span));
        },
        _ => todo!(),
      }
      tv
    },
    h::Unary { op, child } => {
      let child_t =
        type_inference(nodes, child, environment, fresh_type_var, constraints)?;
      use UnaryOp::*;
      let expect_t = match op {
        Minus => Primitive::integer.promote(),
        MinusDot => Primitive::real.promote(),
        Not => Primitive::boolean.promote(),
      };
      constraints.push(TypeConstraint(
        expect_t.clone(),
        child_t,
        nodes[child].span,
      ));
      expect_t
    },
    h::FunctionDef {
      parameter_names,
      parameter_types,
      body,
      ..
    } => {
      let parameter_types = parameter_types
        .into_iter()
        .map(|t| t.map(|t| parse_type(nodes, t, environment)))
        .map(|t| match t {
          Some(Ok(t)) => Ok(t),
          Some(Err(e)) => Err(e),
          None => Ok(fresh_type_var()),
        })
        .try_collect::<Vec<_>>()?;
      parameter_names
        .into_iter()
        .zip(parameter_types.clone())
        .for_each(|(n, t)| {
          environment.insert_type(n, t);
        });
      let return_type =
        type_inference(nodes, body, environment, fresh_type_var, constraints)?;
      Type::Function {
        param_types: parameter_types,
        return_type: return_type.into(),
      }
    },
    h::FunctionCall {
      callee, arguments, ..
    } => {
      let tv = fresh_type_var();
      let callee_t = type_inference(
        nodes,
        callee,
        environment,
        fresh_type_var,
        constraints,
      )?;
      let param_types: Vec<_> = if arguments.len() == 1
        && let HlIrKind::Immediate(ConstValue::Nothing) =
          nodes[arguments[0]].kind
      {
        nodes[arguments[0]].type_ = Type::Primitive(Primitive::nothing);
        vec![]
      } else {
        arguments
          .into_iter()
          .map(|a| {
            type_inference(nodes, a, environment, fresh_type_var, constraints)
          })
          .try_collect()?
      };
      constraints.push(TypeConstraint(
        Type::Function {
          param_types,
          return_type: tv.clone().into(),
        },
        callee_t,
        span,
      ));
      tv
    },
    h::If {
      predicate,
      then,
      else_,
    } => {
      let tv = fresh_type_var();
      let pred_t = type_inference(
        nodes,
        predicate,
        environment,
        fresh_type_var,
        constraints,
      )?;
      let then_t =
        type_inference(nodes, then, environment, fresh_type_var, constraints)?;
      let else_t = if let Some(else_) = else_ {
        type_inference(nodes, else_, environment, fresh_type_var, constraints)?
      } else {
        Type::Primitive(Primitive::nothing)
      };
      constraints.extend_from_slice(&[
        TypeConstraint(
          pred_t,
          Type::Primitive(Primitive::boolean),
          nodes[predicate].span,
        ),
        TypeConstraint(then_t, tv.clone(), nodes[then].span),
        TypeConstraint(
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
  nodes[ptr].type_ = type_.clone();
  Ok(type_)
}
