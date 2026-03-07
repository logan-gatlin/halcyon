use super::{
    TraitConstraint,
    TraitRef,
    Type,
};

pub(crate) fn leading_forall_count(type_: &Type) -> usize {
    let mut count = 0;
    let mut current = type_;
    while let Type::ForAll(body) = current {
        count += 1;
        current = body;
    }
    count
}

pub(crate) fn peel_leading_foralls(type_: &Type) -> (usize, Type) {
    let mut count = 0;
    let mut current = type_.clone();
    while let Type::ForAll(body) = current {
        count += 1;
        current = *body;
    }
    (count, current)
}

pub(crate) fn instantiate_forall_strict(
    type_: &Type,
    arguments: &[Type],
) -> Option<Type> {
    arguments
        .iter()
        .try_fold(type_.clone(), |current, argument| {
            let Type::ForAll(body) = current else {
                return None;
            };
            body.open_forall(argument)
        })
}

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

pub(crate) fn instantiate_predicates(
    predicates: &[TraitConstraint],
    arguments: &[Type],
) -> Option<Vec<TraitConstraint>> {
    predicates
        .iter()
        .map(|predicate| {
            let arguments = predicate
                .arguments
                .iter()
                .map(|argument| instantiate_type_vars(argument, arguments))
                .collect::<Option<Vec<_>>>()?;
            Some(TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments,
            })
        })
        .collect()
}
