//! Type inference and bidirectional type checking.
//!
//! This module implements Hindley-Milner style inference with:
//! - let-generalization,
//! - trait-predicate accumulation,
//! - explicit higher-rank checks via skolemization in checking mode.

use std::collections::{
    HashMap,
    HashSet,
};

use indexmap::IndexMap;

use crate::Span;
use crate::ir::{
    Glob,
    ImmediateValue,
    Path,
    Pattern,
    PatternKind,
    ScopeKind,
    Term,
    TermKind,
    TypeExpr,
    TypeExprKind,
};

use super::instantiation::{
    instantiate_forall_strict,
    instantiate_predicates,
    peel_leading_foralls,
};
use super::kind::{
    constructor_kind,
    infer_scheme_kind,
};
use super::type_expr::{
    TypeExprLowerError,
    TypeExprSymbol,
    lower_type_expr,
    lower_type_scheme_expr,
};
use super::{
    Kind,
    MetaVarId,
    StructMatch,
    TraitConstraint,
    TraitRef,
    Type,
    TypeDefinition,
    TypeScheme,
    TypeTransform,
    for_each_child_type,
    for_each_pattern_binding,
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
    /// Trait constraint application arity mismatch.
    InvalidTraitApplication {
        name: Path,
        expected: usize,
        found: usize,
        span: Span,
    },
    /// Kind mismatch.
    KindMismatch {
        expected: Kind,
        found: Kind,
        span: Span,
    },
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
    /// Pattern set does not cover every possible value.
    NonExhaustivePatterns { span: Span, counterexample: String },
    /// Unification failure with source span context.
    Unification {
        error: UnifyError,
        span: Span,
        context: Option<&'static str>,
    },
}

fn unify_with_context(
    unification_table: &mut UnificationTable,
    left: &Type,
    right: &Type,
    span: Span,
    context: &'static str,
) -> Result<(), TypeError> {
    unification_table.unify(left, right).map_err(|error| {
        TypeError::Unification {
            error,
            span,
            context: Some(context),
        }
    })
}

#[cfg(test)]
mod tests {
    use enum_iterator::all;

