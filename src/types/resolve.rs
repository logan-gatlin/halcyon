use std::collections::HashMap;

use indexmap::IndexMap;

use crate::hc_core::CoreSymbols;
use crate::ir::{
    Module,
    Path,
    Pattern,
    PatternKind,
    Statement,
    Term,
    TermKind,
    TypeDefKind,
    TypeExpr,
    TypeExprKind,
};
use crate::logging::WithContext;
use crate::{
    FileLogger,
    Span,
};

use super::catalog::{
    TypeCatalog,
    TypeDefinition,
};
use super::infer::{
    InferenceContext,
    TypeEnv,
    TypeError,
};
use super::{
    Type,
    TypeName,
    TypeScheme,
};

#[derive(Debug, Clone)]
struct TypeDefEntry {
    parameters: Vec<Path>,
    def: crate::ir::TypeDef,
}

pub fn resolve_module(
    module: Module<()>,
    logger: &mut FileLogger,
) -> Module<Type> {
    let Module { name, statements } = module;
    let statements = Vec::from(statements);
    let type_entries = collect_type_entries(&statements);
    let mut catalog = build_type_catalog(&type_entries, logger);
    let mut env = TypeEnv::new();
    let constructors = build_sum_constructors(&type_entries, &mut catalog, logger);
    env.extend(constructors);
    env.extend(core_array_schemes());

    let mut ctx = InferenceContext::new();
    ctx.set_type_catalog(catalog);

    let typed_statements = statements
        .into_iter()
        .map(|statement| {
            match statement {
                Statement::Term(term) => {
                    let typed = match ctx.infer_term(&mut env, &term) {
                        Ok(typed) => typed,
                        Err(error) => {
                            log_type_error(logger, term.span, error);
                            fallback_term(&term)
                        }
                    };
                    Statement::Term(typed)
                }
                Statement::Type {
                    path,
                    parameters,
                    def,
                } => {
                    Statement::Type {
                        path,
                        parameters,
                        def,
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    Module {
        name,
        statements: typed_statements.into_boxed_slice(),
    }
}

fn collect_type_entries(statements: &[Statement<()>]) -> HashMap<Path, TypeDefEntry> {
    let mut entries = HashMap::new();
    for statement in statements {
        if let Statement::Type {
            path,
            parameters,
            def,
        } = statement
        {
            entries.insert(
                path.clone(),
                TypeDefEntry {
                    parameters: parameters.to_vec(),
                    def: def.clone(),
                },
            );
        }
    }
    entries
}

fn build_type_catalog(
    entries: &HashMap<Path, TypeDefEntry>,
    logger: &mut FileLogger,
) -> TypeCatalog {
    let mut catalog = TypeCatalog::new();
    let mut stack = Vec::new();
    for path in entries.keys() {
        let _ = resolve_type_definition(path, entries, &mut catalog, &mut stack, logger);
    }
    catalog
}

fn resolve_type_definition(
    path: &Path,
    entries: &HashMap<Path, TypeDefEntry>,
    catalog: &mut TypeCatalog,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> TypeDefinition {
    if let Some(definition) = catalog.get(path) {
        return definition.clone();
    }
    let Some(entry) = entries.get(path) else {
        return TypeDefinition {
            parameters: 0,
            body: Type::Unit,
        };
    };
    if stack.contains(path) {
        logger
            .error("Recursive type definitions are not supported yet")
            .primary(
                format!("Type `{}` depends on itself.", format_path(path)),
                entry.def.span(),
            )
            .done();
        let definition = TypeDefinition {
            parameters: entry.parameters.len(),
            body: Type::Unit,
        };
        catalog.insert(path.clone(), definition.clone());
        return definition;
    }

    stack.push(path.clone());
    let param_map = param_index_map(&entry.parameters);
    let body = type_def_kind_to_type(
        entry.def.kind(),
        &param_map,
        entries,
        catalog,
        stack,
        logger,
    );
    let body = wrap_forall(body, entry.parameters.len());
    let definition = TypeDefinition {
        parameters: entry.parameters.len(),
        body,
    };
    catalog.insert(path.clone(), definition.clone());
    stack.pop();
    definition
}

fn type_def_kind_to_type(
    kind: &TypeDefKind,
    param_map: &HashMap<Path, u32>,
    entries: &HashMap<Path, TypeDefEntry>,
    catalog: &mut TypeCatalog,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> Type {
    match kind {
        TypeDefKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            for (name, type_expr) in fields.iter() {
                let field_type =
                    type_expr_to_type_in_def(type_expr, param_map, entries, catalog, stack, logger);
                typed_fields.insert(name.clone(), field_type);
            }
            Type::Struct {
                fields: typed_fields,
            }
        }
        TypeDefKind::Sum(variants) => {
            let mut variant_names = Vec::with_capacity(variants.len());
            let mut variant_types = Vec::with_capacity(variants.len());
            for (name, type_expr) in variants.iter() {
                variant_names.push(name.clone());
                variant_types.push(type_expr_to_type_in_def(
                    type_expr, param_map, entries, catalog, stack, logger,
                ));
            }
            Type::Sum {
                variant_names,
                variant_types,
            }
        }
        TypeDefKind::Expr(type_expr) => {
            type_expr_to_type_in_def(type_expr, param_map, entries, catalog, stack, logger)
        }
    }
}

fn type_expr_to_type_in_def(
    expr: &TypeExpr,
    param_map: &HashMap<Path, u32>,
    entries: &HashMap<Path, TypeDefEntry>,
    catalog: &mut TypeCatalog,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> Type {
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| {
                    type_expr_to_type_in_def(item, param_map, entries, catalog, stack, logger)
                })
                .collect();
            Type::Tuple(items)
        }
        TypeExprKind::Instantiation(path, args) => {
            if let Some(index) = param_map.get(path) {
                if !args.is_empty() {
                    logger
                        .error("Type parameters cannot be applied")
                        .primary(
                            format!(
                                "`{}` is a type parameter but is applied to arguments.",
                                format_path(path)
                            ),
                            expr.span,
                        )
                        .done();
                }
                return Type::TypeVar(*index);
            }

            let arguments = args
                .iter()
                .map(|arg| {
                    type_expr_to_type_in_def(arg, param_map, entries, catalog, stack, logger)
                })
                .collect::<Vec<_>>();

            if path.major == "core" {
                return core_type_from_path(expr.span, path, &arguments, logger);
            }

            let base = if entries.contains_key(path) {
                let definition = resolve_type_definition(path, entries, catalog, stack, logger);
                if definition.parameters != arguments.len() {
                    logger
                        .error("Invalid type application")
                        .primary(
                            format!(
                                "`{}` expects {} type arguments but got {}.",
                                format_path(path),
                                definition.parameters,
                                arguments.len()
                            ),
                            expr.span,
                        )
                        .done();
                }
                Type::Named {
                    name: TypeName::new(path.major.clone(), path.minor.clone()),
                    body: Box::new(definition.body),
                }
            } else {
                Type::Named {
                    name: TypeName::new(path.major.clone(), path.minor.clone()),
                    body: Box::new(Type::Unit),
                }
            };

            apply_type_constructor(base, arguments)
        }
    }
}

fn build_sum_constructors(
    entries: &HashMap<Path, TypeDefEntry>,
    catalog: &mut TypeCatalog,
    logger: &mut FileLogger,
) -> Vec<(Path, TypeScheme)> {
    let mut constructors = Vec::new();
    for (path, entry) in entries.iter() {
        let TypeDefKind::Sum(variants) = entry.def.kind() else {
            continue;
        };

        let param_map = param_index_map(&entry.parameters);
        let mut stack = Vec::new();
        let definition = resolve_type_definition(path, entries, catalog, &mut stack, logger);
        let base = Type::Named {
            name: TypeName::new(path.major.clone(), path.minor.clone()),
            body: Box::new(definition.body.clone()),
        };
        let args = type_vars_for_params(entry.parameters.len());
        let result_type = apply_type_constructor(base, args);

        for (variant, type_expr) in variants.iter() {
            let payload_type = type_expr_to_type_in_def(
                type_expr, &param_map, entries, catalog, &mut stack, logger,
            );
            let constructor_type = if matches!(payload_type, Type::Unit) {
                result_type.clone()
            } else {
                Type::Function(Box::new(payload_type), Box::new(result_type.clone()))
            };
            let scheme_type = wrap_forall(constructor_type, entry.parameters.len());
            constructors.push((
                Path::new(path.major.clone(), variant.clone()),
                TypeScheme::new(scheme_type),
            ));
        }
    }
    constructors
}

fn core_array_schemes() -> Vec<(Path, TypeScheme)> {
    let array_var = Type::TypeVar(0);
    let array_type = Type::Array(Box::new(array_var.clone()));
    let empty = wrap_forall(array_type.clone(), 1);
    let push = wrap_forall(
        Type::Function(
            Box::new(array_var.clone()),
            Box::new(Type::Function(
                Box::new(array_type.clone()),
                Box::new(array_type.clone()),
            )),
        ),
        1,
    );
    let concat = wrap_forall(
        Type::Function(
            Box::new(array_type.clone()),
            Box::new(Type::Function(
                Box::new(array_type.clone()),
                Box::new(array_type.clone()),
            )),
        ),
        1,
    );
    vec![
        (CoreSymbols::EmptyArray.path(), TypeScheme::new(empty)),
        (CoreSymbols::ArrayPush.path(), TypeScheme::new(push)),
        (CoreSymbols::ArrayConcat.path(), TypeScheme::new(concat)),
    ]
}

fn core_type_from_path(
    span: Span,
    path: &Path,
    args: &[Type],
    logger: &mut FileLogger,
) -> Type {
    match path.minor.as_str() {
        "unit" => {
            expect_core_arity(span, path, 0, args.len(), logger).map_or(Type::Unit, |_| Type::Unit)
        }
        "integer" => {
            expect_core_arity(span, path, 0, args.len(), logger)
                .map_or(Type::Integer, |_| Type::Integer)
        }
        "real" => {
            expect_core_arity(span, path, 0, args.len(), logger).map_or(Type::Real, |_| Type::Real)
        }
        "boolean" => {
            expect_core_arity(span, path, 0, args.len(), logger)
                .map_or(Type::Boolean, |_| Type::Boolean)
        }
        "string" => {
            expect_core_arity(span, path, 0, args.len(), logger)
                .map_or(Type::String, |_| Type::String)
        }
        "glyph" => {
            expect_core_arity(span, path, 0, args.len(), logger)
                .map_or(Type::Glyph, |_| Type::Glyph)
        }
        "array" => {
            if expect_core_arity(span, path, 1, args.len(), logger).is_ok() {
                Type::Array(Box::new(args[0].clone()))
            } else {
                Type::Array(Box::new(Type::Unit))
            }
        }
        "function" => {
            if expect_core_arity(span, path, 2, args.len(), logger).is_ok() {
                Type::Function(Box::new(args[0].clone()), Box::new(args[1].clone()))
            } else {
                Type::Function(Box::new(Type::Unit), Box::new(Type::Unit))
            }
        }
        _ => {
            let base = Type::Named {
                name: TypeName::new(path.major.clone(), path.minor.clone()),
                body: Box::new(Type::Unit),
            };
            apply_type_constructor(base, args.to_vec())
        }
    }
}

fn expect_core_arity(
    span: Span,
    path: &Path,
    expected: usize,
    found: usize,
    logger: &mut FileLogger,
) -> Result<(), ()> {
    if expected == found {
        Ok(())
    } else {
        logger
            .error("Invalid type application")
            .primary(
                format!(
                    "`{}` expects {} type arguments but got {}.",
                    format_path(path),
                    expected,
                    found
                ),
                span,
            )
            .done();
        Err(())
    }
}

fn wrap_forall(
    type_: Type,
    count: usize,
) -> Type {
    (0..count).fold(type_, |inner, _| Type::ForAll(Box::new(inner)))
}

fn apply_type_constructor(
    constructor: Type,
    arguments: Vec<Type>,
) -> Type {
    if arguments.is_empty() {
        constructor
    } else {
        Type::Apply {
            constructor: Box::new(constructor),
            arguments,
        }
    }
}

fn param_index_map(parameters: &[Path]) -> HashMap<Path, u32> {
    let count = parameters.len();
    parameters
        .iter()
        .enumerate()
        .map(|(index, path)| (path.clone(), (count - 1 - index) as u32))
        .collect()
}

fn type_vars_for_params(count: usize) -> Vec<Type> {
    (0..count)
        .map(|index| Type::TypeVar((count - 1 - index) as u32))
        .collect()
}

fn format_path(path: &Path) -> String {
    if path.major.is_empty() {
        path.minor.clone()
    } else {
        format!("{}::{}", path.major, path.minor)
    }
}

fn log_type_error(
    logger: &mut FileLogger,
    span: Span,
    error: TypeError,
) {
    match error {
        TypeError::UnknownIdentifier(path) => {
            logger
                .error("Unknown identifier")
                .primary(format!("`{}` is not defined.", format_path(&path)), span)
                .done();
        }
        TypeError::UnknownConstructor(path) => {
            logger
                .error("Unknown constructor")
                .primary(format!("`{}` is not defined.", format_path(&path)), span)
                .done();
        }
        TypeError::InvalidTypeApplication {
            name,
            expected,
            found,
        } => {
            logger
                .error("Invalid type application")
                .primary(
                    format!("`{name}` expects {expected} type arguments but got {found}."),
                    span,
                )
                .done();
        }
        TypeError::MissingField { field, in_type } => {
            logger
                .error("Missing field")
                .primary(format!("Field `{field}` is missing in `{in_type}`."), span)
                .done();
        }
        TypeError::NotAFunction(type_) => {
            logger
                .error("Not a function")
                .primary(format!("`{type_}` is not callable."), span)
                .done();
        }
        TypeError::InvalidScheme => {
            logger
                .error("Invalid type scheme")
                .primary("A type scheme could not be instantiated.", span)
                .done();
        }
        TypeError::Unification(error) => {
            match error {
                super::unify::UnifyError::Occurs { var, in_type } => {
                    logger
                        .error("Occurs check failed")
                        .primary(
                            format!("Type variable ?t{var} occurs in `{in_type}`."),
                            span,
                        )
                        .done();
                }
                super::unify::UnifyError::Mismatch { left, right } => {
                    logger
                        .error("Type mismatch")
                        .primary(format!("`{left}` does not match `{right}`."), span)
                        .done();
                }
            }
        }
    }
}

fn fallback_term(term: &Term<()>) -> Term<Type> {
    let kind = match &term.kind {
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            TermKind::Let {
                assignee: fallback_pattern(assignee),
                scope: *scope,
                value: Box::new(fallback_term(value)),
                then: Box::new(fallback_term(then)),
                else_: Box::new(fallback_term(else_)),
            }
        }
        TermKind::Immediate(value) => TermKind::Immediate(value.clone()),
        TermKind::Identifier(path) => TermKind::Identifier(path.clone()),
        TermKind::Tuple(items) => TermKind::Tuple(items.iter().map(fallback_term).collect()),
        TermKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            for (name, value) in fields.iter() {
                typed_fields.insert(name.clone(), fallback_term(value));
            }
            TermKind::Struct(typed_fields)
        }
        TermKind::Field { of, index } => {
            TermKind::Field {
                of: Box::new(fallback_term(of)),
                index: index.clone(),
            }
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            let typed_captures = captures
                .iter()
                .map(|(path, _)| (path.clone(), Type::Unit))
                .collect::<Vec<_>>();
            TermKind::Function {
                parameter_name: parameter_name.clone(),
                parameter_type: parameter_type.clone(),
                captures: typed_captures.into_boxed_slice(),
                body: Box::new(fallback_term(body)),
            }
        }
        TermKind::Call { callee, argument } => {
            TermKind::Call {
                callee: Box::new(fallback_term(callee)),
                argument: Box::new(fallback_term(argument)),
            }
        }
        TermKind::Semicolon(left, right) => {
            TermKind::Semicolon(
                Box::new(fallback_term(left)),
                Box::new(fallback_term(right)),
            )
        }
        TermKind::Unreachable => TermKind::Unreachable,
    };

    Term {
        comments: term.comments.clone(),
        kind,
        span: term.span,
        type_: Type::Unit,
    }
}

fn fallback_pattern(pattern: &Pattern<()>) -> Pattern<Type> {
    let kind = match &pattern.kind {
        PatternKind::Hole => PatternKind::Hole,
        PatternKind::Identifier(path) => PatternKind::Identifier(path.clone()),
        PatternKind::ConstConstructor(path) => PatternKind::ConstConstructor(path.clone()),
        PatternKind::Constructor(path, payload) => {
            PatternKind::Constructor(path.clone(), Box::new(fallback_pattern(payload)))
        }
        PatternKind::Tuple(items) => {
            PatternKind::Tuple(items.iter().map(fallback_pattern).collect())
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            PatternKind::Array {
                starting: starting.iter().map(fallback_pattern).collect(),
                glob: glob.clone(),
                ending: ending.iter().map(fallback_pattern).collect(),
            }
        }
        PatternKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            for (name, value) in fields.iter() {
                typed_fields.insert(name.clone(), fallback_pattern(value));
            }
            PatternKind::Struct(typed_fields)
        }
        PatternKind::Immediate(value) => PatternKind::Immediate(value.clone()),
        PatternKind::TypeHint(inner, type_expr) => {
            PatternKind::TypeHint(Box::new(fallback_pattern(inner)), type_expr.clone())
        }
    };

    Pattern {
        comments: pattern.comments.clone(),
        kind,
        span: pattern.span,
        type_: Type::Unit,
    }
}
