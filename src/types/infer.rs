//! Type inference and bidirectional type checking.
//!
//! This module implements Hindley-Milner style inference with:
//! - let-generalization,
//! - trait-predicate accumulation,
//! - explicit higher-rank checks via skolemization in checking mode.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::Span;
use crate::ir::{
    Glob,
    Path,
    Pattern,
    PatternKind,
    ScopeKind,
    Term,
    TermKind,
    TypeExpr,
    TypeExprKind,
};

use super::instantiation::instantiate_predicates;
use super::type_expr::{
    TypeExprLowerError,
    TypeExprSymbol,
    lower_type_expr,
    lower_type_scheme_expr,
};
use super::{
    MetaVarId,
    StructMatch,
    TraitConstraint,
    TraitRef,
    Type,
    TypeDefinition,
    TypeScheme,
    TypeTransform,
    for_each_child_type,
};

use super::unify::{
    UnificationTable,
    UnifyError,
};

/// Errors produced during type inference.
#[derive(Debug, Clone)]
pub enum TypeError {
    /// Referenced term path is not in scope.
    UnknownIdentifier { path: Path, span: Span },
    /// Referenced constructor path is not in scope.
    UnknownConstructor { path: Path, span: Span },
    /// Type application arity mismatch.
    InvalidTypeApplication {
        name: Path,
        expected: usize,
        found: usize,
        span: Span,
    },
    /// Placeholder `_` used where not allowed.
    InvalidPlaceholderType { span: Span },
    /// Trait constraints were written where only plain types are allowed.
    TraitConstraintsNotAllowed { span: Span },
    /// Non-function type used in call position.
    NotAFunction { type_: Type, span: Span },
    /// Failed to instantiate a type scheme.
    InvalidScheme { span: Span },
    /// Unannotated function parameter was used at incompatible argument types.
    HigherRankAnnotationRequired { parameter: Path, span: Span },
    /// Explicit `for all` annotation omitted required trait constraints.
    PolymorphicAnnotationMissingConstraints {
        predicates: Vec<TraitConstraint>,
        span: Span,
    },
    /// Unification failure with source span context.
    Unification { error: UnifyError, span: Span },
}

#[cfg(test)]
mod tests {
    use crate::ir::{
        Glob,
        ImmediateValue,
        ScopeKind,
        TypeExprConstraint,
        TypeExprKind,
    };
    use crate::types::TypeDefinitionKind;
    use crate::{
        Span,
        WithSpan,
    };

    use super::*;

