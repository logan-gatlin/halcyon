//! Resolve phase orchestration.
//!
//! This module wires together:
//! - type-definition collection/lowering,
//! - trait definition/implementation registration,
//! - term inference and predicate solving,
//! - final symbol-table publication.

use std::collections::HashSet;

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
    TypeDeclKind,
    TypeDefKind,
    TypeExpr,
};
use crate::logging::WithContext;
use crate::{
    FileLogger,
    Span,
};

use super::infer::{
    InferenceContext,
    TypeEnv,
};
use super::{
    SymbolTable,
    TraitConstraint,
    TraitDef,
    TraitError,
    TraitImpl,
    TraitRef,
    Type,
    TypeDefinition,
    TypeDefinitionKind,
    TypeScheme,
    for_each_child_type,
};

mod common;
mod diagnostics;
mod impls;
mod recovery;
mod traits;
mod type_defs;

use common::predicate_is_ground;
use diagnostics::{
    log_duplicate_definition,
    log_term_duplicates,
    log_type_error,
};
use impls::ImplProcessingContext;
use recovery::{
    fallback_term,
    normalize_term_types,
};
use traits::{
    build_trait_definitions,
    register_trait_definitions,
    solve_predicates,
};
use type_defs::{
    build_sum_constructors,
    build_type_definitions,
    collect_constructor_definitions,
    collect_term_definitions,
    collect_type_entries,
};

#[derive(Debug, Clone)]
/// Intermediate representation of a type declaration before full resolution.
struct PendingTypeDefinitionEntry {
    kind: TypeDefinitionKind,
    parameters: Vec<Path>,
    syntax: crate::ir::TypeDef,
}

#[derive(Debug, Clone)]
/// Intermediate representation of a trait declaration before registration.
struct PendingTraitDefinitionEntry {
    span: Span,
    trait_definition: TraitDef,
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Fully typed module produced by resolve.
    pub module: Module<Type>,

    /// Final inferred/generalized schemes for exported bindings.
    pub schemes: IndexMap<Path, TypeScheme>,
}

/// Resolve a module with a fresh symbol table.
pub fn resolve_module(
    module: Module<()>,
    logger: &mut FileLogger,
) -> Module<Type> {
    let mut symbols = SymbolTable::new();
    resolve_module_with_symbols(&mut symbols, module, logger)
}

/// Resolve a module using an existing symbol table.
pub fn resolve_module_with_symbols(
    symbols: &mut SymbolTable,
    module: Module<()>,
    logger: &mut FileLogger,
) -> Module<Type> {
    resolve_module_with_symbols_and_schemes(symbols, module, logger).module
}

