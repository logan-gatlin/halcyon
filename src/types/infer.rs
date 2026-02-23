use std::collections::HashMap;

use indexmap::IndexMap;

use crate::ir::{
    Glob,
    Path,
    Pattern,
    PatternKind,
    ScopeKind,
    Term,
    TermKind,
    TypeExpr,
    TypeExprKind,
};

use super::{
    MetaVarId,
    StructMatch,
    TraitConstraint,
    TraitRef,
    Type,
    TypeDefinition,
    TypeScheme,
};

use super::unify::{
    UnificationTable,
    UnifyError,
};

#[derive(Debug, Clone)]
pub enum TypeError {
    UnknownIdentifier(Path),
    UnknownConstructor(Path),
    InvalidTypeApplication {
        name: Path,
        expected: usize,
        found: usize,
    },
    MissingField {
        field: String,
        in_type: Type,
    },
    NotAFunction(Type),
    InvalidScheme,
    Unification(UnifyError),
}

impl From<UnifyError> for TypeError {
    fn from(value: UnifyError) -> Self {
        Self::Unification(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    bindings: IndexMap<Path, TypeScheme>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(
        &self,
        path: &Path,
    ) -> Option<&TypeScheme> {
        self.bindings.get(path)
    }

    pub fn bindings(&self) -> &IndexMap<Path, TypeScheme> {
        &self.bindings
    }

    pub fn into_bindings(self) -> IndexMap<Path, TypeScheme> {
        self.bindings
    }

    pub fn with_binding(
        &self,
        path: Path,
        scheme: impl Into<TypeScheme>,
    ) -> Self {
        let mut next = self.clone();
        next.bindings.insert(path, scheme.into());
        next
    }

    pub fn with_bindings<T>(
        &self,
        bindings: impl IntoIterator<Item = (Path, T)>,
    ) -> Self
    where
        T: Into<TypeScheme>,
    {
        let mut next = self.clone();
        next.bindings.extend(
            bindings
                .into_iter()
                .map(|(path, scheme)| (path, scheme.into())),
        );
        next
    }

    pub fn insert(
        &mut self,
        path: Path,
        scheme: impl Into<TypeScheme>,
    ) {
        self.bindings.insert(path, scheme.into());
    }

    pub fn extend<T>(
        &mut self,
        bindings: impl IntoIterator<Item = (Path, T)>,
    ) where
        T: Into<TypeScheme>,
    {
        self.bindings.extend(
            bindings
                .into_iter()
                .map(|(path, scheme)| (path, scheme.into())),
        );
    }
}

#[derive(Debug, Default)]
pub struct InferenceContext {
    table: UnificationTable,
    level: u32,
    type_definitions: HashMap<Path, TypeDefinition>,
}

#[derive(Debug, Clone)]
pub struct SchemeInstance {
    pub type_: Type,
    pub predicates: Vec<TraitConstraint>,
}

#[derive(Debug, Clone)]
pub struct InferenceOutput {
    pub term: Term<Type>,
    pub predicates: Vec<TraitConstraint>,
}

impl InferenceContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_type_definitions(
        &mut self,
        definitions: HashMap<Path, TypeDefinition>,
    ) {
        self.type_definitions = definitions;
    }

    pub fn table(&self) -> &UnificationTable {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut UnificationTable {
        &mut self.table
    }

    pub fn fresh_meta(&mut self) -> Type {
        self.table.new_meta(self.level)
    }

    pub fn instantiate(
        &mut self,
        scheme: &TypeScheme,
    ) -> Result<Type, TypeError> {
        Ok(self.instantiate_scheme(scheme)?.type_)
    }

    pub fn instantiate_scheme(
        &mut self,
        scheme: &TypeScheme,
    ) -> Result<SchemeInstance, TypeError> {
        let mut current = scheme.type_.clone();
        let mut predicates = scheme.predicates.clone();
        loop {
            match current {
                Type::ForAll(body) => {
                    let fresh = self.fresh_meta();
                    current = open_forall(&body, &fresh).ok_or(TypeError::InvalidScheme)?;
                    predicates = open_forall_predicates(&predicates, &fresh)
                        .ok_or(TypeError::InvalidScheme)?;
                }
                other => {
                    return Ok(SchemeInstance {
                        type_: other,
                        predicates,
                    });
                }
            }
        }
    }

    pub fn generalize_at(
        &mut self,
        type_: &Type,
        level: u32,
    ) -> TypeScheme {
        self.generalize_with_predicates(type_, level, Vec::new())
    }

    pub fn generalize_with_predicates(
        &mut self,
        type_: &Type,
        level: u32,
        predicates: Vec<TraitConstraint>,
    ) -> TypeScheme {
        let normalized_type = self.table.normalize(type_);
        let normalized_predicates = normalize_predicates(&mut self.table, &predicates);
        let mut metas = self.table.free_meta_vars(&normalized_type);
        for predicate in normalized_predicates.iter() {
            for argument in predicate.arguments.iter() {
                metas.extend(self.table.free_meta_vars(argument));
            }
        }
        let mut metas = metas
            .into_iter()
            .filter(|id| {
                self.table
                    .level(*id)
                    .is_some_and(|var_level| var_level > level)
            })
            .collect::<Vec<_>>();
        metas.sort_unstable();
        let replacements = metas
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index as u32))
            .collect::<HashMap<_, _>>();
        let type_ = replace_meta_vars(&normalized_type, &replacements).for_all(metas.len());
        let predicates = replace_meta_vars_in_predicates(&normalized_predicates, &replacements);
        type_.scheme_with_predicates(predicates)
    }