    fn term(kind: TermKind<()>) -> Term<()> {
        Term {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    fn pattern(kind: PatternKind<()>) -> Pattern<()> {
        Pattern {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    fn type_expr(kind: TypeExprKind) -> TypeExpr {
        TypeExpr {
            comments: String::new(),
            kind,
            span: Span::Generated,
        }
    }

    fn type_inst(
        path: Path,
        args: Vec<TypeExpr>,
    ) -> TypeExpr {
        type_expr(TypeExprKind::Instantiation(path, args.into_boxed_slice()))
    }

    fn type_forall(
        params: Vec<Path>,
        constraints: Vec<TypeExprConstraint>,
        body: TypeExpr,
    ) -> TypeExpr {
        type_expr(TypeExprKind::ForAll(
            params.into_boxed_slice(),
            constraints.into_boxed_slice(),
            Box::new(body),
        ))
    }

    fn type_constraint(
        trait_name: Path,
        arguments: Vec<TypeExpr>,
    ) -> TypeExprConstraint {
        TypeExprConstraint {
            trait_name,
            arguments: arguments.into_boxed_slice(),
            span: Span::Generated,
        }
    }

    fn forall_identity_type_expr(param: Path) -> TypeExpr {
        type_forall(
            vec![param.clone()],
            Vec::new(),
            type_inst(
                Path::core("function"),
                vec![
                    type_inst(param.clone(), Vec::new()),
                    type_inst(param, Vec::new()),
                ],
            ),
        )
    }

    fn core_type_definitions() -> IndexMap<Path, TypeDefinition> {
        [
            (Path::core("unit"), Type::Unit.def(0)),
            (Path::core("integer"), Type::Integer.def(0)),
            (Path::core("real"), Type::Real.def(0)),
            (Path::core("boolean"), Type::Boolean.def(0)),
            (Path::core("string"), Type::String.def(0)),
            (Path::core("glyph"), Type::Glyph.def(0)),
            (Path::core("array"), Type::array().def(1)),
            (Path::core("function"), Type::function().def(2)),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn type_env_api_roundtrip() {
        let p1 = Path::new("demo", "a");
        let p2 = Path::new("demo", "b");
        let p3 = Path::new("demo", "c");
        let p4 = Path::new("demo", "d");

        let env = TypeEnv::new()
            .with_binding(p1.clone(), Type::Integer)
            .with_bindings([(p2.clone(), Type::Boolean)]);
        assert_eq!(
            env.get(&p1).expect("binding must exist").type_,
            Type::Integer
        );
        assert_eq!(env.bindings().len(), 2);

        let mut mutable = env.clone();
        mutable.insert(p3.clone(), Type::String);
        mutable.extend([(p4.clone(), Type::Glyph)]);

        let bindings = mutable.into_bindings();
        assert_eq!(
            bindings.get(&p3).expect("binding must exist").type_,
            Type::String
        );
        assert_eq!(
            bindings.get(&p4).expect("binding must exist").type_,
            Type::Glyph
        );
    }

    #[test]
    fn instantiate_scheme_rewrites_predicates_with_fresh_meta_vars() {
        let mut ctx = InferenceContext::new();
        let scheme = Type::v(0)
            .for_all(1)
            .scheme_with_predicates(vec![TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::v(0)],
            )]);

        let instance = ctx
            .instantiate_scheme(&scheme, Span::Generated)
            .expect("scheme instantiation should succeed");

        let Type::MetaVar(id) = instance.type_ else {
            panic!("expected a fresh metavariable");
        };
        assert_eq!(
            instance.predicates,
            vec![TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::MetaVar(id)]
            )]
        );
    }

    #[test]
    fn generalize_with_predicates_quantifies_predicate_metas() {
        let mut ctx = InferenceContext::new();
        ctx.level = 1;
        let meta = ctx.fresh_meta();
        let scheme = ctx.generalize_with_predicates(
            &Type::func(meta.clone(), meta.clone()),
            0,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![meta])],
        );

        assert_eq!(scheme.type_, Type::func(Type::v(0), Type::v(0)).for_all(1));
        assert_eq!(
            scheme.predicates,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])]
        );
    }

    #[test]
    fn infer_identifier_and_report_unknown_identifier() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let known = Path::new("demo", "known");
        env.insert(known.clone(), Type::Integer);

        let known_term = term(TermKind::Identifier(known));
        let typed = ctx
            .infer_term(&mut env, &known_term, &mut schemes)
            .expect("known identifier should infer");
        assert_eq!(typed.term.type_, Type::Integer);

        let unknown = Path::new("demo", "missing");
        let unknown_term = term(TermKind::Identifier(unknown.clone()));
        assert!(matches!(
            ctx.infer_term(&mut env, &unknown_term, &mut schemes),
            Err(TypeError::UnknownIdentifier { path, .. }) if path == unknown
        ));
    }

    #[test]
    fn infer_struct_literal_and_named_field_access() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();

        let mut literal_fields = IndexMap::new();
        literal_fields.insert(
            "x".to_string().with_span(Span::Generated),
            term(TermKind::Immediate(ImmediateValue::Integer(1))),
        );
        let typed_literal = ctx
            .infer_term(
                &mut env,
                &term(TermKind::Struct(literal_fields)),
                &mut schemes,
            )
            .expect("struct literal should infer");
        assert!(matches!(
            typed_literal.term.type_,
            Type::StructConstraint {
                mode: StructMatch::Exact,
                ..
            }
        ));

        let point_type = Type::Named {
            name: Path::new("demo", "Point"),
            body: Box::new(Type::Struct {
                fields: [
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Boolean),
                ]
                .into_iter()
                .collect(),
            }),
        };
        env.insert(Path::new("demo", "p"), point_type);

        let field_term = term(TermKind::Field {
            of: term(TermKind::Identifier(Path::new("demo", "p"))).into(),
            index: "x".to_string().with_span(Span::Generated),
        });
        let typed_field = ctx
            .infer_term(&mut env, &field_term, &mut schemes)
            .expect("field access should infer");
        assert_eq!(typed_field.term.type_, Type::Integer);
    }

    #[test]
    fn infer_field_access_reports_missing_field() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let point_type = Type::Named {
            name: Path::new("demo", "Point"),
            body: Box::new(Type::Struct {
                fields: [("x".to_string(), Type::Integer)].into_iter().collect(),
            }),
        };
        env.insert(Path::new("demo", "p"), point_type);

        let field_term = term(TermKind::Field {
            of: term(TermKind::Identifier(Path::new("demo", "p"))).into(),
            index: "y".to_string().with_span(Span::Generated),
        });

        assert!(matches!(
            ctx.infer_term(&mut env, &field_term, &mut schemes),
            Err(TypeError::Unification { .. })
        ));
    }

    #[test]
    fn infer_function_captures_use_environment_types() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let capture_path = Path::new("demo", "captured");
        let parameter = Path::new("demo", "x");
        env.insert(capture_path.clone(), Type::Integer);

        let function = term(TermKind::Function {
            parameter_name: parameter.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [(capture_path.clone(), ())].into(),
            body: term(TermKind::Identifier(parameter.clone())).into(),
        });
        let typed = ctx
            .infer_term(&mut env, &function, &mut schemes)
            .expect("function inference should succeed");

        let TermKind::Function { captures, .. } = typed.term.kind else {
            panic!("expected function term");
        };
        assert_eq!(captures, [(capture_path, Type::Integer)].into());
    }

    #[test]
    fn infer_function_reports_unknown_capture() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();

        let function = term(TermKind::Function {
            parameter_name: Path::new("demo", "x").with_span(Span::Generated),
            parameter_type: None,
            captures: [(Path::new("demo", "missing"), ())].into(),
            body: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
        });

        assert!(matches!(
            ctx.infer_term(&mut env, &function, &mut schemes),
            Err(TypeError::UnknownIdentifier { .. })
        ));
    }

    #[test]
    fn infer_inline_wasm_forall_assertion_instantiates() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();

        let asserted_type = type_forall(
            vec![Path::new("demo", "a")],
            Vec::new(),
            type_inst(Path::new("demo", "a"), Vec::new()),
        );
        let inline = term(TermKind::InlineWasm {
            asserted_type,
            definitions: IndexMap::new(),
            instructions: [].into(),
        });
        let typed = ctx
            .infer_term(&mut env, &inline, &mut schemes)
            .expect("inline wasm should infer");

        assert!(matches!(typed.term.type_, Type::MetaVar(_)));
    }

    #[test]
    fn infer_call_collects_predicates_from_callee_and_argument() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let callee = Path::new("demo", "f");
        let argument = Path::new("demo", "x");

        env.insert(
            callee.clone(),
            Type::func(Type::Integer, Type::Integer).scheme_with_predicates(vec![TraitRef::new(
                Path::new("demo", "Show"),
                vec![Type::Integer],
            )]),
        );
        env.insert(
            argument.clone(),
            Type::Integer.scheme_with_predicates(vec![TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::Integer],
            )]),
        );

        let call = term(TermKind::Call {
            callee: term(TermKind::Identifier(callee)).into(),
            argument: term(TermKind::Identifier(argument)).into(),
        });
        let typed = ctx
            .infer_term(&mut env, &call, &mut schemes)
            .expect("call should infer");

        assert_eq!(typed.term.type_, Type::Integer);
        assert_eq!(
            typed.predicates,
            vec![
                TraitRef::new(Path::new("demo", "Show"), vec![Type::Integer]),
                TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]),
            ]
        );
    }

    #[test]
    fn let_scope_controls_environment_extension() {
        let mut ctx = InferenceContext::new();
        let mut schemes = IndexMap::new();
        let binding = Path::new("demo", "value");
        let assignee = pattern(PatternKind::Identifier(binding.clone()));
        let value = term(TermKind::Immediate(ImmediateValue::Integer(1)));

        let mut local_env = TypeEnv::new();
        let local = term(TermKind::Let {
            assignee: assignee.clone(),
            scope: ScopeKind::Local,
            value: value.clone().into(),
            then: term(TermKind::Immediate(ImmediateValue::Unit)).into(),
            else_: term(TermKind::Unreachable).into(),
        });
        ctx.infer_term(&mut local_env, &local, &mut schemes)
            .expect("local let should infer");
        assert!(local_env.get(&binding).is_none());

        let mut global_env = TypeEnv::new();
        let global = term(TermKind::Let {
            assignee,
            scope: ScopeKind::Global,
            value: value.into(),
            then: term(TermKind::Identifier(binding.clone())).into(),
            else_: term(TermKind::Unreachable).into(),
        });
        ctx.infer_term(&mut global_env, &global, &mut schemes)
            .expect("global let should infer");
        assert!(global_env.get(&binding).is_some());
    }

    #[test]
    fn semicolon_requires_left_unit_and_returns_right_type() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();

        let valid = term(TermKind::Semicolon(
            term(TermKind::Immediate(ImmediateValue::Unit)).into(),
            term(TermKind::Immediate(ImmediateValue::Boolean(true))).into(),
        ));
        let typed = ctx
            .infer_term(&mut env, &valid, &mut schemes)
            .expect("semicolon with unit lhs should infer");
        assert_eq!(typed.term.type_, Type::Boolean);

        let invalid = term(TermKind::Semicolon(
            term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
            term(TermKind::Immediate(ImmediateValue::Boolean(true))).into(),
        ));
        assert!(matches!(
            ctx.infer_term(&mut env, &invalid, &mut schemes),
            Err(TypeError::Unification { .. })
        ));
    }

    #[test]
    fn unreachable_infers_fresh_meta_type() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();

        let typed = ctx
            .infer_term(&mut env, &term(TermKind::Unreachable), &mut schemes)
            .expect("unreachable should infer");
        assert!(matches!(typed.term.type_, Type::MetaVar(_)));
    }

    #[test]
    fn infer_pattern_array_struct_and_type_hint() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let env = TypeEnv::new();
        let mut bindings = Vec::new();

        let array_pattern = pattern(PatternKind::Array {
            starting: [pattern(PatternKind::Identifier(Path::new("demo", "head")))].into(),
            glob: Glob::Named(Path::new("demo", "rest")),
            ending: [pattern(PatternKind::Identifier(Path::new("demo", "tail")))].into(),
        });
        infer_pattern(
            &mut ctx,
            &env,
            &array_pattern,
            &Type::Array(Box::new(Type::Integer)),
            &mut bindings,
        )
        .expect("array pattern should infer");

        assert!(bindings.contains(&(Path::new("demo", "head"), Type::Integer)));
        assert!(bindings.contains(&(Path::new("demo", "tail"), Type::Integer)));
        assert!(bindings.contains(&(
            Path::new("demo", "rest"),
            Type::Array(Box::new(Type::Integer)),
        )));

        bindings.clear();
        let struct_pattern = pattern(PatternKind::Struct(
            [(
                "x".to_string().with_span(Span::Generated),
                pattern(PatternKind::Identifier(Path::new("demo", "x"))),
            )]
            .into_iter()
            .collect(),
        ));
        let expected = Type::Named {
            name: Path::new("demo", "Point"),
            body: Box::new(Type::Struct {
                fields: [("x".to_string(), Type::Integer)].into_iter().collect(),
            }),
        };
        infer_pattern(&mut ctx, &env, &struct_pattern, &expected, &mut bindings)
            .expect("struct pattern should infer");
        assert!(
            bindings
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "x"))
        );

        bindings.clear();
        let hint_pattern = pattern(PatternKind::TypeHint(
            Box::new(pattern(PatternKind::Identifier(Path::new("demo", "value")))),
            type_inst(Path::core("integer"), Vec::new()),
        ));
        infer_pattern(&mut ctx, &env, &hint_pattern, &Type::Integer, &mut bindings)
            .expect("type-hinted pattern should infer");
        assert!(bindings.contains(&(Path::new("demo", "value"), Type::Integer)));
    }

    #[test]
    fn infer_pattern_constructors_cover_success_and_errors() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut bindings = Vec::new();
        let none = Path::new("demo", "None");
        let some = Path::new("demo", "Some");

        env.insert(none.clone(), Type::Boolean);
        env.insert(some.clone(), Type::func(Type::Integer, Type::Boolean));

        infer_pattern(
            &mut ctx,
            &env,
            &pattern(PatternKind::ConstConstructor(none.clone())),
            &Type::Boolean,
            &mut bindings,
        )
        .expect("const constructor pattern should infer");

        infer_pattern(
            &mut ctx,
            &env,
            &pattern(PatternKind::Constructor(
                some.clone(),
                Box::new(pattern(PatternKind::Immediate(ImmediateValue::Integer(1)))),
            )),
            &Type::Boolean,
            &mut bindings,
        )
        .expect("constructor pattern should infer");

        assert!(matches!(
            infer_pattern(
                &mut ctx,
                &env,
                &pattern(PatternKind::ConstConstructor(Path::new("demo", "Missing"))),
                &Type::Boolean,
                &mut bindings,
            ),
            Err(TypeError::UnknownConstructor { .. })
        ));

        env.insert(some.clone(), Type::Boolean);
        assert!(matches!(
            infer_pattern(
                &mut ctx,
                &env,
                &pattern(PatternKind::Constructor(
                    some,
                    Box::new(pattern(PatternKind::Identifier(Path::new("demo", "x")))),
                )),
                &Type::Boolean,
                &mut bindings,
            ),
            Err(TypeError::NotAFunction { .. })
        ));
    }

    #[test]
    fn type_expr_to_type_and_scheme_cover_nominal_alias_unknown_and_placeholder() {
        let mut ctx = InferenceContext::new();
        let pair = Path::new("demo", "Pair");
        ctx.set_type_definitions(
            [
                (
                    pair.clone(),
                    TypeDefinition {
                        parameters: 0,
                        body: Type::Tuple(vec![Type::Integer, Type::Boolean]),
                        kind: TypeDefinitionKind::Named,
                    },
                ),
                (
                    Path::new("demo", "PairAlias"),
                    TypeDefinition {
                        parameters: 0,
                        body: Type::Tuple(vec![Type::Integer, Type::Boolean]),
                        kind: TypeDefinitionKind::Alias,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        );

        let named = type_expr_to_type(&mut ctx, &type_inst(pair.clone(), Vec::new()))
            .expect("named type expression should lower");
        assert!(matches!(named, Type::Named { name, .. } if name == pair));

        let alias = type_expr_to_type(
            &mut ctx,
            &type_inst(Path::new("demo", "PairAlias"), Vec::new()),
        )
        .expect("alias expression should lower");
        assert_eq!(alias, Type::Tuple(vec![Type::Integer, Type::Boolean]));

        let unknown = type_expr_to_type(
            &mut ctx,
            &type_inst(Path::new("demo", "Missing"), Vec::new()),
        )
        .expect("unknown type should recover");
        assert!(matches!(unknown, Type::Named { .. }));

        let placeholder = type_expr_to_type(&mut ctx, &type_expr(TypeExprKind::Placeholder))
            .expect("placeholder should lower to fresh meta");
        assert!(matches!(placeholder, Type::MetaVar(_)));

        let scheme = type_expr_to_scheme(
            &mut ctx,
            &type_forall(
                vec![Path::new("demo", "a")],
                vec![type_constraint(
                    Path::new("demo", "Eq"),
                    vec![type_inst(Path::new("demo", "a"), Vec::new())],
                )],
                type_inst(Path::new("demo", "a"), Vec::new()),
            ),
        )
        .expect("forall scheme should lower");
        assert_eq!(scheme.type_, Type::v(0).for_all(1));
        assert_eq!(
            scheme.predicates,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])]
        );
    }

    #[test]
    fn type_expr_lower_error_mapping_covers_all_variants() {
        let type_param_applied = type_expr_lower_error(TypeExprLowerError::TypeParameterApplied {
            name: Path::new("demo", "a"),
            found: 1,
            span: Span::Generated,
        });
        assert!(matches!(
            type_param_applied,
            TypeError::InvalidTypeApplication {
                expected: 0,
                found: 1,
                ..
            }
        ));

        let invalid_application =
            type_expr_lower_error(TypeExprLowerError::InvalidTypeApplication {
                name: Path::new("demo", "Pair"),
                expected: 2,
                found: 1,
                span: Span::Generated,
            });
        assert!(matches!(
            invalid_application,
            TypeError::InvalidTypeApplication {
                expected: 2,
                found: 1,
                ..
            }
        ));

        let invalid_placeholder =
            type_expr_lower_error(TypeExprLowerError::PlaceholderNotAllowed {
                span: Span::Generated,
            });
        assert!(matches!(
            invalid_placeholder,
            TypeError::InvalidPlaceholderType { .. }
        ));

        let constraints_not_allowed =
            type_expr_lower_error(TypeExprLowerError::TraitConstraintsNotAllowed {
                span: Span::Generated,
            });
        assert!(matches!(
            constraints_not_allowed,
            TypeError::TraitConstraintsNotAllowed { .. }
        ));
    }

    #[test]
    fn higher_rank_parameter_requires_annotation_for_lambda_argument() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let use_path = Path::new("demo", "use");
        let f_path = Path::new("demo", "f");
        let x_path = Path::new("demo", "x");

        let use_fn = term(TermKind::Function {
            parameter_name: f_path.clone().with_span(Span::Generated),
            parameter_type: Some(forall_identity_type_expr(Path::new("demo", "a"))),
            captures: [].into(),
            body: term(TermKind::Tuple(vec![
                term(TermKind::Call {
                    callee: term(TermKind::Identifier(f_path.clone())).into(),
                    argument: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
                }),
                term(TermKind::Call {
                    callee: term(TermKind::Identifier(f_path.clone())).into(),
                    argument: term(TermKind::Immediate(ImmediateValue::Boolean(true))).into(),
                }),
            ]))
            .into(),
        });

        let argument = term(TermKind::Function {
            parameter_name: x_path.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Identifier(x_path.clone())).into(),
        });

        let let_term = term(TermKind::Let {
            assignee: pattern(PatternKind::Identifier(use_path.clone())),
            scope: ScopeKind::Local,
            value: use_fn.into(),
            then: term(TermKind::Call {
                callee: term(TermKind::Identifier(use_path)).into(),
                argument: argument.into(),
            })
            .into(),
            else_: term(TermKind::Unreachable).into(),
        });

        assert!(matches!(
            ctx.infer_term(&mut env, &let_term, &mut schemes),
            Err(TypeError::HigherRankAnnotationRequired { parameter, .. }) if parameter == x_path
        ));
    }

    #[test]
    fn higher_rank_parameter_accepts_polymorphic_identifier_argument() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let id_path = Path::new("demo", "id");
        let use_path = Path::new("demo", "use");
        let f_path = Path::new("demo", "f");
        let x_path = Path::new("demo", "x");

        let id_fn = term(TermKind::Function {
            parameter_name: x_path.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Identifier(x_path.clone())).into(),
        });
        let use_fn = term(TermKind::Function {
            parameter_name: f_path.clone().with_span(Span::Generated),
            parameter_type: Some(forall_identity_type_expr(Path::new("demo", "a"))),
            captures: [].into(),
            body: term(TermKind::Tuple(vec![
                term(TermKind::Call {
                    callee: term(TermKind::Identifier(f_path.clone())).into(),
                    argument: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
                }),
                term(TermKind::Call {
                    callee: term(TermKind::Identifier(f_path.clone())).into(),
                    argument: term(TermKind::Immediate(ImmediateValue::Boolean(true))).into(),
                }),
            ]))
            .into(),
        });

        let let_term = term(TermKind::Let {
            assignee: pattern(PatternKind::Identifier(id_path.clone())),
            scope: ScopeKind::Local,
            value: id_fn.into(),
            then: term(TermKind::Let {
                assignee: pattern(PatternKind::Identifier(use_path.clone())),
                scope: ScopeKind::Local,
                value: use_fn.into(),
                then: term(TermKind::Call {
                    callee: term(TermKind::Identifier(use_path)).into(),
                    argument: term(TermKind::Identifier(id_path)).into(),
                })
                .into(),
                else_: term(TermKind::Unreachable).into(),
            })
            .into(),
            else_: term(TermKind::Unreachable).into(),
        });

        let typed = ctx
            .infer_term(&mut env, &let_term, &mut schemes)
            .expect("higher-rank identifier argument should infer");
        assert_eq!(
            typed.term.type_,
            Type::Tuple(vec![Type::Integer, Type::Boolean])
        );
    }

    #[test]
    fn higher_rank_unannotated_parameter_use_fails() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let use_path = Path::new("demo", "use");
        let f_path = Path::new("demo", "f");

        let use_fn = term(TermKind::Function {
            parameter_name: f_path.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Tuple(vec![
                term(TermKind::Call {
                    callee: term(TermKind::Identifier(f_path.clone())).into(),
                    argument: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
                }),
                term(TermKind::Call {
                    callee: term(TermKind::Identifier(f_path.clone())).into(),
                    argument: term(TermKind::Immediate(ImmediateValue::Boolean(true))).into(),
                }),
            ]))
            .into(),
        });

        let let_term = term(TermKind::Let {
            assignee: pattern(PatternKind::Identifier(use_path.clone())),
            scope: ScopeKind::Local,
            value: use_fn.into(),
            then: term(TermKind::Call {
                callee: term(TermKind::Identifier(use_path)).into(),
                argument: term(TermKind::Function {
                    parameter_name: Path::new("demo", "x").with_span(Span::Generated),
                    parameter_type: None,
                    captures: [].into(),
                    body: term(TermKind::Identifier(Path::new("demo", "x"))).into(),
                })
                .into(),
            })
            .into(),
            else_: term(TermKind::Unreachable).into(),
        });

        assert!(matches!(
            ctx.infer_term(&mut env, &let_term, &mut schemes),
            Err(TypeError::HigherRankAnnotationRequired { parameter, .. }) if parameter == f_path
        ));
    }

    #[test]
    fn check_term_requires_annotation_for_forall_expected_parameter() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let expected = Type::func(Type::v(0), Type::v(0)).for_all(1);
        let lambda = term(TermKind::Function {
            parameter_name: Path::new("demo", "x").with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Identifier(Path::new("demo", "x"))).into(),
        });

        assert!(matches!(
            check_term(&mut ctx, &mut env, &lambda, &expected, &mut schemes),
            Err(TypeError::HigherRankAnnotationRequired { .. })
        ));
    }

    #[test]
    fn polymorphic_annotation_requires_and_accepts_constraints() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let source = Path::new("demo", "source");
        let binding = Path::new("demo", "poly");
        let param = Path::new("demo", "a");
        let eq_trait = Path::new("demo", "Eq");

        env.insert(
            source.clone(),
            Type::v(0)
                .for_all(1)
                .scheme_with_predicates(vec![TraitRef::new(eq_trait.clone(), vec![Type::v(0)])]),
        );

        let missing_constraints = term(TermKind::Let {
            assignee: pattern(PatternKind::TypeHint(
                Box::new(pattern(PatternKind::Identifier(binding.clone()))),
                type_forall(
                    vec![param.clone()],
                    Vec::new(),
                    type_inst(param.clone(), Vec::new()),
                ),
            )),
            scope: ScopeKind::Local,
            value: term(TermKind::Identifier(source.clone())).into(),
            then: term(TermKind::Identifier(binding.clone())).into(),
            else_: term(TermKind::Unreachable).into(),
        });
        assert!(matches!(
            ctx.infer_term(&mut env, &missing_constraints, &mut schemes),
            Err(TypeError::PolymorphicAnnotationMissingConstraints { .. })
        ));

        let with_constraints = term(TermKind::Let {
            assignee: pattern(PatternKind::TypeHint(
                Box::new(pattern(PatternKind::Identifier(binding.clone()))),
                type_forall(
                    vec![param.clone()],
                    vec![type_constraint(
                        eq_trait,
                        vec![type_inst(param.clone(), Vec::new())],
                    )],
                    type_inst(param, Vec::new()),
                ),
            )),
            scope: ScopeKind::Local,
            value: term(TermKind::Identifier(source)).into(),
            then: term(TermKind::Identifier(binding)).into(),
            else_: term(TermKind::Unreachable).into(),
        });
        ctx.infer_term(&mut env, &with_constraints, &mut schemes)
            .expect("annotation with constraints should pass");
    }
}

