//! Kind representation and inference utilities.

use crate::ir::Path;

use super::{
    TraitConstraint,
    Type,
    TypeScheme,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    Type,
    Arrow(Box<Kind>, Box<Kind>),
}

impl Kind {
    pub fn arrow(
        parameter: Kind,
        result: Kind,
    ) -> Self {
        Self::Arrow(parameter.into(), result.into())
    }

    pub fn from_parameter_kinds(parameter_kinds: &[Kind]) -> Self {
        parameter_kinds
            .iter()
            .rev()
            .cloned()
            .fold(Kind::Type, |result, parameter| {
                Kind::arrow(parameter, result)
            })
    }

    pub fn pretty(&self) -> String {
        match self {
            Kind::Type => "Type".to_string(),
            Kind::Arrow(parameter, result) => {
                format!("({} -> {})", parameter.pretty(), result.pretty())
            }
        }
    }
}

pub(crate) fn constructor_kind(
    parameters: usize,
    parameter_kinds: &[Kind],
) -> Kind {
    let normalized_parameter_kinds = if parameter_kinds.len() == parameters {
        parameter_kinds.to_vec()
    } else {
        vec![Kind::Type; parameters]
    };
    Kind::from_parameter_kinds(&normalized_parameter_kinds)
}

impl std::fmt::Display for Kind {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_str(&self.pretty())
    }
}

pub type KindMetaVarId = u32;

#[derive(Debug, Clone)]
pub(crate) enum InferredKind {
    Type,
    Arrow(Box<InferredKind>, Box<InferredKind>),
    MetaVar(KindMetaVarId),
}

