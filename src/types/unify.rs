use std::collections::BTreeSet;

use indexmap::IndexMap;

use super::instantiation::instantiate_forall_strict;
use super::{
    MetaVarId,
    StructMatch,
    TraitConstraint,
    TraitRef,
    Type,
    for_each_child_type,
    normalize_empty_apply,
};

/// State of a unification meta variable.
#[derive(Debug, Clone)]
pub enum MetaVarState {
    Unbound { level: u32 },
    Link(Type),
}

/// Unification table for inference meta variables.
#[derive(Debug, Clone, Default)]
pub struct UnificationTable {
    vars: Vec<MetaVarState>,
}

/// Errors that can arise during unification.
#[derive(Debug, Clone)]
pub enum UnifyError {
    Occurs { var: MetaVarId, in_type: Type },
    Mismatch { left: Type, right: Type },
}

impl UnificationTable {
    pub fn new_meta(
        &mut self,
        level: u32,
    ) -> Type {
        let id = self.vars.len() as MetaVarId;
        self.vars.push(MetaVarState::Unbound { level });
        Type::MetaVar(id)
    }

    pub fn level(
        &self,
        id: MetaVarId,
    ) -> Option<u32> {
        match self.vars.get(id as usize)? {
            MetaVarState::Unbound { level } => Some(*level),
            MetaVarState::Link(_) => None,
        }
    }

    pub fn prune(
        &mut self,
        type_: &Type,
    ) -> Type {
        match type_ {
            Type::MetaVar(id) => {
                match self.vars.get(*id as usize).cloned() {
                    Some(MetaVarState::Link(link)) => {
                        let pruned = self.prune(&link);
                        self.vars[*id as usize] = MetaVarState::Link(pruned.clone());
                        pruned
                    }
                    _ => type_.clone(),
                }
            }
            _ => type_.clone(),
        }
    }

    pub fn normalize(
        &mut self,
        type_: &Type,
    ) -> Type {
        let pruned = self.prune(type_);
        match pruned {
            Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph
            | Type::TypeVar(_)
            | Type::MetaVar(_) => pruned,
            Type::ForAll(body) => Type::ForAll(Box::new(self.normalize(&body))),
            Type::Named { name, body } => Type::Named { name, body },
            Type::StructConstraint { fields, mode } => {
                Type::StructConstraint {
                    fields: fields
                        .into_iter()
                        .map(|(name, type_)| (name, self.normalize(&type_)))
                        .collect(),
                    mode,
                }
            }
            Type::Struct { fields } => {
                Type::Struct {
                    fields: fields
                        .into_iter()
                        .map(|(name, type_)| (name, self.normalize(&type_)))
                        .collect(),
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.normalize(&inner))),
            Type::Tuple(items) => {
                Type::Tuple(
                    items
                        .into_iter()
                        .map(|item| self.normalize(&item))
                        .collect(),
                )
            }
            Type::Sum { variants } => {
                Type::Sum {
                    variants: variants
                        .into_iter()
                        .map(|(name, type_)| (name, self.normalize(&type_)))
                        .collect(),
                }
            }
            Type::Function(parameter, result) => {
                Type::func(self.normalize(&parameter), self.normalize(&result))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                Type::Apply {
                    constructor: Box::new(self.normalize(&constructor)),
                    arguments: arguments
                        .into_iter()
                        .map(|arg| self.normalize(&arg))
                        .collect(),
                }
            }
        }
    }

    pub fn normalize_trait_ref(
        &mut self,
        trait_ref: &TraitRef,
    ) -> TraitRef {
        TraitRef {
            trait_name: trait_ref.trait_name.clone(),
            arguments: trait_ref
                .arguments
                .iter()
                .map(|arg| self.normalize(arg))
                .collect(),
        }
    }

    pub fn normalize_predicates(
        &mut self,
        predicates: &[TraitConstraint],
    ) -> Vec<TraitConstraint> {
        predicates
            .iter()
            .map(|predicate| self.normalize_trait_ref(predicate))
            .collect()
    }