    use crate::hc_core::CoreType;
    use crate::ir::{
        Glob,
        ImmediateValue,
        ScopeKind,
        TypeExprConstraint,
        TypeExprKind,
    };
    use crate::types::TypeDefinitionKind;
    use crate::types::symbol_table::Symbol;
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
                CoreType::Function.path(),
                vec![
                    type_inst(param.clone(), Vec::new()),
                    type_inst(param, Vec::new()),
                ],
            ),
        )
    }

    fn core_type_definitions() -> IndexMap<Path, TypeDefinition> {
        all::<CoreType>().map(|t| (t.path(), t.typedef())).collect()
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
    fn canonical_trait_name_returns_none_for_alias_cycles() {
        let mut ctx = InferenceContext::new();
        let eq = Path::new("demo", "Eq");
        let equal = Path::new("demo", "Equal");
        ctx.set_trait_aliases(
            [(eq.clone(), equal.clone()), (equal.clone(), eq.clone())]
                .into_iter()
                .collect(),
        );

        assert_eq!(ctx.canonical_trait_name(&eq), None);
        assert_eq!(ctx.canonical_trait_name(&equal), None);
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
    fn if_desugared_let_preserves_condition_predicates() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let condition = Path::new("demo", "condition");
        let condition_predicate = TraitRef::new(Path::new("demo", "Compare"), vec![Type::Integer]);

        env.insert(
            condition.clone(),
            Type::Boolean.scheme_with_predicates(vec![condition_predicate.clone()]),
        );

        let if_desugared = term(TermKind::Let {
            assignee: pattern(PatternKind::Immediate(ImmediateValue::Boolean(true))),
            scope: ScopeKind::Local,
            value: term(TermKind::Identifier(condition)).into(),
            then: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
            else_: term(TermKind::Immediate(ImmediateValue::Integer(0))).into(),
        });

        let typed = ctx
            .infer_term(&mut env, &if_desugared, &mut schemes)
            .expect("if-desugared let should infer");

        assert_eq!(typed.term.type_, Type::Integer);
        assert_eq!(typed.predicates, vec![condition_predicate]);
    }

    #[test]
    fn let_binding_is_recursive_while_inferring_value() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let binding = Path::new("demo", "loop");
        let parameter = Path::new("demo", "x");

        let recursive_value = term(TermKind::Function {
            parameter_name: parameter.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Call {
                callee: term(TermKind::Identifier(binding.clone())).into(),
                argument: term(TermKind::Identifier(parameter.clone())).into(),
            })
            .into(),
        });
        let let_term = term(TermKind::Let {
            assignee: pattern(PatternKind::Identifier(binding.clone())),
            scope: ScopeKind::Local,
            value: recursive_value.into(),
            then: term(TermKind::Identifier(binding)).into(),
            else_: term(TermKind::Unreachable).into(),
        });

        let typed = ctx
            .infer_term(&mut env, &let_term, &mut schemes)
            .expect("recursive let should infer");
        assert!(matches!(typed.term.type_, Type::Function(_, _)));
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
            type_inst(CoreType::Integer.path(), Vec::new()),
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
        let vec2 = Path::new("demo", "Vec2");

        env.insert(none.clone(), Type::Boolean);
        env.insert(some.clone(), Type::func(Type::Integer, Type::Boolean));
        env.insert(
            vec2.clone(),
            Type::func(
                Type::Struct {
                    fields: [
                        ("x".to_string(), Type::Integer),
                        ("y".to_string(), Type::Integer),
                    ]
                    .into_iter()
                    .collect(),
                },
                Type::Boolean,
            ),
        );

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

        bindings.clear();
        infer_pattern(
            &mut ctx,
            &env,
            &pattern(PatternKind::Constructor(
                vec2,
                Box::new(pattern(PatternKind::Struct(
                    [
                        (
                            "x".to_string().with_span(Span::Generated),
                            pattern(PatternKind::Identifier(Path::new("demo", "x"))),
                        ),
                        (
                            "y".to_string().with_span(Span::Generated),
                            pattern(PatternKind::Identifier(Path::new("demo", "y"))),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ))),
            )),
            &Type::Boolean,
            &mut bindings,
        )
        .expect("constructor struct payload pattern should infer");
        let normalized_bindings = bindings
            .iter()
            .map(|(path, type_)| (path.clone(), ctx.table.normalize(type_)))
            .collect::<Vec<_>>();
        assert!(normalized_bindings.contains(&(Path::new("demo", "x"), Type::Integer)));
        assert!(normalized_bindings.contains(&(Path::new("demo", "y"), Type::Integer)));

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
                        parameter_kinds: Vec::new(),
                        body: Type::Tuple(vec![Type::Integer, Type::Boolean]),
                        kind: TypeDefinitionKind::Named,
                    },
                ),
                (
                    Path::new("demo", "PairAlias"),
                    TypeDefinition {
                        parameters: 0,
                        parameter_kinds: Vec::new(),
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
    fn higher_rank_parameter_accepts_unannotated_lambda_argument() {
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

        let typed = ctx
            .infer_term(&mut env, &let_term, &mut schemes)
            .expect("higher-rank unannotated lambda argument should infer");
        assert_eq!(
            typed.term.type_,
            Type::Tuple(vec![Type::Integer, Type::Boolean])
        );
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
    fn check_term_accepts_unannotated_lambda_for_forall_expected_parameter() {
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

        let checked = check_term(&mut ctx, &mut env, &lambda, &expected, &mut schemes)
            .expect("unannotated lambda should check against forall expected type");
        match checked.term.type_ {
            Type::Function(parameter, result) => {
                assert_eq!(parameter, result, "identity function should preserve type");
            }
            other => panic!("expected function type, got {other}"),
        }
    }

    #[test]
    fn check_term_requires_annotation_for_forall_parameter_type() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let parameter = Path::new("demo", "x");
        let expected = Type::func(Type::func(Type::v(0), Type::v(0)).for_all(1), Type::Integer);
        let lambda = term(TermKind::Function {
            parameter_name: parameter.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Immediate(ImmediateValue::Integer(0))).into(),
        });

        assert!(matches!(
            check_term(&mut ctx, &mut env, &lambda, &expected, &mut schemes),
            Err(TypeError::HigherRankAnnotationRequired { parameter: found, .. }) if found == parameter
        ));
    }

    #[test]
    fn type_contains_forall_detects_root_and_ignores_named_bodies() {
        let root_forall = Type::func(Type::v(0), Type::v(0)).for_all(1);
        let nested_forall = Type::Tuple(vec![Type::Integer, root_forall.clone()]);
        let nominal_with_forall_body = Type::Named {
            name: Path::new("demo", "Box"),
            body: Box::new(root_forall.clone()),
        };

        assert!(type_contains_forall(&root_forall));
        assert!(type_contains_forall(&nested_forall));
        assert!(!type_contains_forall(&nominal_with_forall_body));
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

    #[test]
    fn strict_forall_annotation_rejects_monomorphic_value() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let binding = Path::new("demo", "a");
        let param = Path::new("demo", "t");

        let let_term = term(TermKind::Let {
            assignee: pattern(PatternKind::TypeHint(
                Box::new(pattern(PatternKind::Identifier(binding.clone()))),
                type_forall(
                    vec![param.clone()],
                    Vec::new(),
                    type_inst(param, Vec::new()),
                ),
            )),
            scope: ScopeKind::Local,
            value: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
            then: term(TermKind::Identifier(binding)).into(),
            else_: term(TermKind::Unreachable).into(),
        });

        assert!(matches!(
            ctx.infer_term(&mut env, &let_term, &mut schemes),
            Err(TypeError::Unification { .. })
        ));
    }

    #[test]
    fn top_level_forall_annotation_preserves_display_names_in_generalized_scheme() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let binding = Path::new("demo", "id");
        let forall_param = Path::new("demo", "item");
        let value_param = Path::new("demo", "x");

        let let_term = term(TermKind::Let {
            assignee: pattern(PatternKind::TypeHint(
                Box::new(pattern(PatternKind::Identifier(binding.clone()))),
                forall_identity_type_expr(forall_param),
            )),
            scope: ScopeKind::Local,
            value: term(TermKind::Function {
                parameter_name: value_param.clone().with_span(Span::Generated),
                parameter_type: None,
                captures: [].into(),
                body: term(TermKind::Identifier(value_param)).into(),
            })
            .into(),
            then: term(TermKind::Identifier(binding.clone())).into(),
            else_: term(TermKind::Unreachable).into(),
        });

        ctx.infer_term(&mut env, &let_term, &mut schemes)
            .expect("let with forall annotation should infer");

        let scheme = schemes
            .get(&binding)
            .expect("generalized scheme should be recorded");
        let Type::ForAll { name, .. } = &scheme.type_ else {
            panic!("expected leading forall in generalized scheme")
        };
        assert_eq!(name.as_deref(), Some("item"));
    }

    #[test]
    fn polymorphic_annotation_checks_all_nested_forall_hints() {
        let mut ctx = InferenceContext::new();
        ctx.set_type_definitions(core_type_definitions());
        let mut env = TypeEnv::new();
        let mut schemes = IndexMap::new();
        let source = Path::new("demo", "source");
        let binding = Path::new("demo", "poly");
        let outer_param = Path::new("demo", "a");
        let inner_param = Path::new("demo", "b");
        let eq_trait = Path::new("demo", "Eq");

        env.insert(
            source.clone(),
            Type::v(0)
                .for_all(1)
                .scheme_with_predicates(vec![TraitRef::new(eq_trait.clone(), vec![Type::v(0)])]),
        );

        let assignee = pattern(PatternKind::TypeHint(
            Box::new(pattern(PatternKind::TypeHint(
                Box::new(pattern(PatternKind::Identifier(binding.clone()))),
                type_forall(
                    vec![inner_param.clone()],
                    Vec::new(),
                    type_inst(inner_param.clone(), Vec::new()),
                ),
            ))),
            type_forall(
                vec![outer_param.clone()],
                vec![type_constraint(
                    eq_trait.clone(),
                    vec![type_inst(outer_param.clone(), Vec::new())],
                )],
                type_inst(outer_param.clone(), Vec::new()),
            ),
        ));

        let missing_nested_constraints = term(TermKind::Let {
            assignee,
            scope: ScopeKind::Local,
            value: term(TermKind::Identifier(source.clone())).into(),
            then: term(TermKind::Identifier(binding.clone())).into(),
            else_: term(TermKind::Unreachable).into(),
        });
        assert!(matches!(
            ctx.infer_term(&mut env, &missing_nested_constraints, &mut schemes),
            Err(TypeError::PolymorphicAnnotationMissingConstraints { .. })
        ));

        let assignee = pattern(PatternKind::TypeHint(
            Box::new(pattern(PatternKind::TypeHint(
                Box::new(pattern(PatternKind::Identifier(binding.clone()))),
                type_forall(
                    vec![inner_param.clone()],
                    vec![type_constraint(
                        eq_trait.clone(),
                        vec![type_inst(inner_param.clone(), Vec::new())],
                    )],
                    type_inst(inner_param, Vec::new()),
                ),
            ))),
            type_forall(
                vec![outer_param.clone()],
                vec![type_constraint(
                    eq_trait,
                    vec![type_inst(outer_param.clone(), Vec::new())],
                )],
                type_inst(outer_param, Vec::new()),
            ),
        ));
        let with_nested_constraints = term(TermKind::Let {
            assignee,
            scope: ScopeKind::Local,
            value: term(TermKind::Identifier(source)).into(),
            then: term(TermKind::Identifier(binding)).into(),
            else_: term(TermKind::Unreachable).into(),
        });
        ctx.infer_term(&mut env, &with_nested_constraints, &mut schemes)
            .expect("all nested polymorphic hints should validate constraints");
    }
}

