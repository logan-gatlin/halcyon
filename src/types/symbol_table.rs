//! Global type-system symbol table.
//!
//! Stores terms, types, trait definitions, and trait implementations, and
//! provides trait-instance selection/resolution utilities shared by resolve and
//! elaboration.

use std::collections::{
    HashMap,
    HashSet,
};

use indexmap::IndexMap;
use rayon::prelude::*;

use crate::ir::Path;

use super::kind::{
    KindError,
    KindInferenceTable,
    constructor_kind,
    infer_type_kind,
};
use super::traits::{
    TraitConstraint,
    TraitDef,
    TraitError,
    TraitImpl,
    TraitRef,
    TypeScheme,
};
use super::unify::UnificationTable;
use super::{
    Kind,
    Type,
    TypeTransform,
    normalize_parameter_kinds,
};

/// Classification of global symbols stored in the symbol table.
pub enum SymbolKind {
    Term(TypeScheme),
    Type(TypeDefinition),
    TraitDef(TraitDef),
}

impl From<TypeScheme> for SymbolKind {
    fn from(value: TypeScheme) -> Self {
        Self::Term(value)
    }
}

impl From<TypeDefinition> for SymbolKind {
    fn from(value: TypeDefinition) -> Self {
        Self::Type(value)
    }
}

impl From<TraitDef> for SymbolKind {
    fn from(value: TraitDef) -> Self {
        Self::TraitDef(value)
    }
}

/// Anything that can be registered in the global symbol table.
pub trait Symbol {
    fn path(&self) -> Path;
    fn symbol_kind(&self) -> SymbolKind;
}

/// Global definitions shared across modules during typechecking.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SymbolTable {
    terms: IndexMap<Path, TypeScheme>,
    types: IndexMap<Path, TypeDefinition>,
    constructors: std::collections::HashSet<Path>,
    constructor_aliases: IndexMap<Path, Path>,
    trait_defs: IndexMap<Path, TraitDef>,
    trait_aliases: IndexMap<Path, Path>,
    trait_impls: IndexMap<Path, Vec<TraitImpl>>,
    #[serde(skip)]
    trait_impl_indexes: IndexMap<Path, TraitImplIndex>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct TraitImplIndex {
    all: Vec<usize>,
    by_position: Vec<IndexMap<TypeHeadKey, Vec<usize>>>,
    wildcard_by_position: Vec<Vec<usize>>,
}

impl TraitImplIndex {
    fn build(implementations: &[TraitImpl]) -> Self {
        let arity = implementations
            .iter()
            .map(|implementation| implementation.head.arguments.len())
            .max()
            .unwrap_or(0);

        let mut index = Self {
            all: Vec::with_capacity(implementations.len()),
            by_position: vec![IndexMap::new(); arity],
            wildcard_by_position: vec![Vec::new(); arity],
        };

        for (impl_index, implementation) in implementations.iter().enumerate() {
            index.all.push(impl_index);
            for (position, argument) in implementation.head.arguments.iter().enumerate() {
                if let Some(head_key) = type_head_key(argument) {
                    index
                        .by_position
                        .get_mut(position)
                        .unwrap_or_else(|| unreachable!("position must be in bounds"))
                        .entry(head_key)
                        .or_default()
                        .push(impl_index);
                } else {
                    index
                        .wildcard_by_position
                        .get_mut(position)
                        .unwrap_or_else(|| unreachable!("position must be in bounds"))
                        .push(impl_index);
                }
            }
        }

        index
    }

    fn candidate_indices(
        &self,
        predicate: &TraitRef,
    ) -> Vec<usize> {
        let mut filtered: Option<Vec<usize>> = None;

        for (position, argument) in predicate.arguments.iter().enumerate() {
            let Some(head_key) = type_head_key(argument) else {
                continue;
            };
            if !head_key.is_rigid() {
                continue;
            };

            let wildcard = self
                .wildcard_by_position
                .get(position)
                .cloned()
                .unwrap_or_default();
            let keyed = self
                .by_position
                .get(position)
                .and_then(|index| index.get(&head_key))
                .cloned()
                .unwrap_or_default();

            if wildcard.is_empty() && keyed.is_empty() {
                return Vec::new();
            }

            let mut constrained = wildcard;
            constrained.extend(keyed);
            constrained.sort_unstable();
            constrained.dedup();

            filtered = Some(match filtered {
                None => constrained,
                Some(existing) => intersect_sorted_indices(&existing, &constrained),
            });

            if filtered.as_ref().is_some_and(Vec::is_empty) {
                return Vec::new();
            }
        }

        filtered.unwrap_or_else(|| self.all.clone())
    }
}

fn intersect_sorted_indices(
    left: &[usize],
    right: &[usize],
) -> Vec<usize> {
    let mut result = Vec::new();
    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            std::cmp::Ordering::Less => {
                left_index += 1;
            }
            std::cmp::Ordering::Greater => {
                right_index += 1;
            }
            std::cmp::Ordering::Equal => {
                result.push(left[left_index]);
                left_index += 1;
                right_index += 1;
            }
        }
    }

    result
}

#[derive(Debug, Clone)]
pub struct MethodSpecialization {
    /// Trait that provided the selected method.
    pub trait_name: Path,

    /// Concrete impl method path to call.
    pub impl_method_path: Path,

    /// Context predicates required by the selected impl.
    pub predicates: Vec<TraitConstraint>,
}

impl SymbolTable {
    /// Create an empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge symbols from another table, preserving existing entries when identical.
    pub fn absorb(
        &mut self,
        other: &SymbolTable,
    ) {
        self.terms.extend(other.terms.clone());
        self.types.extend(other.types.clone());
        self.constructors.extend(other.constructors.iter().cloned());
        self.constructor_aliases
            .extend(other.constructor_aliases.clone());
        self.trait_defs.extend(other.trait_defs.clone());
        self.trait_aliases.extend(other.trait_aliases.clone());
        for (trait_name, implementations) in &other.trait_impls {
            let entry = self.trait_impls.entry(trait_name.clone()).or_default();
            for implementation in implementations {
                if !entry.contains(implementation) {
                    entry.push(implementation.clone());
                }
            }
            self.rebuild_trait_impl_index_for_trait(trait_name);
        }
    }