    pub fn free_meta_vars(
        &mut self,
        type_: &Type,
    ) -> BTreeSet<MetaVarId> {
        let mut vars = BTreeSet::new();
        self.collect_meta_vars(type_, &mut vars);
        vars
    }

    pub fn unify(
        &mut self,
        left: &Type,
        right: &Type,
    ) -> Result<(), UnifyError> {
        match (left, right) {
            (Type::MetaVar(id), Type::MetaVar(other_id)) => self.unify_meta_pair(*id, *other_id),
            (Type::MetaVar(id), other) => self.unify_meta_with_type(*id, other),
            (other, Type::MetaVar(id)) => self.unify_meta_with_type(*id, other),
            _ => {
                let left = self.normalize_for_unify(left);
                let right = self.normalize_for_unify(right);
                self.unify_non_meta(left, right)
            }
        }
    }

    fn normalize_for_unify(
        &mut self,
        type_: &Type,
    ) -> Type {
        normalize_empty_apply(self.prune(type_))
    }

    fn unify_non_meta(
        &mut self,
        left: Type,
        right: Type,
    ) -> Result<(), UnifyError> {
        match (left, right) {
            (Type::Unit, Type::Unit)
            | (Type::Integer, Type::Integer)
            | (Type::Real, Type::Real)
            | (Type::Boolean, Type::Boolean)
            | (Type::String, Type::String)
            | (Type::Glyph, Type::Glyph) => Ok(()),
            (Type::TypeVar(left), Type::TypeVar(right)) if left == right => Ok(()),
            (Type::ForAll(left), Type::ForAll(right)) => self.unify(&left, &right),
            // Named types are nominal on the main unification path.
            (Type::Named { name: left, .. }, Type::Named { name: right, .. }) if left == right => {
                Ok(())
            }
            (
                Type::StructConstraint {
                    fields: left_fields,
                    mode: left_mode,
                },
                Type::StructConstraint {
                    fields: right_fields,
                    mode: right_mode,
                },
            ) => {
                let _ = self.merge_struct_constraints(
                    left_fields,
                    left_mode,
                    right_fields,
                    right_mode,
                )?;
                Ok(())
            }
            (Type::StructConstraint { fields, mode }, other) => {
                self.unify_struct_constraint_with_type(fields, mode, &other)
            }
            (other, Type::StructConstraint { fields, mode }) => {
                self.unify_struct_constraint_with_type(fields, mode, &other)
            }
            (Type::Struct { fields: left }, Type::Struct { fields: right }) => {
                if left.len() != right.len() || !left.keys().eq(right.keys()) {
                    return Err(UnifyError::Mismatch {
                        left: Type::Struct { fields: left },
                        right: Type::Struct { fields: right },
                    });
                }
                left.values()
                    .zip(right.values())
                    .try_for_each(|(left, right)| self.unify(left, right))
            }
            (Type::Array(left), Type::Array(right)) => self.unify(&left, &right),
            (Type::Tuple(left), Type::Tuple(right)) => {
                if left.len() != right.len() {
                    return Err(UnifyError::Mismatch {
                        left: Type::Tuple(left),
                        right: Type::Tuple(right),
                    });
                }
                left.iter()
                    .zip(right.iter())
                    .try_for_each(|(left, right)| self.unify(left, right))
            }
            (Type::Sum { variants: left }, Type::Sum { variants: right }) => {
                if left.len() != right.len() || !left.keys().eq(right.keys()) {
                    return Err(UnifyError::Mismatch {
                        left: Type::Sum { variants: left },
                        right: Type::Sum { variants: right },
                    });
                }
                left.values()
                    .zip(right.values())
                    .try_for_each(|(left, right)| self.unify(left, right))
            }
            (
                Type::Function(left_param, left_result),
                Type::Function(right_param, right_result),
            ) => {
                self.unify(&left_param, &right_param)?;
                self.unify(&left_result, &right_result)
            }
            (
                Type::Apply {
                    constructor: left_constructor,
                    arguments: left_arguments,
                },
                Type::Apply {
                    constructor: right_constructor,
                    arguments: right_arguments,
                },
            ) => {
                if left_arguments.len() != right_arguments.len() {
                    return Err(UnifyError::Mismatch {
                        left: Type::Apply {
                            constructor: left_constructor,
                            arguments: left_arguments,
                        },
                        right: Type::Apply {
                            constructor: right_constructor,
                            arguments: right_arguments,
                        },
                    });
                }
                self.unify(&left_constructor, &right_constructor)?;
                left_arguments
                    .iter()
                    .zip(right_arguments.iter())
                    .try_for_each(|(left, right)| self.unify(left, right))
            }
            (left, right) => Err(UnifyError::Mismatch { left, right }),
        }
    }

