use std::collections::{
    HashMap,
    HashSet,
};

use crate::logging::WithContext;
use indexmap::IndexMap;

use super::super::type_expr::{
    TypeExprSymbol,
    lower_type_expr,
};
use super::diagnostics::log_type_expr_lower_error;
use super::{
    FileLogger,
    Glob,
    Path,
    Pattern,
    PatternKind,
    ScopeKind,
    Span,
    Statement,
    TermKind,
    Type,
    TypeDeclKind,
    TypeDefEntry,
    TypeDefKind,
    TypeDefinition,
    TypeDefinitionKind,
    TypeExpr,
    TypeScheme,
};

pub(super) fn collect_type_entries(statements: &[Statement<()>]) -> IndexMap<Path, TypeDefEntry> {
    let mut entries = IndexMap::new();
    for statement in statements {
        let Statement::Type {
            path,
            parameters,
            def,
            kind,
            ..
        } = statement
        else {
            continue;
        };
        entries.entry(path.clone()).or_insert_with(|| {
            TypeDefEntry {
                kind: type_definition_kind_from_decl_kind(*kind),
                parameters: parameters.to_vec(),
                def: def.clone(),
            }
        });
    }
    entries
}

pub(super) fn collect_term_definitions(statements: &[Statement<()>]) -> Vec<(Path, Span)> {
    let mut definitions = Vec::new();
    for statement in statements {
        match statement {
            Statement::Term(term) => {
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
            Statement::Trait { methods, .. } => {
                definitions.extend(
                    methods
                        .iter()
                        .map(|method| (method.path.clone(), method.span)),
                );
            }
            Statement::Impl { methods, .. } => {
                definitions.extend(
                    methods
                        .iter()
                        .map(|method| (method.impl_path.clone(), method.span)),
                );
            }
            Statement::Type { .. } | Statement::Wasm(_) => {}
        }
    }
    definitions
}

pub(super) fn collect_constructor_definitions(
    entries: &IndexMap<Path, TypeDefEntry>,
    duplicates: &HashSet<Path>,
) -> Vec<(Path, Span)> {
    entries
        .iter()
        .filter(|(path, _)| !duplicates.contains(*path))
        .filter_map(|(path, entry)| {
            let TypeDefKind::Sum(variants) = entry.def.kind() else {
                return None;
            };
            Some((path, variants, entry.def.span()))
        })
        .flat_map(|(path, variants, span)| {
            variants
                .iter()
                .map(move |(variant, _)| (Path::new(path.major.clone(), variant.clone()), span))
        })
        .collect()
}

pub(super) fn build_type_definitions(
    base_definitions: &IndexMap<Path, TypeDefinition>,
    entries: &IndexMap<Path, TypeDefEntry>,
    logger: &mut FileLogger,
) -> IndexMap<Path, TypeDefinition> {
    let mut definitions = base_definitions.clone();
    let mut stack = Vec::new();
    for path in entries.keys() {
        let _ = resolve_type_definition(path, entries, &mut definitions, &mut stack, logger);
    }
    definitions
}

pub(super) fn build_sum_constructors(
    entries: &IndexMap<Path, TypeDefEntry>,
    type_definitions: &IndexMap<Path, TypeDefinition>,
    logger: &mut FileLogger,
) -> Box<[(Path, TypeScheme)]> {
    let mut constructors = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut type_definitions = type_definitions.clone();
    for (path, entry) in entries.iter() {
        if entry.kind != TypeDefinitionKind::Named {
            continue;
        }
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
                kind: TypeDefinitionKind::Named,
            });
        let base = Type::Named {
            name: path.clone(),
            body: Box::new(definition.body.clone()),
        };
        let args = type_vars_for_params(entry.parameters.len());
        let result_type = base.apply(args);

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
            let constructor_path = Path::new(path.major.clone(), variant.clone());
            if seen_paths.insert(constructor_path.clone()) {
                constructors.push((constructor_path, scheme_type.scheme()));
            }
        }
    }
    constructors.into_boxed_slice()
}