fn type_contains_forall(type_: &Type) -> bool {
    matches!(type_, Type::ForAll { .. }) || {
        let mut contains_forall = false;
        for_each_child_type(type_, false, |child| {
            if !contains_forall && type_contains_forall(child) {
                contains_forall = true;
            }
        });
        contains_forall
    }
}

fn top_level_forall_type_hint(pattern: &Pattern<()>) -> Option<&TypeExpr> {
    match &pattern.kind {
        PatternKind::TypeHint(_, type_expr)
            if matches!(type_expr.kind, TypeExprKind::ForAll(_, _, _)) =>
        {
            Some(type_expr)
        }
        _ => None,
    }
}

fn collect_forall_type_hints<'a>(
    pattern: &'a Pattern<()>,
    type_hints: &mut Vec<&'a TypeExpr>,
) {
    match &pattern.kind {
        PatternKind::TypeHint(inner, type_expr) => {
            if matches!(type_expr.kind, TypeExprKind::ForAll(_, _, _)) {
                type_hints.push(type_expr);
            }
            collect_forall_type_hints(inner, type_hints);
        }
        PatternKind::Constructor(_, payload) => collect_forall_type_hints(payload, type_hints),
        PatternKind::Tuple(items) => {
            for item in items.iter() {
                collect_forall_type_hints(item, type_hints);
            }
        }
        PatternKind::Array {
            starting, ending, ..
        } => {
            for item in starting.iter().chain(ending.iter()) {
                collect_forall_type_hints(item, type_hints);
            }
        }
        PatternKind::Struct(fields) => {
            for value in fields.values() {
                collect_forall_type_hints(value, type_hints);
            }
        }
        PatternKind::Hole
        | PatternKind::Identifier(_)
        | PatternKind::ConstConstructor(_)
        | PatternKind::Immediate(_) => {}
    }
}