    fn unify_meta_pair(
        &mut self,
        left: MetaVarId,
        right: MetaVarId,
    ) -> Result<(), UnifyError> {
        if left == right {
            return Ok(());
        }
        // Follow link chains to canonical representatives.
        let left_canonical = self.prune(&Type::MetaVar(left));
        let right_canonical = self.prune(&Type::MetaVar(right));
        match (&left_canonical, &right_canonical) {
            // Both pruned to the same meta — already unified.
            (Type::MetaVar(l), Type::MetaVar(r)) if l == r => Ok(()),
            // Both are (distinct) unbound metas — bind one to the other.
            (Type::MetaVar(l), Type::MetaVar(_)) => self.bind_meta(*l, &right_canonical),
            // Left resolved to a non-meta — delegate to meta-with-type for right.
            (_, Type::MetaVar(r)) => self.unify_meta_with_type(*r, &left_canonical),
            // Right resolved to a non-meta — delegate to meta-with-type for left.
            (Type::MetaVar(l), _) => self.unify_meta_with_type(*l, &right_canonical),
            // Both resolved to non-meta types.
            _ => {
                let left = self.normalize_for_unify(&left_canonical);
                let right = self.normalize_for_unify(&right_canonical);
                self.unify_non_meta(left, right)
            }
        }
    }

    fn unify_meta_with_type(
        &mut self,
        id: MetaVarId,
        other: &Type,
    ) -> Result<(), UnifyError> {
        let other_pruned = self.normalize_for_unify(other);
        match self.vars.get(id as usize).cloned() {
            Some(MetaVarState::Unbound { .. }) => self.bind_meta(id, &other_pruned),
            Some(MetaVarState::Link(link)) => {
                let link_pruned = self.normalize_for_unify(&link);
                match (link_pruned, other_pruned) {
                    (
                        Type::StructConstraint {
                            fields: left_fields,
                            mode: left_mode,
                        },
                        Type::StructConstraint {
                            fields: right_fields,
                            mode: right_mode,
                        },
                    ) => {
                        let merged = self.merge_struct_constraints(
                            left_fields,
                            left_mode,
                            right_fields,
                            right_mode,
                        )?;
                        if let Some(state) = self.vars.get_mut(id as usize) {
                            *state = MetaVarState::Link(merged);
                        }
                        Ok(())
                    }
                    (Type::StructConstraint { fields, mode }, other) => {
                        self.unify_struct_constraint_with_type(fields, mode, &other)?;
                        if let Some(state) = self.vars.get_mut(id as usize) {
                            *state = MetaVarState::Link(other);
                        }
                        Ok(())
                    }
                    (left, right) => self.unify_non_meta(left, right),
                }
            }
            None => self.bind_meta(id, &other_pruned),
        }
    }

