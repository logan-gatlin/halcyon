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
    InferenceConfig,
    InferenceContext,
    TypeEnv,
};
use super::{
    Kind,
    SymbolTable,
    TraitDef,
    TraitError,
    TraitImpl,
    TraitRef,
    Type,
    TypeDefinition,
    TypeDefinitionKind,
    TypeScheme,
    for_each_pattern_binding,
    predicate_is_ground,
    sorted_unique_predicates,
};

mod common;
mod diagnostics;
mod exhaustiveness;
mod impls;
mod recovery;
mod traits;
mod type_defs;

use diagnostics::{
    log_duplicate_definition,
    log_term_duplicates,
    log_type_error,
};
use exhaustiveness::check_term_exhaustiveness;
use impls::ImplProcessingContext;
use recovery::{
    fallback_term,
    normalize_term_types,
};
use traits::{
    build_trait_alias_entries,
    build_trait_definitions,
    register_trait_aliases,
    register_trait_definitions,
    solve_predicates,
};
use type_defs::{
    build_type_constructors,
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
struct PendingTraitAliasEntry {
    span: Span,
    alias: Path,
    target: Path,
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Fully typed module produced by resolve.
    pub module: Module<Type>,

    /// Final inferred/generalized schemes for exported bindings.
    pub schemes: IndexMap<Path, TypeScheme>,

    /// Per-binding runtime evidence requirements after predicate normalization.
    pub evidence_requirements: IndexMap<Path, Vec<TraitRef>>,
}

/// Resolve a module with a fresh symbol table and return typed IR plus schemes.
#[tracing::instrument(skip_all, fields(module = %module.name))]
pub fn resolve(
    module: Module<()>,
    logger: &mut FileLogger,
) -> ResolvedModule {
    let mut symbols = SymbolTable::new();
    resolve_with_symbols(&mut symbols, module, logger)
}

/// Canonical resolve entry point using an existing symbol table.
#[tracing::instrument(skip_all, fields(module = %module.name))]
pub fn resolve_with_symbols(
    symbols: &mut SymbolTable,
    module: Module<()>,
    logger: &mut FileLogger,
) -> ResolvedModule {
    resolve_with_symbols_and_schemes(symbols, module, logger)
}

/// Resolve a module and return both typed IR and finalized binding schemes.
#[tracing::instrument(skip_all, fields(module = %module.name))]
fn resolve_with_symbols_and_schemes(
    symbols: &mut SymbolTable,
    module: Module<()>,
    logger: &mut FileLogger,
) -> ResolvedModule {
    let _profile_total = crate::profiling::scope("resolve.module.total");
    let Module { name, statements } = module;
    let statements = Vec::from(statements);
    let pending_type_definitions = {
        let _profile = crate::profiling::scope("resolve.collect_type_entries");
        collect_type_entries(&statements)
    };
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

    let mut pending_term_definitions = {
        let _profile = crate::profiling::scope("resolve.collect_term_definitions");
        collect_term_definitions(&statements)
    };
    let pending_constructor_definitions = {
        let _profile = crate::profiling::scope("resolve.collect_constructor_definitions");
        collect_constructor_definitions(&pending_type_definitions, &duplicate_type_paths)
    };
    let pending_constructor_paths = pending_constructor_definitions
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<HashSet<_>>();
    pending_term_definitions.extend(pending_constructor_definitions);
    log_term_duplicates(logger, symbols, &pending_term_definitions);

    let type_definitions = {
        let _profile = crate::profiling::scope("resolve.build_type_definitions");
        build_type_definitions(
            symbols.type_definitions(),
            &pending_type_definitions,
            logger,
        )
    };
    tracing::debug!(
        type_count = type_definitions.len(),
        "type definitions built",
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
                    parameter_kinds: vec![Kind::Type; entry.parameters.len()],
                    body: Type::Error,
                    kind: entry.kind,
                },
            );
        }
    }

    let pending_trait_definitions = {
        let _profile = crate::profiling::scope("resolve.build_trait_definitions");
        build_trait_definitions(
            &statements,
            &pending_type_definitions,
            &type_definitions,
            symbols.trait_defs(),
            logger,
        )
    };
    {
        let _profile = crate::profiling::scope("resolve.register_trait_definitions");
        register_trait_definitions(symbols, &pending_trait_definitions, logger);
    }
    let pending_trait_aliases = {
        let _profile = crate::profiling::scope("resolve.build_trait_aliases");
        build_trait_alias_entries(&statements)
    };
    {
        let _profile = crate::profiling::scope("resolve.register_trait_aliases");
        register_trait_aliases(symbols, &pending_trait_aliases, logger);
    }
    let mut type_definitions = type_definitions;
    for (path, definition) in symbols.type_definitions().iter() {
        type_definitions
            .entry(path.clone())
            .or_insert_with(|| definition.clone());
    }
    tracing::debug!(
        trait_count = pending_trait_definitions.len(),
        alias_count = pending_trait_aliases.len(),
        "traits registered",
    );

    let mut type_environment = TypeEnv::new();
    type_environment.extend(
        symbols
            .terms()
            .iter()
            .map(|(path, scheme)| (path.clone(), scheme.clone())),
    );
    let constructors = {
        let _profile = crate::profiling::scope("resolve.build_type_constructors");
        build_type_constructors(&pending_type_definitions, &type_definitions, logger)
    };
    type_environment.extend(constructors);

    let mut inference_context = InferenceContext::with_config(
        InferenceConfig::default()
            .with_type_definitions(
                type_definitions
                    .iter()
                    .map(|(path, definition)| (path.clone(), definition.clone()))
                    .collect(),
            )
            .with_trait_aliases(symbols.trait_aliases().clone())
            .with_trait_parameter_kinds(
                symbols
                    .trait_defs()
                    .iter()
                    .map(|(path, definition)| (path.clone(), definition.parameter_kinds.clone()))
                    .collect(),
            ),
    );
    let mut schemes = IndexMap::new();
    let mut failed_term_paths = HashSet::new();
    let mut resolved_constructor_aliases = Vec::new();

    tracing::debug!(
        statement_count = statements.len(),
        "beginning statement inference",
    );
    let mut typed_statements = Vec::new();
    {
        let _profile = crate::profiling::scope("resolve.infer_statements");
        for statement in statements.into_iter() {
            match statement {
                Statement::Term(term) => {
                    let known_scheme_paths = schemes.keys().cloned().collect::<HashSet<_>>();
                    let type_environment_snapshot = type_environment.clone();
                    let schemes_snapshot = schemes.clone();
                    let inference_result = {
                        let _profile = crate::profiling::scope("resolve.infer_term");
                        inference_context.infer_term(&mut type_environment, &term, &mut schemes)
                    };
                    let mut output = match inference_result {
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

                    output.term = normalize_term_types(output.term, inference_context.table_mut());
                    let mut constructor_aliases = symbols.constructor_aliases().clone();
                    constructor_aliases.extend(resolved_constructor_aliases.iter().cloned());
                    if let Err(error) = check_term_exhaustiveness(
                        &output.term,
                        symbols.type_definitions(),
                        &constructor_aliases,
                    ) {
                        log_type_error(logger, error);
                        if let TermKind::Let {
                            assignee,
                            scope: ScopeKind::Global,
                            ..
                        } = &term.kind
                        {
                            failed_term_paths.extend(pattern_binding_paths(assignee));
                        }
                        type_environment = type_environment_snapshot;
                        schemes = schemes_snapshot;
                        typed_statements.push(Statement::Term(fallback_term(&term)));
                        continue;
                    }

                    {
                        let _profile = crate::profiling::scope("resolve.solve_predicates.direct");
                        solve_predicates(
                            logger,
                            &mut inference_context,
                            symbols,
                            term.span,
                            &output.predicates,
                        );
                    }

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

                    {
                        let _profile = crate::profiling::scope("resolve.solve_predicates.grounded");
                        solve_predicates(
                            logger,
                            &mut inference_context,
                            symbols,
                            term.span,
                            &grounded_predicates,
                        );
                    }

                    typed_statements.push(Statement::Term(output.term));
                }
                Statement::ConstructorAlias {
                    comments,
                    path,
                    target,
                    span,
                } => {
                    let Some(target_scheme) = type_environment.get(&target).cloned() else {
                        failed_term_paths.insert(path.clone());
                        logger
                            .error("Unknown constructor alias target")
                            .primary(
                                format!(
                                    "`{target}` is not available as a constructor term in this scope."
                                ),
                                span,
                            )
                            .done();
                        typed_statements.push(Statement::ConstructorAlias {
                            comments,
                            path,
                            target,
                            span,
                        });
                        continue;
                    };
                    type_environment.insert(path.clone(), target_scheme.clone());
                    schemes.insert(path.clone(), target_scheme);
                    resolved_constructor_aliases.push((path.clone(), target.clone()));
                    typed_statements.push(Statement::ConstructorAlias {
                        comments,
                        path,
                        target,
                        span,
                    });
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
                    associated_types,
                    methods,
                } => {
                    typed_statements.push(Statement::Trait {
                        comments,
                        path,
                        parameters,
                        associated_types,
                        methods,
                    });
                }
                Statement::TraitAlias {
                    comments,
                    path,
                    target,
                } => {
                    typed_statements.push(Statement::TraitAlias {
                        comments,
                        path,
                        target,
                    });
                }
                Statement::Impl {
                    comments,
                    trait_path,
                    arguments,
                    associated_types,
                    methods,
                } => {
                    let (typed_impl, generated_terms) = {
                        let _profile = crate::profiling::scope("resolve.process_impl");
                        ImplProcessingContext {
                            module_name: &name,
                            logger,
                            inference_context: &mut inference_context,
                            type_environment: &mut type_environment,
                            symbols,
                            schemes: &mut schemes,
                            pending_type_definitions: &pending_type_definitions,
                            type_definitions: &type_definitions,
                        }
                        .process(
                            comments,
                            trait_path,
                            arguments,
                            associated_types,
                            methods,
                        )
                    };
                    typed_statements.push(typed_impl);
                    typed_statements.extend(generated_terms.into_iter().map(Statement::Term));
                }
                Statement::Wasm(sexpr) => typed_statements.push(Statement::Wasm(sexpr)),
            }
        }
    }

    let mut published_term_paths = HashSet::new();
    {
        let _profile = crate::profiling::scope("resolve.publish_terms");
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
                    published_term_paths.insert(path.clone());
                    if pending_constructor_paths.contains(path) {
                        symbols.insert_constructor(path.clone());
                    }
                }
                None => {
                    logger
                        .error("Missing term definition")
                        .primary(format!("`{path}` was not assigned a type."), *span)
                        .done();
                }
            }
        }
    }

    for (alias, target) in resolved_constructor_aliases {
        if published_term_paths.contains(&alias) {
            symbols.insert_constructor_alias(alias, target);
        }
    }

    let evidence_requirements = {
        let _profile = crate::profiling::scope("resolve.collect_evidence_requirements");
        collect_evidence_requirements(&schemes)
    };

    ResolvedModule {
        module: Module {
            name,
            statements: typed_statements.into_boxed_slice(),
        },
        schemes,
        evidence_requirements,
    }
}