fn top_level_type_hint_identifier_path(pattern: &Pattern<Type>) -> Option<&Path> {
    let PatternKind::TypeHint(inner, _) = &pattern.kind else {
        return None;
    };
    let PatternKind::Identifier(path) = &inner.kind else {
        return None;
    };
    Some(path)
}

fn leading_forall_display_names(type_: &Type) -> Vec<Option<String>> {
    let mut names = Vec::new();
    let mut current = type_;
    while let Type::ForAll { name, body } = current {
        names.push(name.clone());
        current = body;
    }
    names
}

fn apply_leading_forall_display_names(
    mut type_: Type,
    names: &[Option<String>],
) -> Type {
    let mut remaining = names.iter();
    let mut current = &mut type_;
    while let Type::ForAll { name, body } = current {
        let Some(next_name) = remaining.next() else {
            break;
        };
        *name = next_name.clone();
        current = body;
    }
    type_
}

fn apply_forall_display_names_to_scheme(
    scheme: TypeScheme,
    names: &[Option<String>],
) -> TypeScheme {
    if names.is_empty() {
        return scheme;
    }
    TypeScheme {
        predicates: scheme.predicates,
        type_: apply_leading_forall_display_names(scheme.type_, names),
    }
}

fn rigidify_meta_vars_for_annotation_check(
    type_: &Type,
    replacements: &mut HashMap<MetaVarId, Type>,
) -> Type {
    struct AnnotationMetaRigidifier<'a> {
        replacements: &'a mut HashMap<MetaVarId, Type>,
    }

    impl TypeTransform for AnnotationMetaRigidifier<'_> {
        fn meta_var(
            &mut self,
            id: MetaVarId,
        ) -> Option<Type> {
            Some(
                self.replacements
                    .entry(id)
                    .or_insert_with(|| {
                        Type::Named {
                            name: Path::new("[annotation-meta]", format!("#{id}")),
                            body: Box::new(Type::Unit),
                        }
                    })
                    .clone(),
            )
        }

        fn named(
            &mut self,
            name: &Path,
            body: &Type,
        ) -> Option<Type> {
            let body = self.transform(body)?;
            Some(Type::Named {
                name: name.clone(),
                body: Box::new(body),
            })
        }
    }

    AnnotationMetaRigidifier { replacements }
        .transform(type_)
        .unwrap_or_else(|| type_.clone())
}

