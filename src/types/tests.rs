//! Core type representation tests.

use indexmap::IndexMap;

use super::*;
use crate::ir::Path;
use crate::parse::ast;

fn named(
    module: &str,
    name: &str,
    body: Type,
) -> Type {
    Type::Named {
        name: Path::new(module, name),
        body: Box::new(body),
    }
}

fn fields(pairs: Vec<(&str, Type)>) -> IndexMap<String, Type> {
    pairs
        .into_iter()
        .map(|(name, type_)| (name.to_string(), type_))
        .collect()
}

fn lookup_roundtrip_symbol(path: &Path) -> super::type_expr::TypeExprSymbol {
    use super::type_expr::TypeExprSymbol;

    if *path == Path::core("Fn") {
        return TypeExprSymbol::Definition(Type::function().def(2));
    }
    if *path == Path::core("Array") {
        return TypeExprSymbol::Definition(Type::array().def(1));
    }

    let primitive = match path.minor.as_str() {
        "integer" | "Integer" => Some(Type::Integer.def(0)),
        "real" | "Real" => Some(Type::Real.def(0)),
        "boolean" | "Boolean" => Some(Type::Boolean.def(0)),
        "string" | "String" => Some(Type::String.def(0)),
        "glyph" | "Glyph" => Some(Type::Glyph.def(0)),
        _ => None,
    };

    primitive
        .map(TypeExprSymbol::Definition)
        .unwrap_or(TypeExprSymbol::Unknown)
}

fn parse_pretty_type(pretty: &str) -> Type {
    let source = format!("module M =\n  type RoundTrip = {pretty}\nend\n");
    let mut logger = crate::Logger::new();
    let mut file_logger = logger.new_file("<roundtrip.hc>", source.clone());
    let source_file =
        crate::parse::parse(source.as_str(), &mut file_logger).expect("type should parse");
    assert!(
        file_logger.is_ok(),
        "pretty-printed type should parse without errors: `{pretty}`"
    );

    let module = source_file
        .modules()
        .into_iter()
        .next()
        .expect("roundtrip module should exist");
    let type_statement = module
        .statements()
        .into_iter()
        .find_map(|statement| {
            if let ast::Statement::Type(statement) = statement {
                Some(statement)
            } else {
                None
            }
        })
        .expect("roundtrip type statement should exist");
    let ast::TypeDef::Alias(alias) = type_statement
        .type_def()
        .expect("roundtrip type definition should exist")
    else {
        panic!("roundtrip helper expects alias definitions only")
    };
    let ast_type_expr = alias
        .type_expr()
        .expect("roundtrip type alias expression should exist");

    let mut scope = crate::ir::ModuleScope::new("demo".to_string());
    let ir_type_expr = crate::ir::type_expr(&mut scope, &mut file_logger, ast_type_expr)
        .expect("IR lowering should succeed");
    assert!(
        file_logger.is_ok(),
        "IR lowering should not report errors for pretty-printed type: `{pretty}`"
    );

    let lowered = super::type_expr::lower_type_expr(
        &ir_type_expr,
        &mut |path| lookup_roundtrip_symbol(path),
        &mut |_| None,
    );
    assert!(
        lowered.errors.is_empty(),
        "semantic lowering should not report errors for pretty-printed type: `{pretty}`"
    );
    lowered.type_
}

fn assert_pretty_roundtrip(type_: Type) {
    let pretty = type_.pretty();
    let reparsed = parse_pretty_type(pretty.as_str());
    assert_eq!(reparsed, type_, "pretty roundtrip mismatch for `{pretty}`");
}

#[test]
fn named_equality_is_nominal() {
    let left = named("demo", "Box", Type::Integer);
    let right_same_name = named("demo", "Box", Type::Boolean);
    let right_other_name = named("demo", "Crate", Type::Integer);

    assert_eq!(left, right_same_name);
    assert_ne!(left, right_other_name);
}

#[test]
fn empty_apply_layers_are_ignored_for_equality() {
    let base = named("demo", "List", Type::Unit);
    let wrapped = Type::Apply {
        constructor: Box::new(Type::Apply {
            constructor: Box::new(base.clone()),
            arguments: Vec::new(),
        }),
        arguments: Vec::new(),
    };

    assert_eq!(wrapped, base);
}

#[test]
fn non_empty_apply_preserves_application() {
    let applied = Type::Integer.apply(vec![Type::Boolean]);
    assert!(matches!(applied, Type::Apply { .. }));
}

