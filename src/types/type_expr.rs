//! Lowering from parsed `TypeExpr` syntax into semantic [`Type`] values.
//!
//! This pass is shared by inference and resolve-time type-definition lowering
//! so application rules and recovery behavior stay consistent.

use std::collections::HashMap;

use crate::Span;
use crate::ir::{
    Path,
    TypeExpr,
    TypeExprConstraint,
    TypeExprKind,
};

use super::instantiation::instantiate_forall_strict;
use super::{
    TraitConstraint,
    TraitRef,
    Type,
    TypeDefinition,
    TypeDefinitionKind,
    TypeScheme,
};

#[derive(Debug, Clone)]
pub(crate) enum TypeExprSymbol {
    /// A locally bound type parameter (De Bruijn index).
    TypeParameter(u32),
    /// A resolved type definition from the symbol table.
    Definition(TypeDefinition),
    /// A missing symbol; callers recover to placeholder nominal types.
    Unknown,
}

#[derive(Debug, Clone)]
pub(crate) enum TypeExprLowerError {
    #[cfg_attr(not(test), allow(dead_code))]
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
    TraitConstraintsNotAllowed {
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct LoweredTypeExpr {
    /// Best-effort lowered type, even when recoverable errors were emitted.
    pub type_: Type,
    /// Collected recoverable lowering errors.
    pub errors: Vec<TypeExprLowerError>,
}

#[derive(Debug, Clone)]
pub(crate) struct LoweredTypeSchemeExpr {
    /// Lowered qualified scheme (`predicates => type`).
    pub scheme: TypeScheme,
    /// Collected recoverable lowering errors.
    pub errors: Vec<TypeExprLowerError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
enum AliasLowering {
    Expand,
    PreserveConstructors,
}

pub(crate) fn lower_type_expr(
    expr: &TypeExpr,
    lookup_symbol: &mut impl FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut impl FnMut(Span) -> Option<Type>,
) -> LoweredTypeExpr {
    lower_type_expr_dyn(
        expr,
        lookup_symbol,
        lower_placeholder,
        AliasLowering::Expand,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lower_type_expr_preserving_alias_constructors(
    expr: &TypeExpr,
    lookup_symbol: &mut impl FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut impl FnMut(Span) -> Option<Type>,
) -> LoweredTypeExpr {
    lower_type_expr_dyn(
        expr,
        lookup_symbol,
        lower_placeholder,
        AliasLowering::PreserveConstructors,
    )
}

pub(crate) fn lower_type_scheme_expr(
    expr: &TypeExpr,
    lookup_symbol: &mut impl FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut impl FnMut(Span) -> Option<Type>,
) -> LoweredTypeSchemeExpr {
    lower_type_scheme_expr_dyn(
        expr,
        lookup_symbol,
        lower_placeholder,
        AliasLowering::Expand,
    )
}

#[allow(dead_code)]
pub(crate) fn lower_type_scheme_expr_preserving_alias_constructors(
    expr: &TypeExpr,
    lookup_symbol: &mut impl FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut impl FnMut(Span) -> Option<Type>,
) -> LoweredTypeSchemeExpr {
    lower_type_scheme_expr_dyn(
        expr,
        lookup_symbol,
        lower_placeholder,
        AliasLowering::PreserveConstructors,
    )
}

fn lower_type_expr_dyn(
    expr: &TypeExpr,
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
    alias_lowering: AliasLowering,
) -> LoweredTypeExpr {
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            lower_tuple(items, lookup_symbol, lower_placeholder, alias_lowering)
        }
        TypeExprKind::Instantiation(path, args) => {
            lower_instantiation(
                path,
                args,
                expr.span,
                lookup_symbol,
                lower_placeholder,
                alias_lowering,
            )
        }
        TypeExprKind::ForAll(params, constraints, body) => {
            let count = params.len();
            let forall_parameter_indices: HashMap<Path, u32> = params
                .iter()
                .enumerate()
                .map(|(i, path)| (path.clone(), (count - 1 - i) as u32))
                .collect();
            let mut body = lower_type_expr_dyn(
                body,
                &mut |path| {
                    forall_parameter_indices
                        .get(path)
                        .copied()
                        .map(TypeExprSymbol::TypeParameter)
                        .unwrap_or_else(|| {
                            match lookup_symbol(path) {
                                TypeExprSymbol::TypeParameter(index) => {
                                    TypeExprSymbol::TypeParameter(
                                        index.checked_add(count as u32).unwrap_or(index),
                                    )
                                }
                                other => other,
                            }
                        })
                },
                lower_placeholder,
                alias_lowering,
            );
            if let Some(constraint) = constraints.first() {
                body.errors
                    .push(TypeExprLowerError::TraitConstraintsNotAllowed {
                        span: constraint.span,
                    });
            }
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

fn lower_type_scheme_expr_dyn(
    expr: &TypeExpr,
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
    alias_lowering: AliasLowering,
) -> LoweredTypeSchemeExpr {
    match &expr.kind {
        TypeExprKind::ForAll(params, constraints, body) => {
            let count = params.len();
            let forall_parameter_indices: HashMap<Path, u32> = params
                .iter()
                .enumerate()
                .map(|(i, path)| (path.clone(), (count - 1 - i) as u32))
                .collect();
            let mut lowered_body = lower_type_scheme_expr_dyn(
                body,
                &mut |path| {
                    forall_parameter_indices
                        .get(path)
                        .copied()
                        .map(TypeExprSymbol::TypeParameter)
                        .unwrap_or_else(|| {
                            match lookup_symbol(path) {
                                TypeExprSymbol::TypeParameter(index) => {
                                    TypeExprSymbol::TypeParameter(
                                        index.checked_add(count as u32).unwrap_or(index),
                                    )
                                }
                                other => other,
                            }
                        })
                },
                lower_placeholder,
                alias_lowering,
            );
            let mut current_predicates = Vec::new();
            for constraint in constraints.iter() {
                let (predicate, mut errors) = lower_trait_constraint(
                    constraint,
                    &mut |path| {
                        forall_parameter_indices
                            .get(path)
                            .copied()
                            .map(TypeExprSymbol::TypeParameter)
                            .unwrap_or_else(|| {
                                match lookup_symbol(path) {
                                    TypeExprSymbol::TypeParameter(index) => {
                                        TypeExprSymbol::TypeParameter(
                                            index.checked_add(count as u32).unwrap_or(index),
                                        )
                                    }
                                    other => other,
                                }
                            })
                    },
                    lower_placeholder,
                    alias_lowering,
                );
                current_predicates.push(predicate);
                lowered_body.errors.append(&mut errors);
            }
            current_predicates.append(&mut lowered_body.scheme.predicates);
            lowered_body.scheme.type_ = lowered_body.scheme.type_.for_all(count);
            lowered_body.scheme.predicates = current_predicates;
            lowered_body
        }
        _ => {
            let lowered =
                lower_type_expr_dyn(expr, lookup_symbol, lower_placeholder, alias_lowering);
            LoweredTypeSchemeExpr {
                scheme: TypeScheme::new(lowered.type_),
                errors: lowered.errors,
            }
        }
    }
}

fn lower_trait_constraint(
    constraint: &TypeExprConstraint,
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
    alias_lowering: AliasLowering,
) -> (TraitConstraint, Vec<TypeExprLowerError>) {
    let mut errors = Vec::new();
    let arguments = constraint
        .arguments
        .iter()
        .map(|argument| {
            let mut lowered =
                lower_type_expr_dyn(argument, lookup_symbol, lower_placeholder, alias_lowering);
            errors.append(&mut lowered.errors);
            lowered.type_
        })
        .collect::<Vec<_>>();
    (
        TraitRef {
            trait_name: constraint.trait_name.clone(),
            arguments,
        },
        errors,
    )
}

fn lower_tuple(
    items: &[TypeExpr],
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
    alias_lowering: AliasLowering,
) -> LoweredTypeExpr {
    match items {
        [] => {
            LoweredTypeExpr {
                type_: Type::Unit,
                errors: Vec::new(),
            }
        }
        _ => {
            items.iter().fold(
                LoweredTypeExpr {
                    type_: Type::Tuple(Vec::new()),
                    errors: Vec::new(),
                },
                |mut lowered, item| {
                    let mut item_lowered =
                        lower_type_expr_dyn(item, lookup_symbol, lower_placeholder, alias_lowering);
                    let Type::Tuple(ref mut tuple_items) = lowered.type_ else {
                        return lowered;
                    };
                    tuple_items.push(item_lowered.type_);
                    lowered.errors.append(&mut item_lowered.errors);
                    lowered
                },
            )
        }
    }
}

fn lower_instantiation(
    path: &Path,
    args: &[TypeExpr],
    span: Span,
    lookup_symbol: &mut dyn FnMut(&Path) -> TypeExprSymbol,
    lower_placeholder: &mut dyn FnMut(Span) -> Option<Type>,
    alias_lowering: AliasLowering,
) -> LoweredTypeExpr {
    let mut errors = Vec::new();
    let arguments = args
        .iter()
        .map(|arg| {
            let mut lowered =
                lower_type_expr_dyn(arg, lookup_symbol, lower_placeholder, alias_lowering);
            errors.append(&mut lowered.errors);
            lowered.type_
        })
        .collect::<Vec<_>>();

    let lowered_type = match lookup_symbol(path) {
        TypeExprSymbol::TypeParameter(index) => Type::v(index).apply(arguments),
        TypeExprSymbol::Definition(definition) => {
            if arguments.len() > definition.parameters {
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
                    if arguments.len() < definition.parameters {
                        Type::Named {
                            name: path.clone(),
                            body: Box::new(definition.body),
                        }
                        .apply(arguments)
                    } else {
                        let expanded = instantiate_forall_strict(&definition.body, &arguments)
                            .unwrap_or(definition.body);
                        match alias_lowering {
                            AliasLowering::Expand => expanded,
                            AliasLowering::PreserveConstructors if definition.parameters > 0 => {
                                Type::Named {
                                    name: path.clone(),
                                    body: Box::new(expanded),
                                }
                            }
                            _ => expanded,
                        }
                    }
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

    LoweredTypeExpr {
        type_: lowered_type,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use crate::Span;
    use crate::hc_core::CoreType;
    use crate::ir::{
        Path,
        TypeExpr,
        TypeExprConstraint,
        TypeExprKind,
    };
    use crate::types::Kind;
    use crate::types::symbol_table::Symbol;

    use super::{
        TraitRef,
        Type,
        TypeDefinition,
        TypeDefinitionKind,
        TypeExprLowerError,
        TypeExprSymbol,
        lower_type_expr,
        lower_type_expr_preserving_alias_constructors,
        lower_type_scheme_expr,
    };

    fn expr(kind: TypeExprKind) -> TypeExpr {
        TypeExpr {
            comments: String::new(),
            kind,
            span: Span::Generated,
        }
    }

    fn constraint(
        trait_name: Path,
        arguments: Vec<TypeExpr>,
    ) -> TypeExprConstraint {
        TypeExprConstraint {
            trait_name,
            arguments: arguments.into_boxed_slice(),
            span: Span::Generated,
        }
    }

    #[test]
    fn named_definition_instantiation_stays_nominal() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [expr(TypeExprKind::Instantiation(int.clone(), [].into()))].into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 1,
                        parameter_kinds: vec![Kind::Type],
                        body: Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1),
                        kind: TypeDefinitionKind::Named,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        let Type::Apply {
            constructor,
            arguments,
        } = lowered.type_
        else {
            panic!("expected type application");
        };
        assert!(matches!(*constructor, Type::Named { name, .. } if name == pair));
        assert_eq!(arguments, vec![Type::Integer]);
    }

    #[test]
    fn alias_definition_instantiation_is_structural() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [expr(TypeExprKind::Instantiation(int.clone(), [].into()))].into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 1,
                        parameter_kinds: vec![Kind::Type],
                        body: Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1),
                        kind: TypeDefinitionKind::Alias,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        assert_eq!(
            lowered.type_,
            Type::Tuple(vec![Type::Integer, Type::Integer])
        );
    }

    #[test]
    fn alias_instantiation_can_preserve_constructor_shape() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let lowered = lower_type_expr_preserving_alias_constructors(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [expr(TypeExprKind::Instantiation(int.clone(), [].into()))].into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 2,
                        parameter_kinds: vec![Kind::Type, Kind::Type],
                        body: Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2),
                        kind: TypeDefinitionKind::Alias,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        let Type::Apply {
            constructor,
            arguments,
        } = lowered.type_
        else {
            panic!("expected preserved alias constructor application");
        };
        assert!(matches!(*constructor, Type::Named { name, .. } if name == pair));
        assert_eq!(arguments, vec![Type::Integer]);
    }

    #[test]
    fn alias_instantiation_preserves_constructor_when_fully_applied() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let bool_ = CoreType::Boolean.path();
        let lowered = lower_type_expr_preserving_alias_constructors(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [
                    expr(TypeExprKind::Instantiation(int.clone(), [].into())),
                    expr(TypeExprKind::Instantiation(bool_.clone(), [].into())),
                ]
                .into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 2,
                        parameter_kinds: vec![Kind::Type, Kind::Type],
                        body: Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2),
                        kind: TypeDefinitionKind::Alias,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else if path == &bool_ {
                    TypeExprSymbol::Definition(Type::Boolean.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        assert!(
            matches!(lowered.type_, Type::Named { ref name, .. } if name == &pair),
            "fully applied alias should be wrapped in Named under PreserveConstructors"
        );
    }

    #[test]
    fn unknown_symbols_recover_to_placeholder_nominal_types() {
        let missing = Path::new("demo", "Missing");
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(missing.clone(), [].into())),
            &mut |_| TypeExprSymbol::Unknown,
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        assert!(matches!(
            lowered.type_,
            Type::Named { name, .. } if name == missing
        ));
    }

    #[test]
    fn type_parameter_application_lowers_to_applied_parameter() {
        let a = Path::new("demo", "a");
        let int = CoreType::Integer.path();
        let lowered = lower_type_expr(
            &expr(TypeExprKind::ForAll(
                [a.clone()].into(),
                [].into(),
                expr(TypeExprKind::Instantiation(
                    a.clone(),
                    [expr(TypeExprKind::Instantiation(int.clone(), [].into()))].into(),
                ))
                .into(),
            )),
            &mut |path| {
                if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        assert_eq!(
            lowered.type_,
            Type::v(0).apply(vec![Type::Integer]).for_all(1)
        );
    }

    #[test]
    fn invalid_type_application_reports_error_and_keeps_lowered_shape() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let bool_ = CoreType::Boolean.path();
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [
                    expr(TypeExprKind::Instantiation(int.clone(), [].into())),
                    expr(TypeExprKind::Instantiation(bool_.clone(), [].into())),
                ]
                .into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 1,
                        parameter_kinds: vec![Kind::Type],
                        body: Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1),
                        kind: TypeDefinitionKind::Named,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else if path == &bool_ {
                    TypeExprSymbol::Definition(Type::Boolean.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
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
        assert!(matches!(
            lowered.type_,
            Type::Apply {
                constructor: _,
                arguments: _
            }
        ));
    }

    #[test]
    fn partial_named_type_application_is_allowed() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [expr(TypeExprKind::Instantiation(int.clone(), [].into()))].into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 2,
                        parameter_kinds: vec![Kind::Type, Kind::Type],
                        body: Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2),
                        kind: TypeDefinitionKind::Named,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        let Type::Apply {
            constructor,
            arguments,
        } = lowered.type_
        else {
            panic!("expected partially applied named type");
        };
        assert!(matches!(*constructor, Type::Named { name, .. } if name == pair));
        assert_eq!(arguments, vec![Type::Integer]);
    }

    #[test]
    fn partial_alias_type_application_preserves_constructor_shape() {
        let pair = Path::new("demo", "Pair");
        let int = CoreType::Integer.path();
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [expr(TypeExprKind::Instantiation(int.clone(), [].into()))].into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 2,
                        parameter_kinds: vec![Kind::Type, Kind::Type],
                        body: Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2),
                        kind: TypeDefinitionKind::Alias,
                    })
                } else if path == &int {
                    TypeExprSymbol::Definition(Type::Integer.def(0))
                } else {
                    TypeExprSymbol::Definition(Type::Unit.def(0))
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        let Type::Apply {
            constructor,
            arguments,
        } = lowered.type_
        else {
            panic!("expected partially applied alias constructor");
        };
        assert!(matches!(*constructor, Type::Named { name, .. } if name == pair));
        assert_eq!(arguments, vec![Type::Integer]);
    }

    #[test]
    fn alias_arity_mismatch_recovers_to_original_alias_body() {
        let pair = Path::new("demo", "Pair");
        let lowered = lower_type_expr(
            &expr(TypeExprKind::Instantiation(
                pair.clone(),
                [
                    expr(TypeExprKind::Instantiation(
                        CoreType::Integer.path(),
                        [].into(),
                    )),
                    expr(TypeExprKind::Instantiation(
                        CoreType::Boolean.path(),
                        [].into(),
                    )),
                ]
                .into(),
            )),
            &mut |path| {
                if path == &pair {
                    TypeExprSymbol::Definition(TypeDefinition {
                        parameters: 1,
                        parameter_kinds: vec![Kind::Type],
                        body: Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1),
                        kind: TypeDefinitionKind::Alias,
                    })
                } else {
                    TypeExprSymbol::Definition(Type::Unit.def(0))
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
            [TypeExprLowerError::InvalidTypeApplication { .. }]
        ));
    }

    #[test]
    fn forall_body_shifts_outer_parameter_indices() {
        let outer = Path::new("demo", "a");
        let inner = Path::new("demo", "b");
        let lowered = lower_type_expr(
            &expr(TypeExprKind::ForAll(
                [inner.clone()].into(),
                [].into(),
                expr(TypeExprKind::Tuple(
                    [
                        expr(TypeExprKind::Instantiation(outer.clone(), [].into())),
                        expr(TypeExprKind::Instantiation(inner.clone(), [].into())),
                    ]
                    .into(),
                ))
                .into(),
            )),
            &mut |path| {
                if path == &outer {
                    TypeExprSymbol::TypeParameter(0)
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert_eq!(
            lowered.type_,
            Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(1)
        );
    }

    #[test]
    fn constraints_in_plain_forall_type_report_error() {
        let a = Path::new("demo", "a");
        let lowered = lower_type_expr(
            &expr(TypeExprKind::ForAll(
                [a.clone()].into(),
                [constraint(
                    Path::new("demo", "Eq"),
                    vec![expr(TypeExprKind::Instantiation(a.clone(), [].into()))],
                )]
                .into(),
                expr(TypeExprKind::Instantiation(a, [].into())).into(),
            )),
            &mut |_| TypeExprSymbol::Unknown,
            &mut |_| None,
        );

        assert!(matches!(
            lowered.errors.as_slice(),
            [TypeExprLowerError::TraitConstraintsNotAllowed { .. }]
        ));
    }

    #[test]
    fn lower_type_scheme_expr_collects_forall_constraints() {
        let a = Path::new("demo", "a");
        let function = CoreType::Function.path();
        let lowered = lower_type_scheme_expr(
            &expr(TypeExprKind::ForAll(
                [a.clone()].into(),
                [constraint(
                    Path::new("demo", "Eq"),
                    vec![expr(TypeExprKind::Instantiation(a.clone(), [].into()))],
                )]
                .into(),
                expr(TypeExprKind::Instantiation(
                    function.clone(),
                    [
                        expr(TypeExprKind::Instantiation(a.clone(), [].into())),
                        expr(TypeExprKind::Instantiation(a.clone(), [].into())),
                    ]
                    .into(),
                ))
                .into(),
            )),
            &mut |path| {
                if path == &function {
                    TypeExprSymbol::Definition(Type::function().def(2))
                } else {
                    TypeExprSymbol::Unknown
                }
            },
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        assert_eq!(
            lowered.scheme.type_,
            Type::func(Type::v(0), Type::v(0)).for_all(1)
        );
        assert_eq!(
            lowered.scheme.predicates,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])]
        );
    }

    #[test]
    fn lower_type_scheme_expr_for_non_forall_has_no_predicates() {
        let lowered = lower_type_scheme_expr(
            &expr(TypeExprKind::Instantiation(
                CoreType::Integer.path(),
                [].into(),
            )),
            &mut |_| TypeExprSymbol::Definition(Type::Integer.def(0)),
            &mut |_| None,
        );

        assert!(lowered.errors.is_empty());
        assert!(lowered.scheme.predicates.is_empty());
        assert_eq!(lowered.scheme.type_, Type::Integer);
    }

    #[test]
    fn placeholders_use_callback_or_report_error() {
        let placeholder = expr(TypeExprKind::Placeholder);
        let lowered = lower_type_expr(&placeholder, &mut |_| TypeExprSymbol::Unknown, &mut |_| {
            Some(Type::Boolean)
        });
        assert_eq!(lowered.type_, Type::Boolean);
        assert!(lowered.errors.is_empty());

        let lowered = lower_type_expr(&placeholder, &mut |_| TypeExprSymbol::Unknown, &mut |_| {
            None
        });
        assert_eq!(lowered.type_, Type::Unit);
        assert!(matches!(
            lowered.errors.as_slice(),
            [TypeExprLowerError::PlaceholderNotAllowed { .. }]
        ));
    }

    #[test]
    fn lower_tuple_collects_errors_from_all_items() {
        let a = Path::new("demo", "a");
        let tuple = expr(TypeExprKind::Tuple(
            [
                expr(TypeExprKind::ForAll(
                    [a.clone()].into(),
                    [constraint(
                        Path::new("demo", "Eq"),
                        vec![expr(TypeExprKind::Instantiation(a.clone(), [].into()))],
                    )]
                    .into(),
                    expr(TypeExprKind::Instantiation(a.clone(), [].into())).into(),
                )),
                expr(TypeExprKind::Placeholder),
            ]
            .into(),
        ));

        let lowered = lower_type_expr(&tuple, &mut |_| TypeExprSymbol::Unknown, &mut |_| None);
        assert_eq!(lowered.errors.len(), 2);
    }

    #[test]
    fn singleton_tuple_type_term_lowers_as_singleton_tuple() {
        let grouped = expr(TypeExprKind::Tuple(
            [expr(TypeExprKind::Instantiation(
                CoreType::Integer.path(),
                [].into(),
            ))]
            .into(),
        ));

        let lowered = lower_type_expr(
            &grouped,
            &mut |_| TypeExprSymbol::Definition(Type::Integer.def(0)),
            &mut |_| None,
        );

        assert_eq!(lowered.type_, Type::Tuple(vec![Type::Integer]));
    }

    #[test]
    fn empty_tuple_type_term_lowers_to_unit() {
        let unit = expr(TypeExprKind::Tuple([].into()));

        let lowered = lower_type_expr(&unit, &mut |_| TypeExprSymbol::Unknown, &mut |_| None);

        assert_eq!(lowered.type_, Type::Unit);
    }
}