    pub fn infer_term(
        &mut self,
        env: &mut TypeEnv,
        term: &Term<()>,
    ) -> Result<InferenceOutput, TypeError> {
        infer_term(self, env, term)
    }
}

pub fn infer_term(
    ctx: &mut InferenceContext,
    env: &mut TypeEnv,
    term: &Term<()>,
) -> Result<InferenceOutput, TypeError> {
    let (kind, type_, predicates) = match &term.kind {
        TermKind::Immediate(value) => {
            (
                TermKind::Immediate(value.clone()),
                value.type_of(),
                Vec::new(),
            )
        }
        TermKind::Identifier(path) => {
            let scheme = env
                .get(path)
                .ok_or_else(|| TypeError::UnknownIdentifier(path.clone()))?;
            let instance = ctx.instantiate_scheme(scheme)?;
            (
                TermKind::Identifier(path.clone()),
                instance.type_,
                instance.predicates,
            )
        }
        TermKind::Tuple(items) => {
            let mut typed_items = Vec::with_capacity(items.len());
            let mut types = Vec::with_capacity(items.len());
            let mut predicates = Vec::new();
            for item in items {
                let typed = infer_term(ctx, env, item)?;
                types.push(typed.term.type_.clone());
                predicates.extend(typed.predicates);
                typed_items.push(typed.term);
            }
            (TermKind::Tuple(typed_items), Type::Tuple(types), predicates)
        }
        TermKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            let mut field_types = IndexMap::new();
            let mut predicates = Vec::new();
            for (name, value) in fields {
                let typed = infer_term(ctx, env, value)?;
                field_types.insert(name.inner.clone(), typed.term.type_.clone());
                predicates.extend(typed.predicates);
                typed_fields.insert(name.clone(), typed.term);
            }
            (
                TermKind::Struct(typed_fields),
                Type::StructConstraint {
                    fields: field_types,
                    mode: StructMatch::Exact,
                },
                predicates,
            )
        }
        TermKind::Field { of, index } => {
            let typed_of = infer_term(ctx, env, of)?;
            let field_name = index.inner.clone();
            let field_type = field_access_type(ctx, &typed_of.term.type_, &field_name)?;
            (
                TermKind::Field {
                    of: typed_of.term.into(),
                    index: index.clone(),
                },
                field_type,
                typed_of.predicates,
            )
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            let param_type = match parameter_type {
                Some(type_expr) => type_expr_to_type(ctx, type_expr)?,
                None => ctx.fresh_meta(),
            };
            let mut env_with_param =
                env.with_binding(parameter_name.inner.clone(), param_type.clone());
            let typed_body = infer_term(ctx, &mut env_with_param, body)?;
            let typed_captures = captures
                .iter()
                .map(|(path, _)| {
                    let scheme = env
                        .get(path)
                        .ok_or_else(|| TypeError::UnknownIdentifier(path.clone()))?;
                    Ok((path.clone(), scheme.type_.clone()))
                })
                .collect::<Result<Vec<_>, TypeError>>()?;
            let type_ = Type::func(param_type, typed_body.term.type_.clone());
            (
                TermKind::Function {
                    parameter_name: parameter_name.clone(),
                    parameter_type: parameter_type.clone(),
                    captures: typed_captures.into_boxed_slice(),
                    body: typed_body.term.into(),
                },
                type_,
                typed_body.predicates,
            )
        }
        TermKind::Call { callee, argument } => {
            let typed_callee = infer_term(ctx, env, callee)?;
            let typed_argument = infer_term(ctx, env, argument)?;
            let result_type = ctx.fresh_meta();
            let function_type = Type::func(typed_argument.term.type_.clone(), result_type.clone());
            ctx.table.unify(&typed_callee.term.type_, &function_type)?;
            let mut predicates = typed_callee.predicates;
            predicates.extend(typed_argument.predicates);
            (
                TermKind::Call {
                    callee: typed_callee.term.into(),
                    argument: typed_argument.term.into(),
                },
                result_type,
                predicates,
            )
        }
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            let outer_level = ctx.level;
            ctx.level += 1;
            let typed_value = infer_term(ctx, env, value)?;
            let mut bindings = Vec::new();
            let typed_pattern =
                infer_pattern(ctx, env, assignee, &typed_value.term.type_, &mut bindings)?;
            ctx.level = outer_level;

            let generalized = bindings
                .into_iter()
                .map(|(path, type_)| {
                    (
                        path,
                        ctx.generalize_with_predicates(
                            &type_,
                            outer_level,
                            typed_value.predicates.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>();
            let mut env_with = env.with_bindings(generalized.clone());
            let typed_then = infer_term(ctx, &mut env_with, then)?;
            let typed_else = infer_term(ctx, env, else_)?;
            ctx.table
                .unify(&typed_then.term.type_, &typed_else.term.type_)?;
            let result_type = ctx.table.normalize(&typed_then.term.type_);
            let mut predicates = typed_then.predicates;
            predicates.extend(typed_else.predicates);
            if *scope == ScopeKind::Global {
                env.extend(generalized);
            }
            (
                TermKind::Let {
                    assignee: typed_pattern,
                    scope: *scope,
                    value: typed_value.term.into(),
                    then: typed_then.term.into(),
                    else_: typed_else.term.into(),
                },
                result_type,
                predicates,
            )
        }
        TermKind::Semicolon(left, right) => {
            let typed_left = infer_term(ctx, env, left)?;
            let typed_right = infer_term(ctx, env, right)?;
            ctx.table.unify(&typed_left.term.type_, &Type::Unit)?;
            let result_type = typed_right.term.type_.clone();
            let mut predicates = typed_left.predicates;
            predicates.extend(typed_right.predicates);
            (
                TermKind::Semicolon(typed_left.term.into(), typed_right.term.into()),
                result_type,
                predicates,
            )
        }
        TermKind::Unreachable => (TermKind::Unreachable, ctx.fresh_meta(), Vec::new()),
    };

    let normalized = ctx.table.normalize(&type_);
    let predicates = normalize_predicates(&mut ctx.table, &predicates);
    Ok(InferenceOutput {
        term: Term {
            comments: term.comments.clone(),
            kind,
            span: term.span,
            type_: normalized,
        },
        predicates,
    })
}

fn infer_pattern(
    ctx: &mut InferenceContext,
    env: &TypeEnv,
    pattern: &Pattern<()>,
    expected: &Type,
    bindings: &mut Vec<(Path, Type)>,
) -> Result<Pattern<Type>, TypeError> {
    match &pattern.kind {
        PatternKind::Hole => {
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Hole,
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Identifier(path) => {
            bindings.push((path.clone(), ctx.table.normalize(expected)));
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Identifier(path.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Immediate(value) => {
            let type_ = value.type_of();
            ctx.table.unify(expected, &type_)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Immediate(value.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Tuple(items) => {
            let mut typed_items = Vec::with_capacity(items.len());
            let mut item_types = Vec::with_capacity(items.len());
            for item in items.iter() {
                let item_type = ctx.fresh_meta();
                let typed_item = infer_pattern(ctx, env, item, &item_type, bindings)?;
                item_types.push(item_type);
                typed_items.push(typed_item);
            }
            let tuple_type = Type::Tuple(item_types);
            ctx.table.unify(expected, &tuple_type)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Tuple(typed_items.into_boxed_slice()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let element_type = ctx.fresh_meta();
            let array_type = Type::Array(Box::new(element_type.clone()));
            ctx.table.unify(expected, &array_type)?;
            let mut typed_start = Vec::with_capacity(starting.len());
            let mut typed_end = Vec::with_capacity(ending.len());
            for item in starting.iter() {
                let typed_item = infer_pattern(ctx, env, item, &element_type, bindings)?;
                typed_start.push(typed_item);
            }
            for item in ending.iter() {
                let typed_item = infer_pattern(ctx, env, item, &element_type, bindings)?;
                typed_end.push(typed_item);
            }
            if let Glob::Named(path) = glob {
                bindings.push((path.clone(), ctx.table.normalize(&array_type)));
            }
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Array {
                    starting: typed_start.into_boxed_slice(),
                    glob: glob.clone(),
                    ending: typed_end.into_boxed_slice(),
                },
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            let mut field_types = IndexMap::new();
            for (name, value) in fields.iter() {
                let field_type = ctx.fresh_meta();
                let typed_value = infer_pattern(ctx, env, value, &field_type, bindings)?;
                field_types.insert(name.inner.clone(), field_type);
                typed_fields.insert(name.clone(), typed_value);
            }
            let struct_type = Type::StructConstraint {
                fields: field_types,
                mode: StructMatch::Exact,
            };
            ctx.table.unify(expected, &struct_type)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Struct(typed_fields),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::ConstConstructor(path) => {
            let scheme = env
                .get(path)
                .ok_or_else(|| TypeError::UnknownConstructor(path.clone()))?;
            let type_ = ctx.instantiate(scheme)?;
            ctx.table.unify(expected, &type_)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::ConstConstructor(path.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Constructor(path, payload) => {
            let scheme = env
                .get(path)
                .ok_or_else(|| TypeError::UnknownConstructor(path.clone()))?;
            let type_ = ctx.instantiate(scheme)?;
            let (param_type, result_type) = match ctx.table.normalize(&type_) {
                Type::Function(parameter, result) => (*parameter, *result),
                other => return Err(TypeError::NotAFunction(other)),
            };
            ctx.table.unify(expected, &result_type)?;
            let typed_payload = infer_pattern(ctx, env, payload, &param_type, bindings)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Constructor(path.clone(), Box::new(typed_payload)),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::TypeHint(inner, type_expr) => {
            let hint_type = type_expr_to_type(ctx, type_expr)?;
            ctx.table.unify(expected, &hint_type)?;
            let typed_inner = infer_pattern(ctx, env, inner, &hint_type, bindings)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::TypeHint(Box::new(typed_inner), type_expr.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
    }
}

fn field_access_type(
    ctx: &mut InferenceContext,
    type_: &Type,
    field_name: &str,
) -> Result<Type, TypeError> {
    let field_type = ctx.fresh_meta();
    let mut fields = IndexMap::new();
    fields.insert(field_name.to_string(), field_type.clone());
    let constraint = Type::StructConstraint {
        fields,
        mode: StructMatch::AtLeast,
    };
    ctx.table.unify(type_, &constraint)?;
    Ok(field_type)
}

fn type_expr_to_type(
    ctx: &InferenceContext,
    expr: &TypeExpr,
) -> Result<Type, TypeError> {
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            let mut types = Vec::with_capacity(items.len());
            for item in items.iter() {
                types.push(type_expr_to_type(ctx, item)?);
            }
            Ok(Type::Tuple(types))
        }
        TypeExprKind::Instantiation(path, args) => {
            let arguments = args
                .iter()
                .map(|arg| type_expr_to_type(ctx, arg))
                .collect::<Result<Vec<_>, _>>()?;
            if path.major == "core" {
                return core_type_from_path(path.clone(), &arguments);
            }
            let definition = ctx.type_definitions.get(path);
            if let Some(definition) = definition {
                if definition.parameters != arguments.len() {
                    return Err(TypeError::InvalidTypeApplication {
                        name: path.clone(),
                        expected: definition.parameters,
                        found: arguments.len(),
                    });
                }
                let base = Type::Named {
                    name: path.clone(),
                    body: Box::new(definition.body.clone()),
                };
                return if arguments.is_empty() {
                    Ok(base)
                } else {
                    Ok(Type::Apply {
                        constructor: Box::new(base),
                        arguments,
                    })
                };
            }
            let base = Type::Named {
                name: path.clone(),
                body: Box::new(Type::Unit),
            };
            if arguments.is_empty() {
                Ok(base)
            } else {
                Ok(Type::Apply {
                    constructor: Box::new(base),
                    arguments,
                })
            }
        }
    }
}

#[allow(clippy::missing_asserts_for_indexing)]
fn core_type_from_path(
    path: Path,
    args: &[Type],
) -> Result<Type, TypeError> {
    match path.minor.as_str() {
        "unit" => expect_arity(path, 0, args.len()).map(|_| Type::Unit),
        "integer" => expect_arity(path, 0, args.len()).map(|_| Type::Integer),
        "real" => expect_arity(path, 0, args.len()).map(|_| Type::Real),
        "boolean" => expect_arity(path, 0, args.len()).map(|_| Type::Boolean),
        "string" => expect_arity(path, 0, args.len()).map(|_| Type::String),
        "glyph" => expect_arity(path, 0, args.len()).map(|_| Type::Glyph),
        "array" => {
            expect_arity(path, 1, args.len()).map(|_| Type::Array(Box::new(args[0].clone())))
        }
        "function" => {
            expect_arity(path, 2, args.len()).map(|_| Type::func(args[0].clone(), args[1].clone()))
        }
        _ => {
            let base = Type::Named {
                name: path.clone(),
                body: Box::new(Type::Unit),
            };
            if args.is_empty() {
                Ok(base)
            } else {
                Ok(Type::Apply {
                    constructor: Box::new(base),
                    arguments: args.to_vec(),
                })
            }
        }
    }
}

fn expect_arity(
    name: Path,
    expected: usize,
    found: usize,
) -> Result<(), TypeError> {
    if expected == found {
        Ok(())
    } else {
        Err(TypeError::InvalidTypeApplication {
            name,
            expected,
            found,
        })
    }
}

fn replace_meta_vars(
    type_: &Type,
    mapping: &HashMap<MetaVarId, u32>,
) -> Type {
    match type_ {
        Type::MetaVar(id) => {
            mapping
                .get(id)
                .map(|index| Type::v(*index))
                .unwrap_or_else(|| Type::MetaVar(*id))
        }
        Type::Unit
        | Type::Integer
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::RecVar(_) => type_.clone(),
        Type::ForAll(body) => Type::ForAll(Box::new(replace_meta_vars(body, mapping))),
        Type::Mu(body) => Type::Mu(Box::new(replace_meta_vars(body, mapping))),
        Type::Named { name, body } => {
            Type::Named {
                name: name.clone(),
                body: body.clone(),
            }
        }
        Type::StructConstraint { fields, mode } => {
            Type::StructConstraint {
                fields: fields
                    .iter()
                    .map(|(name, type_)| (name.clone(), replace_meta_vars(type_, mapping)))
                    .collect(),
                mode: *mode,
            }
        }
        Type::Struct { fields } => {
            Type::Struct {
                fields: fields
                    .iter()
                    .map(|(name, type_)| (name.clone(), replace_meta_vars(type_, mapping)))
                    .collect(),
            }
        }
        Type::Array(inner) => Type::Array(Box::new(replace_meta_vars(inner, mapping))),
        Type::Tuple(items) => {
            Type::Tuple(
                items
                    .iter()
                    .map(|item| replace_meta_vars(item, mapping))
                    .collect(),
            )
        }
        Type::Sum {
            variant_names,
            variant_types,
        } => {
            Type::Sum {
                variant_names: variant_names.clone(),
                variant_types: variant_types
                    .iter()
                    .map(|variant| replace_meta_vars(variant, mapping))
                    .collect(),
            }
        }
        Type::Function(parameter, result) => {
            Type::func(
                replace_meta_vars(parameter, mapping),
                replace_meta_vars(result, mapping),
            )
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            Type::Apply {
                constructor: Box::new(replace_meta_vars(constructor, mapping)),
                arguments: arguments
                    .iter()
                    .map(|arg| replace_meta_vars(arg, mapping))
                    .collect(),
            }
        }
    }
}

fn normalize_predicates(
    table: &mut UnificationTable,
    predicates: &[TraitConstraint],
) -> Vec<TraitConstraint> {
    predicates
        .iter()
        .map(|predicate| {
            TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments: predicate
                    .arguments
                    .iter()
                    .map(|arg| table.normalize(arg))
                    .collect(),
            }
        })
        .collect()
}

fn replace_meta_vars_in_predicates(
    predicates: &[TraitConstraint],
    mapping: &HashMap<MetaVarId, u32>,
) -> Vec<TraitConstraint> {
    predicates
        .iter()
        .map(|predicate| {
            TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments: predicate
                    .arguments
                    .iter()
                    .map(|arg| replace_meta_vars(arg, mapping))
                    .collect(),
            }
        })
        .collect()
}

fn open_forall(
    body: &Type,
    replacement: &Type,
) -> Option<Type> {
    body.substitute_type_var(0, replacement)?
        .shift_type_vars(-1, 0)
}

fn open_forall_predicates(
    predicates: &[TraitConstraint],
    replacement: &Type,
) -> Option<Vec<TraitConstraint>> {
    predicates
        .iter()
        .map(|predicate| {
            let arguments = predicate
                .arguments
                .iter()
                .map(|arg| open_forall(arg, replacement))
                .collect::<Option<Vec<_>>>()?;
            Some(TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::ir::{
        ImmediateValue,
        ScopeKind,
    };
    use crate::{
        Span,
        WithSpan,
    };

    use super::*;

    fn term(kind: TermKind<()>) -> Term<()> {
        Term {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    fn pattern(kind: PatternKind<()>) -> Pattern<()> {
        Pattern {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    #[test]
    fn generalize_and_instantiate_roundtrip() {
        let mut ctx = InferenceContext::new();
        ctx.level = 1;
        let meta = ctx.fresh_meta();
        let scheme = ctx.generalize_at(&meta, 0);
        let instantiated = ctx.instantiate(&scheme).expect("instantiate");
        assert!(matches!(scheme.type_, Type::ForAll(_)));
        assert!(matches!(instantiated, Type::MetaVar(_)));
    }

    #[test]
    fn infer_polymorphic_let() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();

        let id_path = Path::new("test", "id");
        let x_path = Path::new("test", "x");

        let id_fn = term(TermKind::Function {
            parameter_name: x_path.clone().with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: term(TermKind::Identifier(x_path.clone())).into(),
        });

        let id_pattern = pattern(PatternKind::Identifier(id_path.clone()));

        let call_id_int = term(TermKind::Call {
            callee: term(TermKind::Identifier(id_path.clone())).into(),
            argument: term(TermKind::Immediate(ImmediateValue::Integer(1))).into(),
        });
        let call_id_bool = term(TermKind::Call {
            callee: term(TermKind::Identifier(id_path.clone())).into(),
            argument: term(TermKind::Immediate(ImmediateValue::Boolean(true))).into(),
        });

        let tuple = term(TermKind::Tuple(vec![call_id_int, call_id_bool]));
        let let_term = term(TermKind::Let {
            assignee: id_pattern,
            scope: ScopeKind::Local,
            value: id_fn.into(),
            then: tuple.into(),
            else_: term(TermKind::Unreachable).into(),
        });

        let typed = ctx.infer_term(&mut env, &let_term).expect("infer");
        assert_eq!(
            typed.term.type_,
            Type::Tuple(vec![Type::Integer, Type::Boolean])
        );
    }

    #[test]
    fn infer_struct_literal_produces_exact_constraint() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut fields = IndexMap::new();
        fields.insert(
            "x".to_string().with_span(Span::Generated),
            term(TermKind::Immediate(ImmediateValue::Integer(1))),
        );
        let literal = term(TermKind::Struct(fields));
        let typed = ctx.infer_term(&mut env, &literal).expect("infer");
        let Type::StructConstraint { fields, mode } = typed.term.type_ else {
            panic!("expected struct constraint");
        };
        assert_eq!(mode, StructMatch::Exact);
        assert_eq!(fields.len(), 1);
        assert!(fields.contains_key("x"));
    }

    #[test]
    fn infer_field_access_uses_named_struct() {
        let mut ctx = InferenceContext::new();
        let mut env = TypeEnv::new();
        let mut struct_fields = IndexMap::new();
        struct_fields.insert("x".to_string(), Type::Integer);
        struct_fields.insert("y".to_string(), Type::Boolean);
        let point_type = Type::Named {
            name: Path::new("test", "Point"),
            body: Box::new(Type::Struct {
                fields: struct_fields,
            }),
        };
        env.insert(Path::new("test", "p"), point_type);

        let field_term = term(TermKind::Field {
            of: term(TermKind::Identifier(Path::new("test", "p"))).into(),
            index: "x".to_string().with_span(Span::Generated),
        });

        let typed = ctx.infer_term(&mut env, &field_term).expect("infer");
        assert_eq!(typed.term.type_, Type::Integer);
    }
}
