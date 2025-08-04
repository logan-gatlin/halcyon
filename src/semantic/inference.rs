use super::*;

pub fn infer_pattern(pattern: &mut Pattern, environment: &mut Environment) -> TypeRef {
    let type_ = match &mut pattern.kind {
        PatternKind::Name(path) => {
            let t = environment.fresh_type_var();
            environment.insert_value_type(path.clone(), t.clone(), false);
            t
        }
        PatternKind::Tuple(patterns) => Type::Product(
            patterns
                .into_iter()
                .map(|p| infer_pattern(p, environment))
                .collect(),
        )
        .into(),
        PatternKind::Literal(const_value) => const_value.type_of(),
    };
    pattern.type_ = type_.clone();
    type_
}

pub fn type_inference(
    nodes: &mut IrModule,
    ptr: IrPtr,
    environment: &mut Environment,
    constraints: &mut Vec<TypeConstraint>,
) -> Result<TypeRef> {
    use IrKind as h;
    use TypeConstraint as tc;
    macro_rules! rec {
        ($e:expr) => {
            type_inference(nodes, $e, environment, constraints)
        };
    }
    let span = nodes[ptr].span;
    let type_ = match nodes[ptr].kind.clone() {
        h::ImportedSymbol(name, type_) => {
            environment.insert_value_type(name.clone(), type_.clone(), true);
            environment.get_value_type(&name)
        }
        // Let declarations
        h::Declaration {
            mut assignee,
            value,
            in_,
        } => {
            let mut new_constraints = constraints.clone();
            let recursive_type_placeholder = infer_pattern(&mut assignee, environment);
            nodes[ptr].kind = h::Declaration {
                assignee: assignee.clone(),
                value,
                in_: in_.clone(),
            };
            type_inference(nodes, value, environment, &mut new_constraints)?;
            new_constraints.push(tc(
                recursive_type_placeholder.clone(),
                nodes[value].type_.clone(),
                span,
            ));
            let solution = unification(&new_constraints)?;
            assignee.unify_all(&solution);
            apply_solution(nodes, ptr, solution);
            let second_solution = unification(&check_structs(environment, nodes, ptr)?)?;
            assignee.unify_all(&second_solution);
            apply_solution(nodes, ptr, second_solution);
            assignee.iter_names(&mut |name, type_| {
                environment.insert_value_type(name.clone(), type_.clone(), true);
            });
            if let Some(in_) = in_ {
                rec!(in_)?
            } else {
                Type::Unit.to_ref()
            }
        }
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
        h::Tuple(items) => {
            Type::Product(items.into_iter().map(|i| rec!(i)).try_collect()?).to_ref()
        }
        h::StructLiteral {
            field_names,
            field_values,
            ..
        } => Type::Struct {
            member_names: field_names,
            member_types: field_values.into_iter().map(|v| rec!(v)).try_collect()?,
        }
        .to_ref(),
        h::Field { of, .. } => {
            rec!(of)?;
            environment.fresh_type_var()
        }
        h::Binary { op, left, right } => {
            let left_t = rec!(left)?;
            let right_t = rec!(right)?;
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
                }
                StarDot | SlashDot | PlusDot | MinusDot => {
                    constraints.extend_from_slice(&[
                        tc(left_t.clone(), right_t.clone(), span),
                        tc(Type::Real.to_ref(), left_t, nodes[left].span),
                        tc(Type::Real.to_ref(), right_t, nodes[right].span),
                    ]);
                    Type::Real.to_ref()
                }
                And | Or | Xor => {
                    constraints.extend_from_slice(&[
                        tc(left_t.clone(), right_t.clone(), span),
                        tc(Type::Boolean.to_ref(), left_t, nodes[left].span),
                        tc(Type::Boolean.to_ref(), right_t, nodes[right].span),
                    ]);
                    Type::Boolean.to_ref()
                }
                DoubleEqual | BangEqual | Less | LessEqual | Greater | GreaterEqual => {
                    constraints.push(tc(left_t.clone(), right_t.clone(), span));
                    constraints.push(tc(left_t, right_t, nodes[right].span));
                    Type::Boolean.to_ref()
                }
                _ => todo!(),
            }
        }
        h::Unary { op, child } => {
            let child_t = rec!(child)?;
            use UnaryOp::*;
            let expect_t = match op {
                Not => Type::Boolean,
                MinusDot => Type::Real,
                Minus => Type::Integer,
            }
            .to_ref();
            constraints.push(tc(expect_t.clone(), child_t, nodes[child].span));
            expect_t
        }
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
            let return_type = rec!(body)?;
            Type::func(parameter_type, return_type)
        }
        h::FunctionCall { callee, argument } => {
            let tv = environment.fresh_type_var();
            let callee_t = rec!(callee)?;
            let arg_t = rec!(argument)?;
            constraints.push(tc(
                callee_t,
                Type::func(arg_t, tv.clone()),
                nodes[argument].span,
            ));
            tv
        }
        h::If {
            predicate,
            then,
            else_,
        } => {
            let tv = environment.fresh_type_var();
            let pred_t = rec!(predicate)?;
            let then_t = rec!(then)?;
            let else_t = if let Some(else_) = else_ {
                rec!(else_)?
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
        }
        h::Match {
            scrutinee,
            mut predicates,
            branches,
        } => {
            let scrutinee_t = rec!(scrutinee)?;
            for p in &mut predicates {
                let predicate_t = infer_pattern(p, environment);
                constraints.push(tc(scrutinee_t.clone(), predicate_t, p.span));
            }
            nodes[ptr].kind = h::Match {
                scrutinee,
                predicates,
                branches: branches.clone(),
            };
            let branches_t = match branches.as_slice() {
                [] => Type::Unit.to_ref(),
                [a] => rec!(*a)?,
                b @ [a, ..] => {
                    let branch_t = rec!(*a)?;
                    for b in &b[1..] {
                        let new_t = rec!(*b)?;
                        constraints.push(tc(branch_t.clone(), new_t, nodes[*b].span));
                    }
                    branch_t
                }
            };
            branches_t
        }
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
