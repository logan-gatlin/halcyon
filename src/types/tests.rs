use super::instantiation::instantiate_forall_strict;
use super::*;
use crate::ir::Path;
use indexmap::IndexMap;

use super::unify::UnificationTable;

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

fn list_of(inner: Type) -> Type {
    let list_type = Type::Named {
        name: Path::new("test", "List"),
        body: Box::new(Type::Unit),
    };
    Type::Apply {
        constructor: Box::new(list_type),
        arguments: vec![inner],
    }
}

#[test]
fn nominal_equality_ignores_body() {
    let left = named("core", "List", Type::Integer);
    let right = named("core", "List", Type::Boolean);
    assert_eq!(left, right);
}

#[test]
fn nominal_equality_distinguishes_names() {
    let left = named("core", "List", Type::Integer);
    let right = named("core", "Tree", Type::Integer);
    assert_ne!(left, right);
}

#[test]
fn nominal_not_equal_structural() {
    let fields = [("value".to_string(), Type::Integer)]
        .into_iter()
        .collect::<IndexMap<_, _>>();
    let structural = Type::Struct {
        fields: fields.clone(),
    };
    let nominal = named("core", "Box", Type::Struct { fields });
    assert_ne!(nominal, structural);
}

#[test]
fn apply_empty_arguments_equates_constructor() {
    let nominal = named("core", "List", Type::Unit);
    let applied = Type::Apply {
        constructor: Box::new(nominal.clone()),
        arguments: vec![],
    };
    assert_eq!(applied, nominal);

    let applied_primitive = Type::Apply {
        constructor: Box::new(Type::Integer),
        arguments: vec![],
    };
    assert_eq!(applied_primitive, Type::Integer);
}

#[test]
fn apply_nonempty_arguments_not_equal_constructor() {
    let applied = Type::Apply {
        constructor: Box::new(Type::Integer),
        arguments: vec![Type::Boolean],
    };
    assert_ne!(applied, Type::Integer);
}

#[test]
fn applied_named_type_does_not_unify_structurally() {
    let mut table = UnificationTable::default();
    let core_function = Type::Named {
        name: Path::new("core", "function"),
        body: Box::new(Type::function()),
    }
    .apply(vec![Type::Integer, Type::Integer]);
    let direct_function = Type::func(Type::Integer, Type::Integer);
    assert!(table.unify(&core_function, &direct_function).is_err());
}

#[test]
fn shift_type_vars_respects_binder() {
    let original = Type::func(Type::v(0), Type::v(0).for_all(1));
    let shifted = original.shift_type_vars(1, 0).expect("shift succeeds");
    let expected = Type::func(Type::v(1), Type::v(0).for_all(1));
    assert_eq!(shifted, expected);
}

#[test]
fn shift_type_vars_underflow_returns_none() {
    let original = Type::v(0);
    assert!(original.shift_type_vars(-1, 0).is_none());
}

#[test]
fn substitute_type_var_respects_binder() {
    let original = Type::func(Type::v(0), Type::v(1)).for_all(1);
    let replaced = original
        .substitute_type_var(0, &Type::Integer)
        .expect("substitution succeeds");
    let expected = Type::func(Type::v(0), Type::Integer).for_all(1);
    assert_eq!(replaced, expected);
}

#[test]
fn substitute_type_var_shifts_replacement() {
    let original = Type::v(1).for_all(1);
    let replacement = Type::func(Type::v(0), Type::v(0));
    let replaced = original
        .substitute_type_var(0, &replacement)
        .expect("substitution succeeds");
    let expected = Type::func(Type::v(1), Type::v(1)).for_all(1);
    assert_eq!(replaced, expected);
}

#[test]
fn instantiate_forall_strict_accepts_type_var_arguments() {
    let instantiated = instantiate_forall_strict(&Type::function(), &[Type::v(0), Type::v(0)])
        .expect("instantiation succeeds");
    let expected = Type::func(Type::v(0), Type::v(0));
    assert_eq!(instantiated, expected);
}

#[test]
fn pretty_prints_named_application() {
    let nominal = named("core", "List", Type::Unit);
    let applied = Type::Apply {
        constructor: Box::new(nominal),
        arguments: vec![Type::Integer],
    };
    assert_eq!(applied.pretty(), "core::List integer");
}

#[test]
fn pretty_prints_letter_sequence() {
    assert_eq!(Type::v(0).pretty(), "'a");
    assert_eq!(Type::v(25).pretty(), "'z");
    assert_eq!(Type::v(26).pretty(), "'aa");
    assert_eq!(Type::v(27).pretty(), "'ab");
}