fn unify_with_span(
    unification_table: &mut UnificationTable,
    left: &Type,
    right: &Type,
    span: Span,
) -> Result<(), TypeError> {
    unification_table
        .unify(left, right)
        .map_err(|error| TypeError::Unification { error, span })
}

fn type_contains_forall(type_: &Type) -> bool {
    let mut contains_forall = false;
    for_each_child_type(type_, true, |child| {
        if matches!(child, Type::ForAll(_)) {
            contains_forall = true;
        }
    });
    contains_forall
}

fn first_forall_type_hint(pattern: &Pattern<()>) -> Option<&TypeExpr> {
    match &pattern.kind {
        PatternKind::TypeHint(inner, type_expr) => {
            if matches!(type_expr.kind, TypeExprKind::ForAll(_, _, _)) {
                Some(type_expr)
            } else {
                first_forall_type_hint(inner)
            }
        }
        PatternKind::Constructor(_, payload) => first_forall_type_hint(payload),
        PatternKind::Tuple(items) => items.iter().find_map(first_forall_type_hint),
        PatternKind::Array {
            starting, ending, ..
        } => {
            starting
                .iter()
                .chain(ending.iter())
                .find_map(first_forall_type_hint)
        }
        PatternKind::Struct(fields) => fields.values().find_map(first_forall_type_hint),
        PatternKind::Hole
        | PatternKind::Identifier(_)
        | PatternKind::ConstConstructor(_)
        | PatternKind::Immediate(_) => None,
    }
}

