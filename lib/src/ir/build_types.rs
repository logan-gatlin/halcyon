use super::*;
use crate::{lint::*, parse::*};

pub fn type_def(
    ns: &mut ModuleNameSpace,
    assignee: String,
    assignee_span: Span,
    type_: TypeDefinition,
    items: &mut Vec<ModuleItem>,
) -> Result<()> {
    use TypeDefinitionKind::*;
    match type_.inner {
        TypeFunction { arguments, body } => {
            for (id, arg) in arguments.iter().enumerate() {
                ns.types.define_local(arg, Type::Variable(id));
            }
            type_def(ns, assignee, assignee_span, *body, items)?;
            ns.types.end_local_scopes(arguments.len());
        }
        Structure { lhs, rhs } => {
            let type_ = Type::Struct {
                member_names: lhs,
                member_types: rhs.into_iter().map(|t| type_expr(ns, t)).try_collect()?,
            };
            let path = ns.define_type(&assignee, type_).span(assignee_span)?;
            items.push(ModuleItem::Type(path));
        }
        Sum {
            variant_names,
            variant_types,
        } => {
            let path = ns.define_type(&assignee, Type::Any).span(assignee_span)?;
            let variant_types: Vec<_> = variant_types
                .into_iter()
                .map(|t| type_expr(ns, t))
                .try_collect()?;
            let sum_type = Type::Sum {
                variant_names: variant_names.clone(),
                variant_types: variant_types.clone(),
            };
            for (id, (type_, name)) in variant_types.iter().zip(&variant_names).enumerate() {
                let constructor = Constructor {
                    variant: id,
                    in_type: type_.clone(),
                    out_type: sum_type.clone(),
                };
                let path = ns.constructors.define_global(name, constructor.clone())?;
                items.push(ModuleItem::Constructor(path, constructor));
            }
            ns.update_type(path.clone(), sum_type);
            items.push(ModuleItem::Type(path))
        }
        Expression(expr) => {
            let path = ns
                .define_type(&assignee, type_expr(ns, expr)?)
                .span(assignee_span)?;
            items.push(ModuleItem::Type(path));
        }
    };
    Ok(())
}

pub fn type_expr(ns: &ModuleNameSpace, type_: TypeExpression) -> Result<Type> {
    use TypeExpressionKind::*;
    Ok(match type_.inner {
        Function(a, b) => Type::func(type_expr(ns, *a)?, type_expr(ns, *b)?),
        Call(..) => {
            let mut parameters = vec![];
            let callee =
                reduce_call(ns, type_, &mut parameters).context(format!("{}", parameters.len()))?;
            let abstract_type = Universe::get().get_named_type(&callee);
            if abstract_type.arity != parameters.len() {
                return Err(lint(
                    TypeLint::PartialInstantiation,
                    callee.span,
                    [
                        format!("{}", abstract_type.arity),
                        format!("{}", parameters.len()),
                    ],
                ));
            }
            let parameters = parameters
                .into_iter()
                .map(|t| type_expr(ns, t))
                .try_collect::<Vec<_>>()?;
            abstract_type.instantiate(&parameters)
        }
        Identifier(id) => {
            let type_ = ns.types.get(&id).span(type_.span)?;
            // Incomplete instantiation
            if let Type::Instantiation(name, types) = &type_
                && Universe::get().get_named_type(name).arity != types.len()
            {
                panic!()
            }
            type_
        }
        Product(items) => Type::Product(items.into_iter().map(|t| type_expr(ns, t)).try_collect()?),
        ModulePath(items) => ns.types.get_exact(&Path::from(items)).span(type_.span)?,
        Unit => Type::Unit,
    })
}

fn reduce_call(
    ns: &ModuleNameSpace,
    expr: TypeExpression,
    parameters: &mut Vec<TypeExpression>,
) -> Result<Spanned<Path>> {
    match expr.inner {
        TypeExpressionKind::Call(callee, argument) => {
            let inner = reduce_call(ns, *callee, parameters);
            parameters.push(*argument);
            inner
        }
        TypeExpressionKind::Identifier(name) => Ok(ns
            .types
            .get_path(&name)
            .span(expr.span)?
            .with_span(expr.span)),
        TypeExpressionKind::ModulePath(items) => {
            let path = Path::from(items);
            ns.types.get_exact(&path)?;
            Ok(path.with_span(expr.span))
        }
        TypeExpressionKind::Unit
        | TypeExpressionKind::Product(_)
        | TypeExpressionKind::Function(..) => Err(lint_nospan(TypeLint::PartialInstantiation))
            .context("0")
            .span(expr.span),
    }
}
