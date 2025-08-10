use super::*;

pub fn check_ir(
    env: &mut Environment,
    module: &mut IrModule,
    ptr: IrPtr,
    expect: TypeRef,
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
    let expect_t = (*expect.borrow()).clone();
    use IrKind as I;
    match (module.nodes[ptr].kind.clone(), expect_t) {
        (
            I::If {
                predicate,
                then,
                else_,
            },
            _,
        ) => {
            rec!(predicate, Type::Boolean.to_ref())?;
            rec!(then, expect.clone())?;
            if let Some(else_) = else_ {
                rec!(else_, expect.clone())?;
            }
        }
        (I::Tuple(items), Type::Product(types)) if items.len() == types.len() => items
            .into_iter()
            .zip(types)
            .try_for_each(|(i, t)| rec!(i, t))?,
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
        ) if (parameter_name.is_none() && expect_param == Type::Unit.to_ref())
            || parameter_name.is_some() =>
        {
            if let Some(parameter_name) = parameter_name.clone() {
                env.define(parameter_name.clone(), expect_param);
            }
            let capture_types = captures
                .iter()
                .map(|p| env.get_symbol(p))
                .collect::<Vec<_>>();
            rec!(body, expect_return)?;
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
            env.constraint(t, expect, span);
            return Ok(());
        }
    };
    module.nodes[ptr].type_ = expect;
    Ok(())
}

pub fn check_pattern(env: &mut Environment, pat: &mut Pattern, expect: TypeRef) {
    let expect_t = (&*expect.borrow()).clone();
    match (&mut pat.kind, expect_t) {
        (PatternKind::Name(path), _) => {
            env.define(path.clone(), expect.clone());
            pat.type_ = expect.clone();
        }
        (PatternKind::Tuple(pats), Type::Product(types)) if pats.len() == types.len() => pats
            .into_iter()
            .zip(types)
            .for_each(|(pat, t)| check_pattern(env, pat, t)),
        _ => {
            let actual = infer_pattern(env, pat);
            env.constraint(actual, expect.clone(), pat.span);
        }
    }
    pat.type_ = expect;
}