    fn bind_meta(
        &mut self,
        id: MetaVarId,
        type_: &Type,
    ) -> Result<(), UnifyError> {
        if let Type::MetaVar(other_id) = type_
            && *other_id == id
        {
            return Ok(());
        }
        if self.occurs(id, type_) {
            return Err(UnifyError::Occurs {
                var: id,
                in_type: type_.clone(),
            });
        }
        if let Some(level) = self.level(id) {
            self.adjust_levels(level, type_)?;
        }
        if let Some(state) = self.vars.get_mut(id as usize) {
            *state = MetaVarState::Link(type_.clone());
        }
        Ok(())
    }

    fn merge_struct_constraints(
        &mut self,
        left_fields: IndexMap<String, Type>,
        left_mode: StructMatch,
        right_fields: IndexMap<String, Type>,
        right_mode: StructMatch,
    ) -> Result<Type, UnifyError> {
        let is_subset = |left: &IndexMap<String, Type>, right: &IndexMap<String, Type>| {
            right.keys().all(|key| left.contains_key(key))
        };
        let has_same_fields = |left: &IndexMap<String, Type>, right: &IndexMap<String, Type>| {
            left.len() == right.len() && left.keys().all(|key| right.contains_key(key))
        };

        let unify_overlap =
            |this: &mut Self, left: &IndexMap<String, Type>, right: &IndexMap<String, Type>| {
                for (name, left_type) in left.iter() {
                    if let Some(right_type) = right.get(name) {
                        this.unify(left_type, right_type)?;
                    }
                }
                Ok(())
            };

        match (left_mode, right_mode) {
            (StructMatch::Exact, StructMatch::Exact) => {
                if !has_same_fields(&left_fields, &right_fields) {
                    return Err(UnifyError::Mismatch {
                        left: Type::StructConstraint {
                            fields: left_fields,
                            mode: left_mode,
                        },
                        right: Type::StructConstraint {
                            fields: right_fields,
                            mode: right_mode,
                        },
                    });
                }
                unify_overlap(self, &left_fields, &right_fields)?;
                Ok(Type::StructConstraint {
                    fields: left_fields,
                    mode: StructMatch::Exact,
                })
            }
            (StructMatch::Exact, StructMatch::AtLeast) => {
                if !is_subset(&left_fields, &right_fields) {
                    return Err(UnifyError::Mismatch {
                        left: Type::StructConstraint {
                            fields: left_fields,
                            mode: left_mode,
                        },
                        right: Type::StructConstraint {
                            fields: right_fields,
                            mode: right_mode,
                        },
                    });
                }
                unify_overlap(self, &left_fields, &right_fields)?;
                Ok(Type::StructConstraint {
                    fields: left_fields,
                    mode: StructMatch::Exact,
                })
            }
            (StructMatch::AtLeast, StructMatch::Exact) => {
                if !is_subset(&right_fields, &left_fields) {
                    return Err(UnifyError::Mismatch {
                        left: Type::StructConstraint {
                            fields: left_fields,
                            mode: left_mode,
                        },
                        right: Type::StructConstraint {
                            fields: right_fields,
                            mode: right_mode,
                        },
                    });
                }
                unify_overlap(self, &left_fields, &right_fields)?;
                Ok(Type::StructConstraint {
                    fields: right_fields,
                    mode: StructMatch::Exact,
                })
            }
            (StructMatch::AtLeast, StructMatch::AtLeast) => {
                unify_overlap(self, &left_fields, &right_fields)?;
                let mut merged = left_fields;
                for (name, type_) in right_fields {
                    merged.entry(name).or_insert(type_);
                }
                Ok(Type::StructConstraint {
                    fields: merged,
                    mode: StructMatch::AtLeast,
                })
            }
        }
    }

