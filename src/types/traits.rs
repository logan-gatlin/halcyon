//! Trait-domain data structures shared by inference and resolution.

use indexmap::IndexMap;

use crate::ir::Path;

use super::{
    Kind,
    Type,
};

/// A trait constraint applied to type arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitRef {
    pub trait_name: Path,
    pub arguments: Vec<Type>,
}

impl TraitRef {
    pub fn new(
        trait_name: Path,
        arguments: Vec<Type>,
    ) -> Self {
        Self {
            trait_name,
            arguments,
        }
    }
}

pub type TraitConstraint = TraitRef;

/// A type scheme with attached trait predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    pub predicates: Vec<TraitConstraint>,
    pub type_: Type,
}

impl TypeScheme {
    pub fn new(type_: Type) -> Self {
        Self {
            predicates: Vec::new(),
            type_,
        }
    }

    pub fn with_predicates(
        type_: Type,
        predicates: Vec<TraitConstraint>,
    ) -> Self {
        Self { predicates, type_ }
    }
}

impl From<Type> for TypeScheme {
    fn from(type_: Type) -> Self {
        Self::new(type_)
    }
}

/// Definition of a trait and its trait-item signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    pub name: Path,
    pub parameters: usize,
    pub parameter_kinds: Vec<Kind>,
    pub methods: IndexMap<Path, TypeScheme>,
}

impl TraitDef {
    pub fn new(
        name: Path,
        parameters: usize,
    ) -> Self {
        Self {
            name,
            parameters,
            parameter_kinds: vec![Kind::Type; parameters],
            methods: Default::default(),
        }
    }

    pub fn method(
        mut self,
        path: Path,
        type_scheme: TypeScheme,
    ) -> Self {
        self.methods.insert(path, type_scheme);
        self
    }
}

/// Return trait items sorted by stable path key for deterministic lowering.
pub(crate) fn ordered_trait_methods(trait_definition: &TraitDef) -> Vec<(Path, TypeScheme)> {
    let mut methods = trait_definition
        .methods
        .iter()
        .map(|(path, scheme)| (path.clone(), scheme.clone()))
        .collect::<Vec<_>>();
    methods.sort_by(|(left, _), (right, _)| {
        (left.major.clone(), left.minor.clone()).cmp(&(right.major.clone(), right.minor.clone()))
    });
    methods
}

/// A trait implementation head with its context predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImpl {
    pub parameters: usize,
    pub head: TraitRef,
    pub predicates: Vec<TraitConstraint>,
    pub methods: IndexMap<Path, Path>,
}

/// Errors that can occur when defining or resolving traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitError {
    UnknownTrait(Path),
    DuplicateTrait(Path),
    ArityMismatch {
        trait_name: Path,
        expected: usize,
        found: usize,
    },
    OverlappingInstance {
        trait_name: Path,
        left: TraitRef,
        right: TraitRef,
    },
    AmbiguousInstance {
        predicate: TraitConstraint,
    },
    RecursivePredicate {
        predicate: TraitConstraint,
    },
    InvalidInstance {
        trait_name: Path,
    },
    InvalidAliasTarget {
        alias: Path,
        target: Path,
    },
    KindMismatch {
        trait_name: Path,
        expected: Kind,
        found: Kind,
    },
    NoInstance {
        predicate: TraitConstraint,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_scheme_constructors_and_from_type_match() {
        let scheme = TypeScheme::new(Type::Integer);
        assert!(scheme.predicates.is_empty());
        assert_eq!(scheme.type_, Type::Integer);

        let with_predicates = TypeScheme::with_predicates(
            Type::Boolean,
            vec![TraitRef::new(
                Path::new("demo", "Show"),
                vec![Type::Boolean],
            )],
        );
        assert_eq!(
            with_predicates.predicates,
            vec![TraitRef::new(
                Path::new("demo", "Show"),
                vec![Type::Boolean]
            )]
        );

        let via_from: TypeScheme = Type::Glyph.into();
        assert_eq!(via_from.type_, Type::Glyph);
    }

    #[test]
    fn trait_def_builder_adds_methods() {
        let trait_def = TraitDef::new(Path::new("demo", "Eq"), 1).method(
            Path::new("demo", "eq"),
            Type::curry(&[Type::v(0), Type::v(0), Type::Boolean]).scheme(),
        );

        assert_eq!(trait_def.methods.len(), 1);
        assert!(trait_def.methods.contains_key(&Path::new("demo", "eq")));
    }

    #[test]
    fn ordered_trait_methods_are_sorted_by_path() {
        let trait_def = TraitDef {
            name: Path::new("demo", "Ord"),
            parameters: 1,
            parameter_kinds: vec![Kind::Type],
            methods: [
                (Path::new("demo", "z"), Type::Integer.scheme()),
                (Path::new("demo", "a"), Type::Integer.scheme()),
                (Path::new("demo", "m"), Type::Integer.scheme()),
            ]
            .into_iter()
            .collect(),
        };

        let ordered = ordered_trait_methods(&trait_def);
        let names = ordered
            .into_iter()
            .map(|(path, _)| path.minor)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a", "m", "z"]);
    }
}
