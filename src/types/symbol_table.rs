use indexmap::IndexMap;

use crate::ir::Path;

use super::Type;
use super::traits::{
    TraitConstraint,
    TraitDef,
    TraitError,
    TraitImpl,
    TraitRef,
    TypeScheme,
};
use super::unify::UnificationTable;

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
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    terms: IndexMap<Path, TypeScheme>,
    types: IndexMap<Path, TypeDefinition>,
    trait_defs: IndexMap<Path, TraitDef>,
    trait_impls: IndexMap<Path, Vec<TraitImpl>>,
}

#[derive(Debug, Clone)]
pub struct MethodSpecialization {
    pub trait_name: Path,
    pub impl_method_path: Path,
    pub predicates: Vec<TraitConstraint>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        s: impl Symbol,
    ) -> Option<SymbolKind> {
        match s.symbol_kind() {
            SymbolKind::Term(v) => self.terms.insert(s.path(), v).map(Into::into),
            SymbolKind::Type(v) => self.types.insert(s.path(), v).map(Into::into),
            SymbolKind::TraitDef(v) => self.trait_defs.insert(s.path(), v).map(Into::into),
        }
    }

    pub fn extend<I, S>(
        &mut self,
        it: I,
    ) where
        I: IntoIterator<Item = S>,
        S: Symbol,
    {
        for s in it.into_iter() {
            self.insert(s);
        }
    }

    pub fn terms(&self) -> &IndexMap<Path, TypeScheme> {
        &self.terms
    }

    pub fn trait_defs(&self) -> &IndexMap<Path, TraitDef> {
        &self.trait_defs
    }

    pub fn trait_impls(&self) -> &IndexMap<Path, Vec<TraitImpl>> {
        &self.trait_impls
    }

    pub fn insert_term(
        &mut self,
        path: Path,
        scheme: impl Into<TypeScheme>,
    ) -> Option<TypeScheme> {
        self.terms.insert(path, scheme.into())
    }

    pub fn type_definitions(&self) -> &IndexMap<Path, TypeDefinition> {
        &self.types
    }

    pub fn insert_type(
        &mut self,
        path: Path,
        definition: TypeDefinition,
    ) -> Option<TypeDefinition> {
        self.types.insert(path, definition)
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
        let has_unknown_method = impl_
            .methods
            .keys()
            .any(|method| !def.methods.contains_key(method));
        if has_unknown_method {
            return Err(TraitError::InvalidInstance {
                trait_name: impl_.head.trait_name.clone(),
            });
        }
        let missing_method = def
            .methods
            .keys()
            .any(|method| !impl_.methods.contains_key(method));
        if missing_method {
            return Err(TraitError::InvalidInstance {
                trait_name: impl_.head.trait_name.clone(),
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

    pub fn select_impl(
        &self,
        predicate: &TraitConstraint,
    ) -> Result<Option<TraitImpl>, TraitError> {
        let mut table = UnificationTable::default();
        let normalized = table.normalize_trait_ref(predicate);
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

        let mut matched: Option<TraitImpl> = None;
        if let Some(candidates) = self.trait_impls.get(&normalized.trait_name) {
            for candidate in candidates {
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

    pub fn resolve_method_specialization(
        &self,
        method_path: &Path,
        arguments: &[Type],
    ) -> Result<Option<MethodSpecialization>, TraitError> {
        let Some((trait_name, _)) = self
            .trait_defs
            .iter()
            .find(|(_, def)| def.methods.contains_key(method_path))
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

    fn resolve_predicate(
        &self,
        table: &mut UnificationTable,
        predicate: &TraitConstraint,
        stack: &mut Vec<TraitConstraint>,
    ) -> Result<Vec<TraitConstraint>, TraitError> {
        let normalized = table.normalize_trait_ref(predicate);
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
            .any(|entry| table.normalize_trait_ref(entry) == normalized)
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

/// Definition of a named type with its parameters and body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDefinitionKind {
    Named,
    Alias,
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub parameters: usize,
    pub body: Type,
    pub kind: TypeDefinitionKind,
}

struct InstantiatedImpl {
    head: TraitRef,
    predicates: Vec<TraitConstraint>,
}

fn instantiate_trait_impl(
    table: &mut UnificationTable,
    impl_: &TraitImpl,
) -> Result<InstantiatedImpl, TraitError> {
    let replacements = std::iter::repeat_with(|| table.new_meta(0))
        .take(impl_.parameters)
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
