use std::collections::HashMap;

use indexmap::IndexMap;

use crate::Span;
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
    TypeTransform,
};

use super::unify::{
    UnificationTable,
    UnifyError,
};

/// Errors produced during type inference.
#[derive(Debug, Clone)]
pub enum TypeError {
    UnknownIdentifier {
        path: Path,
        span: Span,
    },
    UnknownConstructor {
        path: Path,
        span: Span,
    },
    InvalidTypeApplication {
        name: Path,
        expected: usize,
        found: usize,
        span: Span,
    },
    MissingField {
        field: String,
        in_type: Type,
        span: Span,
    },
    NotAFunction {
        type_: Type,
        span: Span,
    },
    InvalidScheme {
        span: Span,
    },
    Unification {
        error: UnifyError,
        span: Span,
    },
}

fn unify_with_span(
    table: &mut UnificationTable,
    left: &Type,
    right: &Type,
    span: Span,
) -> Result<(), TypeError> {
    table
        .unify(left, right)
        .map_err(|error| TypeError::Unification { error, span })
}

/// Mapping of term paths to their type schemes.
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

/// Inference state: unification table, level tracking, and known types.
#[derive(Debug, Default)]
pub struct InferenceContext {
    table: UnificationTable,
    level: u32,
    type_definitions: IndexMap<Path, TypeDefinition>,
}

/// Instantiated scheme paired with its trait predicates.
#[derive(Debug, Clone)]
pub struct SchemeInstance {
    pub type_: Type,
    pub predicates: Vec<TraitConstraint>,
}

/// Result of inference for a term, including remaining predicates.
#[derive(Debug, Clone)]
pub struct InferenceOutput {
    pub term: Term<Type>,
    pub predicates: Vec<TraitConstraint>,
}

struct InferredTermItems {
    items: Box<[Term<Type>]>,
    predicates: Vec<TraitConstraint>,
}

struct InferredPatternItems {
    items: Box<[Pattern<Type>]>,
    types: Vec<Type>,
}

