use super::{
    TraitConstraint,
    TraitRef,
    Type,
    for_each_child_type,
};

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

pub(super) fn predicate_is_ground(predicate: &TraitConstraint) -> bool {
    predicate.arguments.iter().all(type_is_ground)
}

fn type_is_ground(type_: &Type) -> bool {
    match type_ {
        Type::TypeVar(_) | Type::MetaVar(_) => false,
        _ => {
            let mut is_ground = true;
            for_each_child_type(type_, true, |child| {
                if is_ground && !type_is_ground(child) {
                    is_ground = false;
                }
            });
            is_ground
        }
    }
}