    fn rebuild_trait_impl_index_for_trait(
        &mut self,
        trait_name: &Path,
    ) {
        let Some(implementations) = self.trait_impls.get(trait_name) else {
            self.trait_impl_indexes.shift_remove(trait_name);
            return;
        };
        self.trait_impl_indexes
            .insert(trait_name.clone(), TraitImplIndex::build(implementations));
    }

    pub fn rebuild_derived_indexes(&mut self) {
        self.trait_impl_indexes.clear();
        let trait_names = self.trait_impls.keys().cloned().collect::<Vec<_>>();
        for trait_name in trait_names {
            self.rebuild_trait_impl_index_for_trait(&trait_name);
        }
    }

    fn candidate_impl_indices(
        &self,
        predicate: &TraitRef,
    ) -> Vec<usize> {
        let Some(candidates) = self.trait_impls.get(&predicate.trait_name) else {
            return Vec::new();
        };

        self.trait_impl_indexes
            .get(&predicate.trait_name)
            .map(|index| index.candidate_indices(predicate))
            .unwrap_or_else(|| (0..candidates.len()).collect())
    }

    /// Insert any symbol kind and return the previous value if present.
    pub fn insert(
        &mut self,
        symbol: impl Symbol,
    ) -> Option<SymbolKind> {
        match symbol.symbol_kind() {
            SymbolKind::Term(value) => self.terms.insert(symbol.path(), value).map(Into::into),
            SymbolKind::Type(value) => self.types.insert(symbol.path(), value).map(Into::into),
            SymbolKind::TraitDef(value) => {
                self.trait_defs.insert(symbol.path(), value).map(Into::into)
            }
        }
    }

    /// Insert many symbols.
    pub fn extend<I, S>(
        &mut self,
        symbols: I,
    ) where
        I: IntoIterator<Item = S>,
        S: Symbol,
    {
        for symbol in symbols {
            self.insert(symbol);
        }
    }

    /// Borrow all term bindings.
    pub fn terms(&self) -> &IndexMap<Path, TypeScheme> {
        &self.terms
    }

    /// Borrow all trait definitions.
    pub fn trait_defs(&self) -> &IndexMap<Path, TraitDef> {
        &self.trait_defs
    }

    /// Borrow all known constructor paths.
    pub fn constructors(&self) -> &std::collections::HashSet<Path> {
        &self.constructors
    }

    /// Borrow constructor aliases (`alias -> target`).
    pub fn constructor_aliases(&self) -> &IndexMap<Path, Path> {
        &self.constructor_aliases
    }

    /// Register a constructor path.
    pub fn insert_constructor(
        &mut self,
        path: Path,
    ) -> bool {
        self.constructors.insert(path)
    }

    /// Register a constructor alias (`alias -> target`).
    pub fn insert_constructor_alias(
        &mut self,
        alias: Path,
        target: Path,
    ) -> Option<Path> {
        self.constructors.insert(alias.clone());
        self.constructor_aliases.insert(alias, target)
    }

    /// Borrow trait aliases (`alias -> target`).
    pub fn trait_aliases(&self) -> &IndexMap<Path, Path> {
        &self.trait_aliases
    }

    /// Resolve trait aliases to the canonical trait definition path.
    pub fn canonical_trait_path(
        &self,
        trait_name: &Path,
    ) -> Option<Path> {
        let mut current = trait_name.clone();
        let mut seen = HashSet::new();
        loop {
            if self.trait_defs.contains_key(&current) {
                return Some(current);
            }
            if !seen.insert(current.clone()) {
                return None;
            }
            let next = self.trait_aliases.get(&current)?;
            current = next.clone();
        }
    }

    /// Lookup a trait definition through alias indirection.
    pub fn trait_definition(
        &self,
        trait_name: &Path,
    ) -> Option<&TraitDef> {
        let canonical = self.canonical_trait_path(trait_name)?;
        self.trait_defs.get(&canonical)
    }

    /// Borrow all trait implementations grouped by trait path.
    pub fn trait_impls(&self) -> &IndexMap<Path, Vec<TraitImpl>> {
        &self.trait_impls
    }

    /// Insert or replace one term binding.
    pub fn insert_term(
        &mut self,
        path: Path,
        scheme: impl Into<TypeScheme>,
    ) -> Option<TypeScheme> {
        self.terms.insert(path, scheme.into())
    }

    /// Borrow all type definitions.
    pub fn type_definitions(&self) -> &IndexMap<Path, TypeDefinition> {
        &self.types
    }

    /// Insert or replace one type definition.
    pub fn insert_type(
        &mut self,
        path: Path,
        definition: TypeDefinition,
    ) -> Option<TypeDefinition> {
        self.types.insert(path, definition)
    }

    /// Register a new trait definition.
    pub fn insert_trait(
        &mut self,
        trait_definition: TraitDef,
    ) -> Result<(), TraitError> {
        if self.trait_defs.contains_key(&trait_definition.name)
            || self.trait_aliases.contains_key(&trait_definition.name)
        {
            return Err(TraitError::DuplicateTrait(trait_definition.name));
        }
        self.trait_defs
            .insert(trait_definition.name.clone(), trait_definition);
        Ok(())
    }

    /// Register a trait alias (`alias -> target`).
    pub fn insert_trait_alias(
        &mut self,
        alias: Path,
        target: Path,
    ) -> Result<(), TraitError> {
        if self.trait_defs.contains_key(&alias) || self.trait_aliases.contains_key(&alias) {
            return Err(TraitError::DuplicateTrait(alias));
        }
        let canonical_target = self.canonical_trait_path(&target).ok_or_else(|| {
            TraitError::InvalidAliasTarget {
                alias: alias.clone(),
                target,
            }
        })?;
        self.trait_aliases.insert(alias, canonical_target);
        Ok(())
    }