#[test]
fn normalize_empty_apply_strips_nested_empty_layers() {
    let type_ = Type::Apply {
        constructor: Box::new(Type::Apply {
            constructor: Box::new(Type::Integer),
            arguments: Vec::new(),
        }),
        arguments: Vec::new(),
    };
    assert_eq!(normalize_empty_apply(type_), Type::Integer);
}

#[test]
fn for_each_child_type_respects_named_body_flag() {
    let type_ = named(
        "demo",
        "Pair",
        Type::Tuple(vec![Type::Integer, Type::Boolean]),
    );

    let mut without_body = 0;
    for_each_child_type(&type_, false, |_| without_body += 1);

    let mut with_body = 0;
    for_each_child_type(&type_, true, |_| with_body += 1);

    assert_eq!(without_body, 0);
    assert_eq!(with_body, 1);
}

#[test]
fn type_transform_walk_tracks_forall_depth() {
    struct DepthTracker {
        depth: u32,
        max_depth: u32,
    }

    impl TypeTransform for DepthTracker {
        fn enter_forall(&mut self) {
            self.depth += 1;
            self.max_depth = self.max_depth.max(self.depth);
        }

        fn leave_forall(&mut self) {
            self.depth -= 1;
        }
    }

    let type_ = Type::func(Type::v(1), Type::v(0)).for_all(2);
    let mut tracker = DepthTracker {
        depth: 0,
        max_depth: 0,
    };
    tracker.walk(&type_);

    assert_eq!(tracker.depth, 0);
    assert_eq!(tracker.max_depth, 2);
}

#[test]
fn named_transform_hook_can_replace_named_node() {
    struct ExpandNamed;

    impl TypeTransform for ExpandNamed {
        fn named(
            &mut self,
            _name: &Path,
            body: &Type,
        ) -> Option<Type> {
            self.transform(body)
        }
    }

    let type_ = named(
        "demo",
        "Wrapped",
        Type::Struct {
            fields: fields(vec![("value", Type::Integer)]),
        },
    );
    let transformed = ExpandNamed
        .transform(&type_)
        .expect("transform should succeed");

    assert!(matches!(transformed, Type::Struct { .. }));
}

#[test]
fn apply_transform_hook_can_abort() {
    struct RejectApply;

    impl TypeTransform for RejectApply {
        fn apply(
            &mut self,
            _constructor: &Type,
            _arguments: &[Type],
        ) -> Option<Type> {
            None
        }
    }

    let type_ = Type::array().apply(vec![Type::Integer]);
    assert!(RejectApply.transform(&type_).is_none());
}

#[test]
fn shift_type_vars_respects_cutoff_and_forall() {
    let original = Type::func(Type::v(0), Type::func(Type::v(1), Type::v(0)).for_all(1));
    let shifted = original
        .shift_type_vars(1, 0)
        .expect("shift should succeed");

    let expected = Type::func(Type::v(1), Type::func(Type::v(2), Type::v(0)).for_all(1));
    assert_eq!(shifted, expected);
}

#[test]
fn shift_type_vars_underflow_returns_none() {
    assert!(Type::v(0).shift_type_vars(-1, 0).is_none());
}

#[test]
fn substitute_type_var_respects_binder_depth() {
    let type_ = Type::func(Type::v(0), Type::v(1)).for_all(1);
    let replaced = type_
        .substitute_type_var(0, &Type::Integer)
        .expect("substitution should succeed");

    assert_eq!(replaced, Type::func(Type::v(0), Type::Integer).for_all(1));
}

#[test]
fn substitute_type_var_shifts_replacement_under_forall() {
    let type_ = Type::v(1).for_all(1);
    let replacement = Type::func(Type::v(0), Type::v(0));
    let replaced = type_
        .substitute_type_var(0, &replacement)
        .expect("substitution should succeed");

    assert_eq!(replaced, Type::func(Type::v(1), Type::v(1)).for_all(1));
}

#[test]
fn open_forall_opens_outermost_binder() {
    let forall = Type::func(Type::v(0), Type::v(1)).for_all(2);
    let Type::ForAll { body, .. } = forall else {
        panic!("expected forall type");
    };
    let opened = body
        .open_forall(&Type::Integer)
        .expect("open_forall should succeed");

    assert_eq!(opened, Type::func(Type::v(0), Type::Integer).for_all(1));
}

#[test]
fn helper_type_constructors_build_expected_shapes() {
    assert_eq!(Type::array(), Type::Array(Box::new(Type::v(0))).for_all(1));
    assert_eq!(
        Type::function(),
        Type::func(Type::v(1), Type::v(0)).for_all(2)
    );
}

