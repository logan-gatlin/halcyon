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
    Type,
    TypeName,
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
        name: TypeName,
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
    bindings: IndexMap<Path, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(
        &self,
        path: &Path,
    ) -> Option<&Type> {
        self.bindings.get(path)
    }

    pub fn with_binding(
        &self,
        path: Path,
        scheme: Type,
    ) -> Self {
        let mut next = self.clone();
        next.bindings.insert(path, scheme);
        next
    }

    pub fn with_bindings(
        &self,
        bindings: impl IntoIterator<Item = (Path, Type)>,
    ) -> Self {
        let mut next = self.clone();
        next.bindings.extend(bindings);
        next
    }

    pub fn insert(
        &mut self,
        path: Path,
        scheme: Type,
    ) {
        self.bindings.insert(path, scheme);
    }

    pub fn extend(
        &mut self,
        bindings: impl IntoIterator<Item = (Path, Type)>,
    ) {
        self.bindings.extend(bindings);
    }
}

#[derive(Debug, Default)]
pub struct InferenceContext {
    table: UnificationTable,
    level: u32,
}

impl InferenceContext {
    pub fn new() -> Self {
        Self::default()
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
        scheme: &Type,
    ) -> Result<Type, TypeError> {
        let mut current = scheme.clone();
        loop {
            match current {
                Type::ForAll(body) => {
                    let fresh = self.fresh_meta();
                    current = open_forall(&body, &fresh).ok_or(TypeError::InvalidScheme)?;
                }
                other => return Ok(other),
            }
        }
    }

    pub fn generalize_at(
        &mut self,
        type_: &Type,
        level: u32,
    ) -> Type {
        let normalized = self.table.normalize(type_);
        let mut metas = self
            .table
            .free_meta_vars(&normalized)
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
        (0..metas.len()).fold(replace_meta_vars(&normalized, &replacements), |r, _| {
            Type::ForAll(Box::new(r))
        })
    }

    pub fn infer_term(
        &mut self,
        env: &mut TypeEnv,
        term: &Term<()>,
    ) -> Result<Term<Type>, TypeError> {
        infer_term(self, env, term)
    }
}