fn predicate_key(predicate: &TraitConstraint) -> String {
    let args = predicate
        .arguments
        .iter()
        .map(Type::pretty)
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        predicate.trait_name.to_string()
    } else {
        format!("{} {args}", predicate.trait_name)
    }
}

fn inferred_predicates_covered_by_annotation(
    inferred: &[TraitConstraint],
    annotation: &[TraitConstraint],
) -> bool {
    let mut inferred_keys = inferred.iter().map(predicate_key).collect::<Vec<_>>();
    inferred_keys.sort();
    inferred_keys.dedup();

    let mut annotation_keys = annotation.iter().map(predicate_key).collect::<Vec<_>>();
    annotation_keys.sort();
    annotation_keys.dedup();

    inferred_keys
        .iter()
        .all(|key| annotation_keys.binary_search(key).is_ok())
}

/// Mapping of term paths to their type schemes.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    bindings: IndexMap<Path, TypeScheme>,
}

impl TypeEnv {
    /// Create an empty type environment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a term binding by path.
    pub fn get(
        &self,
        path: &Path,
    ) -> Option<&TypeScheme> {
        self.bindings.get(path)
    }

    /// Borrow all bindings.
    pub fn bindings(&self) -> &IndexMap<Path, TypeScheme> {
        &self.bindings
    }

