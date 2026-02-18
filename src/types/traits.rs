use std::collections::HashMap;

use crate::ir::Path;

use super::unify::UnificationTable;
use super::{
    Type,
    TypeParameterIndex,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    pub name: Path,
    pub parameters: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImpl {
    pub parameters: TypeParameterIndex,
    pub head: TraitRef,
    pub predicates: Vec<TraitConstraint>,
}

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

#[derive(Debug, Clone, Default)]
pub struct TraitEnv {
    trait_defs: HashMap<Path, TraitDef>,
    trait_impls: HashMap<Path, Vec<TraitImpl>>,
}

impl TraitEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_trait(
        &mut self,
        def: TraitDef,
    ) -> Result<(), TraitError> {
        if self.trait_defs.contains_key(&def.name) {
            return Err(TraitError::DuplicateTrait(def.name));
        }
        self.trait_defs.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn insert_impl(
        &mut self,
        impl_: TraitImpl,
    ) -> Result<(), TraitError> {
        let def = self
            .trait_defs
            .get(&impl_.head.trait_name)
            .ok_or_else(|| TraitError::UnknownTrait(impl_.head.trait_name.clone()))?;
        if def.parameters != impl_.head.arguments.len() {
            return Err(TraitError::ArityMismatch {
                trait_name: impl_.head.trait_name.clone(),
                expected: def.parameters,
                found: impl_.head.arguments.len(),
            });
        }
        let impls = self
            .trait_impls
            .entry(impl_.head.trait_name.clone())
            .or_default();
        for existing in impls.iter() {
            if instances_overlap(existing, &impl_)? {
                return Err(TraitError::OverlappingInstance {
                    trait_name: impl_.head.trait_name.clone(),
                    left: existing.head.clone(),
                    right: impl_.head.clone(),
                });
            }
        }
        impls.push(impl_);
        Ok(())
    }

    pub fn resolve_predicates(
        &self,
        table: &mut UnificationTable,
        predicates: &[TraitConstraint],
    ) -> Result<Vec<TraitConstraint>, TraitError> {
        let mut unresolved = Vec::new();
        let mut stack = Vec::new();
        for predicate in predicates {
            for remaining in self.resolve_predicate(table, predicate, &mut stack)? {
                push_unique(&mut unresolved, remaining);
            }
        }
        Ok(unresolved)
    }

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

    fn resolve_predicate(
        &self,
        table: &mut UnificationTable,
        predicate: &TraitConstraint,
        stack: &mut Vec<TraitConstraint>,
    ) -> Result<Vec<TraitConstraint>, TraitError> {
        let normalized = normalize_trait_ref(table, predicate);
        let def = self
            .trait_defs
            .get(&normalized.trait_name)
            .ok_or_else(|| TraitError::UnknownTrait(normalized.trait_name.clone()))?;
        if def.parameters != normalized.arguments.len() {
            return Err(TraitError::ArityMismatch {
                trait_name: normalized.trait_name.clone(),
                expected: def.parameters,
                found: normalized.arguments.len(),
            });
        }
        if stack
            .iter()
            .any(|entry| normalize_trait_ref(table, entry) == normalized)
        {
            return Err(TraitError::RecursivePredicate {
                predicate: normalized,
            });
        }

        stack.push(normalized.clone());

        let mut matched: Option<(UnificationTable, Vec<TraitConstraint>)> = None;
        let mut ambiguous = false;
        if let Some(candidates) = self.trait_impls.get(&normalized.trait_name) {
            for candidate in candidates {
                let mut local_table = table.clone();
                let instantiated = instantiate_trait_impl(&mut local_table, candidate)?;
                if matches_trait_ref(&mut local_table, &normalized, &instantiated.head) {
                    if matched.is_some() {
                        ambiguous = true;
                        break;
                    }
                    matched = Some((local_table, instantiated.predicates));
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
            for predicate in context {
                for remaining in self.resolve_predicate(table, &predicate, stack)? {
                    push_unique(&mut pending, remaining);
                }
            }
            Ok(pending)
        } else {
            Ok(vec![normalized])
        };

        stack.pop();
        result
    }
}

struct InstantiatedImpl {
    head: TraitRef,
    predicates: Vec<TraitConstraint>,
}

fn normalize_trait_ref(
    table: &mut UnificationTable,
    trait_ref: &TraitRef,
) -> TraitRef {
    TraitRef {
        trait_name: trait_ref.trait_name.clone(),
        arguments: trait_ref
            .arguments
            .iter()
            .map(|arg| table.normalize(arg))
            .collect(),
    }
}

fn instantiate_trait_impl(
    table: &mut UnificationTable,
    impl_: &TraitImpl,
) -> Result<InstantiatedImpl, TraitError> {
    let replacements = std::iter::repeat_with(|| table.new_meta(0))
        .take(impl_.parameters as usize)
        .collect::<Vec<_>>();
    let head = substitute_type_vars_in_trait_ref(&impl_.head, &replacements)?;
    let predicates = substitute_type_vars_in_predicates(&impl_.predicates, &replacements)?;
    Ok(InstantiatedImpl { head, predicates })
}

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

fn substitute_type_vars_in_predicates(
    predicates: &[TraitConstraint],
    replacements: &[Type],
) -> Result<Vec<TraitConstraint>, TraitError> {
    predicates
        .iter()
        .map(|predicate| substitute_type_vars_in_trait_ref(predicate, replacements))
        .collect()
}

fn substitute_type_vars(
    type_: &Type,
    replacements: &[Type],
) -> Option<Type> {
    let mut current = type_.clone();
    for (index, replacement) in replacements.iter().enumerate() {
        current = current.substitute_type_var(index as u32, replacement)?;
    }
    Some(current)
}

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

fn push_unique(
    predicates: &mut Vec<TraitConstraint>,
    predicate: TraitConstraint,
) {
    if !predicates.contains(&predicate) {
        predicates.push(predicate);
    }
}
