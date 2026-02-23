use std::collections::{
    HashMap,
    HashSet,
};

use indexmap::IndexMap;

use crate::ir::{
    Glob,
    Module,
    Path,
    Pattern,
    PatternKind,
    ScopeKind,
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

use super::infer::{
    InferenceContext,
    TypeEnv,
    TypeError,
};
use super::{
    SymbolTable,
    TraitConstraint,
    TraitError,
    TraitRef,
    Type,
    TypeDefinition,
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
    let mut symbols = SymbolTable::new();
    resolve_module_with_symbols(&mut symbols, module, logger)
}

pub fn resolve_module_with_symbols(
    symbols: &mut SymbolTable,
    module: Module<()>,
    logger: &mut FileLogger,
) -> Module<Type> {
    let Module { name, statements } = module;
    let statements = Vec::from(statements);
    let type_entries = collect_type_entries(&statements);
    let duplicate_types = type_entries
        .keys()
        .filter(|path| symbols.type_definitions().contains_key(*path))
        .cloned()
        .collect::<HashSet<_>>();
    for (path, entry) in type_entries.iter() {
        if duplicate_types.contains(path) {
            log_duplicate_definition(logger, entry.def.span(), "type", path);
        }
    }
    let mut term_definitions = collect_term_definitions(&statements);
    term_definitions.extend(collect_constructor_definitions(
        &type_entries,
        &duplicate_types,
    ));
    log_term_duplicates(logger, symbols, &term_definitions);
    let type_definitions =
        build_type_definitions(symbols.type_definitions(), &type_entries, logger);
    let mut env = TypeEnv::new();
    env.extend(
        symbols
            .terms()
            .iter()
            .map(|(path, scheme)| (path.clone(), scheme.clone())),
    );
    let constructors = build_sum_constructors(&type_entries, &type_definitions, logger);
    env.extend(constructors);

    for (path, entry) in type_entries.iter() {
        if symbols.type_definitions().contains_key(path) {
            continue;
        }
        if let Some(definition) = type_definitions.get(path) {
            symbols.insert_type(path.clone(), definition.clone());
        } else {
            let definition = TypeDefinition {
                parameters: entry.parameters.len(),
                body: Type::Unit,
            };
            symbols.insert_type(path.clone(), definition);
        }
    }

    let mut ctx = InferenceContext::new();
    ctx.set_type_definitions(type_definitions.clone());

    let typed_statements = statements
        .into_iter()
        .map(|statement| {
            match statement {
                Statement::Term(term) => {
                    let output = match ctx.infer_term(&mut env, &term) {
                        Ok(output) => output,
                        Err(error) => {
                            log_type_error(logger, term.span, error);
                            return Statement::Term(fallback_term(&term));
                        }
                    };
                    solve_predicates(logger, &mut ctx, symbols, term.span, &output.predicates);
                    Statement::Term(output.term)
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

    for (path, span) in term_definitions.iter() {
        if symbols.terms().contains_key(path) {
            continue;
        }
        match env.get(path).cloned() {
            Some(scheme) => {
                symbols.insert_term(path.clone(), scheme);
            }
            None => {
                logger
                    .error("Missing term definition")
                    .primary(
                        format!("`{}` was not assigned a type.", format_path(path)),
                        *span,
                    )
                    .done();
            }
        }
    }

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

fn collect_term_definitions(statements: &[Statement<()>]) -> Vec<(Path, Span)> {
    let mut definitions = Vec::new();
    for statement in statements {
        let Statement::Term(term) = statement else {
            continue;
        };
        let TermKind::Let {
            assignee,
            scope: ScopeKind::Global,
            ..
        } = &term.kind
        else {
            continue;
        };
        definitions.extend(collect_pattern_bindings(assignee));
    }
    definitions
}

fn collect_constructor_definitions(
    entries: &HashMap<Path, TypeDefEntry>,
    duplicates: &HashSet<Path>,
) -> Vec<(Path, Span)> {
    let mut definitions = Vec::new();
    for (path, entry) in entries.iter() {
        if duplicates.contains(path) {
            continue;
        }
        let TypeDefKind::Sum(variants) = entry.def.kind() else {
            continue;
        };
        for (variant, _) in variants.iter() {
            definitions.push((
                Path::new(path.major.clone(), variant.clone()),
                entry.def.span(),
            ));
        }
    }
    definitions
}

fn collect_pattern_bindings(pattern: &Pattern<()>) -> Vec<(Path, Span)> {
    match &pattern.kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => {
            Vec::new()
        }
        PatternKind::Identifier(path) => vec![(path.clone(), pattern.span)],
        PatternKind::Constructor(_, payload) => collect_pattern_bindings(payload),
        PatternKind::Tuple(items) => items.iter().flat_map(collect_pattern_bindings).collect(),
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let mut bindings = Vec::new();
            bindings.extend(starting.iter().flat_map(collect_pattern_bindings));
            bindings.extend(ending.iter().flat_map(collect_pattern_bindings));
            if let Glob::Named(path) = glob {
                bindings.push((path.clone(), pattern.span));
            }
            bindings
        }
        PatternKind::Struct(fields) => fields.values().flat_map(collect_pattern_bindings).collect(),
        PatternKind::TypeHint(inner, _) => collect_pattern_bindings(inner),
    }
}

fn build_type_definitions(
    base_definitions: &HashMap<Path, TypeDefinition>,
    entries: &HashMap<Path, TypeDefEntry>,
    logger: &mut FileLogger,
) -> HashMap<Path, TypeDefinition> {
    let mut definitions = base_definitions.clone();
    let mut stack = Vec::new();
    for path in entries.keys() {
        let _ = resolve_type_definition(path, entries, &mut definitions, &mut stack, logger);
    }
    definitions
}

fn resolve_type_definition(
    path: &Path,
    entries: &HashMap<Path, TypeDefEntry>,
    type_definitions: &mut HashMap<Path, TypeDefinition>,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> TypeDefinition {
    if let Some(definition) = type_definitions.get(path) {
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
        type_definitions.insert(path.clone(), definition.clone());
        return definition;
    }

    stack.push(path.clone());
    let param_map = param_index_map(&entry.parameters);
    let body = type_def_kind_to_type(
        entry.def.kind(),
        &param_map,
        entries,
        type_definitions,
        stack,
        logger,
    );
    let body = body.for_all(entry.parameters.len());
    let definition = TypeDefinition {
        parameters: entry.parameters.len(),
        body,
    };
    type_definitions.insert(path.clone(), definition.clone());
    stack.pop();
    definition
}

fn type_def_kind_to_type(
    kind: &TypeDefKind,
    param_map: &HashMap<Path, u32>,
    entries: &HashMap<Path, TypeDefEntry>,
    type_definitions: &mut HashMap<Path, TypeDefinition>,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> Type {
    match kind {
        TypeDefKind::Struct(fields) => {
            let mut typed_fields = IndexMap::new();
            for (name, type_expr) in fields.iter() {
                let field_type = type_expr_to_type_in_def(
                    type_expr,
                    param_map,
                    entries,
                    type_definitions,
                    stack,
                    logger,
                );
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
                    type_expr,
                    param_map,
                    entries,
                    type_definitions,
                    stack,
                    logger,
                ));
            }
            Type::Sum {
                variant_names,
                variant_types,
            }
        }
        TypeDefKind::Expr(type_expr) => {
            type_expr_to_type_in_def(
                type_expr,
                param_map,
                entries,
                type_definitions,
                stack,
                logger,
            )
        }
    }
}

fn type_expr_to_type_in_def(
    expr: &TypeExpr,
    param_map: &HashMap<Path, u32>,
    entries: &HashMap<Path, TypeDefEntry>,
    type_definitions: &mut HashMap<Path, TypeDefinition>,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> Type {
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| {
                    type_expr_to_type_in_def(
                        item,
                        param_map,
                        entries,
                        type_definitions,
                        stack,
                        logger,
                    )
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
                return Type::v(*index);
            }

            let arguments = args
                .iter()
                .map(|arg| {
                    type_expr_to_type_in_def(
                        arg,
                        param_map,
                        entries,
                        type_definitions,
                        stack,
                        logger,
                    )
                })
                .collect::<Vec<_>>();

            if path.major == "core" {
                return core_type_from_path(expr.span, path, &arguments, logger);
            }

            let definition = type_definitions.get(path).cloned().or_else(|| {
                entries.contains_key(path).then(|| {
                    resolve_type_definition(path, entries, type_definitions, stack, logger)
                })
            });

            if let Some(definition) = definition {
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
                let base = Type::Named {
                    name: path.clone(),
                    body: Box::new(definition.body),
                };
                return apply_type_constructor(base, arguments);
            }

            let base = Type::Named {
                name: path.clone(),
                body: Box::new(Type::Unit),
            };
            apply_type_constructor(base, arguments)
        }
    }
}

fn build_sum_constructors(
    entries: &HashMap<Path, TypeDefEntry>,
    type_definitions: &HashMap<Path, TypeDefinition>,
    logger: &mut FileLogger,
) -> Vec<(Path, TypeScheme)> {
    let mut constructors = Vec::new();
    let mut type_definitions = type_definitions.clone();
    for (path, entry) in entries.iter() {
        let TypeDefKind::Sum(variants) = entry.def.kind() else {
            continue;
        };

        let param_map = param_index_map(&entry.parameters);
        let definition = type_definitions
            .get(path)
            .cloned()
            .unwrap_or(TypeDefinition {
                parameters: entry.parameters.len(),
                body: Type::Unit,
            });
        let base = Type::Named {
            name: path.clone(),
            body: Box::new(definition.body.clone()),
        };
        let args = type_vars_for_params(entry.parameters.len());
        let result_type = apply_type_constructor(base, args);

        for (variant, type_expr) in variants.iter() {
            let payload_type = type_expr_to_type_in_def(
                type_expr,
                &param_map,
                entries,
                &mut type_definitions,
                &mut Vec::new(),
                logger,
            );
            let constructor_type = if matches!(payload_type, Type::Unit) {
                result_type.clone()
            } else {
                Type::func(payload_type, result_type.clone())
            };
            let scheme_type = constructor_type.for_all(entry.parameters.len());
            constructors.push((
                Path::new(path.major.clone(), variant.clone()),
                scheme_type.scheme(),
            ));
        }
    }
    constructors
}

#[allow(clippy::missing_asserts_for_indexing)]
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
                Type::func(args[0].clone(), args[1].clone())
            } else {
                Type::func(Type::Unit, Type::Unit)
            }
        }
        _ => {
            let base = Type::Named {
                name: Path::new(path.major.clone(), path.minor.clone()),
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
        .map(|index| Type::v((count - 1 - index) as u32))
        .collect()
}

fn log_term_duplicates(
    logger: &mut FileLogger,
    symbols: &SymbolTable,
    definitions: &[(Path, Span)],
) {
    let mut seen = HashSet::new();
    for (path, span) in definitions.iter() {
        if !seen.insert(path.clone()) {
            continue;
        }
        if symbols.terms().contains_key(path) {
            log_duplicate_definition(logger, *span, "term", path);
        }
    }
}

fn log_duplicate_definition(
    logger: &mut FileLogger,
    span: Span,
    kind: &str,
    path: &Path,
) {
    logger
        .error(format!("Duplicate {kind} definition"))
        .primary(format!("`{}` is already defined.", format_path(path)), span)
        .done();
}

fn format_path(path: &Path) -> String {
    path.to_string()
}

fn format_trait_ref(trait_ref: &TraitRef) -> String {
    if trait_ref.arguments.is_empty() {
        trait_ref.trait_name.to_string()
    } else {
        let args = trait_ref
            .arguments
            .iter()
            .map(Type::pretty)
            .collect::<Vec<_>>()
            .join(" ");
        format!("{} {args}", trait_ref.trait_name)
    }
}

fn solve_predicates(
    logger: &mut FileLogger,
    ctx: &mut InferenceContext,
    symbols: &SymbolTable,
    span: Span,
    predicates: &[TraitConstraint],
) {
    if predicates.is_empty() {
        return;
    }
    match symbols.resolve_predicates(ctx.table_mut(), predicates) {
        Ok(unresolved) => {
            for predicate in unresolved {
                logger
                    .error("Unresolved trait constraint")
                    .primary(
                        format!("`{}` is required here.", format_trait_ref(&predicate)),
                        span,
                    )
                    .done();
            }
        }
        Err(error) => log_trait_error(logger, span, error),
    }
}

fn log_trait_error(
    logger: &mut FileLogger,
    span: Span,
    error: TraitError,
) {
    match error {
        TraitError::UnknownTrait(path) => {
            logger
                .error("Unknown trait")
                .primary(format!("`{}` is not defined.", format_path(&path)), span)
                .done();
        }
        TraitError::DuplicateTrait(path) => {
            logger
                .error("Duplicate trait definition")
                .primary(
                    format!("`{}` is already defined.", format_path(&path)),
                    span,
                )
                .done();
        }
        TraitError::ArityMismatch {
            trait_name,
            expected,
            found,
        } => {
            logger
                .error("Invalid trait application")
                .primary(
                    format!(
                        "`{}` expects {expected} type arguments but got {found}.",
                        format_path(&trait_name)
                    ),
                    span,
                )
                .done();
        }
        TraitError::MissingMethod { trait_name, method } => {
            logger
                .error("Missing trait method")
                .primary(
                    format!(
                        "`{}` is missing method `{}`.",
                        format_path(&trait_name),
                        format_path(&method)
                    ),
                    span,
                )
                .done();
        }
        TraitError::ExtraMethod { trait_name, method } => {
            logger
                .error("Unknown trait method")
                .primary(
                    format!(
                        "`{}` does not declare method `{}`.",
                        format_path(&trait_name),
                        format_path(&method)
                    ),
                    span,
                )
                .done();
        }
        TraitError::MethodTypeMismatch {
            method,
            expected,
            found,
            ..
        } => {
            logger
                .error("Trait method type mismatch")
                .primary(
                    format!(
                        "`{}` expects `{}` but got `{}`.",
                        format_path(&method),
                        expected.type_.pretty(),
                        found.type_.pretty()
                    ),
                    span,
                )
                .done();
        }
        TraitError::OverlappingInstance { trait_name, .. } => {
            logger
                .error("Overlapping trait instance")
                .primary(
                    format!("Instances for `{}` overlap.", format_path(&trait_name)),
                    span,
                )
                .done();
        }
        TraitError::AmbiguousInstance { predicate } => {
            logger
                .error("Ambiguous trait instance")
                .primary(
                    format!(
                        "Multiple instances match `{}`.",
                        format_trait_ref(&predicate)
                    ),
                    span,
                )
                .done();
        }
        TraitError::RecursivePredicate { predicate } => {
            logger
                .error("Recursive trait constraint")
                .primary(
                    format!("`{}` depends on itself.", format_trait_ref(&predicate)),
                    span,
                )
                .done();
        }
        TraitError::InvalidInstance { trait_name } => {
            logger
                .error("Invalid trait instance")
                .primary(
                    format!("Instance for `{}` is invalid.", format_path(&trait_name)),
                    span,
                )
                .done();
        }
        TraitError::NoInstance { predicate } => {
            logger
                .error("Missing trait instance")
                .primary(
                    format!("No instance found for `{}`.", format_trait_ref(&predicate)),
                    span,
                )
                .done();
        }
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
