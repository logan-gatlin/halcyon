//! Type-definition collection and lowering for resolve.

use std::collections::{
    HashMap,
    HashSet,
};

use crate::logging::WithContext;
use indexmap::IndexMap;

use super::super::type_expr::{
    TypeExprSymbol,
    lower_type_expr,
    lower_type_scheme_expr,
};
use super::diagnostics::log_type_expr_lower_error;
use super::{
    FileLogger,
    Glob,
    Path,
    Pattern,
    PatternKind,
    PendingTypeDefinitionEntry,
    ScopeKind,
    Span,
    Statement,
    TermKind,
    Type,
    TypeDeclKind,
    TypeDefKind,
    TypeDefinition,
    TypeDefinitionKind,
    TypeExpr,
    TypeScheme,
};

/// Collect pending type declarations from module statements.
pub(super) fn collect_type_entries(
    statements: &[Statement<()>]
) -> IndexMap<Path, PendingTypeDefinitionEntry> {
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
            PendingTypeDefinitionEntry {
                kind: type_definition_kind_from_decl_kind(*kind),
                parameters: parameters.to_vec(),
                syntax: def.clone(),
            }
        });
    }
    entries
}

/// Collect top-level term paths that should be published after inference.
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

/// Collect constructor symbols contributed by non-duplicate sum types.
pub(super) fn collect_constructor_definitions(
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
    duplicates: &HashSet<Path>,
) -> Vec<(Path, Span)> {
    entries
        .iter()
        .filter(|(path, _)| !duplicates.contains(*path))
        .filter_map(|(path, entry)| {
            let TypeDefKind::Sum(variants) = entry.syntax.kind() else {
                return None;
            };
            Some((path, variants, entry.syntax.span()))
        })
        .flat_map(|(path, variants, span)| {
            variants
                .iter()
                .map(move |(variant, _)| (path.sibling(variant), span))
        })
        .collect()
}

/// Build resolved type-definition bodies for all pending declarations.
pub(super) fn build_type_definitions(
    base_definitions: &IndexMap<Path, TypeDefinition>,
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
    logger: &mut FileLogger,
) -> IndexMap<Path, TypeDefinition> {
    let mut definitions = base_definitions.clone();
    let mut stack = Vec::new();
    for path in entries.keys() {
        let _ = resolve_type_definition(path, entries, &mut definitions, &mut stack, logger);
    }
    definitions
}

/// Build constructor schemes for every resolved named sum type.
pub(super) fn build_sum_constructors(
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
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
        let TypeDefKind::Sum(variants) = entry.syntax.kind() else {
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
            let constructor_path = path.sibling(variant);
            if seen_paths.insert(constructor_path.clone()) {
                constructors.push((constructor_path, scheme_type.scheme()));
            }
        }
    }
    constructors.into_boxed_slice()
}

/// Lower a type expression in type-definition context.
pub(super) fn type_expr_to_type_in_def(
    expr: &TypeExpr,
    param_map: &HashMap<Path, u32>,
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
    type_definitions: &mut IndexMap<Path, TypeDefinition>,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> Type {
    TypeDefinitionExprLowerer {
        param_map,
        entries,
        type_definitions,
        stack,
        logger,
    }
    .lower(expr)
}

/// Lower a type expression to a scheme in type-definition context.
pub(super) fn type_expr_to_scheme_in_def(
    expr: &TypeExpr,
    param_map: &HashMap<Path, u32>,
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
    type_definitions: &mut IndexMap<Path, TypeDefinition>,
    stack: &mut Vec<Path>,
    logger: &mut FileLogger,
) -> TypeScheme {
    TypeDefinitionExprLowerer {
        param_map,
        entries,
        type_definitions,
        stack,
        logger,
    }
    .lower_scheme(expr)
}

/// Map type parameter paths to De Bruijn indices used in this definition.
pub(super) fn param_index_map(parameters: &[Path]) -> HashMap<Path, u32> {
    let count = parameters.len();
    parameters
        .iter()
        .enumerate()
        .map(|(index, path)| (path.clone(), (count - 1 - index) as u32))
        .collect()
}

/// Build type variables corresponding to `count` parameters.
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
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
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
        entry.syntax.kind(),
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
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
) -> bool {
    cycle.iter().all(|path| {
        let Some(entry) = entries.get(path) else {
            return false;
        };
        entry.kind == TypeDefinitionKind::Named
            && matches!(entry.syntax.kind(), TypeDefKind::Sum(_))
    })
}

