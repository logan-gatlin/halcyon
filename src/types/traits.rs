//! Trait-domain data structures shared by inference and resolution.

use indexmap::IndexMap;

use crate::ir::Path;

use super::{
    Kind,
    Type,
    predicate_is_ground,
};

/// A trait constraint applied to type arguments.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    pub fn pretty(&self) -> String {
        let type_ = self.type_.pretty();
        let constraints = self
            .predicates
            .iter()
            .filter(|predicate| !predicate_is_ground(predicate))
            .map(format_trait_constraint)
            .collect::<Vec<_>>();
        if constraints.is_empty() {
            return type_;
        }

        let constraints = constraints.join(", ");
        format!("{type_} where {constraints}")
    }
}

impl std::fmt::Display for TypeScheme {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_str(&self.pretty())
    }
}

impl From<Type> for TypeScheme {
    fn from(type_: Type) -> Self {
        Self::new(type_)
    }
}

/// Definition of a trait and its trait-item signatures.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TraitDef {
    pub name: Path,
    pub parameters: usize,
    pub parameter_kinds: Vec<Kind>,
    pub associated_types: IndexMap<Path, Kind>,
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
            associated_types: Default::default(),
            methods: Default::default(),
        }
    }

    pub fn associated_type(
        mut self,
        path: Path,
        kind: Kind,
    ) -> Self {
        self.associated_types.insert(path, kind);
        self
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

fn format_trait_constraint(constraint: &TraitConstraint) -> String {
    if constraint.arguments.is_empty() {
        return constraint.trait_name.to_string();
    }

    let arguments = constraint
        .arguments
        .iter()
        .map(format_trait_constraint_argument)
        .collect::<Vec<_>>()
        .join(" ");
    format!("{} {arguments}", constraint.trait_name)
}

fn format_trait_constraint_argument(type_: &Type) -> String {
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

/// A trait implementation head with its context predicates.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TraitImpl {
    pub parameters: usize,
    pub head: TraitRef,
    pub predicates: Vec<TraitConstraint>,
    pub associated_types: IndexMap<Path, Type>,
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
    InvalidInstanceItems {
        trait_name: Path,
        unknown_items: Vec<Path>,
        missing_items: Vec<Path>,
        unknown_associated_types: Vec<Path>,
        missing_associated_types: Vec<Path>,
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
    AssociatedTypeKindMismatch {
        trait_name: Path,
        associated_type: Path,
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
            associated_types: Default::default(),
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

    #[test]
    fn type_scheme_pretty_uses_where_clause_for_predicates() {
        let scheme = TypeScheme::with_predicates(
            Type::ForAll {
                name: None,
                body: Box::new(Type::func(Type::v(0), Type::v(0))),
            },
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])],
        );

        assert_eq!(scheme.pretty(), "for a in a -> a where demo::Eq a");
        assert_eq!(scheme.to_string(), "for a in a -> a where demo::Eq a");
    }

    #[test]
    fn type_scheme_pretty_hides_ground_predicates() {
        let scheme = TypeScheme::with_predicates(
            Type::Integer,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer])],
        );

        assert_eq!(scheme.pretty(), "Integer");
        assert_eq!(scheme.to_string(), "Integer");
    }
}