pub(super) fn type_expr_to_type_in_def(
    expr: &TypeExpr,
    param_map: &HashMap<Path, u32>,
    entries: &IndexMap<Path, TypeDefEntry>,
    type_definitions: &mut IndexMap<Path, TypeDefinition>,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> Type {
    TypeDefTypeExprLowering {
        param_map,
        entries,
        type_definitions,
        stack,
        logger,
    }
    .lower(expr)
}

pub(super) fn param_index_map(parameters: &[Path]) -> HashMap<Path, u32> {
    let count = parameters.len();
    parameters
        .iter()
        .enumerate()
        .map(|(index, path)| (path.clone(), (count - 1 - index) as u32))
        .collect()
}

pub(super) fn type_vars_for_params(count: usize) -> Vec<Type> {
    (0..count)
        .map(|index| Type::v((count - 1 - index) as u32))
        .collect()
}

fn type_definition_kind_from_decl_kind(kind: TypeDeclKind) -> TypeDefinitionKind {
    match kind {
        TypeDeclKind::Named => TypeDefinitionKind::Named,
        TypeDeclKind::Alias => TypeDefinitionKind::Alias,
    }
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

fn resolve_type_definition(
    path: &Path,
    entries: &IndexMap<Path, TypeDefEntry>,
    type_definitions: &mut IndexMap<Path, TypeDefinition>,
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
            kind: TypeDefinitionKind::Named,
        };
    };
    if let Some(cycle_start) = stack.iter().position(|candidate| candidate == path) {
        let cycle = &stack[cycle_start..];
        if recursive_cycle_allowed(cycle, entries) {
            let definition = TypeDefinition {
                parameters: entry.parameters.len(),
                body: Type::Unit,
                kind: entry.kind,
            };
            type_definitions.insert(path.clone(), definition.clone());
            return definition;
        }
        log_invalid_recursive_cycle(logger, cycle, entries);
        let definition = TypeDefinition {
            parameters: entry.parameters.len(),
            body: Type::Unit,
            kind: entry.kind,
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
        kind: entry.kind,
    };
    type_definitions.insert(path.clone(), definition.clone());
    stack.pop();
    definition
}

fn recursive_cycle_allowed(
    cycle: &[Path],
    entries: &IndexMap<Path, TypeDefEntry>,
) -> bool {
    cycle.iter().all(|path| {
        let Some(entry) = entries.get(path) else {
            return false;
        };
        entry.kind == TypeDefinitionKind::Named && matches!(entry.def.kind(), TypeDefKind::Sum(_))
    })
}

fn log_invalid_recursive_cycle(
    logger: &mut FileLogger,
    cycle: &[Path],
    entries: &IndexMap<Path, TypeDefEntry>,
) {
    let cycle_text = format_recursive_cycle(cycle);
    if let Some(path) = cycle.iter().find(|path| {
        entries
            .get(*path)
            .is_some_and(|entry| entry.kind == TypeDefinitionKind::Alias)
    }) {
        let span = entries
            .get(path)
            .map(|entry| entry.def.span())
            .unwrap_or(Span::Generated);
        logger
            .error("Recursive type aliases are not allowed")
            .primary(
                format!("`{path}` is part of recursive cycle `{cycle_text}`."),
                span,
            )
            .done();
        return;
    }

    if let Some(path) = cycle.iter().find(|path| {
        entries
            .get(*path)
            .is_some_and(|entry| !matches!(entry.def.kind(), TypeDefKind::Sum(_)))
    }) {
        let span = entries
            .get(path)
            .map(|entry| entry.def.span())
            .unwrap_or(Span::Generated);
        logger
            .error("Invalid recursive type definition")
            .primary(
                format!(
                    "`{path}` is part of recursive cycle `{cycle_text}`. Only sum type definitions may be recursive."
                ),
                span,
            )
            .done();
        return;
    }

    if let Some(path) = cycle.first() {
        let span = entries
            .get(path)
            .map(|entry| entry.def.span())
            .unwrap_or(Span::Generated);
        logger
            .error("Invalid recursive type definition")
            .primary(
                format!("Recursive cycle `{cycle_text}` is not supported."),
                span,
            )
            .done();
    }
}

fn format_recursive_cycle(cycle: &[Path]) -> String {
    let mut names = cycle.iter().map(ToString::to_string).collect::<Vec<_>>();
    if let Some(first) = cycle.first() {
        names.push(first.to_string());
    }
    names.join(" -> ")
}

fn type_def_kind_to_type(
    kind: &TypeDefKind,
    param_map: &HashMap<Path, u32>,
    entries: &IndexMap<Path, TypeDefEntry>,
    type_definitions: &mut IndexMap<Path, TypeDefinition>,
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
            let mut typed_variants = IndexMap::new();
            for (name, type_expr) in variants.iter() {
                let variant_type = type_expr_to_type_in_def(
                    type_expr,
                    param_map,
                    entries,
                    type_definitions,
                    stack,
                    logger,
                );
                typed_variants.insert(name.clone(), variant_type);
            }
            Type::Sum {
                variants: typed_variants,
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

struct TypeDefTypeExprLowering<'a> {
    param_map: &'a HashMap<Path, u32>,
    entries: &'a IndexMap<Path, TypeDefEntry>,
    type_definitions: &'a mut IndexMap<Path, TypeDefinition>,
    stack: &'a mut Vec<Path>,
    logger: &'a mut FileLogger,
}

impl TypeDefTypeExprLowering<'_> {
    fn lower(
        &mut self,
        expr: &TypeExpr,
    ) -> Type {
        let lowered = lower_type_expr(expr, &mut |path| self.symbol_for_path(path), &mut |_| None);
        lowered
            .errors
            .into_iter()
            .for_each(|error| log_type_expr_lower_error(self.logger, error));
        lowered.type_
    }

    fn symbol_for_path(
        &mut self,
        path: &Path,
    ) -> TypeExprSymbol {
        if let Some(index) = self.param_map.get(path).copied() {
            return TypeExprSymbol::TypeParameter(index);
        }

        self.type_definitions
            .get(path)
            .cloned()
            .or_else(|| {
                self.entries.contains_key(path).then(|| {
                    resolve_type_definition(
                        path,
                        self.entries,
                        self.type_definitions,
                        self.stack,
                        self.logger,
                    )
                })
            })
            .map(TypeExprSymbol::Definition)
            .unwrap_or(TypeExprSymbol::Unknown)
    }
}
