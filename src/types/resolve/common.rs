//! Shared helpers for resolve submodules.

use super::{
    TraitConstraint,
    TraitRef,
    Type,
    for_each_child_type,
};

/// Render a trait reference in source-like form for diagnostics.
pub(super) fn format_trait_ref(trait_ref: &TraitRef) -> String {
    if trait_ref.arguments.is_empty() {
        trait_ref.trait_name.to_string()
    } else {
        let args = trait_ref
            .arguments
            .iter()
            .map(Type::pretty)
            .collect::<Vec<_>>()
            .join(" ");
        format!("{} {args}", trait_ref.trait_name)
    }
}

/// Return `true` when a predicate has no type/meta variables.
pub(super) fn predicate_is_ground(predicate: &TraitConstraint) -> bool {
    predicate.arguments.iter().all(is_ground_type)
}

fn is_ground_type(type_: &Type) -> bool {
    if matches!(type_, Type::TypeVar(_) | Type::MetaVar(_)) {
        return false;
    }
    let mut is_ground = true;
    for_each_child_type(type_, false, |child| {
        if is_ground && !is_ground_type(child) {
            is_ground = false;
        }
    });
    is_ground
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Path;

    #[test]
    fn format_trait_ref_renders_with_and_without_arguments() {
        let no_args = TraitRef::new(Path::new("demo", "Eq"), Vec::new());
        let with_args = TraitRef::new(
            Path::new("demo", "Eq"),
            vec![Type::Integer, Type::Array(Box::new(Type::Boolean))],
        );

        assert_eq!(format_trait_ref(&no_args), "demo::Eq");
        assert_eq!(format_trait_ref(&with_args), "demo::Eq integer [] boolean");
    }

    #[test]
    fn predicate_is_ground_detects_variables_and_nested_ground_types() {
        let non_ground_type_var = TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)]);
        let non_ground_meta = TraitRef::new(Path::new("demo", "Eq"), vec![Type::MetaVar(0)]);
        let ground = TraitRef::new(
            Path::new("demo", "Eq"),
            vec![Type::Tuple(vec![
                Type::Integer,
                Type::Named {
                    name: Path::new("demo", "Token"),
                    body: Box::new(Type::Unit),
                },
            ])],
        );

        assert!(!predicate_is_ground(&non_ground_type_var));
        assert!(!predicate_is_ground(&non_ground_meta));
        assert!(predicate_is_ground(&ground));
    }

    #[test]
    fn predicate_is_ground_treats_applied_named_types_as_ground_by_arguments() {
        let generic_box = Type::Named {
            name: Path::new("demo", "Box"),
            body: Box::new(
                Type::Struct {
                    fields: [("value".to_string(), Type::v(0))].into_iter().collect(),
                }
                .for_all(1),
            ),
        };
        let ground = TraitRef::new(
            Path::new("demo", "Show"),
            vec![generic_box.clone().apply(vec![Type::Integer])],
        );
        let non_ground = TraitRef::new(
            Path::new("demo", "Show"),
            vec![generic_box.apply(vec![Type::MetaVar(0)])],
        );

        assert!(predicate_is_ground(&ground));
        assert!(!predicate_is_ground(&non_ground));
    }
}