    /// Consume this environment into raw bindings.
    pub fn into_bindings(self) -> IndexMap<Path, TypeScheme> {
        self.bindings
    }

    /// Return a cloned environment extended with a single binding.
    pub fn with_binding(
        &self,
        path: Path,
        scheme: impl Into<TypeScheme>,
    ) -> Self {
        let mut next = self.clone();
        next.bindings.insert(path, scheme.into());
        next
    }

    /// Return a cloned environment extended with many bindings.
    pub fn with_bindings<T>(
        &self,
        bindings: impl IntoIterator<Item = (Path, T)>,
    ) -> Self
    where
        T: Into<TypeScheme>,
    {
        let mut next = self.clone();
        next.bindings.extend(
            bindings
                .into_iter()
                .map(|(path, scheme)| (path, scheme.into())),
        );
        next
    }

    /// Insert or replace one binding in place.
    pub fn insert(
        &mut self,
        path: Path,
        scheme: impl Into<TypeScheme>,
    ) {
        self.bindings.insert(path, scheme.into());
    }

    /// Extend this environment in place with many bindings.
    pub fn extend<T>(
        &mut self,
        bindings: impl IntoIterator<Item = (Path, T)>,
    ) where
        T: Into<TypeScheme>,
    {
        self.bindings.extend(
            bindings
                .into_iter()
                .map(|(path, scheme)| (path, scheme.into())),
        );
    }
}

/// Inference state: unification table, level tracking, and known types.
#[derive(Debug, Default)]
pub struct InferenceContext {
    table: UnificationTable,
    level: u32,
    type_definitions: IndexMap<Path, TypeDefinition>,
    skolem_salt: usize,
    unannotated_parameter_argument_types: Vec<(Path, Option<Type>)>,
}

/// Instantiated scheme paired with its trait predicates.
#[derive(Debug, Clone)]
pub struct SchemeInstance {
    pub type_: Type,
    pub predicates: Vec<TraitConstraint>,
}

/// Result of inference for a term, including remaining predicates.
#[derive(Debug, Clone)]
pub struct InferenceOutput {
    pub term: Term<Type>,
    pub predicates: Vec<TraitConstraint>,
}

struct InferredTermCollection {
    items: Box<[Term<Type>]>,
    predicates: Vec<TraitConstraint>,
}

struct InferredPatternCollection {
    items: Box<[Pattern<Type>]>,
    types: Vec<Type>,
}

impl InferenceContext {
    /// Create a fresh inference context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace currently known type definitions used by type-expression lowering.
    pub fn set_type_definitions(
        &mut self,
        definitions: IndexMap<Path, TypeDefinition>,
    ) {
        self.type_definitions = definitions;
    }

    /// Borrow the unification table.
    pub fn table(&self) -> &UnificationTable {
        &self.table
    }

    /// Mutably borrow the unification table.
    pub fn table_mut(&mut self) -> &mut UnificationTable {
        &mut self.table
    }

    /// Allocate a fresh inference metavariable at the current level.
    pub fn fresh_meta(&mut self) -> Type {
        self.table.new_meta(self.level)
    }

    /// Allocate a fresh rigid skolem type used for rank checking.
    fn fresh_skolem(&mut self) -> Type {
        let name = Path::new("[skolem]", format!("#{}", self.skolem_salt));
        self.skolem_salt += 1;
        Type::Named {
            name,
            body: Box::new(Type::Unit),
        }
    }

    /// Replace every leading `for all` binder with fresh skolems.
    fn skolemize_forall(
        &mut self,
        type_: &Type,
        span: Span,
    ) -> Result<Type, TypeError> {
        let mut current = self.table.normalize(type_);
        while let Type::ForAll(body) = current {
            let skolem = self.fresh_skolem();
            current = body
                .open_forall(&skolem)
                .ok_or(TypeError::InvalidScheme { span })?;
        }
        Ok(current)
    }

    /// Enter a function scope for an unannotated parameter.
    fn push_unannotated_parameter(
        &mut self,
        parameter: Path,
    ) {
        self.unannotated_parameter_argument_types
            .push((parameter, None));
    }

    /// Exit the most recent unannotated-parameter scope.
    fn pop_unannotated_parameter(&mut self) {
        self.unannotated_parameter_argument_types.pop();
    }

    /// Record an argument type for calls to an unannotated parameter.
    fn record_unannotated_parameter_argument(
        &mut self,
        parameter: &Path,
        argument_type: &Type,
        span: Span,
    ) -> Result<(), TypeError> {
        let argument_type = self.table.normalize(argument_type);
        let Some((_, seen_argument_type)) = self
            .unannotated_parameter_argument_types
            .iter_mut()
            .rev()
            .find(|(tracked_parameter, _)| tracked_parameter == parameter)
        else {
            return Ok(());
        };
        match seen_argument_type {
            None => {
                *seen_argument_type = Some(argument_type);
                Ok(())
            }
            Some(seen_argument_type) => {
                let mut trial_table = self.table.clone();
                if trial_table
                    .unify(seen_argument_type, &argument_type)
                    .is_err()
                {
                    return Err(TypeError::HigherRankAnnotationRequired {
                        parameter: parameter.clone(),
                        span,
                    });
                }
                *seen_argument_type = argument_type;
                Ok(())
            }
        }
    }

    /// Check whether `path` refers to an in-scope unannotated parameter.
    fn has_unannotated_parameter(
        &self,
        path: &Path,
    ) -> bool {
        self.unannotated_parameter_argument_types
            .iter()
            .rev()
            .any(|(tracked_parameter, _)| tracked_parameter == path)
    }

    pub fn instantiate(
        &mut self,
        scheme: &TypeScheme,
        span: Span,
    ) -> Result<Type, TypeError> {
        Ok(self.instantiate_scheme(scheme, span)?.type_)
    }

    /// Instantiate a scheme and its predicates into fresh metavariables.
    pub fn instantiate_scheme(
        &mut self,
        scheme: &TypeScheme,
        span: Span,
    ) -> Result<SchemeInstance, TypeError> {
        let mut current = scheme.type_.clone();
        let mut predicates = scheme.predicates.clone();
        loop {
            match current {
                Type::ForAll(body) => {
                    let fresh = self.fresh_meta();
                    current = body
                        .open_forall(&fresh)
                        .ok_or(TypeError::InvalidScheme { span })?;
                    predicates = instantiate_predicates(&predicates, std::slice::from_ref(&fresh))
                        .ok_or(TypeError::InvalidScheme { span })?;
                }
                other => {
                    return Ok(SchemeInstance {
                        type_: other,
                        predicates,
                    });
                }
            }
        }
    }

    pub fn generalize_at(
        &mut self,
        type_: &Type,
        level: u32,
    ) -> TypeScheme {
        self.generalize_with_predicates(type_, level, Vec::new())
    }

    /// Generalize all metavariables above `level` into `for all` binders.
    pub fn generalize_with_predicates(
        &mut self,
        type_: &Type,
        level: u32,
        predicates: Vec<TraitConstraint>,
    ) -> TypeScheme {
        let normalized_type = self.table.normalize(type_);
        let normalized_predicates = self.table.normalize_predicates(&predicates);
        let mut free_meta_vars = self.table.free_meta_vars(&normalized_type);
        for predicate in normalized_predicates.iter() {
            for argument in predicate.arguments.iter() {
                free_meta_vars.extend(self.table.free_meta_vars(argument));
            }
        }
        let mut free_meta_vars = free_meta_vars
            .into_iter()
            .filter(|id| {
                self.table
                    .level(*id)
                    .is_some_and(|var_level| var_level > level)
            })
            .collect::<Vec<_>>();
        free_meta_vars.sort_unstable();
        let meta_var_to_type_var = free_meta_vars
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index as u32))
            .collect::<HashMap<_, _>>();
        let type_ = MetaVarToTypeVarSubstitution {
            meta_var_to_type_var: &meta_var_to_type_var,
        }
        .transform(&normalized_type)
        .unwrap_or_else(|| normalized_type.clone())
        .for_all(free_meta_vars.len());
        let predicates =
            replace_meta_vars_in_predicates(&normalized_predicates, &meta_var_to_type_var);
        type_.scheme_with_predicates(predicates)
    }

    /// Infer a term type and collect deferred trait predicates.
    pub fn infer_term(
        &mut self,
        type_environment: &mut TypeEnv,
        term: &Term<()>,
        schemes: &mut IndexMap<Path, TypeScheme>,
    ) -> Result<InferenceOutput, TypeError> {
        infer_term(self, type_environment, term, schemes)
    }
}