#[test]
fn struct_constraint_exact_matches_named_struct() {
    let mut table = UnificationTable::default();
    let named_struct = named(
        "test",
        "Point",
        Type::Struct {
            fields: fields(vec![("x", Type::Integer), ("y", Type::Boolean)]),
        },
    );
    let constraint = Type::StructConstraint {
        fields: fields(vec![("y", Type::Boolean), ("x", Type::Integer)]),
        mode: StructMatch::Exact,
    };
    assert!(table.unify(&constraint, &named_struct).is_ok());
}

#[test]
fn struct_constraint_exact_rejects_extra_fields() {
    let mut table = UnificationTable::default();
    let named_struct = named(
        "test",
        "Point",
        Type::Struct {
            fields: fields(vec![("x", Type::Integer)]),
        },
    );
    let constraint = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer), ("y", Type::Boolean)]),
        mode: StructMatch::Exact,
    };
    assert!(table.unify(&constraint, &named_struct).is_err());
}

#[test]
fn struct_constraint_at_least_allows_subset() {
    let mut table = UnificationTable::default();
    let named_struct = named(
        "test",
        "Point",
        Type::Struct {
            fields: fields(vec![("x", Type::Integer), ("y", Type::Boolean)]),
        },
    );
    let constraint = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer)]),
        mode: StructMatch::AtLeast,
    };
    assert!(table.unify(&constraint, &named_struct).is_ok());
}

#[test]
fn struct_constraint_at_least_rejects_missing() {
    let mut table = UnificationTable::default();
    let named_struct = named(
        "test",
        "Point",
        Type::Struct {
            fields: fields(vec![("x", Type::Integer)]),
        },
    );
    let constraint = Type::StructConstraint {
        fields: fields(vec![("y", Type::Boolean)]),
        mode: StructMatch::AtLeast,
    };
    assert!(table.unify(&constraint, &named_struct).is_err());
}

#[test]
fn struct_constraints_merge_on_meta_var() {
    let mut table = UnificationTable::default();
    let meta = table.new_meta(0);
    let left = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer)]),
        mode: StructMatch::AtLeast,
    };
    let right = Type::StructConstraint {
        fields: fields(vec![("y", Type::Boolean)]),
        mode: StructMatch::AtLeast,
    };
    table.unify(&meta, &left).expect("bind left");
    table.unify(&meta, &right).expect("merge right");
    let resolved = table.prune(&meta);
    let Type::StructConstraint { fields, mode } = resolved else {
        panic!("expected merged struct constraint");
    };
    assert_eq!(mode, StructMatch::AtLeast);
    assert_eq!(fields.len(), 2);
    assert!(fields.contains_key("x"));
    assert!(fields.contains_key("y"));
}

#[test]
fn struct_constraints_exact_rejects_extra_at_least_fields() {
    let mut table = UnificationTable::default();
    let meta = table.new_meta(0);
    let exact = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer)]),
        mode: StructMatch::Exact,
    };
    let at_least = Type::StructConstraint {
        fields: fields(vec![("x", Type::Integer), ("y", Type::Boolean)]),
        mode: StructMatch::AtLeast,
    };
    table.unify(&meta, &exact).expect("bind exact");
    assert!(table.unify(&meta, &at_least).is_err());
}

#[test]
fn resolve_trait_instance() {
    let mut symbols = SymbolTable::new();
    let eq = Path::new("test", "Eq");
    symbols
        .insert_trait(TraitDef {
            name: eq.clone(),
            parameters: 1,
            methods: IndexMap::new(),
        })
        .expect("trait def");
    symbols
        .insert_impl(TraitImpl {
            parameters: 0,
            head: TraitRef::new(eq.clone(), vec![Type::Integer]),
            predicates: Vec::new(),
            methods: IndexMap::new(),
        })
        .expect("impl");

    let mut table = UnificationTable::default();
    let unresolved = symbols
        .resolve_predicates(
            &mut table,
            &[TraitRef::new(eq.clone(), vec![Type::Integer])],
        )
        .expect("resolve");
    assert!(unresolved.is_empty());
}

#[test]
fn resolve_trait_instance_with_context() {
    let mut symbols = SymbolTable::new();
    let eq = Path::new("test", "Eq");
    symbols
        .insert_trait(TraitDef {
            name: eq.clone(),
            parameters: 1,
            methods: IndexMap::new(),
        })
        .expect("trait def");
    symbols
        .insert_impl(TraitImpl {
            parameters: 0,
            head: TraitRef::new(eq.clone(), vec![Type::Integer]),
            predicates: Vec::new(),
            methods: IndexMap::new(),
        })
        .expect("impl eq int");
    symbols
        .insert_impl(TraitImpl {
            parameters: 1,
            head: TraitRef::new(eq.clone(), vec![list_of(Type::v(0))]),
            predicates: vec![TraitRef::new(eq.clone(), vec![Type::v(0)])],
            methods: IndexMap::new(),
        })
        .expect("impl eq list");

    let mut table = UnificationTable::default();
    let unresolved = symbols
        .resolve_predicates(
            &mut table,
            &[TraitRef::new(eq.clone(), vec![list_of(Type::Integer)])],
        )
        .expect("resolve");
    assert!(unresolved.is_empty());
}