#[test]
fn curry_builds_right_associative_function_types() {
    assert_eq!(Type::curry(&[]), Type::Unit);
    assert_eq!(Type::curry(&[Type::Integer]), Type::Integer);
    assert_eq!(
        Type::curry(&[Type::Integer, Type::Boolean, Type::String]),
        Type::func(Type::Integer, Type::func(Type::Boolean, Type::String))
    );
}

#[test]
fn scheme_helpers_attach_predicates() {
    let predicate = TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]);
    let scheme = Type::Integer.scheme_with_predicates(vec![predicate.clone()]);

    assert_eq!(scheme.type_, Type::Integer);
    assert_eq!(scheme.predicates, vec![predicate]);
}

#[test]
fn pretty_prints_struct_constraint_modes() {
    let exact = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer)]),
        mode: StructMatch::Exact,
    };
    let at_least = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer)]),
        mode: StructMatch::AtLeast,
    };

    assert_eq!(exact.pretty(), "{x: integer}");
    assert_eq!(at_least.pretty(), "{x: integer, ..}");
}

#[test]
fn pretty_prints_sum_and_function_with_wrapping() {
    let sum = Type::Sum {
        variants: fields(vec![
            ("none", Type::Unit),
            ("some", Type::Tuple(vec![Type::Integer, Type::Boolean])),
        ]),
    };
    let function = Type::func(sum, Type::Integer);

    assert_eq!(
        function.pretty(),
        "(| none | some (integer, boolean) ) -> integer"
    );
}

#[test]
fn pretty_prints_right_associative_function_without_nested_parentheses() {
    let function = Type::curry(&[Type::v(0), Type::v(1), Type::v(2)]);
    assert_eq!(function.pretty(), "a -> b -> c");
}

#[test]
fn pretty_prints_type_variable_names_past_z() {
    assert_eq!(Type::v(0).pretty(), "a");
    assert_eq!(Type::v(25).pretty(), "z");
    assert_eq!(Type::v(26).pretty(), "aa");
    assert_eq!(Type::v(27).pretty(), "ab");
}

#[test]
fn type_variable_names_skip_reserved_keywords() {
    for index in 0..200u32 {
        let name = type_var_name(index);
        assert!(
            !is_reserved_type_variable_name(name.as_str()),
            "type variable name `{name}` at index {index} collides with a keyword"
        );
    }
}

#[test]
fn pretty_prints_consecutive_forall_binders_compactly() {
    let type_ = Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2);
    assert_eq!(type_.pretty(), "for a b in (a, b)");
}

#[test]
fn pretty_prints_explicit_forall_names_from_source() {
    let type_ = Type::func(Type::v(0), Type::v(0)).for_all_with_names([Some("item".to_string())]);
    assert_eq!(type_.pretty(), "for item in item -> item");
}

#[test]
fn generated_forall_names_avoid_explicit_name_collisions() {
    let type_ = Type::ForAll {
        name: None,
        body: Box::new(Type::ForAll {
            name: Some("a".to_string()),
            body: Box::new(Type::func(Type::v(1), Type::v(0))),
        }),
    };

    assert_eq!(type_.pretty(), "for b a in b -> a");
}

#[test]
fn display_uses_pretty_output() {
    let type_ = Type::func(Type::Integer, Type::Boolean);
    assert_eq!(format!("{type_}"), "integer -> boolean");
}

#[test]
fn pretty_round_trips_source_expressible_types() {
    assert_pretty_roundtrip(Type::Integer);
    assert_pretty_roundtrip(Type::Tuple(vec![Type::Integer]));
    assert_pretty_roundtrip(Type::Tuple(vec![Type::Integer, Type::Boolean]));
    assert_pretty_roundtrip(Type::Array(Box::new(Type::Integer)));
    assert_pretty_roundtrip(Type::Array(Box::new(Type::Array(Box::new(Type::Integer)))));
    assert_pretty_roundtrip(Type::func(
        Type::Integer,
        Type::func(Type::Boolean, Type::String),
    ));
    assert_pretty_roundtrip(Type::func(
        Type::func(Type::Integer, Type::Boolean),
        Type::String,
    ));
    assert_pretty_roundtrip(Type::func(Type::v(1), Type::v(0)).for_all(2));
    assert_pretty_roundtrip(Type::func(Type::Tuple(vec![Type::v(0)]), Type::v(0)).for_all(1));
    assert_pretty_roundtrip(named("demo", "List", Type::Unit).apply(vec![Type::Integer]));
    assert_pretty_roundtrip(named("demo", "Mapper", Type::Unit).apply(vec![
        Type::func(Type::Integer, Type::Boolean),
        Type::Array(Box::new(Type::String)),
    ]));
}
