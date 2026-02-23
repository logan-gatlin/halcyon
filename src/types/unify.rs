use std::collections::BTreeSet;

use indexmap::IndexMap;

use super::{
    MetaVarId,
    StructMatch,
    Type,
};

#[derive(Debug, Clone)]
pub enum MetaVarState {
    Unbound { level: u32 },
    Link(Type),
}

#[derive(Debug, Clone, Default)]
pub struct UnificationTable {
    vars: Vec<MetaVarState>,
}

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
            | Type::MetaVar(_)
            | Type::RecVar(_) => pruned,
            Type::ForAll(body) => Type::ForAll(Box::new(self.normalize(&body))),
            Type::Mu(body) => Type::Mu(Box::new(self.normalize(&body))),
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
            Type::Sum {
                variant_names,
                variant_types,
            } => {
                Type::Sum {
                    variant_names,
                    variant_types: variant_types
                        .into_iter()
                        .map(|variant| self.normalize(&variant))
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
                let left = normalize_empty_apply(self.prune(left));
                let right = normalize_empty_apply(self.prune(right));
                self.unify_non_meta(left, right)
            }
        }
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
            (Type::RecVar(left), Type::RecVar(right)) if left == right => Ok(()),
            (Type::ForAll(left), Type::ForAll(right)) => self.unify(&left, &right),
            (Type::Mu(left), Type::Mu(right)) => self.unify(&left, &right),
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
            (
                Type::Sum {
                    variant_names: left_names,
                    variant_types: left_types,
                },
                Type::Sum {
                    variant_names: right_names,
                    variant_types: right_types,
                },
            ) => {
                if left_names != right_names || left_types.len() != right_types.len() {
                    return Err(UnifyError::Mismatch {
                        left: Type::Sum {
                            variant_names: left_names,
                            variant_types: left_types,
                        },
                        right: Type::Sum {
                            variant_names: right_names,
                            variant_types: right_types,
                        },
                    });
                }
                left_types
                    .iter()
                    .zip(right_types.iter())
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
        match (
            self.vars.get(left as usize).cloned(),
            self.vars.get(right as usize).cloned(),
        ) {
            (Some(MetaVarState::Unbound { .. }), _) => self.bind_meta(left, &Type::MetaVar(right)),
            (_, Some(MetaVarState::Unbound { .. })) => self.bind_meta(right, &Type::MetaVar(left)),
            (Some(MetaVarState::Link(left_link)), _) => {
                self.unify_meta_with_type(right, &left_link)
            }
            _ => Ok(()),
        }
    }

    fn unify_meta_with_type(
        &mut self,
        id: MetaVarId,
        other: &Type,
    ) -> Result<(), UnifyError> {
        let other_pruned = normalize_empty_apply(self.prune(other));
        match self.vars.get(id as usize).cloned() {
            Some(MetaVarState::Unbound { .. }) => self.bind_meta(id, &other_pruned),
            Some(MetaVarState::Link(link)) => {
                let link_pruned = normalize_empty_apply(self.prune(&link));
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
        if let Some(named_fields) = self.resolve_named_struct_fields(other) {
            match mode {
                StructMatch::Exact => {
                    if named_fields.len() != fields.len()
                        || fields.keys().any(|key| !named_fields.contains_key(key))
                    {
                        return Err(UnifyError::Mismatch {
                            left: Type::StructConstraint { fields, mode },
                            right: other.clone(),
                        });
                    }
                }
                StructMatch::AtLeast => {
                    if fields.keys().any(|key| !named_fields.contains_key(key)) {
                        return Err(UnifyError::Mismatch {
                            left: Type::StructConstraint { fields, mode },
                            right: other.clone(),
                        });
                    }
                }
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

    fn resolve_named_struct_fields(
        &mut self,
        type_: &Type,
    ) -> Option<IndexMap<String, Type>> {
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
            Type::MetaVar(other_id) => {
                if id == other_id {
                    true
                } else {
                    match self.vars.get(other_id as usize).cloned() {
                        Some(MetaVarState::Link(link)) => self.occurs(id, &link),
                        _ => false,
                    }
                }
            }
            Type::ForAll(body) | Type::Mu(body) | Type::Array(body) => self.occurs(id, &body),
            Type::Function(parameter, result) => {
                self.occurs(id, &parameter) || self.occurs(id, &result)
            }
            Type::Tuple(items) => items.iter().any(|item| self.occurs(id, item)),
            Type::StructConstraint { fields, .. } => {
                fields.values().any(|field| self.occurs(id, field))
            }
            Type::Struct { fields } => fields.values().any(|field| self.occurs(id, field)),
            Type::Sum { variant_types, .. } => {
                variant_types.iter().any(|variant| self.occurs(id, variant))
            }
            Type::Apply {
                constructor,
                arguments,
            } => self.occurs(id, &constructor) || arguments.iter().any(|arg| self.occurs(id, arg)),
            Type::Named { .. }
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph
            | Type::TypeVar(_)
            | Type::RecVar(_) => false,
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
            Type::ForAll(body) | Type::Mu(body) | Type::Array(body) => {
                self.adjust_levels(level, &body)
            }
            Type::Function(parameter, result) => {
                self.adjust_levels(level, &parameter)?;
                self.adjust_levels(level, &result)
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .try_for_each(|item| self.adjust_levels(level, item))
            }
            Type::StructConstraint { fields, .. } => {
                fields
                    .values()
                    .try_for_each(|field| self.adjust_levels(level, field))
            }
            Type::Struct { fields } => {
                fields
                    .values()
                    .try_for_each(|field| self.adjust_levels(level, field))
            }
            Type::Sum { variant_types, .. } => {
                variant_types
                    .iter()
                    .try_for_each(|variant| self.adjust_levels(level, variant))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                self.adjust_levels(level, &constructor)?;
                arguments
                    .iter()
                    .try_for_each(|arg| self.adjust_levels(level, arg))
            }
            Type::Named { .. }
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph
            | Type::TypeVar(_)
            | Type::RecVar(_) => Ok(()),
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
                } else if let Some(MetaVarState::Link(link)) = self.vars.get(id as usize).cloned() {
                    self.collect_meta_vars(&link, vars);
                }
            }
            Type::ForAll(body) | Type::Mu(body) | Type::Array(body) => {
                self.collect_meta_vars(&body, vars)
            }
            Type::Function(parameter, result) => {
                self.collect_meta_vars(&parameter, vars);
                self.collect_meta_vars(&result, vars);
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .for_each(|item| self.collect_meta_vars(item, vars));
            }
            Type::StructConstraint { fields, .. } => {
                fields
                    .values()
                    .for_each(|field| self.collect_meta_vars(field, vars));
            }
            Type::Struct { fields } => {
                fields
                    .values()
                    .for_each(|field| self.collect_meta_vars(field, vars));
            }
            Type::Sum { variant_types, .. } => {
                variant_types
                    .iter()
                    .for_each(|variant| self.collect_meta_vars(variant, vars));
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                self.collect_meta_vars(&constructor, vars);
                arguments
                    .iter()
                    .for_each(|arg| self.collect_meta_vars(arg, vars));
            }
            Type::Named { .. }
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph
            | Type::TypeVar(_)
            | Type::RecVar(_) => {}
        }
    }
}

fn normalize_empty_apply(type_: Type) -> Type {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } if arguments.is_empty() => normalize_empty_apply(*constructor),
        other => other,
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
    let mut current = body.clone();
    for arg in arguments {
        if let Type::ForAll(inner) = current {
            current = open_forall(&inner, arg)?;
        } else {
            return None;
        }
    }
    Some(current)
}

fn open_forall(
    body: &Type,
    replacement: &Type,
) -> Option<Type> {
    body.substitute_type_var(0, replacement)?
        .shift_type_vars(-1, 0)
}