/// Resolve a module and return both typed IR and finalized binding schemes.
pub fn resolve_module_with_symbols_and_schemes(
    symbols: &mut SymbolTable,
    module: Module<()>,
    logger: &mut FileLogger,
) -> ResolvedModule {
    let Module { name, statements } = module;
    let statements = Vec::from(statements);
    let pending_type_definitions = collect_type_entries(&statements);
    let duplicate_type_paths = pending_type_definitions
        .keys()
        .filter(|path| symbols.type_definitions().contains_key(*path))
        .cloned()
        .collect::<HashSet<_>>();
    for (path, entry) in pending_type_definitions.iter() {
        if duplicate_type_paths.contains(path) {
            log_duplicate_definition(logger, entry.syntax.span(), "type", path);
        }
    }

    let mut pending_term_definitions = collect_term_definitions(&statements);
    pending_term_definitions.extend(collect_constructor_definitions(
        &pending_type_definitions,
        &duplicate_type_paths,
    ));
    log_term_duplicates(logger, symbols, &pending_term_definitions);

    let type_definitions = build_type_definitions(
        symbols.type_definitions(),
        &pending_type_definitions,
        logger,
    );
    for (path, entry) in pending_type_definitions.iter() {
        if symbols.type_definitions().contains_key(path) {
            continue;
        }
        if let Some(definition) = type_definitions.get(path) {
            symbols.insert_type(path.clone(), definition.clone());
        } else {
            symbols.insert_type(
                path.clone(),
                TypeDefinition {
                    parameters: entry.parameters.len(),
                    body: Type::Unit,
                    kind: entry.kind,
                },
            );
        }
    }

    let pending_trait_definitions = build_trait_definitions(
        &statements,
        &pending_type_definitions,
        &type_definitions,
        logger,
    );
    register_trait_definitions(symbols, &pending_trait_definitions, logger);

    let mut type_environment = TypeEnv::new();
    type_environment.extend(
        symbols
            .terms()
            .iter()
            .map(|(path, scheme)| (path.clone(), scheme.clone())),
    );
    let constructors = build_sum_constructors(&pending_type_definitions, &type_definitions, logger);
    type_environment.extend(constructors);

    let mut inference_context = InferenceContext::new();
    let mut schemes = IndexMap::new();
    let mut failed_term_paths = HashSet::new();
    inference_context.set_type_definitions(
        type_definitions
            .iter()
            .map(|(path, def)| (path.clone(), def.clone()))
            .collect::<IndexMap<_, _>>(),
    );

    let mut typed_statements = Vec::new();
    for statement in statements.into_iter() {
        match statement {
            Statement::Term(term) => {
                let known_scheme_paths = schemes.keys().cloned().collect::<HashSet<_>>();
                let output = match inference_context.infer_term(
                    &mut type_environment,
                    &term,
                    &mut schemes,
                ) {
                    Ok(output) => output,
                    Err(error) => {
                        log_type_error(logger, error);
                        if let TermKind::Let {
                            assignee,
                            scope: ScopeKind::Global,
                            ..
                        } = &term.kind
                        {
                            failed_term_paths.extend(pattern_binding_paths(assignee));
                        }
                        typed_statements.push(Statement::Term(fallback_term(&term)));
                        continue;
                    }
                };

                solve_predicates(
                    logger,
                    &mut inference_context,
                    symbols,
                    term.span,
                    &output.predicates,
                );
                let mut grounded_predicates = Vec::new();
                for (path, scheme) in schemes.iter() {
                    if known_scheme_paths.contains(path) {
                        continue;
                    }
                    for predicate in scheme.predicates.iter() {
                        if predicate_is_ground(predicate)
                            && !grounded_predicates.contains(predicate)
                        {
                            grounded_predicates.push(predicate.clone());
                        }
                    }
                }
                solve_predicates(
                    logger,
                    &mut inference_context,
                    symbols,
                    term.span,
                    &grounded_predicates,
                );
                typed_statements.push(Statement::Term(normalize_term_types(
                    output.term,
                    inference_context.table_mut(),
                )));
            }
            Statement::Type {
                comments,
                path,
                parameters,
                def,
                kind,
            } => {
                typed_statements.push(Statement::Type {
                    comments,
                    path,
                    parameters,
                    def,
                    kind,
                });
            }
            Statement::Trait {
                comments,
                path,
                parameters,
                methods,
            } => {
                typed_statements.push(Statement::Trait {
                    comments,
                    path,
                    parameters,
                    methods,
                });
            }
            Statement::Impl {
                comments,
                trait_path,
                arguments,
                methods,
            } => {
                let (typed_impl, generated_terms) = ImplProcessingContext {
                    module_name: &name,
                    logger,
                    inference_context: &mut inference_context,
                    type_environment: &mut type_environment,
                    symbols,
                    schemes: &mut schemes,
                    pending_type_definitions: &pending_type_definitions,
                    type_definitions: &type_definitions,
                }
                .process(comments, trait_path, arguments, methods);
                typed_statements.push(typed_impl);
                typed_statements.extend(generated_terms.into_iter().map(Statement::Term));
            }
            Statement::Wasm(sexpr) => typed_statements.push(Statement::Wasm(sexpr)),
        }
    }

    for (path, span) in pending_term_definitions.iter() {
        if symbols.terms().contains_key(path) {
            continue;
        }
        if failed_term_paths.contains(path) {
            continue;
        }
        match type_environment.get(path).cloned() {
            Some(scheme) => {
                symbols.insert_term(path.clone(), scheme);
            }
            None => {
                logger
                    .error("Missing term definition")
                    .primary(format!("`{path}` was not assigned a type."), *span)
                    .done();
            }
        }
    }

    ResolvedModule {
        module: Module {
            name,
            statements: typed_statements.into_boxed_slice(),
        },
        schemes,
    }
}