impl InferenceContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_type_definitions(
        &mut self,
        definitions: IndexMap<Path, TypeDefinition>,
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
        span: Span,
    ) -> Result<Type, TypeError> {
        Ok(self.instantiate_scheme(scheme, span)?.type_)
    }

    pub fn instantiate_scheme(
        &mut self,
        scheme: &TypeScheme,
        span: Span,
    ) -> Result<SchemeInstance, TypeError> {
        let mut current = scheme.type_.clone();
        let mut predicates = scheme.predicates.clone();
        loop {
            match current {
                Type::ForAll(body) => {
                    let fresh = self.fresh_meta();
                    current = body
                        .open_forall(&fresh)
                        .ok_or(TypeError::InvalidScheme { span })?;
                    predicates = open_forall_predicates(&predicates, &fresh)
                        .ok_or(TypeError::InvalidScheme { span })?;
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
        let normalized_predicates = self.table.normalize_predicates(&predicates);
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
        let type_ = ReplaceMetaVars {
            mapping: &replacements,
        }
        .transform(&normalized_type)
        .unwrap_or_else(|| normalized_type.clone())
        .for_all(metas.len());
        let predicates = replace_meta_vars_in_predicates(&normalized_predicates, &replacements);
        type_.scheme_with_predicates(predicates)
    }

    pub fn infer_term(
        &mut self,
        env: &mut TypeEnv,
        term: &Term<()>,
        schemes: &mut IndexMap<Path, TypeScheme>,
    ) -> Result<InferenceOutput, TypeError> {
        infer_term(self, env, term, schemes)
    }
}

pub fn infer_term(
    ctx: &mut InferenceContext,
    env: &mut TypeEnv,
    term: &Term<()>,
    schemes: &mut IndexMap<Path, TypeScheme>,
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
            let scheme = env.get(path).ok_or_else(|| {
                TypeError::UnknownIdentifier {
                    path: path.clone(),
                    span: term.span,
                }
            })?;
            let instance = ctx.instantiate_scheme(scheme, term.span)?;
            (
                TermKind::Identifier(path.clone()),
                instance.type_,
                instance.predicates,
            )
        }
        TermKind::Tuple(items) => {
            let InferredTermItems {
                items: typed_items,
                predicates,
            } = infer_term_items(ctx, env, items, schemes)?;
            let types = typed_items.iter().map(|item| item.type_.clone()).collect();
            (
                TermKind::Tuple(Vec::from(typed_items)),
                Type::Tuple(types),
                predicates,
            )
        }
        TermKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            let mut field_types = IndexMap::new();
            let mut predicates = Vec::new();
            for (name, value) in fields {
                let typed = infer_term(ctx, env, value, schemes)?;
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
            let typed_of = infer_term(ctx, env, of, schemes)?;
            let field_name = index.inner.clone();
            let field_type = field_access_type(ctx, &typed_of.term.type_, &field_name, index.span)?;
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
            let typed_body = infer_term(ctx, &mut env_with_param, body, schemes)?;
            let typed_captures = captures
                .iter()
                .map(|(path, _)| {
                    let scheme = env.get(path).ok_or_else(|| {
                        TypeError::UnknownIdentifier {
                            path: path.clone(),
                            span: Span::Generated,
                        }
                    })?;
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
            let typed_callee = infer_term(ctx, env, callee, schemes)?;
            let typed_argument = infer_term(ctx, env, argument, schemes)?;
            let result_type = ctx.fresh_meta();
            let function_type = Type::func(typed_argument.term.type_.clone(), result_type.clone());
            unify_with_span(
                &mut ctx.table,
                &typed_callee.term.type_,
                &function_type,
                term.span,
            )?;
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
            let typed_value = infer_term(ctx, env, value, schemes)?;
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
            schemes.extend(generalized.iter().cloned());
            let mut env_with = env.with_bindings(generalized.clone());
            let typed_then = infer_term(ctx, &mut env_with, then, schemes)?;
            let typed_else = infer_term(ctx, env, else_, schemes)?;
            unify_with_span(
                &mut ctx.table,
                &typed_then.term.type_,
                &typed_else.term.type_,
                term.span,
            )?;
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
            let typed_left = infer_term(ctx, env, left, schemes)?;
            let typed_right = infer_term(ctx, env, right, schemes)?;
            unify_with_span(
                &mut ctx.table,
                &typed_left.term.type_,
                &Type::Unit,
                typed_left.term.span,
            )?;
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
    let predicates = ctx.table.normalize_predicates(&predicates);
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
            unify_with_span(&mut ctx.table, expected, &type_, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Immediate(value.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Tuple(items) => {
            let InferredPatternItems {
                items: typed_items,
                types: item_types,
            } = infer_pattern_items(ctx, env, items, bindings)?;
            let tuple_type = Type::Tuple(item_types);
            unify_with_span(&mut ctx.table, expected, &tuple_type, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Tuple(typed_items),
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
            unify_with_span(&mut ctx.table, expected, &array_type, pattern.span)?;
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
            unify_with_span(&mut ctx.table, expected, &struct_type, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::Struct(typed_fields),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::ConstConstructor(path) => {
            let scheme = env.get(path).ok_or_else(|| {
                TypeError::UnknownConstructor {
                    path: path.clone(),
                    span: pattern.span,
                }
            })?;
            let type_ = ctx.instantiate(scheme, pattern.span)?;
            unify_with_span(&mut ctx.table, expected, &type_, pattern.span)?;
            Ok(Pattern {
                comments: pattern.comments.clone(),
                kind: PatternKind::ConstConstructor(path.clone()),
                span: pattern.span,
                type_: ctx.table.normalize(expected),
            })
        }
        PatternKind::Constructor(path, payload) => {
            let scheme = env.get(path).ok_or_else(|| {
                TypeError::UnknownConstructor {
                    path: path.clone(),
                    span: pattern.span,
                }
            })?;
            let type_ = ctx.instantiate(scheme, pattern.span)?;
            let (param_type, result_type) = match ctx.table.normalize(&type_) {
                Type::Function(parameter, result) => (*parameter, *result),
                other => {
                    return Err(TypeError::NotAFunction {
                        type_: other,
                        span: pattern.span,
                    });
                }
            };
            unify_with_span(&mut ctx.table, expected, &result_type, pattern.span)?;
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
            unify_with_span(&mut ctx.table, expected, &hint_type, type_expr.span)?;
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
    span: Span,
) -> Result<Type, TypeError> {
    let field_type = ctx.fresh_meta();
    let mut fields = IndexMap::new();
    fields.insert(field_name.to_string(), field_type.clone());
    let constraint = Type::StructConstraint {
        fields,
        mode: StructMatch::AtLeast,
    };
    unify_with_span(&mut ctx.table, type_, &constraint, span)?;
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
            let definition = ctx.type_definitions.get(path);
            if let Some(definition) = definition {
                if definition.parameters != arguments.len() {
                    return Err(TypeError::InvalidTypeApplication {
                        name: path.clone(),
                        expected: definition.parameters,
                        found: arguments.len(),
                        span: expr.span,
                    });
                }
                let base = Type::Named {
                    name: path.clone(),
                    body: Box::new(definition.body.clone()),
                };
                return Ok(base.apply(arguments));
            }
            let base = Type::Named {
                name: path.clone(),
                body: Box::new(Type::Unit),
            };
            Ok(base.apply(arguments))
        }
    }
}

fn infer_term_items(
    ctx: &mut InferenceContext,
    env: &mut TypeEnv,
    items: &[Term<()>],
    schemes: &mut IndexMap<Path, TypeScheme>,
) -> Result<InferredTermItems, TypeError> {
    items
        .iter()
        .try_fold(
            (Vec::with_capacity(items.len()), Vec::new()),
            |(mut typed_items, mut predicates), item| {
                let typed = infer_term(ctx, env, item, schemes)?;
                predicates.extend(typed.predicates);
                typed_items.push(typed.term);
                Ok((typed_items, predicates))
            },
        )
        .map(|(items, predicates)| {
            InferredTermItems {
                items: items.into_boxed_slice(),
                predicates,
            }
        })
}

fn infer_pattern_items(
    ctx: &mut InferenceContext,
    env: &TypeEnv,
    items: &[Pattern<()>],
    bindings: &mut Vec<(Path, Type)>,
) -> Result<InferredPatternItems, TypeError> {
    items
        .iter()
        .try_fold(
            (
                Vec::with_capacity(items.len()),
                Vec::with_capacity(items.len()),
            ),
            |(mut typed_items, mut item_types), item| {
                let item_type = ctx.fresh_meta();
                let typed_item = infer_pattern(ctx, env, item, &item_type, bindings)?;
                item_types.push(item_type);
                typed_items.push(typed_item);
                Ok((typed_items, item_types))
            },
        )
        .map(|(items, types)| {
            InferredPatternItems {
                items: items.into_boxed_slice(),
                types,
            }
        })
}

struct ReplaceMetaVars<'a> {
    mapping: &'a HashMap<MetaVarId, u32>,
}

impl TypeTransform for ReplaceMetaVars<'_> {
    fn meta_var(
        &mut self,
        id: MetaVarId,
    ) -> Option<Type> {
        Some(
            self.mapping
                .get(&id)
                .map(|index| Type::v(*index))
                .unwrap_or_else(|| Type::MetaVar(id)),
        )
    }
}

fn replace_meta_vars_in_predicates(
    predicates: &[TraitConstraint],
    mapping: &HashMap<MetaVarId, u32>,
) -> Vec<TraitConstraint> {
    let mut replacer = ReplaceMetaVars { mapping };
    predicates
        .iter()
        .map(|predicate| {
            TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments: predicate
                    .arguments
                    .iter()
                    .map(|arg| replacer.transform(arg).unwrap_or_else(|| arg.clone()))
                    .collect(),
            }
        })
        .collect()
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
                .map(|arg| arg.open_forall(replacement))
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
        let instantiated = ctx
            .instantiate(&scheme, Span::Generated)
            .expect("instantiate");
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

        let mut schemes = IndexMap::new();
        let typed = ctx
            .infer_term(&mut env, &let_term, &mut schemes)
            .expect("infer");
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
        let mut schemes = IndexMap::new();
        let typed = ctx
            .infer_term(&mut env, &literal, &mut schemes)
            .expect("infer");
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

        let mut schemes = IndexMap::new();
        let typed = ctx
            .infer_term(&mut env, &field_term, &mut schemes)
            .expect("infer");
        assert_eq!(typed.term.type_, Type::Integer);
    }
}