#[test]
fn resolve_trait_unresolved_predicate_is_retained() {
    let mut symbols = SymbolTable::new();
    let show = Path::new("test", "Show");
    symbols
        .insert_trait(TraitDef {
            name: show.clone(),
            parameters: 1,
            methods: IndexMap::new(),
        })
        .expect("trait def");

    let mut table = UnificationTable::default();
    let meta = table.new_meta(0);
    let predicate = TraitRef::new(show.clone(), vec![meta.clone()]);
    let unresolved = symbols
        .resolve_predicates(&mut table, std::slice::from_ref(&predicate))
        .expect("resolve");
    assert_eq!(unresolved, vec![predicate]);
}

#[test]
fn resolve_trait_recursive_predicate_errors() {
    let mut symbols = SymbolTable::new();
    let eq = Path::new("test", "Eq");
    symbols
        .insert_trait(TraitDef {
            name: eq.clone(),
            parameters: 1,
            methods: IndexMap::new(),
        })
        .expect("trait def");
    symbols
        .insert_impl(TraitImpl {
            parameters: 1,
            head: TraitRef::new(eq.clone(), vec![Type::v(0)]),
            predicates: vec![TraitRef::new(eq.clone(), vec![Type::v(0)])],
            methods: IndexMap::new(),
        })
        .expect("impl");

    let mut table = UnificationTable::default();
    let meta = table.new_meta(0);
    let predicate = TraitRef::new(eq.clone(), vec![meta]);
    let result = symbols.resolve_predicates(&mut table, &[predicate]);
    assert!(matches!(result, Err(TraitError::RecursivePredicate { .. })));
}

#[test]
fn overlap_detection_rejects_conflicting_instances() {
    let mut symbols = SymbolTable::new();
    let eq = Path::new("test", "Eq");
    symbols
        .insert_trait(TraitDef {
            name: eq.clone(),
            parameters: 1,
            methods: IndexMap::new(),
        })
        .expect("trait def");
    symbols
        .insert_impl(TraitImpl {
            parameters: 1,
            head: TraitRef::new(eq.clone(), vec![list_of(Type::v(0))]),
            predicates: Vec::new(),
            methods: IndexMap::new(),
        })
        .expect("impl list");

    let result = symbols.insert_impl(TraitImpl {
        parameters: 0,
        head: TraitRef::new(eq.clone(), vec![list_of(Type::Integer)]),
        predicates: Vec::new(),
        methods: IndexMap::new(),
    });
    assert!(matches!(
        result,
        Err(TraitError::OverlappingInstance { .. })
    ));
}

#[test]
fn insert_impl_requires_all_trait_methods() {
    let mut symbols = SymbolTable::new();
    let trait_path = Path::new("test", "Eq");
    let method_path = Path::new("test", "eq");
    symbols
        .insert_trait(TraitDef {
            name: trait_path.clone(),
            parameters: 1,
            methods: [(
                method_path,
                Type::curry(&[Type::v(0), Type::v(0), Type::Boolean]).scheme(),
            )]
            .into_iter()
            .collect(),
        })
        .expect("trait def");

    let result = symbols.insert_impl(TraitImpl {
        parameters: 0,
        head: TraitRef::new(trait_path, vec![Type::Integer]),
        predicates: Vec::new(),
        methods: IndexMap::new(),
    });
    assert!(matches!(result, Err(TraitError::InvalidInstance { .. })));
}

#[test]
fn resolve_method_specialization_uses_impl_mapping() {
    let mut symbols = SymbolTable::new();
    let trait_path = Path::new("test", "Eq");
    let method_path = Path::new("test", "eq");
    let impl_method_path = Path::new("test", "eq_integer");
    symbols
        .insert_trait(TraitDef {
            name: trait_path.clone(),
            parameters: 1,
            methods: [(
                method_path.clone(),
                Type::curry(&[Type::v(0), Type::v(0), Type::Boolean]).scheme(),
            )]
            .into_iter()
            .collect(),
        })
        .expect("trait def");
    symbols
        .insert_impl(TraitImpl {
            parameters: 0,
            head: TraitRef::new(trait_path.clone(), vec![Type::Integer]),
            predicates: Vec::new(),
            methods: [(method_path.clone(), impl_method_path.clone())]
                .into_iter()
                .collect(),
        })
        .expect("impl");

    let resolved = symbols
        .resolve_method_specialization(&method_path, &[Type::Integer])
        .expect("resolve")
        .expect("specialization");
    assert_eq!(resolved.trait_name, trait_path);
    assert_eq!(resolved.impl_method_path, impl_method_path);
    assert!(resolved.predicates.is_empty());
}
