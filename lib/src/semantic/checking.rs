use super::*;

pub fn check_ir(
    env: &mut Environment,
    module: &mut IrModule,
    ptr: IrPtr,
    expect: Type,
) -> Result<()> {
    macro_rules! infer {
        ($ptr:expr) => {
            infer_ir(env, module, $ptr)
        };
    }
    macro_rules! rec {
        ($ptr:expr, $expect:expr) => {
            check_ir(env, module, $ptr, $expect)
        };
    }
    let span = module.nodes[ptr].span;
    use IrKind as I;
    match (module.nodes[ptr].kind.clone(), &expect) {
        (
            I::If {
                predicate,
                then,
                else_,
            },
            _,
        ) => {
            rec!(predicate, Type::Boolean)?;
            rec!(then, expect.clone())?;
            if let Some(else_) = else_ {
                rec!(else_, expect.clone())?;
            }
        }
        (I::Tuple(items), Type::Product(types)) if items.len() == types.len() => items
            .into_iter()
            .zip(types)
            .try_for_each(|(i, t)| rec!(i, t.clone()))?,
        (
            I::FunctionDef {
                parameter_name,
                parameter_span,
                parameter_type,
                captures,
                body,
                ..
            },
            Type::Function(expect_param, expect_return),
        ) if (parameter_name.is_none() && **expect_param == Type::Unit)
            || parameter_name.is_some() =>
        {
            if let Some(parameter_name) = parameter_name.clone() {
                env.define(parameter_name.clone(), *expect_param.clone());
            }
            let capture_types = captures
                .iter()
                .map(|p| env.get_symbol(p))
                .collect::<Vec<_>>();
            rec!(body, *expect_return.clone())?;
            module.nodes[ptr].kind = IrKind::FunctionDef {
                parameter_name,
                parameter_span,
                parameter_type,
                captures,
                capture_types,
                body,
            };
        }
        _ => {
            let t = infer!(ptr)?;
            env.type_constraint(t, expect, span);
            return Ok(());
        }
    };
    module.nodes[ptr].type_ = expect;
    Ok(())
}

pub fn check_pattern(env: &mut Environment, pat: &mut Pattern, expect: Type) {
    match (&mut pat.kind, &expect) {
        (PatternKind::Name(path), _) => {
            env.define(path.clone(), expect.clone());
            pat.type_ = expect.clone();
        }
        (PatternKind::Tuple(pats), Type::Product(types)) if pats.len() == types.len() => pats
            .iter_mut()
            .zip(types)
            .for_each(|(pat, t)| check_pattern(env, pat, t.clone())),
        _ => {
            let actual = infer_pattern(env, pat);
            env.type_constraint(actual, expect.clone(), pat.span);
            return;
        }
    }
    pat.type_ = expect;
}