fn collect_evidence_requirements(
    schemes: &IndexMap<Path, TypeScheme>
) -> IndexMap<Path, Vec<TraitRef>> {
    schemes
        .iter()
        .filter_map(|(path, scheme)| {
            let requirements = sorted_unique_predicates(&scheme.predicates);
            (!requirements.is_empty()).then_some((path.clone(), requirements))
        })
        .collect()
}

fn pattern_binding_paths<T>(pattern: &Pattern<T>) -> Vec<Path> {
    let mut paths = Vec::new();
    for_each_pattern_binding(pattern, |path, _| paths.push(path.clone()));
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hc_core::compile_core_module;
    use crate::ir::Glob;
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
            .and_then(|module| {
                ir::lower_module(module, &mut file_logger, ir::LoweringOptions::default())
                    .map(|lowered| lowered.module)
            })
            .expect("source should lower to IR module");
        let resolved = resolve_with_symbols_and_schemes(symbols, module, &mut file_logger);
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
        assert_eq!(term.type_, Type::Error);
    }

    #[test]
    fn resolve_module_reports_duplicate_type_against_existing_symbols() {
        let source = "module demo =\n  type Token = { value: core::Integer }\nend\n";
        let mut symbols = SymbolTable::new();
        symbols.insert_type(
            Path::new("demo", "Token"),
            TypeDefinition {
                parameters: 0,
                parameter_kinds: Vec::new(),
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
        let source = "module demo =\n  trait Id : a =\n    let id : a -> a\n  end\n  impl Id core::Integer =\n    let id = fn x => x\n  end\n  let value : core::Integer = id 1\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
        assert!(resolved.schemes.contains_key(&Path::new("demo", "value")));
        assert!(symbols.trait_impls().contains_key(&Path::new("demo", "Id")));
    }

    #[test]
    fn resolve_module_allows_recursive_trait_calls_in_impl_methods() {
        let source = "module demo =\n  trait Id : a =\n    let id : a -> a\n  end\n  impl Id for a in a =\n    let id = fn value => demo::id value\n  end\n  let value : core::Integer = demo::id 1\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
        assert!(resolved.schemes.contains_key(&Path::new("demo", "value")));

        let trait_path = Path::new("demo", "Id");
        let method_path = Path::new("demo", "id");
        let impl_method_path = symbols
            .trait_impls()
            .get(&trait_path)
            .and_then(|implementations| implementations.first())
            .and_then(|implementation| implementation.methods.get(&method_path))
            .cloned()
            .expect("expected generated impl method path");

        let impl_scheme = symbols
            .terms()
            .get(&impl_method_path)
            .expect("expected impl method scheme");
        assert!(
            impl_scheme.predicates.is_empty(),
            "impl method scheme should not retain recursive self predicate"
        );
        assert!(
            !resolved
                .evidence_requirements
                .contains_key(&impl_method_path),
            "impl method should not publish self evidence requirements"
        );
    }

    #[test]
    fn resolve_module_collects_evidence_requirements_for_predicated_bindings() {
        let source = "module demo =\n  trait Addish : a =\n    let plus : a -> a -> a\n  end\n  let double = fn x => plus x x\nend\n";
        let mut symbols = SymbolTable::new();

        let (resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());

        let evidence = resolved
            .evidence_requirements
            .get(&Path::new("demo", "double"))
            .expect("expected evidence requirements for predicated binding");
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].trait_name, Path::new("demo", "Addish"));
    }

    #[test]
    fn resolve_module_reports_unresolved_ground_predicates() {
        let source = "module demo =\n  trait Eq : a =\n    let eq : a -> a -> core::Boolean\n  end\n  let value = eq 1 1\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
    }

    #[test]
    fn resolve_module_reports_unresolved_ground_predicates_for_applied_named_types() {
        let source = "module demo =\n  type Box: a = { value: a }\n  trait Show : a =\n    let show : a -> core::String\n  end\n  let boxed : Box core::Integer = { value = 1 }\n  let rendered = show boxed\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
    }

    #[test]
    fn resolve_module_reports_unresolved_predicates_for_do_expressions() {
        let source = "module demo =\n  do show default\nend\n";
        let mut symbols = SymbolTable::new();
        let mut logger = Logger::new();
        let _ = compile_core_module(&mut symbols, &mut logger);

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
    }

    #[test]
    fn resolve_module_reports_non_exhaustive_boolean_match_with_counterexample() {
        let source = "module demo =\n  let value =\n    match true with\n    | true => 1\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
        let Some(diagnostic) = file_logger
            .iter()
            .find(|diagnostic| diagnostic.message == "Non-exhaustive patterns")
        else {
            panic!("expected non-exhaustive pattern diagnostic");
        };
        assert!(
            diagnostic.notes.iter().any(|note| note.contains("false")),
            "expected boolean counterexample note"
        );
    }

    #[test]
    fn resolve_module_accepts_exhaustive_boolean_match() {
        let source = "module demo =\n  let value =\n    match true with\n    | true => 1\n    | false => 0\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
    }

    #[test]
    fn resolve_module_reports_non_exhaustive_sum_match_with_counterexample() {
        let source = "module demo =\n  type Option: a = | Some a | None\n  let pick = fn value =>\n    match value with\n    | Some _ => 1\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
        let Some(diagnostic) = file_logger
            .iter()
            .find(|diagnostic| diagnostic.message == "Non-exhaustive patterns")
        else {
            panic!("expected non-exhaustive pattern diagnostic");
        };
        assert!(
            diagnostic.notes.iter().any(|note| note.contains("None")),
            "expected missing constructor counterexample"
        );
    }

    #[test]
    fn resolve_module_accepts_nested_pattern_exhaustiveness() {
        let source = "module demo =\n  let classify = fn value =>\n    match value with\n    | (true, true) => 1\n    | (true, false) => 2\n    | (false, true) => 3\n    | (false, false) => 4\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
    }

    #[test]
    fn resolve_module_reports_non_exhaustive_nested_pattern_with_counterexample() {
        let source = "module demo =\n  let classify = fn value =>\n    match value with\n    | (true, true) => 1\n    | (true, false) => 2\n    | (false, true) => 3\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
        let Some(diagnostic) = file_logger
            .iter()
            .find(|diagnostic| diagnostic.message == "Non-exhaustive patterns")
        else {
            panic!("expected non-exhaustive pattern diagnostic");
        };
        assert!(
            diagnostic
                .notes
                .iter()
                .any(|note| note.contains("(false, false)")),
            "expected nested counterexample note"
        );
    }

    #[test]
    fn resolve_module_accepts_deep_array_partition_exhaustiveness() {
        let source = "module demo =\n  let classify = fn xs =>\n    match xs with\n    | [] => 0\n    | [true, ..] => 1\n    | [false, ..] => 2\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
    }

    #[test]
    fn resolve_module_reports_non_exhaustive_array_partition_with_counterexample() {
        let source = "module demo =\n  let classify = fn xs =>\n    match xs with\n    | [true, ..] => 1\n    | [false, ..] => 2\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(!file_logger.is_ok());
        let Some(diagnostic) = file_logger
            .iter()
            .find(|diagnostic| diagnostic.message == "Non-exhaustive patterns")
        else {
            panic!("expected non-exhaustive pattern diagnostic");
        };
        assert!(
            diagnostic.notes.iter().any(|note| note.contains("[]")),
            "expected empty-array counterexample"
        );
    }

    #[test]
    fn resolve_module_allows_positive_branch_refinement_for_nested_lets() {
        let source = "module demo =\n  let value = fn input =>\n    match input with\n    | true => let true = input in 1\n    | false => 0\nend\n";
        let mut symbols = SymbolTable::new();

        let (_resolved, file_logger) = resolve_source(source, &mut symbols);
        assert!(file_logger.is_ok());
    }
}