fn log_invalid_recursive_cycle(
    logger: &mut FileLogger,
    cycle: &[Path],
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
) {
    let cycle_text = format_recursive_cycle(cycle);
    if let Some(path) = cycle.iter().find(|path| {
        entries
            .get(*path)
            .is_some_and(|entry| entry.kind == TypeDefinitionKind::Alias)
    }) {
        let span = entries
            .get(path)
            .map(|entry| entry.syntax.span())
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
            .is_some_and(|entry| !matches!(entry.syntax.kind(), TypeDefKind::Sum(_)))
    }) {
        let span = entries
            .get(path)
            .map(|entry| entry.syntax.span())
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
            .map(|entry| entry.syntax.span())
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
    entries: &IndexMap<Path, PendingTypeDefinitionEntry>,
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

struct TypeDefinitionExprLowerer<'a> {
    param_map: &'a HashMap<Path, u32>,
    entries: &'a IndexMap<Path, PendingTypeDefinitionEntry>,
    type_definitions: &'a mut IndexMap<Path, TypeDefinition>,
    stack: &'a mut Vec<Path>,
    logger: &'a mut FileLogger,
}

impl TypeDefinitionExprLowerer<'_> {
    fn lower(
        &mut self,
        expr: &TypeExpr,
    ) -> Type {
        let lowered = lower_type_expr(
            expr,
            &mut |path| self.lookup_type_expr_symbol(path),
            &mut |_| None,
        );
        lowered
            .errors
            .into_iter()
            .for_each(|error| log_type_expr_lower_error(self.logger, error));
        lowered.type_
    }

    fn lower_scheme(
        &mut self,
        expr: &TypeExpr,
    ) -> TypeScheme {
        let lowered = lower_type_scheme_expr(
            expr,
            &mut |path| self.lookup_type_expr_symbol(path),
            &mut |_| None,
        );
        lowered
            .errors
            .into_iter()
            .for_each(|error| log_type_expr_lower_error(self.logger, error));
        lowered.scheme
    }

    fn lookup_type_expr_symbol(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        TypeExprConstraint,
        TypeExprKind,
    };
    use crate::types::TraitRef;
    use crate::{
        Logger,
        ir,
        parse,
    };

    fn parse_module_statements(source: &str) -> Vec<Statement<()>> {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", source);
        let module = parse::parse(source, &mut file_logger)
            .and_then(|source_file| source_file.modules().into_iter().next())
            .and_then(|module| ir::module(module, &mut file_logger))
            .expect("source should lower to module");
        module.statements.into_vec()
    }

    fn type_expr(kind: TypeExprKind) -> TypeExpr {
        TypeExpr {
            comments: String::new(),
            kind,
            span: Span::Generated,
        }
    }

    #[test]
    fn collect_type_entries_keeps_first_definition_per_path() {
        let statements = parse_module_statements(
            "module demo =\n  type Token = { value: core::integer }\n  type Token = { value: core::boolean }\nend\n",
        );
        let entries = collect_type_entries(&statements);

        assert_eq!(entries.len(), 1);
        let entry = entries
            .get(&Path::new("demo", "Token"))
            .expect("entry should exist");
        assert_eq!(entry.kind, TypeDefinitionKind::Named);
    }

    #[test]
    fn collect_term_and_constructor_definitions_include_all_sources() {
        let statements = parse_module_statements(
            "module demo =\n  type Option: a = | Some a | None\n  trait Eq : a =\n    let eq : a -> a -> core::boolean\n  end\n  impl Eq : core::integer =\n    let eq = fn x => x\n  end\n  let value = 1\nend\n",
        );
        let entries = collect_type_entries(&statements);
        let duplicates = HashSet::new();

        let term_defs = collect_term_definitions(&statements);
        let constructors = collect_constructor_definitions(&entries, &duplicates);

        assert!(
            term_defs
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "value"))
        );
        assert!(
            term_defs
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "eq"))
        );
        assert!(
            constructors
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "Some"))
        );
        assert!(
            constructors
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "None"))
        );
    }

    #[test]
    fn param_index_map_and_type_vars_follow_debruijn_ordering() {
        let parameters = vec![Path::new("demo", "a"), Path::new("demo", "b")];
        let index_map = param_index_map(&parameters);
        assert_eq!(index_map.get(&Path::new("demo", "a")), Some(&1));
        assert_eq!(index_map.get(&Path::new("demo", "b")), Some(&0));

        assert_eq!(type_vars_for_params(2), vec![Type::v(1), Type::v(0)]);
    }

    #[test]
    fn build_type_definitions_handles_recursive_rules() {
        let mut logger = Logger::new();

        let alias_statements = parse_module_statements("module demo =\n  type ~Loop = Loop\nend\n");
        let alias_entries = collect_type_entries(&alias_statements);
        let mut alias_file = logger.new_file("alias.hc", "");
        let alias_defs = build_type_definitions(&IndexMap::new(), &alias_entries, &mut alias_file);
        assert_eq!(
            alias_defs
                .get(&Path::new("demo", "Loop"))
                .expect("alias definition should exist")
                .body,
            Type::Unit
        );

        let sum_statements =
            parse_module_statements("module demo =\n  type List = | Nil | Cons List\nend\n");
        let sum_entries = collect_type_entries(&sum_statements);
        let mut sum_file = logger.new_file("sum.hc", "");
        let sum_defs = build_type_definitions(&IndexMap::new(), &sum_entries, &mut sum_file);
        let list_def = sum_defs
            .get(&Path::new("demo", "List"))
            .expect("sum definition should exist");
        assert_eq!(list_def.kind, TypeDefinitionKind::Named);
        assert!(matches!(list_def.body, Type::Sum { .. }));
    }

    #[test]
    fn build_sum_constructors_generates_constructor_schemes() {
        let statements =
            parse_module_statements("module demo =\n  type Option: a = | Some a | None\nend\n");
        let entries = collect_type_entries(&statements);
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let mut defs = [
            (Path::core("function"), Type::function().def(2)),
            (Path::core("array"), Type::array().def(1)),
        ]
        .into_iter()
        .collect::<IndexMap<_, _>>();
        defs.extend(build_type_definitions(&defs, &entries, &mut file_logger));

        let constructors = build_sum_constructors(&entries, &defs, &mut file_logger);
        assert!(
            constructors
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "Some"))
        );
        assert!(
            constructors
                .iter()
                .any(|(path, _)| path == &Path::new("demo", "None"))
        );
    }

    #[test]
    fn type_expr_to_type_in_def_and_scheme_in_def_apply_recovery_and_constraints() {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");

        let lowered = type_expr_to_type_in_def(
            &type_expr(TypeExprKind::Placeholder),
            &HashMap::new(),
            &IndexMap::new(),
            &mut IndexMap::new(),
            &mut Vec::new(),
            &mut file_logger,
        );
        assert_eq!(lowered, Type::Unit);

        let a = Path::new("demo", "a");
        let scheme = type_expr_to_scheme_in_def(
            &TypeExpr {
                comments: String::new(),
                kind: TypeExprKind::ForAll(
                    [a.clone()].into(),
                    [TypeExprConstraint {
                        trait_name: Path::new("demo", "Eq"),
                        arguments: [type_expr(TypeExprKind::Instantiation(a.clone(), [].into()))]
                            .into(),
                        span: Span::Generated,
                    }]
                    .into(),
                    Box::new(type_expr(TypeExprKind::Instantiation(a.clone(), [].into()))),
                ),
                span: Span::Generated,
            },
            &HashMap::new(),
            &IndexMap::new(),
            &mut IndexMap::new(),
            &mut Vec::new(),
            &mut file_logger,
        );

        assert_eq!(scheme.type_, Type::v(0).for_all(1));
        assert_eq!(
            scheme.predicates,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])]
        );
    }
}