fn instantiate_leading_foralls_with_metas(
    table: &mut UnificationTable,
    type_: &Type,
    level: u32,
    span: Span,
) -> Result<Type, TypeError> {
    let mut current = table.normalize(type_);
    while let Type::ForAll { body, .. } = current {
        let fresh = table.new_meta(level);
        current = body
            .open_forall(&fresh)
            .ok_or(TypeError::InvalidScheme { span })?;
    }
    Ok(current)
}

fn skolemize_leading_foralls_for_annotation_check(
    table: &mut UnificationTable,
    type_: &Type,
    span: Span,
) -> Result<Type, TypeError> {
    let mut current = table.normalize(type_);
    let mut index = 0usize;
    while let Type::ForAll { body, .. } = current {
        let skolem = Type::Named {
            name: Path::new("[annotation-skolem]", format!("#{index}")),
            body: Box::new(Type::Unit),
        };
        index += 1;
        current = body
            .open_forall(&skolem)
            .ok_or(TypeError::InvalidScheme { span })?;
    }
    Ok(current)
}

fn ensure_forall_annotation_is_compatible_with_inferred(
    context: &InferenceContext,
    inferred: &TypeScheme,
    annotation: &TypeScheme,
    span: Span,
) -> Result<(), TypeError> {
    let mut trial_table = context.table.clone();
    let normalized_inferred = trial_table.normalize(&inferred.type_);
    let rigidified_inferred =
        rigidify_meta_vars_for_annotation_check(&normalized_inferred, &mut HashMap::new());
    let inferred_instance = instantiate_leading_foralls_with_metas(
        &mut trial_table,
        &rigidified_inferred,
        context.level,
        span,
    )?;
    let annotation_expected =
        skolemize_leading_foralls_for_annotation_check(&mut trial_table, &annotation.type_, span)?;
    trial_table
        .unify(&inferred_instance, &annotation_expected)
        .map_err(|error| {
            let error = match error {
                UnifyError::Mismatch { .. } => {
                    UnifyError::Mismatch {
                        left: inferred.type_.clone(),
                        right: annotation.type_.clone(),
                    }
                }
                other => other,
            };
            TypeError::Unification {
                error,
                span,
                context: Some(
                    "checking whether the explicit annotation matches the inferred value type",
                ),
            }
        })
}

fn predicate_key(
    predicate: &TraitConstraint,
    canonical_trait_name: impl Fn(&Path) -> Option<Path>,
) -> String {
    let trait_name =
        canonical_trait_name(&predicate.trait_name).unwrap_or_else(|| predicate.trait_name.clone());
    let args = predicate
        .arguments
        .iter()
        .map(Type::pretty)
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        trait_name.to_string()
    } else {
        format!("{} {args}", trait_name)
    }
}