    /// Register a trait implementation after coherence and shape checks.
    pub fn insert_impl(
        &mut self,
        trait_implementation: TraitImpl,
    ) -> Result<(), TraitError> {
        let mut trait_implementation = trait_implementation;
        let canonical_trait_name = self
            .canonical_trait_path(&trait_implementation.head.trait_name)
            .ok_or_else(|| {
                TraitError::UnknownTrait(trait_implementation.head.trait_name.clone())
            })?;
        trait_implementation.head.trait_name = canonical_trait_name.clone();
        let trait_definition = self.trait_defs.get(&canonical_trait_name).ok_or_else(|| {
            TraitError::UnknownTrait(trait_implementation.head.trait_name.clone())
        })?;
        if trait_definition.parameters != trait_implementation.head.arguments.len() {
            return Err(TraitError::ArityMismatch {
                trait_name: trait_implementation.head.trait_name.clone(),
                expected: trait_definition.parameters,
                found: trait_implementation.head.arguments.len(),
            });
        }
        validate_impl_head_kinds(self, trait_definition, &trait_implementation)?;
        let has_unknown_method = trait_implementation
            .methods
            .keys()
            .any(|method| !trait_definition.methods.contains_key(method));
        if has_unknown_method {
            return Err(TraitError::InvalidInstance {
                trait_name: trait_implementation.head.trait_name.clone(),
            });
        }
        let missing_method = trait_definition
            .methods
            .keys()
            .any(|method| !trait_implementation.methods.contains_key(method));
        if missing_method {
            return Err(TraitError::InvalidInstance {
                trait_name: trait_implementation.head.trait_name.clone(),
            });
        }
        let impls = self
            .trait_impls
            .entry(canonical_trait_name.clone())
            .or_default();
        for existing in impls.iter() {
            if instances_overlap(existing, &trait_implementation)? {
                return Err(TraitError::OverlappingInstance {
                    trait_name: trait_implementation.head.trait_name.clone(),
                    left: existing.head.clone(),
                    right: trait_implementation.head.clone(),
                });
            }
        }
        impls.push(trait_implementation);
        self.rebuild_trait_impl_index_for_trait(&canonical_trait_name);
        Ok(())
    }

    /// Resolve predicates recursively, returning unresolved residual predicates.
    pub fn resolve_predicates(
        &self,
        table: &mut UnificationTable,
        predicates: &[TraitConstraint],
    ) -> Result<Vec<TraitConstraint>, TraitError> {
        self.resolve_predicates_with_assumptions(table, predicates, &[])
    }

    /// Resolve predicates recursively under additional assumed predicates.
    pub fn resolve_predicates_with_assumptions(
        &self,
        table: &mut UnificationTable,
        predicates: &[TraitConstraint],
        assumptions: &[TraitConstraint],
    ) -> Result<Vec<TraitConstraint>, TraitError> {
        let _profile_total = crate::profiling::scope("symbols.resolve_predicates.total");
        let mut unresolved = Vec::new();
        let mut stack = Vec::new();
        let mut memo = HashMap::<Vec<u8>, Result<Vec<TraitConstraint>, TraitError>>::new();
        for predicate in predicates {
            for remaining in
                self.resolve_predicate(table, predicate, assumptions, &mut stack, &mut memo)?
            {
                if self.predicate_matches_assumptions(table, &remaining, assumptions) {
                    continue;
                }
                push_unique(&mut unresolved, remaining);
            }
        }
        Ok(unresolved)
    }

    /// Resolve predicates and fail if any remain unresolved.
    pub fn resolve_predicates_strict(
        &self,
        table: &mut UnificationTable,
        predicates: &[TraitConstraint],
    ) -> Result<(), TraitError> {
        let unresolved = self.resolve_predicates(table, predicates)?;
        if let Some(predicate) = unresolved.into_iter().next() {
            Err(TraitError::NoInstance { predicate })
        } else {
            Ok(())
        }
    }

    /// Select a unique implementation matching `predicate`.
    pub fn select_impl(
        &self,
        predicate: &TraitConstraint,
    ) -> Result<Option<TraitImpl>, TraitError> {
        let mut table = UnificationTable::default();
        let mut normalized = table.normalize_trait_ref(predicate);
        let canonical_trait_name = self
            .canonical_trait_path(&normalized.trait_name)
            .ok_or_else(|| TraitError::UnknownTrait(normalized.trait_name.clone()))?;
        normalized.trait_name = canonical_trait_name;
        let trait_definition = self
            .trait_defs
            .get(&normalized.trait_name)
            .ok_or_else(|| TraitError::UnknownTrait(normalized.trait_name.clone()))?;
        if trait_definition.parameters != normalized.arguments.len() {
            return Err(TraitError::ArityMismatch {
                trait_name: normalized.trait_name.clone(),
                expected: trait_definition.parameters,
                found: normalized.arguments.len(),
            });
        }

        let mut matched: Option<TraitImpl> = None;
        if let Some(candidates) = self.trait_impls.get(&normalized.trait_name) {
            for candidate in self
                .candidate_impl_indices(&normalized)
                .into_iter()
                .filter_map(|index| candidates.get(index))
            {
                if !impl_head_may_match_predicate(&normalized, &candidate.head) {
                    continue;
                }
                let mut local_table = UnificationTable::default();
                let instantiated = instantiate_trait_impl(&mut local_table, candidate)?;
                if matches_trait_ref(&mut local_table, &normalized, &instantiated.head) {
                    if matched.is_some() {
                        return Err(TraitError::AmbiguousInstance {
                            predicate: normalized,
                        });
                    }
                    matched = Some(candidate.clone());
                }
            }
        }
        Ok(matched)
    }

    /// Resolve trait method dispatch to a concrete impl method and context predicates.
    pub fn resolve_method_specialization(
        &self,
        method_path: &Path,
        arguments: &[Type],
    ) -> Result<Option<MethodSpecialization>, TraitError> {
        let Some((trait_name, _)) = self
            .trait_defs
            .iter()
            .find(|(_, trait_definition)| trait_definition.methods.contains_key(method_path))
        else {
            return Ok(None);
        };
        let predicate = TraitRef::new(trait_name.clone(), arguments.to_vec());
        let Some(selected_impl) = self.select_impl(&predicate)? else {
            return Ok(None);
        };
        let Some(impl_method_path) = selected_impl.methods.get(method_path).cloned() else {
            return Err(TraitError::InvalidInstance {
                trait_name: trait_name.clone(),
            });
        };

        let mut table = UnificationTable::default();
        let instantiated = instantiate_trait_impl(&mut table, &selected_impl)?;
        if !matches_trait_ref(&mut table, &predicate, &instantiated.head) {
            return Ok(None);
        }
        let predicates = instantiated
            .predicates
            .into_iter()
            .map(|predicate| table.normalize_trait_ref(&predicate))
            .collect();

        Ok(Some(MethodSpecialization {
            trait_name: trait_name.clone(),
            impl_method_path,
            predicates,
        }))
    }