pub fn infer_term(
    ctx: &mut InferenceContext,
    env: &mut TypeEnv,
    term: &Term<()>,
) -> Result<Term<Type>, TypeError> {
    let (kind, type_) = match &term.kind {
        TermKind::Immediate(value) => (TermKind::Immediate(value.clone()), value.type_of()),
        TermKind::Identifier(path) => {
            let scheme = env
                .get(path)
                .ok_or_else(|| TypeError::UnknownIdentifier(path.clone()))?;
            let type_ = ctx.instantiate(scheme)?;
            (TermKind::Identifier(path.clone()), type_)
        }
        TermKind::Tuple(items) => {
            let mut typed_items = Vec::with_capacity(items.len());
            let mut types = Vec::with_capacity(items.len());
            for item in items {
                let typed = infer_term(ctx, env, item)?;
                types.push(typed.type_.clone());
                typed_items.push(typed);
            }
            (TermKind::Tuple(typed_items), Type::Tuple(types))
        }
        TermKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            let mut field_types = IndexMap::new();
            for (name, value) in fields {
                let typed = infer_term(ctx, env, value)?;
                field_types.insert(name.inner.clone(), typed.type_.clone());
                typed_fields.insert(name.clone(), typed);
            }
            (
                TermKind::Struct(typed_fields),
                Type::StructConstraint {
                    fields: field_types,
                    mode: StructMatch::Exact,
                },
            )
        }
        TermKind::Field { of, index } => {
            let typed_of = infer_term(ctx, env, of)?;
            let field_name = index.inner.clone();
            let field_type = field_access_type(ctx, &typed_of.type_, &field_name)?;
            (
                TermKind::Field {
                    of: typed_of.into(),
                    index: index.clone(),
                },
                field_type,
            )
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            let param_type = match parameter_type {
                Some(type_expr) => type_expr_to_type(type_expr)?,
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
                    Ok((path.clone(), scheme.clone()))
                })
                .collect::<Result<Vec<_>, TypeError>>()?;
            let type_ = Type::Function(Box::new(param_type), Box::new(typed_body.type_.clone()));
            (
                TermKind::Function {
                    parameter_name: parameter_name.clone(),
                    parameter_type: parameter_type.clone(),
                    captures: typed_captures.into_boxed_slice(),
                    body: typed_body.into(),
                },
                type_,
            )
        }
        TermKind::Call { callee, argument } => {
            let typed_callee = infer_term(ctx, env, callee)?;
            let typed_argument = infer_term(ctx, env, argument)?;
            let result_type = ctx.fresh_meta();
            let function_type = Type::Function(
                Box::new(typed_argument.type_.clone()),
                Box::new(result_type.clone()),
            );
            ctx.table.unify(&typed_callee.type_, &function_type)?;
            (
                TermKind::Call {
                    callee: typed_callee.into(),
                    argument: typed_argument.into(),
                },
                result_type,
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
                infer_pattern(ctx, env, assignee, &typed_value.type_, &mut bindings)?;
            ctx.level = outer_level;

            let generalized = bindings
                .into_iter()
                .map(|(path, type_)| (path, ctx.generalize_at(&type_, outer_level)))
                .collect::<Vec<_>>();
            let mut env_with = env.with_bindings(generalized.clone());
            let typed_then = infer_term(ctx, &mut env_with, then)?;
            let typed_else = infer_term(ctx, env, else_)?;
            ctx.table.unify(&typed_then.type_, &typed_else.type_)?;
            let result_type = ctx.table.normalize(&typed_then.type_);
            if *scope == ScopeKind::Global {
                env.extend(generalized);
            }
            (
                TermKind::Let {
                    assignee: typed_pattern,
                    scope: *scope,
                    value: typed_value.into(),
                    then: typed_then.into(),
                    else_: typed_else.into(),
                },
                result_type,
            )
        }
        TermKind::Semicolon(left, right) => {
            let typed_left = infer_term(ctx, env, left)?;
            let typed_right = infer_term(ctx, env, right)?;
            ctx.table.unify(&typed_left.type_, &Type::Unit)?;
            let result_type = typed_right.type_.clone();
            (
                TermKind::Semicolon(typed_left.into(), typed_right.into()),
                result_type,
            )
        }
        TermKind::Unreachable => (TermKind::Unreachable, ctx.fresh_meta()),
    };

    let normalized = ctx.table.normalize(&type_);
    Ok(Term {
        comments: term.comments.clone(),
        kind,
        span: term.span,
        type_: normalized,
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
            let hint_type = type_expr_to_type(type_expr)?;
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

fn type_expr_to_type(expr: &TypeExpr) -> Result<Type, TypeError> {
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            let mut types = Vec::with_capacity(items.len());
            for item in items.iter() {
                types.push(type_expr_to_type(item)?);
            }
            Ok(Type::Tuple(types))
        }
        TypeExprKind::Instantiation(path, args) => {
            let arguments = args
                .iter()
                .map(type_expr_to_type)
                .collect::<Result<Vec<_>, _>>()?;
            if path.major == "core" {
                return core_type_from_path(path, &arguments);
            }
            let name = TypeName::new(path.major.clone(), path.minor.clone());
            let base = Type::Named {
                name,
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
    path: &Path,
    args: &[Type],
) -> Result<Type, TypeError> {
    let name = TypeName::new(path.major.clone(), path.minor.clone());
    match path.minor.as_str() {
        "unit" => expect_arity(name, 0, args.len()).map(|_| Type::Unit),
        "integer" => expect_arity(name, 0, args.len()).map(|_| Type::Integer),
        "real" => expect_arity(name, 0, args.len()).map(|_| Type::Real),
        "boolean" => expect_arity(name, 0, args.len()).map(|_| Type::Boolean),
        "string" => expect_arity(name, 0, args.len()).map(|_| Type::String),
        "glyph" => expect_arity(name, 0, args.len()).map(|_| Type::Glyph),
        "array" => {
            expect_arity(name, 1, args.len()).map(|_| Type::Array(Box::new(args[0].clone())))
        }
        "function" => {
            expect_arity(name, 2, args.len())
                .map(|_| Type::Function(Box::new(args[0].clone()), Box::new(args[1].clone())))
        }
        _ => {
            let base = Type::Named {
                name,
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
    name: TypeName,
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
                .map(|index| Type::TypeVar(*index))
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
            Type::Function(
                Box::new(replace_meta_vars(parameter, mapping)),
                Box::new(replace_meta_vars(result, mapping)),
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

fn open_forall(
    body: &Type,
    replacement: &Type,
) -> Option<Type> {
    body.substitute_type_var(0, replacement)?
        .shift_type_vars(-1, 0)
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
        assert!(matches!(scheme, Type::ForAll(_)));
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
        assert_eq!(typed.type_, Type::Tuple(vec![Type::Integer, Type::Boolean]));
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
        let Type::StructConstraint { fields, mode } = typed.type_ else {
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
            name: TypeName::new("test", "Point"),
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
        assert_eq!(typed.type_, Type::Integer);
    }
}
