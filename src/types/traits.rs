use indexmap::IndexMap;

use crate::ir::Path;

use super::Type;

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

/// Definition of a trait and its method signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    pub name: Path,
    pub parameters: usize,
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

pub(crate) fn ordered_trait_methods(def: &TraitDef) -> Vec<(Path, TypeScheme)> {
    let mut methods = def
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
    NoInstance {
        predicate: TraitConstraint,
    },
}