fn inferred_predicates_covered_by_annotation(
    context: &InferenceContext,
    inferred: &[TraitConstraint],
    annotation: &[TraitConstraint],
) -> bool {
    let mut inferred_keys = inferred
        .iter()
        .map(|predicate| {
            predicate_key(predicate, |trait_name| {
                context.canonical_trait_name(trait_name)
            })
        })
        .collect::<Vec<_>>();
    inferred_keys.sort();
    inferred_keys.dedup();

    let mut annotation_keys = annotation
        .iter()
        .map(|predicate| {
            predicate_key(predicate, |trait_name| {
                context.canonical_trait_name(trait_name)
            })
        })
        .collect::<Vec<_>>();
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
    trait_aliases: IndexMap<Path, Path>,
    trait_parameter_kinds: IndexMap<Path, Vec<Kind>>,
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

    pub fn set_trait_aliases(
        &mut self,
        aliases: IndexMap<Path, Path>,
    ) {
        self.trait_aliases = aliases;
    }

    pub fn set_trait_parameter_kinds(
        &mut self,
        kinds: IndexMap<Path, Vec<Kind>>,
    ) {
        self.trait_parameter_kinds = kinds;
    }

    pub fn canonical_trait_name(
        &self,
        trait_name: &Path,
    ) -> Option<Path> {
        let mut current = trait_name.clone();
        let mut seen = HashSet::new();
        while let Some(next) = self.trait_aliases.get(&current) {
            if !seen.insert(current.clone()) {
                return None;
            }
            current = next.clone();
        }
        Some(current)
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
        while let Type::ForAll { body, .. } = current {
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
        let (count, body) = peel_leading_foralls(&scheme.type_);
        if count == 0 {
            return Ok(SchemeInstance {
                type_: body,
                predicates: scheme.predicates.clone(),
            });
        }
        let metas = std::iter::repeat_with(|| self.table.new_meta(self.level))
            .take(count)
            .collect::<Vec<_>>();
        let type_ = instantiate_forall_strict(&scheme.type_, &metas)
            .ok_or(TypeError::InvalidScheme { span })?;
        // `instantiate_forall_strict` peels ForAlls outside-in: metas[0] opens
        // the outermost binder (= TypeVar(count-1) in the body), while
        // `instantiate_type_vars` maps TypeVar(0) → args[0].  Reverse the
        // metas so TypeVar(k) maps to the same concrete meta in both paths.
        let reversed_metas: Vec<_> = metas.iter().rev().cloned().collect();
        let predicates = instantiate_predicates(&scheme.predicates, &reversed_metas)
            .ok_or(TypeError::InvalidScheme { span })?;
        Ok(SchemeInstance { type_, predicates })
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
        let mut type_ = MetaVarToTypeVarSubstitution {
            meta_var_to_type_var: &meta_var_to_type_var,
        }
        .transform(&normalized_type)
        .unwrap_or_else(|| normalized_type.clone())
        .for_all(free_meta_vars.len());
        let predicates =
            replace_meta_vars_in_predicates(&normalized_predicates, &meta_var_to_type_var);
        let extra_foralls = required_outer_type_var_binders(&type_, &predicates);
        if extra_foralls > 0 {
            type_ = type_.for_all(extra_foralls);
        }
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
                            span: term.span,
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
                forall @ Type::ForAll { .. } => {
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
            unify_with_context(
                &mut ctx.table,
                &typed_callee.term.type_,
                &function_type,
                callee.span,
                "checking the type of the called expression",
            )?;
            let expected_argument_type = ctx.table.normalize(&parameter_type);
            let typed_argument =
                match check_term(ctx, env, argument, &expected_argument_type, schemes) {
                    Ok(typed_argument) => typed_argument,
                    Err(TypeError::Unification {
                        error,
                        span,
                        context,
                    }) => {
                        if let TermKind::Identifier(path) = &typed_callee.term.kind
                            && ctx.has_unannotated_parameter(path)
                        {
                            return Err(TypeError::HigherRankAnnotationRequired {
                                parameter: path.clone(),
                                span: argument.span,
                            });
                        }
                        return Err(TypeError::Unification {
                            error,
                            span,
                            context: context.or(Some(
                                "checking this call argument against the parameter type",
                            )),
                        });
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
            let mut recursive_bindings = IndexMap::<Path, Type>::new();
            for_each_pattern_binding(assignee, |path, _| {
                recursive_bindings
                    .entry(path.clone())
                    .or_insert_with(|| ctx.fresh_meta());
            });
            let mut env_for_value = env.with_bindings(
                recursive_bindings
                    .iter()
                    .map(|(path, type_)| (path.clone(), type_.clone())),
            );
            let typed_value = infer_term(ctx, &mut env_for_value, value, schemes)?;
            tracing::debug!(
                assignee = ?assignee.kind,
                value_type = %typed_value.term.type_,
                ctx_level = ctx.level,
                outer_level,
                "let binding",
            );
            let mut forall_type_hints = Vec::new();
            collect_forall_type_hints(assignee, &mut forall_type_hints);
            let mut top_level_forall_annotation_names = None;
            if !forall_type_hints.is_empty() {
                let inferred = ctx.generalize_with_predicates(
                    &typed_value.term.type_,
                    outer_level,
                    typed_value.predicates.clone(),
                );
                for type_expr in forall_type_hints {
                    let annotation = type_expr_to_scheme(ctx, type_expr)?;
                    if !inferred_predicates_covered_by_annotation(
                        ctx,
                        &inferred.predicates,
                        &annotation.predicates,
                    ) {
                        return Err(TypeError::PolymorphicAnnotationMissingConstraints {
                            predicates: inferred.predicates,
                            span: type_expr.span,
                        });
                    }
                }
                if let Some(type_expr) = top_level_forall_type_hint(assignee) {
                    let annotation = type_expr_to_scheme(ctx, type_expr)?;
                    ensure_forall_annotation_is_compatible_with_inferred(
                        ctx,
                        &inferred,
                        &annotation,
                        type_expr.span,
                    )?;
                    top_level_forall_annotation_names =
                        Some(leading_forall_display_names(&annotation.type_));
                }
            }
            let mut bindings = Vec::new();
            let typed_pattern = infer_pattern(
                ctx,
                &env_for_value,
                assignee,
                &typed_value.term.type_,
                &mut bindings,
            )?;
            for (path, binding_type) in bindings.iter() {
                if let Some(recursive_type) = recursive_bindings.get(path) {
                    unify_with_context(
                        &mut ctx.table,
                        recursive_type,
                        binding_type,
                        assignee.span,
                        "checking recursive let references against the bound value type",
                    )?;
                }
            }
            ctx.level = outer_level;

            let include_value_predicates = matches!(
                typed_pattern.kind,
                PatternKind::Immediate(ImmediateValue::Boolean(true))
            );
            let top_level_hint_binding_path =
                top_level_type_hint_identifier_path(&typed_pattern).cloned();
            let generalized = bindings
                .into_iter()
                .map(|(path, type_)| {
                    let scheme = ctx.generalize_with_predicates(
                        &type_,
                        outer_level,
                        typed_value.predicates.clone(),
                    );
                    let scheme = if top_level_hint_binding_path
                        .as_ref()
                        .is_some_and(|hint_path| hint_path == &path)
                    {
                        top_level_forall_annotation_names
                            .as_deref()
                            .map_or(scheme.clone(), |names| {
                                apply_forall_display_names_to_scheme(scheme, names)
                            })
                    } else {
                        scheme
                    };
                    tracing::trace!(
                        path = %path,
                        raw_type = %type_.pretty(),
                        scheme = %scheme.type_.pretty(),
                        "let binding generalized",
                    );
                    (path, scheme)
                })
                .collect::<Vec<_>>();
            schemes.extend(generalized.iter().cloned());
            let mut env_with = env.with_bindings(generalized.clone());
            let typed_then = infer_term(ctx, &mut env_with, then, schemes)?;
            let typed_else = infer_term(ctx, env, else_, schemes)?;
            tracing::trace!(
                then_type = %ctx.table.normalize(&typed_then.term.type_).pretty(),
                else_type = %ctx.table.normalize(&typed_else.term.type_).pretty(),
                "let branch unification",
            );
            unify_with_context(
                &mut ctx.table,
                &typed_then.term.type_,
                &typed_else.term.type_,
                term.span,
                "checking that both branches of this let expression return the same type",
            )?;
            let result_type = ctx.table.normalize(&typed_then.term.type_);
            let mut predicates = typed_then.predicates;
            predicates.extend(typed_else.predicates);
            if include_value_predicates {
                predicates.extend(typed_value.predicates);
            }
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
            unify_with_context(
                &mut ctx.table,
                &typed_left.term.type_,
                &Type::Unit,
                typed_left.term.span,
                "checking that the left side of `;` is unit",
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
    unify_with_context(
        &mut inference_context.table,
        &inferred.term.type_,
        &expected,
        term.span,
        "checking this expression against the expected type",
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
            unify_with_context(
                &mut inference_context.table,
                &annotated,
                expected_parameter,
                type_expr.span,
                "checking the annotated function parameter type against the expected parameter type",
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
                    span: term.span,
                }
            })?;
            Ok((path.clone(), scheme.type_.clone()))
        })
        .collect::<Result<Vec<_>, TypeError>>()?;
    let expected_type = Type::func(expected_parameter.clone(), expected_result.clone());
    let value_type = Type::func(parameter_type, typed_body.term.type_.clone());
    unify_with_context(
        &mut inference_context.table,
        &value_type,
        &expected_type,
        term.span,
        "checking this function body against the expected function type",
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
            unify_with_context(
                &mut ctx.table,
                expected,
                &type_,
                pattern.span,
                "checking this literal pattern against the expected type",
            )?;
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
            unify_with_context(
                &mut ctx.table,
                expected,
                &tuple_type,
                pattern.span,
                "checking this tuple pattern against the expected type",
            )?;
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
            unify_with_context(
                &mut ctx.table,
                expected,
                &array_type,
                pattern.span,
                "checking this array pattern against the expected type",
            )?;
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
            unify_with_context(
                &mut ctx.table,
                expected,
                &struct_type,
                pattern.span,
                "checking this struct pattern against the expected type",
            )?;
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
            unify_with_context(
                &mut ctx.table,
                expected,
                &type_,
                pattern.span,
                "checking this constructor pattern against the expected type",
            )?;
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
            tracing::debug!(
                path = %path,
                scheme_type = %scheme.type_,
                instantiated = %type_,
                expected = %ctx.table.normalize(expected),
                "constructor pattern",
            );
            let (param_type, result_type) = match ctx.table.normalize(&type_) {
                Type::Function(parameter, result) => (*parameter, *result),
                other => {
                    return Err(TypeError::NotAFunction {
                        type_: other,
                        span: pattern.span,
                    });
                }
            };
            unify_with_context(
                &mut ctx.table,
                expected,
                &result_type,
                pattern.span,
                "checking this constructor pattern result type against the expected type",
            )?;
            let payload_expected = match (&payload.kind, ctx.table.normalize(&param_type)) {
                (PatternKind::Struct(_), Type::Struct { fields }) => {
                    Type::StructConstraint {
                        fields,
                        mode: StructMatch::Exact,
                    }
                }
                (_, other) => other,
            };
            let typed_payload = infer_pattern(ctx, env, payload, &payload_expected, bindings)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Constructor(path.clone(), Box::new(typed_payload)),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::TypeHint(inner, type_expr) => {
            let hint_scheme = type_expr_to_scheme(ctx, type_expr)?;
            tracing::debug!(
                hint = %hint_scheme.type_.pretty(),
                expected = %ctx.table.normalize(expected).pretty(),
                predicates = hint_scheme.predicates.len(),
                "type hint pattern",
            );
            let type_definitions = ctx.type_definitions.clone();
            let hint_kind = infer_scheme_kind(
                &hint_scheme,
                0,
                &|path| {
                    type_definitions
                        .get(path)
                        .map(|def| constructor_kind(def.parameters, &def.parameter_kinds))
                },
                &|trait_name| ctx.trait_parameter_kinds.get(trait_name).cloned(),
            );
            if let Ok(inference) = hint_kind
                && inference.kind != Kind::Type
            {
                return Err(TypeError::KindMismatch {
                    expected: Kind::Type,
                    found: inference.kind,
                    span: type_expr.span,
                });
            }
            let hint_type = hint_scheme.type_;
            let expected_type = ctx.table.normalize(expected);
            let hint_type = match (hint_type, expected_type) {
                (forall @ Type::ForAll { .. }, Type::ForAll { .. }) => forall,
                (forall @ Type::ForAll { .. }, _) => {
                    let scheme = TypeScheme::new(forall);
                    ctx.instantiate(&scheme, type_expr.span)?
                }
                (other, _) => other,
            };
            unify_with_context(
                &mut ctx.table,
                expected,
                &hint_type,
                type_expr.span,
                "checking this pattern type annotation against the expected type",
            )?;
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
    unify_with_context(
        &mut ctx.table,
        type_,
        &constraint,
        span,
        "checking whether this value has the requested field",
    )?;
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

fn required_outer_type_var_binders(
    type_: &Type,
    predicates: &[TraitConstraint],
) -> usize {
    let type_max = max_free_type_var_index(type_, 0);
    let predicate_depth = leading_forall_count(type_);
    let predicate_max = predicates
        .iter()
        .flat_map(|predicate| predicate.arguments.iter())
        .filter_map(|argument| max_free_type_var_index(argument, predicate_depth))
        .max();
    type_max
        .into_iter()
        .chain(predicate_max)
        .max()
        .map(|index| index as usize + 1)
        .unwrap_or(0)
}

fn leading_forall_count(type_: &Type) -> u32 {
    let mut current = type_;
    let mut count = 0;
    while let Type::ForAll { body, .. } = current {
        count += 1;
        current = body;
    }
    count
}

fn max_free_type_var_index(
    type_: &Type,
    depth: u32,
) -> Option<u32> {
    match type_ {
        Type::TypeVar(index) => (*index >= depth).then(|| index - depth),
        Type::ForAll { body, .. } => max_free_type_var_index(body, depth + 1),
        Type::Array(inner) => max_free_type_var_index(inner, depth),
        Type::Tuple(items) => {
            items
                .iter()
                .filter_map(|item| max_free_type_var_index(item, depth))
                .max()
        }
        Type::Struct { fields } | Type::StructConstraint { fields, .. } => {
            fields
                .values()
                .filter_map(|field| max_free_type_var_index(field, depth))
                .max()
        }
        Type::Sum { variants } => {
            variants
                .values()
                .filter_map(|variant| max_free_type_var_index(variant, depth))
                .max()
        }
        Type::Function(parameter, result) => {
            max_free_type_var_index(parameter, depth)
                .into_iter()
                .chain(max_free_type_var_index(result, depth))
                .max()
        }
        Type::Named { body, .. } => max_free_type_var_index(body, depth),
        Type::Apply {
            constructor,
            arguments,
        } => {
            max_free_type_var_index(constructor, depth)
                .into_iter()
                .chain(
                    arguments
                        .iter()
                        .filter_map(|argument| max_free_type_var_index(argument, depth)),
                )
                .max()
        }
        Type::Unit
        | Type::Integer
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::MetaVar(_) => None,
    }
}
