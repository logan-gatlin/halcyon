use crate::operator::{BinaryOp, UnaryOp};

use super::*;

pub fn infer_types(module: &mut IrModule) -> Result<ModuleInterface> {
    let mut interface = ModuleInterface::default();
    let mut env = Environment::new();
    for id in 0..module.items.len() {
        match module.items[id].clone() {
            ModuleItem::Let(mut pattern, ptr) => {
                env.begin_let();
                let pattern_t = infer_pattern(&mut env, &mut pattern);
                let body_t = infer_ir(&mut env, module, ptr)?;
                env.constraint(pattern_t, body_t, module.nodes[ptr].span);
                let solution = env.end_let()?;
                module.unify_all(&solution);
                pattern.iter_names(&mut |n, _| {
                    env.make_let_bound(n);
                    interface.values.insert(n.clone(), env.get_symbol(n));
                });
                pattern.unify_all(&solution);
                module.items[id] = ModuleItem::Let(pattern, ptr);
            }
            ModuleItem::Type(path, t) => {
                interface.types.insert(path, t);
            }
            ModuleItem::Constructor(path, constructor) => {
                env.constructors.insert(path.clone(), constructor.clone());
                env.define(
                    path.clone(),
                    Type::func(constructor.in_type.clone(), constructor.out_type.clone()),
                );
                env.make_let_bound(&path);
                interface.constructors.insert(path, constructor);
            }
        }
    }
    Ok(interface)
}

pub fn infer_pattern(env: &mut Environment, pat: &mut Pattern) -> TypeRef {
    let t = match &mut pat.kind {
        PatternKind::Name(path) => {
            let t = Type::TypeVariable(env.fresh_type_variable()).to_ref();
            env.define(path.clone(), t.clone());
            t
        }
        PatternKind::Tuple(patterns) => Type::Product(
            patterns
                .into_iter()
                .map(|p| infer_pattern(env, p))
                .collect(),
        )
        .to_ref(),
        PatternKind::Constructor(constructor, pattern) => {
            check_pattern(env, pattern, constructor.in_type.clone());
            constructor.out_type.clone()
        }
        PatternKind::Literal(const_value) => const_value.type_of(),
    };
    pat.type_ = t.clone();
    t
}