    fn predicate_matches_assumptions(
        &self,
        table: &mut UnificationTable,
        predicate: &TraitConstraint,
        assumptions: &[TraitConstraint],
    ) -> bool {
        let _profile = crate::profiling::scope("symbols.predicate_matches_assumptions");
        for assumption in assumptions {
            let mut local_table = table.clone();
            let mut normalized_predicate = local_table.normalize_trait_ref(predicate);
            if let Some(canonical) = self.canonical_trait_path(&normalized_predicate.trait_name) {
                normalized_predicate.trait_name = canonical;
            }
            let mut normalized_assumption = local_table.normalize_trait_ref(assumption);
            if let Some(canonical) = self.canonical_trait_path(&normalized_assumption.trait_name) {
                normalized_assumption.trait_name = canonical;
            }
            if matches_trait_ref(
                &mut local_table,
                &normalized_predicate,
                &normalized_assumption,
            ) {
                *table = local_table;
                return true;
            }
        }
        false
    }

    /// Internal recursive predicate resolver used by [`Self::resolve_predicates`].
    fn resolve_predicate(
        &self,
        table: &mut UnificationTable,
        predicate: &TraitConstraint,
        assumptions: &[TraitConstraint],
        stack: &mut Vec<TraitConstraint>,
        memo: &mut HashMap<Vec<u8>, Result<Vec<TraitConstraint>, TraitError>>,
    ) -> Result<Vec<TraitConstraint>, TraitError> {
        let _profile_total = crate::profiling::scope("symbols.resolve_predicate.total");
        if self.predicate_matches_assumptions(table, predicate, assumptions) {
            return Ok(Vec::new());
        }
        let mut normalized = table.normalize_trait_ref(predicate);
        let canonical_trait_name = self
            .canonical_trait_path(&normalized.trait_name)
            .ok_or_else(|| TraitError::UnknownTrait(normalized.trait_name.clone()))?;
        normalized.trait_name = canonical_trait_name;
        let trait_definition = self
            .trait_defs
            .get(&normalized.trait_name)
            .ok_or_else(|| TraitError::UnknownTrait(normalized.trait_name.clone()))?;
        if trait_definition.parameters != normalized.arguments.len() {
            return Err(TraitError::ArityMismatch {
                trait_name: normalized.trait_name.clone(),
                expected: trait_definition.parameters,
                found: normalized.arguments.len(),
            });
        }
        if stack
            .iter()
            .any(|entry| table.normalize_trait_ref(entry) == normalized)
        {
            return Err(TraitError::RecursivePredicate {
                predicate: normalized,
            });
        }

        let memo_key = predicate_solver_memo_key(&normalized, assumptions);
        if let Some(key) = memo_key.as_ref()
            && let Some(cached) = memo.get(key)
        {
            return cached.clone();
        }

        stack.push(normalized.clone());

        let mut matched: Option<(UnificationTable, Vec<TraitConstraint>)> = None;
        let mut ambiguous = false;
        if let Some(candidates) = self.trait_impls.get(&normalized.trait_name) {
            {
                let _profile = crate::profiling::scope("symbols.resolve_predicate.candidates");
                let candidate_indices = self.candidate_impl_indices(&normalized);
                let selected_candidates = candidate_indices
                    .into_iter()
                    .filter_map(|index| candidates.get(index))
                    .collect::<Vec<_>>();

                let candidate_results = if selected_candidates.len() > 8 {
                    selected_candidates
                        .par_iter()
                        .map(|candidate| {
                            if !impl_head_may_match_predicate(&normalized, &candidate.head) {
                                return Ok(None);
                            }
                            let mut local_table = table.clone();
                            let instantiated = instantiate_trait_impl(&mut local_table, candidate)?;
                            if matches_trait_ref(&mut local_table, &normalized, &instantiated.head) {
                                Ok(Some((local_table, instantiated.predicates)))
                            } else {
                                Ok(None)
                            }
                        })
                        .collect::<Vec<Result<Option<(UnificationTable, Vec<TraitConstraint>)>, TraitError>>>()
                } else {
                    selected_candidates
                        .iter()
                        .map(|candidate| {
                            if !impl_head_may_match_predicate(&normalized, &candidate.head) {
                                return Ok(None);
                            }
                            let mut local_table = table.clone();
                            let instantiated = instantiate_trait_impl(&mut local_table, candidate)?;
                            if matches_trait_ref(&mut local_table, &normalized, &instantiated.head) {
                                Ok(Some((local_table, instantiated.predicates)))
                            } else {
                                Ok(None)
                            }
                        })
                        .collect::<Vec<Result<Option<(UnificationTable, Vec<TraitConstraint>)>, TraitError>>>()
                };

                for candidate_result in candidate_results {
                    if let Some((candidate_table, candidate_predicates)) = candidate_result? {
                        if matched.is_some() {
                            ambiguous = true;
                            break;
                        }
                        matched = Some((candidate_table, candidate_predicates));
                    }
                }
            }
        }

        let result = if ambiguous {
            Err(TraitError::AmbiguousInstance {
                predicate: normalized,
            })
        } else if let Some((winning_table, context)) = matched {
            *table = winning_table;
            let mut pending = Vec::new();
            {
                let _profile = crate::profiling::scope("symbols.resolve_predicate.context");
                for predicate in context {
                    for remaining in
                        self.resolve_predicate(table, &predicate, assumptions, stack, memo)?
                    {
                        if self.predicate_matches_assumptions(table, &remaining, assumptions) {
                            continue;
                        }
                        push_unique(&mut pending, remaining);
                    }
                }
            }
            Ok(pending)
        } else {
            Ok(vec![normalized])
        };

        stack.pop();
        if let Some(key) = memo_key {
            memo.insert(key, result.clone());
        }
        result
    }
}

/// Definition of a named type with its parameters and body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeDefinitionKind {
    /// Nominal type; identity is name-based.
    Named,

    /// Alias type; expanded structurally when referenced.
    Alias,
}