fn pattern_binding_paths<T>(pattern: &Pattern<T>) -> Vec<Path> {
    match &pattern.kind {
        PatternKind::Identifier(path) => vec![path.clone()],
        PatternKind::Constructor(_, payload) => pattern_binding_paths(payload),
        PatternKind::Tuple(items) => items.iter().flat_map(pattern_binding_paths).collect(),
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let mut paths = starting
                .iter()
                .chain(ending.iter())
                .flat_map(pattern_binding_paths)
                .collect::<Vec<_>>();
            if let Glob::Named(path) = glob {
                paths.push(path.clone());
            }
            paths
        }
        PatternKind::Struct(fields) => fields.values().flat_map(pattern_binding_paths).collect(),
        PatternKind::TypeHint(inner, _) => pattern_binding_paths(inner),
        PatternKind::Hole | PatternKind::ConstConstructor(_) | PatternKind::Immediate(_) => {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hc_core::compile_core_module;
    use crate::{
        Logger,
        ir,
        parse,
    };

    fn resolve_source(
        source: &str,
        symbols: &mut SymbolTable,
    ) -> (ResolvedModule, crate::FileLogger) {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", source);
        let module = parse::parse(source, &mut file_logger)
            .and_then(|source_file| source_file.modules().into_iter().next())
            .and_then(|module| ir::module(module, &mut file_logger))
            .expect("source should lower to IR module");
        let resolved = resolve_module_with_symbols_and_schemes(symbols, module, &mut file_logger);
        (resolved, file_logger)
    }

    #[test]
    fn pattern_binding_paths_collect_nested_bindings() {
        let pattern = Pattern {
            comments: String::new(),
            kind: PatternKind::Tuple(
                [
                    Pattern {
                        comments: String::new(),
                        kind: PatternKind::Identifier(Path::new("demo", "a")),
                        span: Span::Generated,
                        type_: (),
                    },
                    Pattern {
                        comments: String::new(),
                        kind: PatternKind::Array {
                            starting: [Pattern {
                                comments: String::new(),
                                kind: PatternKind::Identifier(Path::new("demo", "b")),
                                span: Span::Generated,
                                type_: (),
                            }]
                            .into(),
                            glob: Glob::Named(Path::new("demo", "rest")),
                            ending: [Pattern {
                                comments: String::new(),
                                kind: PatternKind::Identifier(Path::new("demo", "c")),
                                span: Span::Generated,
                                type_: (),
                            }]
                            .into(),
                        },
                        span: Span::Generated,
                        type_: (),
                    },
                ]
                .into(),
            ),
            span: Span::Generated,
            type_: (),
        };

        let mut paths = pattern_binding_paths(&pattern);
        paths.sort_by_key(ToString::to_string);
        assert_eq!(
            paths,
            vec![
                Path::new("demo", "a"),
                Path::new("demo", "b"),
                Path::new("demo", "c"),
                Path::new("demo", "rest"),
            ]
        );
    }

    #[test]
    fn resolve_module_publishes_global_binding_schemes_and_symbols() {
        let source = "module demo =\n  let id = fn x => x\nend\n";
        let mut symbols = SymbolTable::new();
        let (resolved, file_logger) = resolve_source(source, &mut symbols);

        assert!(file_logger.is_ok());
        assert!(resolved.schemes.contains_key(&Path::new("demo", "id")));
        assert!(symbols.terms().contains_key(&Path::new("demo", "id")));
    }

    #[test]
    fn resolve_module_recovers_failed_global_terms_without_publishing_symbol() {
        let source = "module demo =\n  let bad = missing\nend\n";
        let mut symbols = SymbolTable::new();
        let (resolved, file_logger) = resolve_source(source, &mut symbols);

        assert!(!file_logger.is_ok());
        assert!(!resolved.schemes.contains_key(&Path::new("demo", "bad")));
        assert!(!symbols.terms().contains_key(&Path::new("demo", "bad")));

        let Some(Statement::Term(term)) = resolved.module.statements.first() else {
            panic!("expected term statement");
        };
        assert_eq!(term.type_, Type::Unit);
    }

    #[test]
    fn resolve_module_reports_duplicate_type_against_existing_symbols() {
        let source = "module demo =\n  type Token = { value: core::integer }\nend\n";
        let mut symbols = SymbolTable::new();
        symbols.insert_type(
            Path::new("demo", "Token"),
            TypeDefinition {
                parameters: 0,
                body: Type::Unit,
                kind: TypeDefinitionKind::Named,
            },
        );

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(
            file_logger
                .iter()
                .any(|diagnostic| diagnostic.message == "Duplicate type definition")
        );
    }

    #[test]
    fn resolve_module_solves_trait_predicates_for_impl_calls() {
        let source = "module demo =\n  trait Id : a =\n    let id : a -> a\n  end\n  impl Id : core::integer =\n    let id = fn x => x\n  end\n  let value : core::integer = id 1\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
        assert!(resolved.schemes.contains_key(&Path::new("demo", "value")));
        assert!(symbols.trait_impls().contains_key(&Path::new("demo", "Id")));
    }

    #[test]
    fn resolve_module_reports_unresolved_ground_predicates() {
        let source = "module demo =\n  trait Eq : a =\n    let eq : a -> a -> core::boolean\n  end\n  let value = eq 1 1\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
        assert!(
            file_logger
                .iter()
                .any(|diagnostic| diagnostic.message == "Unresolved trait constraint")
        );
    }
}