#[derive(Debug, Clone)]
enum KindMetaVarState {
    Unbound,
    Link(InferredKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KindError {
    Occurs { var: KindMetaVarId, in_kind: Kind },
    Mismatch { left: Kind, right: Kind },
}

#[derive(Debug, Clone, Default)]
pub(crate) struct KindInferenceTable {
    meta_var_states: Vec<KindMetaVarState>,
}

#[derive(Debug, Clone)]
pub(crate) struct SchemeKindInference {
    pub parameter_kinds: Vec<Kind>,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SchemeKindError {
    Kind(KindError),
    PredicateArityMismatch {
        trait_name: Path,
        expected: usize,
        found: usize,
    },
    PredicateKindMismatch {
        trait_name: Path,
        expected: Kind,
        found: Kind,
    },
}

impl From<KindError> for SchemeKindError {
    fn from(value: KindError) -> Self {
        SchemeKindError::Kind(value)
    }
}

impl KindInferenceTable {
    pub(crate) fn new_meta(&mut self) -> InferredKind {
        let id = self.meta_var_states.len() as KindMetaVarId;
        self.meta_var_states.push(KindMetaVarState::Unbound);
        InferredKind::MetaVar(id)
    }

    pub(crate) fn from_kind(kind: &Kind) -> InferredKind {
        match kind {
            Kind::Type => InferredKind::Type,
            Kind::Arrow(parameter, result) => {
                InferredKind::Arrow(
                    Self::from_kind(parameter).into(),
                    Self::from_kind(result).into(),
                )
            }
        }
    }

    pub(crate) fn normalize(
        &mut self,
        kind: &InferredKind,
    ) -> InferredKind {
        let pruned = self.prune(kind);
        match pruned {
            InferredKind::Type | InferredKind::MetaVar(_) => pruned,
            InferredKind::Arrow(parameter, result) => {
                InferredKind::Arrow(
                    self.normalize(&parameter).into(),
                    self.normalize(&result).into(),
                )
            }
        }
    }

    pub(crate) fn resolve(
        &mut self,
        kind: &InferredKind,
    ) -> Kind {
        match self.normalize(kind) {
            InferredKind::Type => Kind::Type,
            InferredKind::Arrow(parameter, result) => {
                Kind::arrow(self.resolve(&parameter), self.resolve(&result))
            }
            InferredKind::MetaVar(_) => Kind::Type,
        }
    }

    pub(crate) fn unify(
        &mut self,
        left: &InferredKind,
        right: &InferredKind,
    ) -> Result<(), KindError> {
        let left = self.prune(left);
        let right = self.prune(right);
        match (left, right) {
            (InferredKind::Type, InferredKind::Type) => Ok(()),
            (InferredKind::MetaVar(left_id), InferredKind::MetaVar(right_id)) => {
                self.unify_meta_pair(left_id, right_id)
            }
            (InferredKind::MetaVar(id), kind) | (kind, InferredKind::MetaVar(id)) => {
                self.bind_meta(id, &kind)
            }
            (
                InferredKind::Arrow(left_parameter, left_result),
                InferredKind::Arrow(right_parameter, right_result),
            ) => {
                self.unify(&left_parameter, &right_parameter)?;
                self.unify(&left_result, &right_result)
            }
            (left, right) => {
                Err(KindError::Mismatch {
                    left: self.resolve(&left),
                    right: self.resolve(&right),
                })
            }
        }
    }

    fn prune(
        &mut self,
        kind: &InferredKind,
    ) -> InferredKind {
        match kind {
            InferredKind::MetaVar(id) => {
                match self.meta_var_states.get(*id as usize).cloned() {
                    Some(KindMetaVarState::Link(link)) => {
                        let pruned = self.prune(&link);
                        if let Some(state) = self.meta_var_states.get_mut(*id as usize) {
                            *state = KindMetaVarState::Link(pruned.clone());
                        }
                        pruned
                    }
                    _ => kind.clone(),
                }
            }
            _ => kind.clone(),
        }
    }

    fn unify_meta_pair(
        &mut self,
        left: KindMetaVarId,
        right: KindMetaVarId,
    ) -> Result<(), KindError> {
        if left == right {
            return Ok(());
        }
        let left_kind = self.prune(&InferredKind::MetaVar(left));
        let right_kind = self.prune(&InferredKind::MetaVar(right));
        match (&left_kind, &right_kind) {
            (InferredKind::MetaVar(left_id), InferredKind::MetaVar(right_id))
                if left_id == right_id =>
            {
                Ok(())
            }
            (InferredKind::MetaVar(left_id), _) => self.bind_meta(*left_id, &right_kind),
            (_, InferredKind::MetaVar(right_id)) => self.bind_meta(*right_id, &left_kind),
            _ => self.unify(&left_kind, &right_kind),
        }
    }

    fn bind_meta(
        &mut self,
        id: KindMetaVarId,
        kind: &InferredKind,
    ) -> Result<(), KindError> {
        if let InferredKind::MetaVar(other_id) = kind
            && *other_id == id
        {
            return Ok(());
        }
        if self.occurs(id, kind) {
            return Err(KindError::Occurs {
                var: id,
                in_kind: self.resolve(kind),
            });
        }
        if let Some(state) = self.meta_var_states.get_mut(id as usize) {
            *state = KindMetaVarState::Link(kind.clone());
        }
        Ok(())
    }

    fn occurs(
        &mut self,
        id: KindMetaVarId,
        kind: &InferredKind,
    ) -> bool {
        match self.prune(kind) {
            InferredKind::MetaVar(other_id) => id == other_id,
            InferredKind::Arrow(parameter, result) => {
                self.occurs(id, &parameter) || self.occurs(id, &result)
            }
            InferredKind::Type => false,
        }
    }
}

pub(crate) fn infer_scheme_kind(
    scheme: &TypeScheme,
    leading_parameters: usize,
    lookup_type_kind: &impl Fn(&Path) -> Option<Kind>,
    lookup_trait_kinds: &impl Fn(&Path) -> Option<Vec<Kind>>,
) -> Result<SchemeKindInference, SchemeKindError> {
    let mut table = KindInferenceTable::default();
    let mut bound_kinds = Vec::new();
    let mut parameter_kinds = Vec::new();

    let mut current = &scheme.type_;
    while let Type::ForAll { body, .. } = current {
        let kind = table.new_meta();
        if parameter_kinds.len() < leading_parameters {
            parameter_kinds.push(kind.clone());
        }
        bound_kinds.push(kind);
        current = body;
    }

    let inferred_kind = infer_type_kind(&mut table, current, &mut bound_kinds, lookup_type_kind)?;

    for predicate in scheme.predicates.iter() {
        check_predicate_kind(
            predicate,
            &mut table,
            &mut bound_kinds,
            lookup_type_kind,
            lookup_trait_kinds,
        )?;
    }

    Ok(SchemeKindInference {
        parameter_kinds: parameter_kinds
            .into_iter()
            .map(|kind| table.resolve(&kind))
            .collect(),
        kind: table.resolve(&inferred_kind),
    })
}

pub(crate) fn infer_type_kind(
    table: &mut KindInferenceTable,
    type_: &Type,
    bound_kinds: &mut Vec<InferredKind>,
    lookup_type_kind: &impl Fn(&Path) -> Option<Kind>,
) -> Result<InferredKind, KindError> {
    match type_ {
        Type::Unit
        | Type::Integer
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::MetaVar(_) => Ok(InferredKind::Type),
        Type::TypeVar(index) => {
            let Some(kind) = lookup_bound_kind(bound_kinds, *index).cloned() else {
                return Ok(InferredKind::Type);
            };
            Ok(kind)
        }
        Type::ForAll { body, .. } => {
            bound_kinds.push(table.new_meta());
            let kind = infer_type_kind(table, body, bound_kinds, lookup_type_kind)?;
            let _ = bound_kinds.pop();
            Ok(kind)
        }
        Type::Named { name, .. } => {
            if let Some(kind) = lookup_type_kind(name) {
                Ok(KindInferenceTable::from_kind(&kind))
            } else {
                Ok(table.new_meta())
            }
        }
        Type::StructConstraint { fields, .. } | Type::Struct { fields } => {
            check_field_kinds(fields.values(), table, bound_kinds, lookup_type_kind)?;
            Ok(InferredKind::Type)
        }
        Type::Array(inner) => {
            let kind = infer_type_kind(table, inner, bound_kinds, lookup_type_kind)?;
            table.unify(&kind, &InferredKind::Type)?;
            Ok(InferredKind::Type)
        }
        Type::Tuple(items) => {
            check_field_kinds(items.iter(), table, bound_kinds, lookup_type_kind)?;
            Ok(InferredKind::Type)
        }
        Type::Sum { variants } => {
            check_field_kinds(variants.values(), table, bound_kinds, lookup_type_kind)?;
            Ok(InferredKind::Type)
        }
        Type::Function(parameter, result) => {
            let parameter_kind = infer_type_kind(table, parameter, bound_kinds, lookup_type_kind)?;
            table.unify(&parameter_kind, &InferredKind::Type)?;
            let result_kind = infer_type_kind(table, result, bound_kinds, lookup_type_kind)?;
            table.unify(&result_kind, &InferredKind::Type)?;
            Ok(InferredKind::Type)
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            let mut constructor_kind =
                infer_type_kind(table, constructor, bound_kinds, lookup_type_kind)?;
            for argument in arguments.iter() {
                let argument_kind =
                    infer_type_kind(table, argument, bound_kinds, lookup_type_kind)?;
                let result_kind = table.new_meta();
                let expected =
                    InferredKind::Arrow(argument_kind.into(), result_kind.clone().into());
                table.unify(&constructor_kind, &expected)?;
                constructor_kind = result_kind;
            }
            Ok(constructor_kind)
        }
    }
}

fn check_predicate_kind(
    predicate: &TraitConstraint,
    table: &mut KindInferenceTable,
    bound_kinds: &mut Vec<InferredKind>,
    lookup_type_kind: &impl Fn(&Path) -> Option<Kind>,
    lookup_trait_kinds: &impl Fn(&Path) -> Option<Vec<Kind>>,
) -> Result<(), SchemeKindError> {
    let Some(expected_argument_kinds) = lookup_trait_kinds(&predicate.trait_name) else {
        return Ok(());
    };
    if expected_argument_kinds.len() != predicate.arguments.len() {
        return Err(SchemeKindError::PredicateArityMismatch {
            trait_name: predicate.trait_name.clone(),
            expected: expected_argument_kinds.len(),
            found: predicate.arguments.len(),
        });
    }
    for (argument, expected_kind) in predicate
        .arguments
        .iter()
        .zip(expected_argument_kinds.iter())
    {
        let argument_kind = infer_type_kind(table, argument, bound_kinds, lookup_type_kind)?;
        let expected_kind_inferred = KindInferenceTable::from_kind(expected_kind);
        if let Err(error) = table.unify(&argument_kind, &expected_kind_inferred) {
            let (found, expected) = match error {
                KindError::Mismatch { left, right } => (left, right),
                KindError::Occurs { in_kind, .. } => (in_kind, expected_kind.clone()),
            };
            return Err(SchemeKindError::PredicateKindMismatch {
                trait_name: predicate.trait_name.clone(),
                expected,
                found,
            });
        }
    }
    Ok(())
}

fn lookup_bound_kind(
    bound_kinds: &[InferredKind],
    index: u32,
) -> Option<&InferredKind> {
    let position = bound_kinds.len().checked_sub(index as usize + 1)?;
    bound_kinds.get(position)
}

fn check_field_kinds<'a>(
    fields: impl Iterator<Item = &'a Type>,
    table: &mut KindInferenceTable,
    bound_kinds: &mut Vec<InferredKind>,
    lookup_type_kind: &impl Fn(&Path) -> Option<Kind>,
) -> Result<(), KindError> {
    for field in fields {
        let kind = infer_type_kind(table, field, bound_kinds, lookup_type_kind)?;
        table.unify(&kind, &InferredKind::Type)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ir::Path;

    use super::*;

    #[test]
    fn infer_simple_application_kind() {
        let mut table = KindInferenceTable::default();
        let list = Type::Named {
            name: Path::new("demo", "List"),
            body: Box::new(Type::Unit),
        }
        .apply(vec![Type::Integer]);
        let kind = infer_type_kind(&mut table, &list, &mut Vec::new(), &|path| {
            (path == &Path::new("demo", "List")).then(|| Kind::arrow(Kind::Type, Kind::Type))
        })
        .expect("kind inference should succeed");
        assert_eq!(table.resolve(&kind), Kind::Type);
    }

    #[test]
    fn infer_forall_parameter_kinds() {
        let scheme = Type::v(0).apply(vec![Type::Integer]).for_all(1).scheme();
        let inferred = infer_scheme_kind(&scheme, 1, &|_| None, &|_| None)
            .expect("kind inference should succeed");
        assert_eq!(
            inferred.parameter_kinds,
            vec![Kind::arrow(Kind::Type, Kind::Type)]
        );
        assert_eq!(inferred.kind, Kind::Type);
    }

    #[test]
    fn predicate_kind_mismatch_is_reported() {
        let scheme = TypeScheme::with_predicates(
            Type::Integer,
            vec![TraitConstraint {
                trait_name: Path::new("demo", "Monad"),
                arguments: vec![Type::Integer],
            }],
        );
        assert!(matches!(
            infer_scheme_kind(&scheme, 0, &|_| None, &|path| {
                (path == &Path::new("demo", "Monad"))
                    .then(|| vec![Kind::arrow(Kind::Type, Kind::Type)])
            }),
            Err(SchemeKindError::PredicateKindMismatch { .. })
        ));
    }
}
