use indexmap::IndexMap;

use super::*;
use crate::{LResult, Span, Spanned, WithSpan, parse::*};

pub fn type_def(
    ns: &mut ModuleNameSpace,
    assignee: String,
    assignee_span: Span,
    type_: TypeDefinition,
    items: &mut Vec<ModuleItem>,
    parameters: usize,
) -> LResult<()> {
    use TypeDefinitionKind::*;
    match type_.inner {
        TypeFunction { arguments, body } => {
            for (id, arg) in arguments.iter().enumerate() {
                ns.new_local_type(arg, Type::Variable(id));
            }
            type_def(ns, assignee, assignee_span, *body, items, arguments.len())?;
            ns.end_type_scopes(arguments.len());
        }
        Structure { lhs, rhs } => {
            let member_types: Vec<Type> =
                rhs.into_iter().map(|t| type_expr(ns, t)).try_collect()?;
            let path = ns
                .new_global_type(&assignee)
                .map_err(|e| e.span(assignee_span))?;
            let mut map = IndexMap::new();
            for (name, type_) in lhs.into_iter().zip(member_types) {
                map.insert(name, type_);
            }
            let type_ = Type::Struct {
                name: path.clone(),
                fields: map,
            };
            Universe::get().new_named_type(path.clone(), type_);
            items.push(ModuleItem::Type(path));
        }
        Sum {
            variant_names,
            variant_types,
        } => {
            let path = ns
                .new_global_type(&assignee)
                .map_err(|e| e.span(assignee_span))?;
            // HACK: Create an arbitrary type here with N type parameters.
            // This causes the type scheme to have the correct kindedness
            Universe::get().new_named_type(
                path.clone(),
                Type::Product((0..parameters).map(|tv| Type::Variable(tv)).collect()),
            );
            let variant_types: Vec<_> = variant_types
                .into_iter()
                .map(|t| {
                    if let Some(t) = t {
                        let t = type_expr(ns, t)?;
                        Ok(Some(t))
                    } else {
                        Ok(None)
                    }
                })
                .try_collect()?;
            let sum_type = Type::Sum {
                name: path.clone(),
                variant_names: variant_names.clone(),
                variant_types: variant_types
                    .clone()
                    .into_iter()
                    .map(|t| t.unwrap_or(Type::Unit))
                    .collect(),
            };
            Universe::get().new_named_type(path.clone(), sum_type);
            let named_type =
                Type::Instantiation(path.clone(), (0..parameters).map(Type::Variable).collect());
            for (id, (type_, name)) in variant_types.iter().zip(&variant_names).enumerate() {
                let constructor = Constructor {
                    variant_id: id,
                    kind: if let Some(type_) = type_ {
                        ConstructorKind::Function(type_.clone(), named_type.clone())
                    } else {
                        ConstructorKind::Unitary(named_type.clone())
                    },
                };
                let path = ns
                    .new_constructor(name, constructor.clone())
                    .map_err(|e| e.span(assignee_span))?;
                let value_path = ns
                    .new_global_value(name)
                    .map_err(|e| e.span(assignee_span))?;
                ns.finalize_value(&value_path);
                items.push(ModuleItem::Constructor(path, constructor));
            }
            items.push(ModuleItem::Type(path))
        }
        Expression(expr) => {
            let path = ns
                .new_global_type(&assignee)
                .map_err(|e| e.span(assignee_span))?;
            let type_ = type_expr(ns, expr)?;
            Universe::get().new_named_type(path.clone(), type_);
            items.push(ModuleItem::Type(path));
        }
    };
    Ok(())
}

pub fn type_expr(ns: &ModuleNameSpace, type_: TypeExpression) -> LResult<Type> {
    use TypeExpressionKind::*;
    let span = type_.span;
    Ok(match type_.inner {
        Function(a, b) => Type::func(type_expr(ns, *a)?, type_expr(ns, *b)?),
        Call(..) => {
            let mut parameters = vec![];
            let callee = reduce_call(ns, type_, &mut parameters)?;
            let abstract_type = Universe::get().get_named_type(&callee);
            if abstract_type.arity != parameters.len() {
                return Err(partial_instantiation_error(
                    abstract_type.arity,
                    parameters.len(),
                ));
            }
            let parameters = parameters
                .into_iter()
                .map(|t| type_expr(ns, t))
                .try_collect::<Vec<_>>()?;
            Type::Instantiation(callee.inner, parameters)
        }
        Identifier(id) => ns.get_type(&id).map_err(|e| e.span(span))?,
        ModulePath(items) => ns
            .get_type_exact(&Path::from(items))
            .map_err(|e| e.span(span))?,
        Product(items) => Type::Product(items.into_iter().map(|t| type_expr(ns, t)).try_collect()?),
        Array(inner) => Type::Array(type_expr(ns, *inner)?.into()),
        Unit => Type::Unit,
    })
}

fn reduce_call(
    ns: &ModuleNameSpace,
    expr: TypeExpression,
    parameters: &mut Vec<TypeExpression>,
) -> LResult<Spanned<Path>> {
    match expr.inner {
        TypeExpressionKind::Call(callee, argument) => {
            let inner = reduce_call(ns, *callee, parameters);
            parameters.push(*argument);
            inner
        }
        TypeExpressionKind::Identifier(name) => Ok(ns
            .get_type_path(&name)
            .map_err(|e| e.span(expr.span))?
            .with_span(expr.span)),
        TypeExpressionKind::ModulePath(items) => Ok(ns
            .get_type_path_exact(&Path::from(items))
            .map_err(|e| e.span(expr.span))?
            .with_span(expr.span)),
        TypeExpressionKind::Unit
        | TypeExpressionKind::Array(_)
        | TypeExpressionKind::Product(_)
        | TypeExpressionKind::Function(..) => {
            Err(err("This type cannot be instantiated").span(expr.span))
        }
    }
}
