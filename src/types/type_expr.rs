use std::collections::HashMap;

use crate::Span;
use crate::ir::{
    Path,
    TypeExpr,
    TypeExprKind,
};

use super::instantiation::instantiate_forall_strict;
use super::{
    Type,
    TypeDefinition,
    TypeDefinitionKind,
};

#[derive(Debug, Clone)]
pub(crate) enum TypeExprSymbol {
    TypeParameter(u32),
    Definition(TypeDefinition),
    Unknown,
}

#[derive(Debug, Clone)]
pub(crate) enum TypeExprLowerError {
    TypeParameterApplied {
        name: Path,
        found: usize,
        span: Span,
    },
    InvalidTypeApplication {
        name: Path,
        expected: usize,
        found: usize,
        span: Span,
    },
    PlaceholderNotAllowed {
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct LoweredTypeExpr {
    pub type_: Type,
    pub errors: Vec<TypeExprLowerError>,
}

pub(crate) fn lower_type_expr(
    expr: &TypeExpr,
    lookup_symbol: &mut impl FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut impl FnMut(Span) -> Option<Type>,
) -> LoweredTypeExpr {
    lower_type_expr_dyn(expr, lookup_symbol, lower_placeholder)
}

fn lower_type_expr_dyn(
    expr: &TypeExpr,
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
) -> LoweredTypeExpr {
    match &expr.kind {
        TypeExprKind::Tuple(items) => lower_tuple(items, lookup_symbol, lower_placeholder),
        TypeExprKind::Instantiation(path, args) => {
            lower_instantiation(path, args, expr.span, lookup_symbol, lower_placeholder)
        }
        TypeExprKind::ForAll(params, body) => {
            let count = params.len();
            let param_map: HashMap<Path, u32> = params
                .iter()
                .enumerate()
                .map(|(i, path)| (path.clone(), (count - 1 - i) as u32))
                .collect();
            let mut body = lower_type_expr_dyn(
                body,
                &mut |path| {
                    param_map
                        .get(path)
                        .copied()
                        .map(TypeExprSymbol::TypeParameter)
                        .unwrap_or_else(|| lookup_symbol(path))
                },
                lower_placeholder,
            );
            body.type_ = body.type_.for_all(count);
            body
        }
        TypeExprKind::Placeholder => {
            if let Some(type_) = lower_placeholder(expr.span) {
                LoweredTypeExpr {
                    type_,
                    errors: Vec::new(),
                }
            } else {
                LoweredTypeExpr {
                    type_: Type::Unit,
                    errors: vec![TypeExprLowerError::PlaceholderNotAllowed { span: expr.span }],
                }
            }
        }
    }
}

fn lower_tuple(
    items: &[TypeExpr],
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
) -> LoweredTypeExpr {
    items.iter().fold(
        LoweredTypeExpr {
            type_: Type::Tuple(Vec::new()),
            errors: Vec::new(),
        },
        |mut lowered, item| {
            let mut item_lowered = lower_type_expr_dyn(item, lookup_symbol, lower_placeholder);
            let Type::Tuple(ref mut tuple_items) = lowered.type_ else {
                return lowered;
            };
            tuple_items.push(item_lowered.type_);
            lowered.errors.append(&mut item_lowered.errors);
            lowered
        },
    )
}

fn lower_instantiation(
    path: &Path,
    args: &[TypeExpr],
    span: Span,
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
) -> LoweredTypeExpr {
    let mut errors = Vec::new();
    let arguments = args
        .iter()
        .map(|arg| {
            let mut lowered = lower_type_expr_dyn(arg, lookup_symbol, lower_placeholder);
            errors.append(&mut lowered.errors);
            lowered.type_
        })
        .collect::<Vec<_>>();

    let type_ = match lookup_symbol(path) {
        TypeExprSymbol::TypeParameter(index) => {
            if !arguments.is_empty() {
                errors.push(TypeExprLowerError::TypeParameterApplied {
                    name: path.clone(),
                    found: arguments.len(),
                    span,
                });
            }
            Type::v(index)
        }
        TypeExprSymbol::Definition(definition) => {
            if definition.parameters != arguments.len() {
                errors.push(TypeExprLowerError::InvalidTypeApplication {
                    name: path.clone(),
                    expected: definition.parameters,
                    found: arguments.len(),
                    span,
                });
            }
            match definition.kind {
                TypeDefinitionKind::Named => {
                    Type::Named {
                        name: path.clone(),
                        body: Box::new(definition.body),
                    }
                    .apply(arguments)
                }
                TypeDefinitionKind::Alias => {
                    instantiate_forall_strict(&definition.body, &arguments)
                        .unwrap_or(definition.body)
                }
            }
        }
        TypeExprSymbol::Unknown => {
            Type::Named {
                name: path.clone(),
                body: Box::new(Type::Unit),
            }
            .apply(arguments)
        }
    };

    LoweredTypeExpr { type_, errors }
}

#[cfg(test)]
mod tests {
    use crate::Span;
    use crate::ir::{
        Path,
        TypeExpr,
        TypeExprKind,
    };

    use super::{
        Type,
        TypeDefinition,
        TypeDefinitionKind,
        TypeExprLowerError,
        TypeExprSymbol,
        lower_type_expr,
    };

    fn expr(kind: TypeExprKind) -> TypeExpr {
        TypeExpr {
            comments: String::new(),
            kind,
            span: Span::Generated,
        }
    }

    #[test]
    fn type_parameter_application_reports_error_and_recovers_to_parameter() {
        let a = Path::new("test", "a");
        let int = Path::core("integer");
        let forall = expr(TypeExprKind::ForAll(
            [a.clone()].into(),
            expr(TypeExprKind::Instantiation(
                a.clone(),
                [expr(TypeExprKind::Instantiation(int, [].into()))].into(),
            ))
            .into(),
        ));

        let lowered = lower_type_expr(&forall, &mut |_| TypeExprSymbol::Unknown, &mut |_| None);
        assert_eq!(lowered.type_, Type::v(0).for_all(1));
        assert!(matches!(
            lowered.errors.as_slice(),
            [TypeExprLowerError::TypeParameterApplied { name, found: 1, .. }] if name == &a
        ));
    }

    #[test]
    fn alias_arity_mismatch_recovers_to_alias_body_without_truncation() {
        let pair = Path::new("test", "Pair");
        let int = Path::core("integer");
        let bool_ = Path::core("boolean");
        let expr = expr(TypeExprKind::Instantiation(
            pair.clone(),
            [
                expr(TypeExprKind::Instantiation(int, [].into())),
                expr(TypeExprKind::Instantiation(bool_, [].into())),
            ]
            .into(),
        ));

        let lowered = lower_type_expr(
            &expr,
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 1,
                        body: Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1),
                        kind: TypeDefinitionKind::Alias,
                    })
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert_eq!(
            lowered.type_,
            Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1)
        );
        assert!(matches!(
            lowered.errors.as_slice(),
            [TypeExprLowerError::InvalidTypeApplication {
                name,
                expected: 1,
                found: 2,
                ..
            }] if name == &pair
        ));
    }

    #[test]
    fn placeholder_in_type_expr_uses_callback_type() {
        let placeholder = expr(TypeExprKind::Placeholder);
        let lowered = lower_type_expr(&placeholder, &mut |_| TypeExprSymbol::Unknown, &mut |_| {
            Some(Type::Integer)
        });
        assert_eq!(lowered.type_, Type::Integer);
        assert!(lowered.errors.is_empty());
    }

    #[test]
    fn placeholder_in_type_expr_reports_error_when_disallowed() {
        let placeholder = expr(TypeExprKind::Placeholder);
        let lowered = lower_type_expr(&placeholder, &mut |_| TypeExprSymbol::Unknown, &mut |_| {
            None
        });
        assert_eq!(lowered.type_, Type::Unit);
        assert!(matches!(
            lowered.errors.as_slice(),
            [TypeExprLowerError::PlaceholderNotAllowed { .. }]
        ));
    }
}