    fn unify_struct_constraint_with_type(
        &mut self,
        fields: IndexMap<String, Type>,
        mode: StructMatch,
        other: &Type,
    ) -> Result<(), UnifyError> {
        // The only structural escape hatch for named types is field constraints.
        // Named-vs-named and named-vs-struct unification remains nominal elsewhere.
        if let Some(named_fields) = self.resolve_named_struct_fields_for_constraint(other) {
            if !struct_constraint_matches_fields(&fields, mode, &named_fields) {
                return Err(UnifyError::Mismatch {
                    left: Type::StructConstraint { fields, mode },
                    right: other.clone(),
                });
            }
            for (name, field_type) in fields.iter() {
                if let Some(named_type) = named_fields.get(name) {
                    self.unify(field_type, named_type)?;
                }
            }
            return Ok(());
        }
        Err(UnifyError::Mismatch {
            left: Type::StructConstraint { fields, mode },
            right: other.clone(),
        })
    }

    fn resolve_named_struct_fields_for_constraint(
        &mut self,
        type_: &Type,
    ) -> Option<IndexMap<String, Type>> {
        // Only named types expose structural fields here, and only for
        // struct-constraint matching.
        let (base, arguments) = split_apply(type_);
        let Type::Named { body, .. } = base else {
            return None;
        };
        let instantiated = instantiate_named_body(&body, &arguments)?;
        match instantiated {
            Type::Struct { fields } => Some(fields),
            _ => None,
        }
    }

    fn occurs(
        &mut self,
        id: MetaVarId,
        type_: &Type,
    ) -> bool {
        let pruned = self.prune(type_);
        match pruned {
            Type::MetaVar(other_id) => id == other_id,
            _ => {
                let mut found = false;
                for_each_child_type(&pruned, false, |child| {
                    if !found && self.occurs(id, child) {
                        found = true;
                    }
                });
                found
            }
        }
    }

    fn adjust_levels(
        &mut self,
        level: u32,
        type_: &Type,
    ) -> Result<(), UnifyError> {
        let pruned = self.prune(type_);
        match pruned {
            Type::MetaVar(id) => {
                match self.vars.get(id as usize).cloned() {
                    Some(MetaVarState::Unbound { level: var_level }) => {
                        if var_level > level
                            && let Some(state) = self.vars.get_mut(id as usize)
                        {
                            *state = MetaVarState::Unbound { level };
                        }
                        Ok(())
                    }
                    Some(MetaVarState::Link(link)) => self.adjust_levels(level, &link),
                    None => Ok(()),
                }
            }
            _ => {
                let mut result = Ok(());
                for_each_child_type(&pruned, false, |child| {
                    if result.is_ok() {
                        result = self.adjust_levels(level, child);
                    }
                });
                result
            }
        }
    }

    fn collect_meta_vars(
        &mut self,
        type_: &Type,
        vars: &mut BTreeSet<MetaVarId>,
    ) {
        let pruned = self.prune(type_);
        match pruned {
            Type::MetaVar(id) => {
                if matches!(
                    self.vars.get(id as usize),
                    Some(MetaVarState::Unbound { .. })
                ) {
                    vars.insert(id);
                }
            }
            _ => {
                for_each_child_type(&pruned, false, |child| self.collect_meta_vars(child, vars));
            }
        }
    }
}

fn struct_constraint_matches_fields(
    constraint_fields: &IndexMap<String, Type>,
    mode: StructMatch,
    target_fields: &IndexMap<String, Type>,
) -> bool {
    match mode {
        StructMatch::Exact => {
            target_fields.len() == constraint_fields.len()
                && constraint_fields
                    .keys()
                    .all(|key| target_fields.contains_key(key))
        }
        StructMatch::AtLeast => {
            constraint_fields
                .keys()
                .all(|key| target_fields.contains_key(key))
        }
    }
}

fn split_apply(type_: &Type) -> (Type, Vec<Type>) {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } => {
            let (base, mut args) = split_apply(constructor);
            args.extend(arguments.iter().cloned());
            (base, args)
        }
        other => (other.clone(), Vec::new()),
    }
}

fn instantiate_named_body(
    body: &Type,
    arguments: &[Type],
) -> Option<Type> {
    instantiate_forall_strict(body, arguments)
}