/// Stored type definition with arity, body, and nominal/alias kind.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeDefinition {
    pub parameters: usize,
    pub parameter_kinds: Vec<Kind>,
    pub body: Type,
    pub kind: TypeDefinitionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
enum TypeHeadKey {
    Unit,
    Integer,
    Real,
    Boolean,
    String,
    Glyph,
    Named(Path),
    Array,
    Tuple(usize),
    Struct,
    Sum,
    Function,
}

impl TypeHeadKey {
    fn is_rigid(&self) -> bool {
        match self {
            Self::Unit
            | Self::Integer
            | Self::Real
            | Self::Boolean
            | Self::String
            | Self::Glyph
            | Self::Array
            | Self::Tuple(_)
            | Self::Sum
            | Self::Function => true,
            Self::Named(_) | Self::Struct => false,
        }
    }
}

fn type_head_key(type_: &Type) -> Option<TypeHeadKey> {
    match type_ {
        Type::Unit => Some(TypeHeadKey::Unit),
        Type::Integer => Some(TypeHeadKey::Integer),
        Type::Real => Some(TypeHeadKey::Real),
        Type::Boolean => Some(TypeHeadKey::Boolean),
        Type::String => Some(TypeHeadKey::String),
        Type::Glyph => Some(TypeHeadKey::Glyph),
        Type::Named { name, .. } => Some(TypeHeadKey::Named(name.clone())),
        Type::Array(_) => Some(TypeHeadKey::Array),
        Type::Tuple(items) => Some(TypeHeadKey::Tuple(items.len())),
        Type::Struct { .. } | Type::StructConstraint { .. } => Some(TypeHeadKey::Struct),
        Type::Sum { .. } => Some(TypeHeadKey::Sum),
        Type::Function(..) => Some(TypeHeadKey::Function),
        Type::Apply {
            constructor,
            arguments: _,
        } => type_head_key(constructor),
        Type::TypeVar(_) | Type::MetaVar(_) | Type::ForAll { .. } => None,
    }
}

fn argument_head_compatible(
    predicate_argument: &Type,
    impl_head_argument: &Type,
) -> bool {
    let Some(predicate_key) = type_head_key(predicate_argument) else {
        return true;
    };
    let Some(impl_head_key) = type_head_key(impl_head_argument) else {
        return true;
    };
    if !predicate_key.is_rigid() || !impl_head_key.is_rigid() {
        return true;
    }
    impl_head_key == predicate_key
}

fn impl_head_may_match_predicate(
    predicate: &TraitRef,
    impl_head: &TraitRef,
) -> bool {
    if predicate.arguments.len() != impl_head.arguments.len() {
        return false;
    }

    predicate
        .arguments
        .iter()
        .zip(impl_head.arguments.iter())
        .all(|(predicate_argument, impl_head_argument)| {
            argument_head_compatible(predicate_argument, impl_head_argument)
        })
}

fn type_is_ground_for_memo(type_: &Type) -> bool {
    match type_ {
        Type::Unit | Type::Integer | Type::Real | Type::Boolean | Type::String | Type::Glyph => {
            true
        }
        Type::Named { .. } => true,
        Type::Array(item) => type_is_ground_for_memo(item),
        Type::Tuple(items) => items.iter().all(type_is_ground_for_memo),
        Type::Struct { fields } | Type::StructConstraint { fields, .. } => {
            fields.values().all(type_is_ground_for_memo)
        }
        Type::Sum { variants } => variants.values().all(type_is_ground_for_memo),
        Type::Function(parameter, result) => {
            type_is_ground_for_memo(parameter) && type_is_ground_for_memo(result)
        }
        Type::Apply {
            constructor,
            arguments,
        } => type_is_ground_for_memo(constructor) && arguments.iter().all(type_is_ground_for_memo),
        Type::TypeVar(_) | Type::MetaVar(_) | Type::ForAll { .. } => false,
    }
}

fn trait_ref_is_ground_for_memo(trait_ref: &TraitRef) -> bool {
    trait_ref.arguments.iter().all(type_is_ground_for_memo)
}

fn predicate_solver_memo_key(
    predicate: &TraitConstraint,
    assumptions: &[TraitConstraint],
) -> Option<Vec<u8>> {
    if !trait_ref_is_ground_for_memo(predicate)
        || assumptions
            .iter()
            .any(|assumption| !trait_ref_is_ground_for_memo(assumption))
    {
        return None;
    }

    postcard::to_stdvec(&(predicate, assumptions)).ok()
}

struct InstantiatedTraitImpl {
    head: TraitRef,
    predicates: Vec<TraitConstraint>,
}

/// Instantiate a trait impl head/context with fresh metavariables.
fn instantiate_trait_impl(
    table: &mut UnificationTable,
    trait_implementation: &TraitImpl,
) -> Result<InstantiatedTraitImpl, TraitError> {
    let replacements = std::iter::repeat_with(|| table.new_meta(0))
        .take(trait_implementation.parameters)
        .collect::<Vec<_>>();
    let head = substitute_type_vars_in_trait_ref(&trait_implementation.head, &replacements)?;
    let predicates =
        substitute_type_vars_in_predicates(&trait_implementation.predicates, &replacements)?;
    Ok(InstantiatedTraitImpl { head, predicates })
}

/// Substitute impl parameters in a trait reference.
fn substitute_type_vars_in_trait_ref(
    trait_ref: &TraitRef,
    replacements: &[Type],
) -> Result<TraitRef, TraitError> {
    let arguments = trait_ref
        .arguments
        .iter()
        .map(|arg| substitute_type_vars(arg, replacements))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            TraitError::InvalidInstance {
                trait_name: trait_ref.trait_name.clone(),
            }
        })?;
    Ok(TraitRef {
        trait_name: trait_ref.trait_name.clone(),
        arguments,
    })
}

/// Substitute impl parameters in a predicate list.
fn substitute_type_vars_in_predicates(
    predicates: &[TraitConstraint],
    replacements: &[Type],
) -> Result<Vec<TraitConstraint>, TraitError> {
    predicates
        .iter()
        .map(|predicate| substitute_type_vars_in_trait_ref(predicate, replacements))
        .collect()
}

