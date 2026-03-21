//! Shared predicate helpers for resolve and elaboration.

use super::{
    TraitConstraint,
    Type,
    for_each_child_type,
};

/// Return `true` when a predicate has no type/meta variables.
pub(crate) fn predicate_is_ground(predicate: &TraitConstraint) -> bool {
    predicate.arguments.iter().all(is_ground_type)
}

/// Deterministic key used to order predicates.
pub(crate) fn predicate_sort_key(predicate: &TraitConstraint) -> String {
    let args = predicate
        .arguments
        .iter()
        .map(predicate_type_key)
        .collect::<Vec<_>>()
        .join("_");
    if args.is_empty() {
        format!(
            "{}::{}",
            predicate.trait_name.major, predicate.trait_name.minor
        )
    } else {
        format!(
            "{}::{} {args}",
            predicate.trait_name.major, predicate.trait_name.minor
        )
    }
}

/// Stable sorted + deduplicated predicate list.
pub(crate) fn sorted_unique_predicates(predicates: &[TraitConstraint]) -> Vec<TraitConstraint> {
    let mut sorted = predicates.to_vec();
    sorted.sort_by_key(predicate_sort_key);
    sorted.dedup();
    sorted
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

fn predicate_type_key(type_: &Type) -> String {
    type_
        .pretty()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Path;

    #[test]
    fn predicate_is_ground_detects_variables_and_nested_ground_types() {
        let non_ground_type_var = TraitConstraint::new(Path::new("demo", "Eq"), vec![Type::v(0)]);
        let non_ground_meta = TraitConstraint::new(Path::new("demo", "Eq"), vec![Type::MetaVar(0)]);
        let ground = TraitConstraint::new(
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
        let ground = TraitConstraint::new(
            Path::new("demo", "Show"),
            vec![generic_box.clone().apply(vec![Type::Integer])],
        );
        let non_ground = TraitConstraint::new(
            Path::new("demo", "Show"),
            vec![generic_box.apply(vec![Type::MetaVar(0)])],
        );

        assert!(predicate_is_ground(&ground));
        assert!(!predicate_is_ground(&non_ground));
    }

    #[test]
    fn sorted_unique_predicates_orders_and_deduplicates() {
        let show_int = TraitConstraint::new(Path::new("demo", "Show"), vec![Type::Integer]);
        let show_bool = TraitConstraint::new(Path::new("demo", "Show"), vec![Type::Boolean]);
        let sorted =
            sorted_unique_predicates(&[show_int.clone(), show_bool.clone(), show_int.clone()]);

        assert_eq!(sorted, vec![show_bool, show_int]);
    }
}
