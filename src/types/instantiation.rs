//! Utilities for instantiating `Type::ForAll` binders and trait predicates.

use super::{
    TraitRef,
    Type,
};

/// Count contiguous `for all` binders at the head of `type_`.
pub(crate) fn leading_forall_count(type_: &Type) -> usize {
    let mut count = 0;
    let mut current = type_;
    while let Type::ForAll { body, .. } = current {
        count += 1;
        current = body;
    }
    count
}

/// Peel contiguous leading `for all` binders, returning `(count, body)`.
pub(crate) fn peel_leading_foralls(type_: &Type) -> (usize, Type) {
    let mut count = 0;
    let mut current = type_.clone();
    while let Type::ForAll { body, .. } = current {
        count += 1;
        current = *body;
    }
    (count, current)
}

/// Instantiate exactly one binder per provided argument.
///
/// Returns `None` when arguments outnumber available leading binders.
pub(crate) fn instantiate_forall_strict(
    type_: &Type,
    arguments: &[Type],
) -> Option<Type> {
    arguments
        .iter()
        .try_fold(type_.clone(), |current, argument| {
            let Type::ForAll { body, .. } = current else {
                return None;
            };
            body.open_forall(argument)
        })
}

/// Repeatedly open binders with arguments until arguments are exhausted.
pub(crate) fn instantiate_type_vars(
    type_: &Type,
    arguments: &[Type],
) -> Option<Type> {
    arguments
        .iter()
        .try_fold(type_.clone(), |current, argument| {
            current.open_forall(argument)
        })
}

/// Instantiate every predicate argument with `arguments`.
pub(crate) fn instantiate_predicates(
    predicates: &[TraitRef],
    arguments: &[Type],
) -> Option<Vec<TraitRef>> {
    predicates
        .iter()
        .map(|predicate| {
            let instantiated_arguments = predicate
                .arguments
                .iter()
                .map(|argument| instantiate_type_vars(argument, arguments))
                .collect::<Option<Vec<_>>>()?;
            Some(TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments: instantiated_arguments,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Path;

    #[test]
    fn leading_forall_count_only_counts_head_binders() {
        let type_ = Type::func(Type::v(0).for_all(1), Type::Integer).for_all(2);
        assert_eq!(leading_forall_count(&type_), 2);
    }

    #[test]
    fn peel_leading_foralls_returns_count_and_body() {
        let type_ = Type::func(Type::v(1), Type::v(0)).for_all(2);
        let (count, body) = peel_leading_foralls(&type_);

        assert_eq!(count, 2);
        assert_eq!(body, Type::func(Type::v(1), Type::v(0)));
    }

    #[test]
    fn instantiate_forall_strict_succeeds_for_matching_arity() {
        let type_ = Type::func(Type::v(1), Type::v(0)).for_all(2);
        let instantiated = instantiate_forall_strict(&type_, &[Type::Integer, Type::Boolean])
            .expect("instantiation should succeed");

        assert_eq!(instantiated, Type::func(Type::Integer, Type::Boolean));
    }

    #[test]
    fn instantiate_forall_strict_rejects_extra_arguments() {
        let type_ = Type::v(0).for_all(1);
        assert!(instantiate_forall_strict(&type_, &[Type::Integer, Type::Boolean]).is_none());
    }

    #[test]
    fn instantiate_type_vars_can_open_non_forall_bodies() {
        let type_ = Type::func(Type::v(0), Type::v(1));
        let instantiated = instantiate_type_vars(&type_, &[Type::Integer])
            .expect("open_forall semantics should apply");

        assert_eq!(instantiated, Type::func(Type::Integer, Type::v(0)));
    }

    #[test]
    fn instantiate_type_vars_fails_when_replacement_shift_overflows() {
        let type_ = Type::v(0);
        assert!(instantiate_type_vars(&type_, &[Type::v(u32::MAX)]).is_none());
    }

    #[test]
    fn instantiate_predicates_rewrites_all_arguments() {
        let predicate = TraitRef::new(
            Path::new("demo", "Eq"),
            vec![Type::func(Type::v(1), Type::v(0))],
        );
        let instantiated = instantiate_predicates(&[predicate], &[Type::Integer, Type::Boolean])
            .expect("predicate instantiation should succeed");

        assert_eq!(
            instantiated,
            vec![TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::func(Type::Boolean, Type::Integer)],
            )]
        );
    }

    /// `instantiate_type_vars` maps TypeVar(k) → args[k] via sequential
    /// `open_forall` calls. Each call replaces TypeVar(0) and shifts the
    /// rest down, so TypeVar(k) ends up matching args[k].
    #[test]
    fn instantiate_type_vars_maps_index_to_same_position() {
        let args = [Type::Integer, Type::Boolean, Type::String];
        assert_eq!(
            instantiate_type_vars(&Type::v(0), &args).unwrap(),
            Type::Integer,
        );
        assert_eq!(
            instantiate_type_vars(&Type::v(1), &args).unwrap(),
            Type::Boolean,
        );
        assert_eq!(
            instantiate_type_vars(&Type::v(2), &args).unwrap(),
            Type::String,
        );
    }

    /// `instantiate_forall_strict` opens ForAlls from the outside in, so
    /// the *outermost* binder (TypeVar(N-1) in the body) gets args[0],
    /// while the *innermost* binder (TypeVar(0)) gets args[N-1].
    ///
    /// This means `instantiate_forall_strict` and `instantiate_type_vars`
    /// have REVERSED mappings: TypeVar(k) → args[k] in type_vars, but
    /// TypeVar(k) → args[N-1-k] in forall_strict.
    #[test]
    fn instantiate_forall_strict_reverses_mapping_relative_to_type_vars() {
        // Scheme: for a in for b in (a -> b)
        // In body: TypeVar(1) = a (outermost), TypeVar(0) = b (innermost)
        let scheme_type = Type::func(Type::v(1), Type::v(0)).for_all(2);

        let body = instantiate_forall_strict(&scheme_type, &[Type::Integer, Type::Boolean])
            .expect("forall instantiation should succeed");

        // forall_strict: TypeVar(1) → args[0] = Integer, TypeVar(0) → args[1] = Boolean
        assert_eq!(body, Type::func(Type::Integer, Type::Boolean));

        // type_vars gives the opposite mapping:
        // TypeVar(1) → args[1] = Boolean, TypeVar(0) → args[0] = Integer
        assert_eq!(
            instantiate_type_vars(&Type::v(1), &[Type::Integer, Type::Boolean]).unwrap(),
            Type::Boolean,
        );
        assert_eq!(
            instantiate_type_vars(&Type::v(0), &[Type::Integer, Type::Boolean]).unwrap(),
            Type::Integer,
        );
    }
}