/// Substitute De Bruijn type variables simultaneously using `replacements`.
///
/// All substitutions are performed in one pass so that variables introduced by
/// one replacement are never captured by a subsequent substitution.
fn substitute_type_vars(
    type_: &Type,
    replacements: &[Type],
) -> Option<Type> {
    struct SimultaneousSubstitution<'a> {
        replacements: &'a [Type],
        depth: super::TypeParameterIndex,
    }

    impl TypeTransform for SimultaneousSubstitution<'_> {
        fn type_var(
            &mut self,
            index: super::TypeParameterIndex,
        ) -> Option<Type> {
            if index >= self.depth
                && let Some(replacement) = self.replacements.get((index - self.depth) as usize)
            {
                replacement.shift_type_vars(self.depth as i32, 0)
            } else {
                Some(Type::TypeVar(index))
            }
        }

        fn enter_forall(&mut self) {
            self.depth += 1;
        }

        fn leave_forall(&mut self) {
            self.depth -= 1;
        }
    }

    SimultaneousSubstitution {
        replacements,
        depth: 0,
    }
    .transform(type_)
}

/// Check whether `predicate` matches an instantiated impl head.
fn matches_trait_ref(
    table: &mut UnificationTable,
    predicate: &TraitRef,
    head: &TraitRef,
) -> bool {
    if predicate.trait_name != head.trait_name || predicate.arguments.len() != head.arguments.len()
    {
        return false;
    }
    predicate
        .arguments
        .iter()
        .zip(head.arguments.iter())
        .try_for_each(|(left, right)| table.unify(left, right))
        .is_ok()
}

/// Conservative overlap check for two impl heads.
fn instances_overlap(
    left: &TraitImpl,
    right: &TraitImpl,
) -> Result<bool, TraitError> {
    if left.head.trait_name != right.head.trait_name
        || left.head.arguments.len() != right.head.arguments.len()
    {
        return Ok(false);
    }
    let mut table = UnificationTable::default();
    let left_inst = instantiate_trait_impl(&mut table, left)?;
    let right_inst = instantiate_trait_impl(&mut table, right)?;
    Ok(matches_trait_ref(
        &mut table,
        &left_inst.head,
        &right_inst.head,
    ))
}

fn validate_impl_head_kinds(
    symbols: &SymbolTable,
    trait_definition: &TraitDef,
    trait_implementation: &TraitImpl,
) -> Result<(), TraitError> {
    let expected_kinds = normalize_parameter_kinds(
        trait_definition.parameter_kinds.clone(),
        trait_definition.parameters,
    );
    let mut kind_table = KindInferenceTable::default();
    let mut bound_kinds = std::iter::repeat_with(|| kind_table.new_meta())
        .take(trait_implementation.parameters)
        .collect::<Vec<_>>();
    for (argument, expected_kind) in trait_implementation
        .head
        .arguments
        .iter()
        .zip(expected_kinds.iter())
    {
        let inferred_kind =
            infer_type_kind(&mut kind_table, argument, &mut bound_kinds, &|type_path| {
                symbols.types.get(type_path).map(|definition| {
                    constructor_kind(definition.parameters, &definition.parameter_kinds)
                })
            })
            .map_err(|error| {
                match error {
                    KindError::Mismatch { left, right } => {
                        TraitError::KindMismatch {
                            trait_name: trait_implementation.head.trait_name.clone(),
                            expected: right,
                            found: left,
                        }
                    }
                    KindError::Occurs { in_kind, .. } => {
                        TraitError::KindMismatch {
                            trait_name: trait_implementation.head.trait_name.clone(),
                            expected: expected_kind.clone(),
                            found: in_kind,
                        }
                    }
                }
            })?;
        let expected_kind_inferred = KindInferenceTable::from_kind(expected_kind);
        if let Err(error) = kind_table.unify(&inferred_kind, &expected_kind_inferred) {
            let (found, expected) = match error {
                KindError::Mismatch { left, right } => (left, right),
                KindError::Occurs { in_kind, .. } => (in_kind, expected_kind.clone()),
            };
            return Err(TraitError::KindMismatch {
                trait_name: trait_implementation.head.trait_name.clone(),
                expected,
                found,
            });
        }
    }
    Ok(())
}