pub fn infer_ir(env: &mut Environment, module: &mut IrModule, ptr: IrPtr) -> Result<TypeRef> {
    macro_rules! rec {
        ($ptr:expr) => {
            infer_ir(env, module, $ptr)
        };
    }
    macro_rules! check {
        ($ptr:expr, $expect:expr) => {
            check_ir(env, module, $ptr, $expect)
        };
    }
    let span = module.nodes[ptr].span;
    let mk = |k, t: TypeRef| IrNode {
        kind: k,
        span,
        type_: t,
    };
    use IrKind as I;
    module.nodes[ptr] = match module.nodes[ptr].kind.clone() {
        I::Declaration {
            mut assignee,
            value,
            in_,
        } => {
            env.begin_let();
            let t = infer_pattern(env, &mut assignee);
            let value_t = rec!(value)?;
            env.constraint(t, value_t, span);
            let solution = env.end_let()?;
            module.unify_all(&solution);
            assignee.iter_names(&mut |name, _| {
                env.make_let_bound(name);
            });
            assignee.unify_all(&solution);
            let in_t = if let Some(in_) = in_ {
                rec!(in_)?
            } else {
                Type::Unit.to_ref()
            };
            mk(
                I::Declaration {
                    assignee,
                    value,
                    in_,
                },
                in_t,
            )
        }
        I::Immediate(const_value) => {
            let t = const_value.type_of();
            mk(I::Immediate(const_value), t)
        }
        I::Identifier(path) => {
            let t = env.get_symbol(&path);
            mk(I::Identifier(path), t)
        }
        I::Tuple(items) => {
            let t = Type::Product(items.iter().map(|i| rec!(*i)).try_collect()?).to_ref();
            mk(I::Tuple(items), t)
        }
        I::StructLiteral {
            field_names,
            field_values,
        } => {
            let field_types = field_values
                .iter()
                .map(|i| rec!(*i))
                .try_collect::<Vec<_>>()?;
            let t = Type::Struct {
                member_names: field_names.clone(),
                member_types: field_types,
            }
            .to_ref();
            mk(
                I::StructLiteral {
                    field_names,
                    field_values,
                },
                t,
            )
        }
        I::Field { of, index } => {
            let of_t = rec!(of)?;
            let t = if let Type::Struct {
                member_names,
                member_types,
            } = &*of_t.borrow()
                && let Some(index) = member_names.iter().position(|n| n == &index)
            {
                member_types[index].clone()
            } else {
                Type::TypeVariable(env.fresh_type_variable()).to_ref()
            };
            mk(I::Field { of, index }, t)
        }
        I::Binary { op, left, right } => {
            use BinaryOp::*;
            match op {
                Plus | Minus | Star | Slash | Percent => {
                    let t = Type::Integer.to_ref();
                    check!(left, t.clone())?;
                    check!(right, t.clone())?;
                    mk(I::Binary { op, left, right }, t)
                }
                PlusDot | MinusDot | StarDot | SlashDot => {
                    let t = Type::Real.to_ref();
                    check!(left, t.clone())?;
                    check!(right, t.clone())?;
                    mk(I::Binary { op, left, right }, t)
                }
                And | Or | Xor => {
                    let t = Type::Boolean.to_ref();
                    check!(left, t.clone())?;
                    check!(right, t.clone())?;
                    mk(I::Binary { op, left, right }, t)
                }
                DoubleEqual | BangEqual | Less | LessEqual | Greater | GreaterEqual => {
                    let t1 = rec!(left)?;
                    let t2 = rec!(right)?;
                    env.constraint(t1, t2, span);
                    mk(I::Binary { op, left, right }, Type::Boolean.to_ref())
                }
                Semicolon => {
                    let _ = rec!(left)?;
                    let t = rec!(right)?;
                    mk(I::Binary { op, left, right }, t)
                }
                Apply => {
                    let arg_t = rec!(left)?;
                    let return_t = Type::TypeVariable(env.fresh_type_variable()).to_ref();
                    let func_t = Type::func(arg_t, return_t.clone());
                    check!(right, func_t.clone())?;
                    mk(I::Binary { op, left, right }, return_t)
                }
            }
        }
        I::Unary { op, child } => {
            use UnaryOp::*;
            match op {
                Minus => {
                    let t = Type::Integer.to_ref();
                    check!(child, t.clone())?;
                    mk(I::Unary { op, child }, t)
                }
                MinusDot => {
                    let t = Type::Real.to_ref();
                    check!(child, t.clone())?;
                    mk(I::Unary { op, child }, t)
                }
                Not => {
                    let t = Type::Boolean.to_ref();
                    check!(child, t.clone())?;
                    mk(I::Unary { op, child }, t)
                }
            }
        }
        I::FunctionDef {
            parameter_name,
            parameter_span,
            parameter_type,
            captures,
            body,
            ..
        } => {
            let param_t = if parameter_name.is_none() {
                Type::Unit.to_ref()
            } else if let Some(parameter_type) = parameter_type.clone() {
                parameter_type
            } else {
                Type::TypeVariable(env.fresh_type_variable()).to_ref()
            };
            if let Some(parameter_name) = parameter_name.clone() {
                env.define(parameter_name, param_t.clone());
            }
            let body_t = rec!(body)?;
            let func_t = Type::func(param_t, body_t);
            let capture_types = captures
                .iter()
                .map(|p| env.get_symbol(p))
                .collect::<Vec<_>>();
            mk(
                I::FunctionDef {
                    parameter_name,
                    parameter_span,
                    parameter_type,
                    captures,
                    capture_types,
                    body,
                },
                func_t,
            )
        }
        I::FunctionCall { callee, argument } => {
            let arg_t = rec!(argument)?;
            let return_t = Type::TypeVariable(env.fresh_type_variable()).to_ref();
            let func_t = Type::func(arg_t, return_t.clone());
            check!(callee, func_t.clone())?;
            mk(I::FunctionCall { callee, argument }, return_t)
        }
        I::If {
            predicate,
            then,
            else_,
        } => {
            check!(predicate, Type::Boolean.to_ref())?;
            let then_t = rec!(then)?;
            if let Some(else_) = else_ {
                check!(else_, then_t.clone())?;
            } else {
                env.constraint(then_t.clone(), Type::Unit.to_ref(), span);
            }
            mk(
                I::If {
                    predicate,
                    then,
                    else_,
                },
                then_t,
            )
        }
        I::Match {
            scrutinee,
            mut predicates,
            branches,
        } => {
            let scrutinee_t = rec!(scrutinee)?;
            predicates
                .iter_mut()
                .for_each(|p| check_pattern(env, p, scrutinee_t.clone()));
            predicates.iter().fold(scrutinee_t, |a, b| {
                env.constraint(a.clone(), b.type_.clone(), b.span);
                a
            });
            let branch_t = branches.iter().map(|b| rec!(*b)).try_collect::<Vec<_>>()?;
            let branch_t = branch_t
                .iter()
                .fold(branch_t.first().unwrap().clone(), |a, b| {
                    env.constraint(a.clone(), b.clone(), span);
                    a.clone()
                });
            mk(
                I::Match {
                    scrutinee,
                    predicates,
                    branches,
                },
                branch_t,
            )
        }
        I::ImportedSymbol(path, type_) => mk(
            I::ImportedSymbol(path, type_.clone()),
            TypeScheme::new(type_).instantiate(|| env.fresh_type_variable()),
        ),
    };
    Ok(module.nodes[ptr].type_.clone())
}
