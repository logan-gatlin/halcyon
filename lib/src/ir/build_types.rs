use super::*;
use crate::{lint::*, parse::*};

#[derive(Debug, Clone)]
enum TypeIrKind {
    Identifier(Path),
    Call {
        callee: Box<TypeIr>,
        argument: Box<TypeIr>,
    },
    Function {
        argument: Path,
        body: Box<TypeIr>,
    },
}

type TypeIr = Spanned<TypeIrKind>;

pub fn type_def(
    ns: &mut ModuleNameSpace,
    assignee: String,
    assignee_span: Span,
    mut parameters: Vec<String>,
    type_: TypeDefinition,
    items: &mut Vec<ModuleItem>,
) -> Result<()> {
    use TypeDefinitionKind::*;
    let span = type_.span;
    // This is kinda dumb, prevents duplicate definitions when recursing on
    // type functions
    let (type_path, named_type) = if !matches!(type_.inner, TypeFunction { .. }) {
        let type_path = ns.define_named_type(&assignee).span(assignee_span)?;
        let named_type = Type::Named(type_path.clone(), vec![]);
        (type_path, named_type)
    } else {
        (Path::from(""), Type::Any)
    };
    let defined_type = match type_.inner {
        TypeFunction { arguments, body } => {
            for (id, arg) in arguments.iter().enumerate() {
                ns.types
                    .define_local(arg, Type::Variable(parameters.len() + id));
            }
            parameters.extend_from_slice(&arguments);
            return type_def(ns, assignee, assignee_span, parameters, *body, items);
        }
        Structure { lhs, rhs } => Type::Struct {
            member_names: lhs,
            member_types: rhs.into_iter().map(|t| type_expr(ns, t)).try_collect()?,
        },
        Sum {
            variant_names,
            variant_types,
        } => {
            let mut constructors = vec![];
            let mut in_types = vec![];
            for (id, (name, type_)) in variant_names.iter().zip(variant_types).enumerate() {
                let in_type = type_expr(ns, type_)?;
                let cons = Constructor {
                    variant: id,
                    in_type: in_type.clone(),
                    out_type: named_type.clone(),
                };
                in_types.push(in_type.clone());
                constructors.push((name, cons.clone()));
            }
            // Definitions enter the namespace after parsing, otherwise constructors
            // may appear within their own definitions
            for (name, constructor) in constructors {
                let path = ns
                    .constructors
                    .define_global(name, constructor.clone())
                    .span(assignee_span)?;
                ns.define_global_value(name).span(span)?;
                items.push(ModuleItem::Constructor(path, constructor));
            }
            Type::Sum {
                variant_names,
                variant_types: in_types,
            }
        }
        Expression(expression) => type_expr(ns, expression)?,
    };
    Universe::get().new_named_type(type_path.clone(), defined_type);
    items.push(ModuleItem::Type(type_path));
    Ok(())
}

pub fn type_expr(ns: &ModuleNameSpace, type_: TypeExpression) -> Result<Type> {
    use TypeExpressionKind::*;
    let span = type_.span;
    Ok(match type_.inner {
        Function(arg, returns) => Type::Function(
            Box::new(type_expr(ns, *arg)?),
            Box::new(type_expr(ns, *returns)?),
        ),
        Identifier(name) => ns.types.get(&name).span(span)?,
        ModulePath(items) => ns.types.get_exact(&Path::from(items)).span(span)?,
        Product(expressions) => Type::Product(
            expressions
                .into_iter()
                .map(|t| type_expr(ns, t))
                .try_collect()?,
        ),
        Unit => Type::Unit,
        Call(callee, argument) => {
            let Type::Named(name, mut args) = type_expr(ns, *callee)? else {
                panic!("Kindness error")
            };
            args.push(type_expr(ns, *argument)?);
            Type::Named(name, args)
        }
    })
}