/// Push `predicate` only if it is not already present.
fn push_unique(
    predicates: &mut Vec<TraitConstraint>,
    predicate: TraitConstraint,
) {
    if !predicates.contains(&predicate) {
        predicates.push(predicate);
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::Logger;
    use crate::hc_core::compile_core_module;

    use super::*;

    fn list_of(inner: Type) -> Type {
        Type::Named {
            name: Path::new("demo", "List"),
            body: Box::new(Type::Unit),
        }
        .apply(vec![inner])
    }

    fn eq_method_scheme() -> TypeScheme {
        Type::curry(&[Type::v(0), Type::v(0), Type::Boolean]).scheme()
    }

    fn trait_with_eq_method(name: &str) -> TraitDef {
        TraitDef {
            name: Path::new("demo", name),
            parameters: 1,
            parameter_kinds: vec![Kind::Type],
            methods: [(Path::new("demo", "eq"), eq_method_scheme())]
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn insert_and_extend_route_symbols_by_kind() {
        #[derive(Clone)]
        struct TermSymbol {
            path: Path,
            scheme: TypeScheme,
        }

        impl Symbol for TermSymbol {
            fn path(&self) -> Path {
                self.path.clone()
            }

            fn symbol_kind(&self) -> SymbolKind {
                SymbolKind::Term(self.scheme.clone())
            }
        }

        #[derive(Clone)]
        struct TypeSymbol {
            path: Path,
            definition: TypeDefinition,
        }

        impl Symbol for TypeSymbol {
            fn path(&self) -> Path {
                self.path.clone()
            }

            fn symbol_kind(&self) -> SymbolKind {
                SymbolKind::Type(self.definition.clone())
            }
        }

        #[derive(Clone)]
        struct TraitSymbol {
            definition: TraitDef,
        }

        impl Symbol for TraitSymbol {
            fn path(&self) -> Path {
                self.definition.name.clone()
            }

            fn symbol_kind(&self) -> SymbolKind {
                SymbolKind::TraitDef(self.definition.clone())
            }
        }

        let mut symbols = SymbolTable::new();
        symbols.extend([
            TermSymbol {
                path: Path::new("demo", "id"),
                scheme: Type::func(Type::Integer, Type::Integer).scheme(),
            },
            TermSymbol {
                path: Path::new("demo", "const"),
                scheme: Type::Integer.scheme(),
            },
        ]);
        symbols.insert(TypeSymbol {
            path: Path::new("demo", "Token"),
            definition: Type::Integer.def_named(0),
        });
        symbols.insert(TraitSymbol {
            definition: TraitDef::new(Path::new("demo", "Eq"), 1),
        });

        assert!(symbols.terms().contains_key(&Path::new("demo", "id")));
        assert!(
            symbols
                .type_definitions()
                .contains_key(&Path::new("demo", "Token"))
        );
        assert!(symbols.trait_defs().contains_key(&Path::new("demo", "Eq")));
    }

    #[test]
    fn insert_trait_rejects_duplicates() {
        let mut symbols = SymbolTable::new();
        let trait_def = TraitDef::new(Path::new("demo", "Eq"), 1);

        symbols
            .insert_trait(trait_def.clone())
            .expect("first trait insertion should succeed");
        assert!(matches!(
            symbols.insert_trait(trait_def),
            Err(TraitError::DuplicateTrait(_))
        ));
    }

    #[test]
    fn trait_aliases_canonicalize_to_trait_definitions() {
        let mut symbols = SymbolTable::new();
        let trait_name = Path::new("demo", "Eq");
        let alias = Path::new("demo", "Equal");
        symbols
            .insert_trait(TraitDef::new(trait_name.clone(), 1))
            .expect("trait insertion should succeed");
        symbols
            .insert_trait_alias(alias.clone(), trait_name.clone())
            .expect("trait alias insertion should succeed");

        assert_eq!(
            symbols.canonical_trait_path(&alias),
            Some(trait_name.clone())
        );
        assert!(symbols.trait_definition(&alias).is_some());
        assert_eq!(symbols.trait_aliases().get(&alias), Some(&trait_name));
    }

    #[test]
    fn insert_trait_alias_rejects_non_trait_targets() {
        let mut symbols = SymbolTable::new();
        let alias = Path::new("demo", "Equal");
        let target = Path::new("demo", "Token");

        assert!(matches!(
            symbols.insert_trait_alias(alias.clone(), target.clone()),
            Err(TraitError::InvalidAliasTarget {
                alias: failed_alias,
                target: failed_target,
            }) if failed_alias == alias && failed_target == target
        ));
    }

    #[test]
    fn insert_impl_validates_trait_shape_and_methods() {
        let mut symbols = SymbolTable::new();
        let trait_def = trait_with_eq_method("Eq");
        symbols
            .insert_trait(trait_def.clone())
            .expect("trait insertion should succeed");

        assert!(matches!(
            symbols.insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Missing"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            }),
            Err(TraitError::UnknownTrait(_))
        ));

        assert!(matches!(
            symbols.insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer, Type::Boolean]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            }),
            Err(TraitError::ArityMismatch { .. })
        ));

        assert!(matches!(
            symbols.insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: [(Path::new("demo", "unknown"), Path::new("demo", "impl_eq"))]
                    .into_iter()
                    .collect(),
            }),
            Err(TraitError::InvalidInstance { .. })
        ));

        assert!(matches!(
            symbols.insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            }),
            Err(TraitError::InvalidInstance { .. })
        ));

        symbols
            .insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: [(Path::new("demo", "eq"), Path::new("demo", "eq_integer"))]
                    .into_iter()
                    .collect(),
            })
            .expect("valid impl should insert");
    }

    #[test]
    fn insert_impl_rejects_wrong_kind_trait_arguments() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef {
                name: Path::new("demo", "Monad"),
                parameters: 1,
                parameter_kinds: vec![Kind::arrow(Kind::Type, Kind::Type)],
                methods: IndexMap::new(),
            })
            .expect("trait insertion should succeed");

        assert!(matches!(
            symbols.insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Monad"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            }),
            Err(TraitError::KindMismatch {
                trait_name,
                expected,
                found,
            }) if trait_name == Path::new("demo", "Monad")
                && expected == Kind::arrow(Kind::Type, Kind::Type)
                && found == Kind::Type
        ));
    }

    #[test]
    fn insert_impl_and_select_impl_work_through_trait_aliases() {
        let mut symbols = SymbolTable::new();
        let trait_name = Path::new("demo", "Eq");
        let alias = Path::new("demo", "Equal");
        symbols
            .insert_trait(TraitDef::new(trait_name.clone(), 1))
            .expect("trait insertion should succeed");
        symbols
            .insert_trait_alias(alias.clone(), trait_name.clone())
            .expect("trait alias insertion should succeed");

        symbols
            .insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(alias.clone(), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            })
            .expect("impl insertion through alias should succeed");

        let selected = symbols
            .select_impl(&TraitRef::new(alias, vec![Type::Integer]))
            .expect("selection should succeed")
            .expect("expected matching impl");
        assert_eq!(selected.head.trait_name, trait_name);
    }

    #[test]
    fn overlap_detection_rejects_conflicting_impls() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef::new(Path::new("demo", "Eq"), 1))
            .expect("trait insertion should succeed");

        symbols
            .insert_impl(TraitImpl {
                parameters: 1,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![list_of(Type::v(0))]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            })
            .expect("generic list impl should insert");

        assert!(matches!(
            symbols.insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![list_of(Type::Integer)]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            }),
            Err(TraitError::OverlappingInstance { .. })
        ));
    }

    #[test]
    fn resolve_predicates_handles_resolution_assumptions_and_strict_mode() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef::new(Path::new("demo", "Show"), 1))
            .expect("trait insertion should succeed");
        symbols
            .insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Show"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            })
            .expect("impl insertion should succeed");

        let mut table = UnificationTable::default();
        let unresolved = symbols
            .resolve_predicates(
                &mut table,
                &[TraitRef::new(
                    Path::new("demo", "Show"),
                    vec![Type::Integer],
                )],
            )
            .expect("resolution should succeed");
        assert!(unresolved.is_empty());

        let meta = table.new_meta(0);
        let predicate = TraitRef::new(Path::new("demo", "Show"), vec![meta.clone()]);
        let unresolved = symbols
            .resolve_predicates_with_assumptions(
                &mut table,
                std::slice::from_ref(&predicate),
                std::slice::from_ref(&predicate),
            )
            .expect("assumption should discharge predicate");
        assert!(unresolved.is_empty());

        let missing = TraitRef::new(Path::new("demo", "Show"), vec![Type::Boolean]);
        assert!(matches!(
            symbols.resolve_predicates_strict(&mut table, &[missing]),
            Err(TraitError::NoInstance { .. })
        ));
    }

    #[test]
    fn resolve_predicates_detects_recursive_predicates() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef::new(Path::new("demo", "Eq"), 1))
            .expect("trait insertion should succeed");
        symbols
            .insert_impl(TraitImpl {
                parameters: 1,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)]),
                predicates: vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])],
                methods: IndexMap::new(),
            })
            .expect("recursive impl insertion should succeed");

        let mut table = UnificationTable::default();
        let predicate = TraitRef::new(Path::new("demo", "Eq"), vec![table.new_meta(0)]);
        assert!(matches!(
            symbols.resolve_predicates(&mut table, &[predicate]),
            Err(TraitError::RecursivePredicate { .. })
        ));
    }

    #[test]
    fn select_impl_returns_none_or_errors_as_expected() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef::new(Path::new("demo", "Eq"), 1))
            .expect("trait insertion should succeed");

        assert!(matches!(
            symbols.select_impl(&TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer])),
            Ok(None)
        ));

        symbols
            .insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: IndexMap::new(),
            })
            .expect("impl insertion should succeed");

        let selected = symbols
            .select_impl(&TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer]))
            .expect("selection should succeed")
            .expect("expected matching impl");
        assert_eq!(selected.head.arguments, vec![Type::Integer]);

        assert!(matches!(
            symbols.select_impl(&TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::Integer, Type::Boolean],
            )),
            Err(TraitError::ArityMismatch { .. })
        ));

        assert!(matches!(
            symbols.select_impl(&TraitRef::new(
                Path::new("demo", "Missing"),
                vec![Type::Integer]
            )),
            Err(TraitError::UnknownTrait(_))
        ));
    }

    #[test]
    fn select_impl_detects_ambiguity_when_candidates_are_manually_conflicting() {
        let mut symbols = SymbolTable::new();
        let trait_name = Path::new("demo", "Eq");
        symbols
            .insert_trait(TraitDef::new(trait_name.clone(), 1))
            .expect("trait insertion should succeed");

        symbols
            .trait_impls
            .entry(trait_name.clone())
            .or_default()
            .extend([
                TraitImpl {
                    parameters: 0,
                    head: TraitRef::new(trait_name.clone(), vec![Type::Integer]),
                    predicates: Vec::new(),
                    methods: IndexMap::new(),
                },
                TraitImpl {
                    parameters: 0,
                    head: TraitRef::new(trait_name.clone(), vec![Type::Integer]),
                    predicates: Vec::new(),
                    methods: IndexMap::new(),
                },
            ]);

        assert!(matches!(
            symbols.select_impl(&TraitRef::new(trait_name, vec![Type::Integer])),
            Err(TraitError::AmbiguousInstance { .. })
        ));
    }

    #[test]
    fn resolve_method_specialization_handles_success_and_missing_cases() {
        let mut symbols = SymbolTable::new();
        let trait_name = Path::new("demo", "Eq");
        let method_path = Path::new("demo", "eq");
        let impl_method = Path::new("demo", "eq_integer");
        symbols
            .insert_trait(trait_with_eq_method("Eq"))
            .expect("trait insertion should succeed");

        assert!(matches!(
            symbols.resolve_method_specialization(&method_path, &[Type::Integer]),
            Ok(None)
        ));

        symbols
            .insert_impl(TraitImpl {
                parameters: 0,
                head: TraitRef::new(trait_name.clone(), vec![Type::Integer]),
                predicates: Vec::new(),
                methods: [(method_path.clone(), impl_method.clone())]
                    .into_iter()
                    .collect(),
            })
            .expect("impl insertion should succeed");

        let specialization = symbols
            .resolve_method_specialization(&method_path, &[Type::Integer])
            .expect("resolution should succeed")
            .expect("expected specialization");
        assert_eq!(specialization.trait_name, trait_name);
        assert_eq!(specialization.impl_method_path, impl_method);
        assert!(specialization.predicates.is_empty());

        assert!(matches!(
            symbols.resolve_method_specialization(&Path::new("demo", "unknown"), &[Type::Integer]),
            Ok(None)
        ));
    }

    #[test]
    fn resolve_method_specialization_propagates_context_predicates() {
        let mut symbols = SymbolTable::new();
        let trait_name = Path::new("demo", "Eq");
        let method_path = Path::new("demo", "eq");

        symbols
            .insert_trait(trait_with_eq_method("Eq"))
            .expect("trait insertion should succeed");
        symbols
            .insert_impl(TraitImpl {
                parameters: 1,
                head: TraitRef::new(trait_name.clone(), vec![list_of(Type::v(0))]),
                predicates: vec![TraitRef::new(trait_name.clone(), vec![Type::v(0)])],
                methods: [(method_path.clone(), Path::new("demo", "eq_list"))]
                    .into_iter()
                    .collect(),
            })
            .expect("impl insertion should succeed");

        let specialization = symbols
            .resolve_method_specialization(&method_path, &[list_of(Type::Integer)])
            .expect("resolution should succeed")
            .expect("expected specialization");

        assert_eq!(
            specialization.predicates,
            vec![TraitRef::new(trait_name, vec![Type::Integer])]
        );
    }

    #[test]
    fn core_show_option_specialization_propagates_inner_show_predicate() {
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let method_path = Path::new("core", "show::show");
        let option_integer = Type::Named {
            name: Path::new("core", "opt::Option"),
            body: Box::new(Type::Unit),
        }
        .apply(vec![Type::Integer]);

        let specialization = symbols
            .resolve_method_specialization(&method_path, &[option_integer])
            .expect("resolution should succeed")
            .expect("expected specialization");

        println!("specialization: {:?}", specialization);
        println!(
            "impl scheme: {:?}",
            symbols.terms().get(&specialization.impl_method_path)
        );

        assert_eq!(
            specialization.predicates,
            vec![TraitRef::new(
                Path::new("core", "show::Show"),
                vec![Type::Integer]
            )]
        );
    }
}
