//! Shared helpers for resolve submodules.

use super::{
    TraitRef,
    Type,
};

/// Render a trait reference in source-like form for diagnostics.
pub(super) fn format_trait_ref(trait_ref: &TraitRef) -> String {
    if trait_ref.arguments.is_empty() {
        trait_ref.trait_name.to_string()
    } else {
        let args = trait_ref
            .arguments
            .iter()
            .map(format_trait_argument)
            .collect::<Vec<_>>()
            .join(" ");
        format!("{} {args}", trait_ref.trait_name)
    }
}

fn format_trait_argument(type_: &Type) -> String {
    let rendered = type_.pretty();
    match type_ {
        Type::Unit
        | Type::Integer
        | Type::Natural
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_)
        | Type::Named { .. }
        | Type::Tuple(_) => rendered,
        _ => format!("({rendered})"),
    }
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
        assert_eq!(
            format_trait_ref(&with_args),
            "demo::Eq Integer ([] Boolean)"
        );
    }

    #[test]
    fn format_trait_ref_wraps_complex_arguments() {
        let with_function_arg = TraitRef::new(
            Path::new("demo", "Show"),
            vec![Type::func(Type::Integer, Type::Boolean)],
        );

        assert_eq!(
            format_trait_ref(&with_function_arg),
            "demo::Show (Integer -> Boolean)"
        );
    }
}