/// Infer a semantic type for `term`.
///
/// This is the synthesis entry point; when checking against an expected type,
/// use [`check_term`] instead.
pub fn infer_term(
    ctx: &mut InferenceContext,
    env: &mut TypeEnv,
    term: &Term<()>,
    schemes: &mut IndexMap<Path, TypeScheme>,
) -> Result<InferenceOutput, TypeError> {
    let (kind, type_, predicates) = match &term.kind {
        TermKind::Immediate(value) => {
            (
                TermKind::Immediate(value.clone()),
                value.type_of(),
                Vec::new(),
            )
        }
        TermKind::Identifier(path) => {
            let scheme = env.get(path).ok_or_else(|| {
                TypeError::UnknownIdentifier {
                    path: path.clone(),
                    span: term.span,
                }
            })?;
            let instance = ctx.instantiate_scheme(scheme, term.span)?;
            (
                TermKind::Identifier(path.clone()),
                instance.type_,
                instance.predicates,
            )
        }
        TermKind::Tuple(items) => {
            let InferredTermCollection {
                items: typed_items,
                predicates,
            } = infer_term_items(ctx, env, items, schemes)?;
            let types = typed_items.iter().map(|item| item.type_.clone()).collect();
            (
                TermKind::Tuple(Vec::from(typed_items)),
                Type::Tuple(types),
                predicates,
            )
        }
        TermKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            let mut field_types = IndexMap::new();
            let mut predicates = Vec::new();
            for (name, value) in fields {
                let typed = infer_term(ctx, env, value, schemes)?;
                field_types.insert(name.inner.clone(), typed.term.type_.clone());
                predicates.extend(typed.predicates);
                typed_fields.insert(name.clone(), typed.term);
            }
            (
                TermKind::Struct(typed_fields),
                Type::StructConstraint {
                    fields: field_types,
                    mode: StructMatch::Exact,
                },
                predicates,
            )
        }
        TermKind::Field { of, index } => {
            let typed_of = infer_term(ctx, env, of, schemes)?;
            let field_name = index.inner.clone();
            let field_type = field_access_type(ctx, &typed_of.term.type_, &field_name, index.span)?;
            (
                TermKind::Field {
                    of: typed_of.term.into(),
                    index: index.clone(),
                },
                field_type,
                typed_of.predicates,
            )
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            let param_type = match parameter_type {
                Some(type_expr) => type_expr_to_type(ctx, type_expr)?,
                None => ctx.fresh_meta(),
            };
            let mut env_with_param =
                env.with_binding(parameter_name.inner.clone(), param_type.clone());
            let parameter_is_unannotated = parameter_type.is_none();
            if parameter_is_unannotated {
                ctx.push_unannotated_parameter(parameter_name.inner.clone());
            }
            let typed_body_result = infer_term(ctx, &mut env_with_param, body, schemes);
            if parameter_is_unannotated {
                ctx.pop_unannotated_parameter();
            }
            let typed_body = typed_body_result?;
            let typed_captures = captures
                .iter()
                .map(|(path, _)| {
                    let scheme = env.get(path).ok_or_else(|| {
                        TypeError::UnknownIdentifier {
                            path: path.clone(),
                            span: Span::Generated,
                        }
                    })?;
                    Ok((path.clone(), scheme.type_.clone()))
                })
                .collect::<Result<Vec<_>, TypeError>>()?;
            let type_ = Type::func(param_type, typed_body.term.type_.clone());
            (
                TermKind::Function {
                    parameter_name: parameter_name.clone(),
                    parameter_type: parameter_type.clone(),
                    captures: typed_captures.into_boxed_slice(),
                    body: typed_body.term.into(),
                },
                type_,
                typed_body.predicates,
            )
        }
        TermKind::InlineWasm {
            asserted_type,
            definitions,
            instructions,
        } => {
            let asserted_type_value = type_expr_to_type(ctx, asserted_type)?;
            let inferred_type = match asserted_type_value {
                forall @ Type::ForAll(_) => {
                    let scheme = TypeScheme::new(forall);
                    ctx.instantiate(&scheme, asserted_type.span)?
                }
                other => other,
            };
            (
                TermKind::InlineWasm {
                    asserted_type: asserted_type.clone(),
                    definitions: definitions.clone(),
                    instructions: instructions.clone(),
                },
                inferred_type,
                Vec::new(),
            )
        }
        TermKind::Call { callee, argument } => {
            let typed_callee = infer_term(ctx, env, callee, schemes)?;
            let parameter_type = ctx.fresh_meta();
            let result_type = ctx.fresh_meta();
            let function_type = Type::func(parameter_type.clone(), result_type.clone());
            unify_with_span(
                &mut ctx.table,
                &typed_callee.term.type_,
                &function_type,
                term.span,
            )?;
            let expected_argument_type = ctx.table.normalize(&parameter_type);
            let typed_argument =
                match check_term(ctx, env, argument, &expected_argument_type, schemes) {
                    Ok(typed_argument) => typed_argument,
                    Err(error @ TypeError::Unification { .. }) => {
                        if let TermKind::Identifier(path) = &typed_callee.term.kind
                            && ctx.has_unannotated_parameter(path)
                        {
                            return Err(TypeError::HigherRankAnnotationRequired {
                                parameter: path.clone(),
                                span: argument.span,
                            });
                        }
                        return Err(error);
                    }
                    Err(other) => return Err(other),
                };
            if let TermKind::Identifier(path) = &typed_callee.term.kind {
                ctx.record_unannotated_parameter_argument(
                    path,
                    &typed_argument.term.type_,
                    typed_argument.term.span,
                )?;
            }
            let mut predicates = typed_callee.predicates;
            predicates.extend(typed_argument.predicates);
            (
                TermKind::Call {
                    callee: typed_callee.term.into(),
                    argument: typed_argument.term.into(),
                },
                ctx.table.normalize(&result_type),
                predicates,
            )
        }
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            let outer_level = ctx.level;
            ctx.level += 1;
            let typed_value = infer_term(ctx, env, value, schemes)?;
            if let Some(type_expr) = first_forall_type_hint(assignee) {
                let annotation = type_expr_to_scheme(ctx, type_expr)?;
                let inferred = ctx.generalize_with_predicates(
                    &typed_value.term.type_,
                    outer_level,
                    typed_value.predicates.clone(),
                );
                if !inferred_predicates_covered_by_annotation(
                    &inferred.predicates,
                    &annotation.predicates,
                ) {
                    return Err(TypeError::PolymorphicAnnotationMissingConstraints {
                        predicates: inferred.predicates,
                        span: type_expr.span,
                    });
                }
            }
            let mut bindings = Vec::new();
            let typed_pattern =
                infer_pattern(ctx, env, assignee, &typed_value.term.type_, &mut bindings)?;
            ctx.level = outer_level;

            let generalized = bindings
                .into_iter()
                .map(|(path, type_)| {
                    (
                        path,
                        ctx.generalize_with_predicates(
                            &type_,
                            outer_level,
                            typed_value.predicates.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>();
            schemes.extend(generalized.iter().cloned());
            let mut env_with = env.with_bindings(generalized.clone());
            let typed_then = infer_term(ctx, &mut env_with, then, schemes)?;
            let typed_else = infer_term(ctx, env, else_, schemes)?;
            unify_with_span(
                &mut ctx.table,
                &typed_then.term.type_,
                &typed_else.term.type_,
                term.span,
            )?;
            let result_type = ctx.table.normalize(&typed_then.term.type_);
            let mut predicates = typed_then.predicates;
            predicates.extend(typed_else.predicates);
            if *scope == ScopeKind::Global {
                env.extend(generalized);
            }
            (
                TermKind::Let {
                    assignee: typed_pattern,
                    scope: *scope,
                    value: typed_value.term.into(),
                    then: typed_then.term.into(),
                    else_: typed_else.term.into(),
                },
                result_type,
                predicates,
            )
        }
        TermKind::Semicolon(left, right) => {
            let typed_left = infer_term(ctx, env, left, schemes)?;
            let typed_right = infer_term(ctx, env, right, schemes)?;
            unify_with_span(
                &mut ctx.table,
                &typed_left.term.type_,
                &Type::Unit,
                typed_left.term.span,
            )?;
            let result_type = typed_right.term.type_.clone();
            let mut predicates = typed_left.predicates;
            predicates.extend(typed_right.predicates);
            (
                TermKind::Semicolon(typed_left.term.into(), typed_right.term.into()),
                result_type,
                predicates,
            )
        }
        TermKind::Unreachable => (TermKind::Unreachable, ctx.fresh_meta(), Vec::new()),
    };

    let normalized = ctx.table.normalize(&type_);
    let predicates = ctx.table.normalize_predicates(&predicates);
    Ok(InferenceOutput {
        term: Term {
            comments: term.comments.clone(),
            kind,
            span: term.span,
            type_: normalized,
        },
        predicates,
    })
}

/// Check `term` against an expected type.
///
/// Leading `for all` binders are skolemized to enforce predicative higher-rank
/// checking.
fn check_term(
    inference_context: &mut InferenceContext,
    type_environment: &mut TypeEnv,
    term: &Term<()>,
    expected: &Type,
    schemes: &mut IndexMap<Path, TypeScheme>,
) -> Result<InferenceOutput, TypeError> {
    let normalized_expected = inference_context.table.normalize(expected);
    if let TermKind::Function {
        parameter_name,
        parameter_type: None,
        ..
    } = &term.kind
        && matches!(&normalized_expected, Type::ForAll(_))
    {
        return Err(TypeError::HigherRankAnnotationRequired {
            parameter: parameter_name.inner.clone(),
            span: term.span,
        });
    }
    let expected = inference_context.skolemize_forall(&normalized_expected, term.span)?;
    if let (
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        },
        Type::Function(expected_parameter, expected_result),
    ) = (&term.kind, &expected)
    {
        return check_function_term(
            inference_context,
            type_environment,
            term,
            parameter_name,
            parameter_type.as_ref(),
            captures,
            body,
            expected_parameter,
            expected_result,
            schemes,
        );
    }

    let mut inferred = infer_term(inference_context, type_environment, term, schemes)?;
    unify_with_span(
        &mut inference_context.table,
        &inferred.term.type_,
        &expected,
        term.span,
    )?;
    inferred.term.type_ = inference_context.table.normalize(&expected);
    inferred.predicates = inference_context
        .table
        .normalize_predicates(&inferred.predicates);
    Ok(inferred)
}

#[allow(clippy::too_many_arguments)]
/// Specialized checker for function terms under an expected function type.
fn check_function_term(
    inference_context: &mut InferenceContext,
    type_environment: &mut TypeEnv,
    term: &Term<()>,
    parameter_name: &crate::Spanned<Path>,
    parameter_type_expr: Option<&TypeExpr>,
    captures: &[(Path, ())],
    body: &Term<()>,
    expected_parameter: &Type,
    expected_result: &Type,
    schemes: &mut IndexMap<Path, TypeScheme>,
) -> Result<InferenceOutput, TypeError> {
    let parameter_type = match parameter_type_expr {
        Some(type_expr) => {
            let annotated = type_expr_to_type(inference_context, type_expr)?;
            unify_with_span(
                &mut inference_context.table,
                &annotated,
                expected_parameter,
                type_expr.span,
            )?;
            inference_context.table.normalize(&annotated)
        }
        None => {
            let normalized_expected_parameter =
                inference_context.table.normalize(expected_parameter);
            if type_contains_forall(&normalized_expected_parameter) {
                return Err(TypeError::HigherRankAnnotationRequired {
                    parameter: parameter_name.inner.clone(),
                    span: term.span,
                });
            }
            normalized_expected_parameter
        }
    };
    let mut environment_with_parameter =
        type_environment.with_binding(parameter_name.inner.clone(), parameter_type.clone());
    let typed_body = check_term(
        inference_context,
        &mut environment_with_parameter,
        body,
        expected_result,
        schemes,
    )?;
    let typed_captures = captures
        .iter()
        .map(|(path, _)| {
            let scheme = type_environment.get(path).ok_or_else(|| {
                TypeError::UnknownIdentifier {
                    path: path.clone(),
                    span: Span::Generated,
                }
            })?;
            Ok((path.clone(), scheme.type_.clone()))
        })
        .collect::<Result<Vec<_>, TypeError>>()?;
    let expected_type = Type::func(expected_parameter.clone(), expected_result.clone());
    let value_type = Type::func(parameter_type, typed_body.term.type_.clone());
    unify_with_span(
        &mut inference_context.table,
        &value_type,
        &expected_type,
        term.span,
    )?;
    let normalized_type = inference_context.table.normalize(&expected_type);
    let predicates = inference_context
        .table
        .normalize_predicates(&typed_body.predicates);
    Ok(InferenceOutput {
        term: Term {
            comments: term.comments.clone(),
            kind: TermKind::Function {
                parameter_name: parameter_name.clone(),
                parameter_type: parameter_type_expr.cloned(),
                captures: typed_captures.into_boxed_slice(),
                body: typed_body.term.into(),
            },
            span: term.span,
            type_: normalized_type,
        },
        predicates,
    })
}

/// Infer and validate a pattern against `expected`, collecting bound names.
fn infer_pattern(
    ctx: &mut InferenceContext,
    env: &TypeEnv,
    pattern: &Pattern<()>,
    expected: &Type,
    bindings: &mut Vec<(Path, Type)>,
) -> Result<Pattern<Type>, TypeError> {
    match &pattern.kind {
        PatternKind::Hole => {
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Hole,
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Identifier(path) => {
            bindings.push((path.clone(), ctx.table.normalize(expected)));
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Identifier(path.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Immediate(value) => {
            let type_ = value.type_of();
            unify_with_span(&mut ctx.table, expected, &type_, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Immediate(value.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Tuple(items) => {
            let InferredPatternCollection {
                items: typed_items,
                types: item_types,
            } = infer_pattern_items(ctx, env, items, bindings)?;
            let tuple_type = Type::Tuple(item_types);
            unify_with_span(&mut ctx.table, expected, &tuple_type, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Tuple(typed_items),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let element_type = ctx.fresh_meta();
            let array_type = Type::Array(Box::new(element_type.clone()));
            unify_with_span(&mut ctx.table, expected, &array_type, pattern.span)?;
            let mut typed_start = Vec::with_capacity(starting.len());
            let mut typed_end = Vec::with_capacity(ending.len());
            for item in starting.iter() {
                let typed_item = infer_pattern(ctx, env, item, &element_type, bindings)?;
                typed_start.push(typed_item);
            }
            for item in ending.iter() {
                let typed_item = infer_pattern(ctx, env, item, &element_type, bindings)?;
                typed_end.push(typed_item);
            }
            if let Glob::Named(path) = glob {
                bindings.push((path.clone(), ctx.table.normalize(&array_type)));
            }
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Array {
                    starting: typed_start.into_boxed_slice(),
                    glob: glob.clone(),
                    ending: typed_end.into_boxed_slice(),
                },
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            let mut field_types = IndexMap::new();
            for (name, value) in fields.iter() {
                let field_type = ctx.fresh_meta();
                let typed_value = infer_pattern(ctx, env, value, &field_type, bindings)?;
                field_types.insert(name.inner.clone(), field_type);
                typed_fields.insert(name.clone(), typed_value);
            }
            let struct_type = Type::StructConstraint {
                fields: field_types,
                mode: StructMatch::Exact,
            };
            unify_with_span(&mut ctx.table, expected, &struct_type, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Struct(typed_fields),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::ConstConstructor(path) => {
            let scheme = env.get(path).ok_or_else(|| {
                TypeError::UnknownConstructor {
                    path: path.clone(),
                    span: pattern.span,
                }
            })?;
            let type_ = ctx.instantiate(scheme, pattern.span)?;
            unify_with_span(&mut ctx.table, expected, &type_, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::ConstConstructor(path.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Constructor(path, payload) => {
            let scheme = env.get(path).ok_or_else(|| {
                TypeError::UnknownConstructor {
                    path: path.clone(),
                    span: pattern.span,
                }
            })?;
            let type_ = ctx.instantiate(scheme, pattern.span)?;
            let (param_type, result_type) = match ctx.table.normalize(&type_) {
                Type::Function(parameter, result) => (*parameter, *result),
                other => {
                    return Err(TypeError::NotAFunction {
                        type_: other,
                        span: pattern.span,
                    });
                }
            };
            unify_with_span(&mut ctx.table, expected, &result_type, pattern.span)?;
            let typed_payload = infer_pattern(ctx, env, payload, &param_type, bindings)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Constructor(path.clone(), Box::new(typed_payload)),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::TypeHint(inner, type_expr) => {
            let hint_scheme = type_expr_to_scheme(ctx, type_expr)?;
            let hint_type = hint_scheme.type_;
            let expected_type = ctx.table.normalize(expected);
            let hint_type = match (hint_type, expected_type) {
                (forall @ Type::ForAll(_), Type::ForAll(_)) => forall,
                (forall @ Type::ForAll(_), _) => {
                    let scheme = TypeScheme::new(forall);
                    ctx.instantiate(&scheme, type_expr.span)?
                }
                (other, _) => other,
            };
            unify_with_span(&mut ctx.table, expected, &hint_type, type_expr.span)?;
            let typed_inner = infer_pattern(ctx, env, inner, &hint_type, bindings)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::TypeHint(Box::new(typed_inner), type_expr.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
    }
}

/// Infer the resulting type of `type_.field_name` field access.
fn field_access_type(
    ctx: &mut InferenceContext,
    type_: &Type,
    field_name: &str,
    span: Span,
) -> Result<Type, TypeError> {
    let field_type = ctx.fresh_meta();
    let mut fields = IndexMap::new();
    fields.insert(field_name.to_string(), field_type.clone());
    let constraint = Type::StructConstraint {
        fields,
        mode: StructMatch::AtLeast,
    };
    unify_with_span(&mut ctx.table, type_, &constraint, span)?;
    Ok(field_type)
}

/// Lower a parsed type expression in inference context.
fn type_expr_to_type(
    ctx: &mut InferenceContext,
    expr: &TypeExpr,
) -> Result<Type, TypeError> {
    let type_definitions = ctx.type_definitions.clone();
    let lowered = lower_type_expr(
        expr,
        &mut |path| {
            type_definitions
                .get(path)
                .cloned()
                .map(TypeExprSymbol::Definition)
                .unwrap_or(TypeExprSymbol::Unknown)
        },
        &mut |_| Some(ctx.fresh_meta()),
    );
    lowered
        .errors
        .into_iter()
        .next()
        .map_or(Ok(lowered.type_), |error| Err(type_expr_lower_error(error)))
}

fn type_expr_to_scheme(
    ctx: &mut InferenceContext,
    expr: &TypeExpr,
) -> Result<TypeScheme, TypeError> {
    let type_definitions = ctx.type_definitions.clone();
    let lowered = lower_type_scheme_expr(
        expr,
        &mut |path| {
            type_definitions
                .get(path)
                .cloned()
                .map(TypeExprSymbol::Definition)
                .unwrap_or(TypeExprSymbol::Unknown)
        },
        &mut |_| Some(ctx.fresh_meta()),
    );
    lowered
        .errors
        .into_iter()
        .next()
        .map_or(Ok(lowered.scheme), |error| {
            Err(type_expr_lower_error(error))
        })
}

/// Convert shared type-expression lowering errors into inference errors.
fn type_expr_lower_error(error: TypeExprLowerError) -> TypeError {
    match error {
        TypeExprLowerError::TypeParameterApplied { name, found, span } => {
            TypeError::InvalidTypeApplication {
                name,
                expected: 0,
                found,
                span,
            }
        }
        TypeExprLowerError::InvalidTypeApplication {
            name,
            expected,
            found,
            span,
        } => {
            TypeError::InvalidTypeApplication {
                name,
                expected,
                found,
                span,
            }
        }
        TypeExprLowerError::PlaceholderNotAllowed { span } => {
            TypeError::InvalidPlaceholderType { span }
        }
        TypeExprLowerError::TraitConstraintsNotAllowed { span } => {
            TypeError::TraitConstraintsNotAllowed { span }
        }
    }
}

/// Infer a list of terms and accumulate all predicates.
fn infer_term_items(
    inference_context: &mut InferenceContext,
    type_environment: &mut TypeEnv,
    items: &[Term<()>],
    schemes: &mut IndexMap<Path, TypeScheme>,
) -> Result<InferredTermCollection, TypeError> {
    items
        .iter()
        .try_fold(
            (Vec::with_capacity(items.len()), Vec::new()),
            |(mut typed_items, mut predicates), item| {
                let typed = infer_term(inference_context, type_environment, item, schemes)?;
                predicates.extend(typed.predicates);
                typed_items.push(typed.term);
                Ok((typed_items, predicates))
            },
        )
        .map(|(items, predicates)| {
            InferredTermCollection {
                items: items.into_boxed_slice(),
                predicates,
            }
        })
}

/// Infer a list of patterns and return both typed patterns and their expected element types.
fn infer_pattern_items(
    inference_context: &mut InferenceContext,
    type_environment: &TypeEnv,
    items: &[Pattern<()>],
    bindings: &mut Vec<(Path, Type)>,
) -> Result<InferredPatternCollection, TypeError> {
    items
        .iter()
        .try_fold(
            (
                Vec::with_capacity(items.len()),
                Vec::with_capacity(items.len()),
            ),
            |(mut typed_items, mut item_types), item| {
                let item_type = inference_context.fresh_meta();
                let typed_item = infer_pattern(
                    inference_context,
                    type_environment,
                    item,
                    &item_type,
                    bindings,
                )?;
                item_types.push(item_type);
                typed_items.push(typed_item);
                Ok((typed_items, item_types))
            },
        )
        .map(|(items, types)| {
            InferredPatternCollection {
                items: items.into_boxed_slice(),
                types,
            }
        })
}

/// Transformation that rewrites generalized metavariables to bound type vars.
struct MetaVarToTypeVarSubstitution<'a> {
    meta_var_to_type_var: &'a HashMap<MetaVarId, u32>,
}

impl TypeTransform for MetaVarToTypeVarSubstitution<'_> {
    fn meta_var(
        &mut self,
        id: MetaVarId,
    ) -> Option<Type> {
        Some(
            self.meta_var_to_type_var
                .get(&id)
                .map(|index| Type::v(*index))
                .unwrap_or_else(|| Type::MetaVar(id)),
        )
    }
}

fn replace_meta_vars_in_predicates(
    predicates: &[TraitConstraint],
    meta_var_to_type_var: &HashMap<MetaVarId, u32>,
) -> Vec<TraitConstraint> {
    let mut replacer = MetaVarToTypeVarSubstitution {
        meta_var_to_type_var,
    };
    predicates
        .iter()
        .map(|predicate| {
            TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments: predicate
                    .arguments
                    .iter()
                    .map(|arg| replacer.transform(arg).unwrap_or_else(|| arg.clone()))
                    .collect(),
            }
        })
        .collect()
}
