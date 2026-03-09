//! Core type representation tests.

use indexmap::IndexMap;

use super::*;
use crate::ir::Path;

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
    let Type::ForAll(body) = forall else {
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
        "(((| none | some (integer, boolean) )) -> integer)"
    );
}

#[test]
fn pretty_prints_type_variable_names_past_z() {
    assert_eq!(Type::v(0).pretty(), "'a");
    assert_eq!(Type::v(25).pretty(), "'z");
    assert_eq!(Type::v(26).pretty(), "'aa");
    assert_eq!(Type::v(27).pretty(), "'ab");
}

#[test]
fn display_uses_pretty_output() {
    let type_ = Type::func(Type::Integer, Type::Boolean);
    assert_eq!(format!("{type_}"), "(integer -> boolean)");
}
