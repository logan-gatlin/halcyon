//! Trait-definition construction and predicate solving for resolve.

use std::collections::HashSet;

use crate::logging::WithContext;
use indexmap::IndexMap;

use super::super::infer::InferenceContext;
use super::common::format_trait_ref;
use super::diagnostics::log_trait_error;
use super::type_defs::{
    param_index_map,
    type_expr_to_scheme_in_def,
    type_vars_for_params,
};
use super::{
    FileLogger,
    Path,
    PendingTraitAliasEntry,
    PendingTraitDefinitionEntry,
    PendingTypeDefinitionEntry,
    Span,
    Statement,
    SymbolTable,
    TraitConstraint,
    TraitDef,
    TypeDefinition,
    TypeScheme,
};

/// Build trait definitions from syntax-level statements.
pub(super) fn build_trait_definitions(
    statements: &[Statement<()>],
    pending_type_definitions: &IndexMap<Path, PendingTypeDefinitionEntry>,
    type_definitions: &IndexMap<Path, TypeDefinition>,
    logger: &mut FileLogger,
) -> Vec<PendingTraitDefinitionEntry> {
    let mut resolved_type_definitions = type_definitions.clone();
    let mut seen_traits = HashSet::new();
    statements
        .iter()
        .filter_map(|statement| {
            let Statement::Trait {
                path,
                parameters,
                methods: method_decls,
                ..
            } = statement
            else {
                return None;
            };
            if !seen_traits.insert(path.clone()) {
                return None;
            }
            let trait_parameter_indices = param_index_map(parameters);
            let span = method_decls
                .first()
                .map(|method| method.span)
                .unwrap_or(Span::Generated);
            let methods = method_decls
                .iter()
                .map(|method| {
                    let mut scheme = type_expr_to_scheme_in_def(
                        &method.type_expr,
                        &trait_parameter_indices,
                        pending_type_definitions,
                        &mut resolved_type_definitions,
                        &mut Vec::new(),
                        logger,
                    );
                    scheme.type_ = scheme.type_.for_all(parameters.len());
                    (method.path.clone(), scheme)
                })
                .collect();
            Some(PendingTraitDefinitionEntry {
                span,
                trait_definition: TraitDef {
                    name: path.clone(),
                    parameters: parameters.len(),
                    methods,
                },
            })
        })
        .collect()
}

/// Register trait definitions and publish trait method schemes into term symbols.
pub(super) fn register_trait_definitions(
    symbols: &mut SymbolTable,
    entries: &[PendingTraitDefinitionEntry],
    logger: &mut FileLogger,
) {
    for entry in entries {
        match symbols.insert_trait(entry.trait_definition.clone()) {
            Ok(()) => {
                for (method_path, scheme) in entry.trait_definition.methods.iter() {
                    if symbols.terms().contains_key(method_path) {
                        continue;
                    }
                    symbols.insert_term(
                        method_path.clone(),
                        trait_method_term_scheme(
                            &entry.trait_definition.name,
                            entry.trait_definition.parameters,
                            scheme,
                        ),
                    );
                }
            }
            Err(error) => log_trait_error(logger, entry.span, error),
        }
    }
}

pub(super) fn build_trait_alias_entries(
    statements: &[Statement<()>]
) -> Vec<PendingTraitAliasEntry> {
    statements
        .iter()
        .filter_map(|statement| {
            let Statement::TraitAlias { path, target, .. } = statement else {
                return None;
            };
            Some(PendingTraitAliasEntry {
                span: Span::Generated,
                alias: path.clone(),
                target: target.clone(),
            })
        })
        .collect()
}

pub(super) fn register_trait_aliases(
    symbols: &mut SymbolTable,
    entries: &[PendingTraitAliasEntry],
    logger: &mut FileLogger,
) {
    for entry in entries {
        if let Err(error) = symbols.insert_trait_alias(entry.alias.clone(), entry.target.clone()) {
            log_trait_error(logger, entry.span, error);
        }
    }
}

/// Attempt to solve accumulated trait predicates and emit diagnostics on failure.
pub(super) fn solve_predicates(
    logger: &mut FileLogger,
    inference_context: &mut InferenceContext,
    symbols: &SymbolTable,
    span: Span,
    predicates: &[TraitConstraint],
) {
    solve_predicates_with_assumptions(logger, inference_context, symbols, span, predicates, &[]);
}

/// Attempt to solve predicates under additional assumed constraints.
pub(super) fn solve_predicates_with_assumptions(
    logger: &mut FileLogger,
    inference_context: &mut InferenceContext,
    symbols: &SymbolTable,
    span: Span,
    predicates: &[TraitConstraint],
    assumptions: &[TraitConstraint],
) {
    if predicates.is_empty() {
        return;
    }
    match symbols.resolve_predicates_with_assumptions(
        inference_context.table_mut(),
        predicates,
        assumptions,
    ) {
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

/// Convert a trait-item scheme into a globally callable term scheme.
fn trait_method_term_scheme(
    trait_name: &Path,
    parameters: usize,
    method_scheme: &TypeScheme,
) -> TypeScheme {
    let mut predicates = method_scheme.predicates.clone();
    predicates.push(super::TraitRef::new(
        trait_name.clone(),
        type_vars_for_params(parameters),
    ));
    TypeScheme {
        predicates,
        type_: method_scheme.type_.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hc_core::CoreType;
    use crate::types::symbol_table::Symbol;
    use crate::types::{
        TraitRef,
        Type,
    };
    use crate::{
        Logger,
        ir,
        parse,
    };

    fn parse_statements(source: &str) -> Vec<Statement<()>> {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", source);
        let module = parse::parse(source, &mut file_logger)
            .and_then(|source_file| source_file.modules().into_iter().next())
            .and_then(|module| ir::module(module, &mut file_logger))
            .expect("source should lower to module");
        module.statements.into_vec()
    }

    fn core_like_type_definitions() -> IndexMap<Path, TypeDefinition> {
        [
            (CoreType::Function.path(), Type::function().def(2)),
            (CoreType::Boolean.path(), Type::Boolean.def(0)),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn build_trait_definitions_quantifies_methods_over_trait_parameters() {
        let statements = parse_statements(
            "module demo =\n  trait Eq : a =\n    let eq : a -> a -> core::Boolean\n  end\nend\n",
        );
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");

        let built = build_trait_definitions(
            &statements,
            &IndexMap::new(),
            &core_like_type_definitions(),
            &mut file_logger,
        );

        assert_eq!(built.len(), 1);
        let trait_def = &built[0].trait_definition;
        assert_eq!(trait_def.name, Path::new("demo", "Eq"));
        assert_eq!(trait_def.parameters, 1);
        assert_eq!(trait_def.methods.len(), 1);

        let method_scheme = trait_def
            .methods
            .get(&Path::new("demo", "eq"))
            .expect("method scheme should exist");
        assert_eq!(
            method_scheme.type_,
            Type::func(Type::v(0), Type::func(Type::v(0), Type::Boolean)).for_all(1)
        );
    }

    #[test]
    fn register_trait_definitions_inserts_trait_and_method_terms() {
        let statements = parse_statements(
            "module demo =\n  trait Eq : a =\n    let eq : a -> a -> core::Boolean\n  end\nend\n",
        );
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let entries = build_trait_definitions(
            &statements,
            &IndexMap::new(),
            &core_like_type_definitions(),
            &mut file_logger,
        );

        let mut symbols = SymbolTable::new();
        register_trait_definitions(&mut symbols, &entries, &mut file_logger);

        let trait_path = Path::new("demo", "Eq");
        let method_path = Path::new("demo", "eq");
        assert!(symbols.trait_defs().contains_key(&trait_path));

        let term_scheme = symbols
            .terms()
            .get(&method_path)
            .expect("method term scheme should be published");
        assert!(
            term_scheme
                .predicates
                .contains(&TraitRef::new(trait_path, vec![Type::v(0)]))
        );
    }

    #[test]
    fn solve_predicates_emits_unresolved_diagnostic_when_unsolved() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef::new(Path::new("demo", "Show"), 1))
            .expect("trait insertion should succeed");

        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let mut inference_context = InferenceContext::new();

        solve_predicates(
            &mut file_logger,
            &mut inference_context,
            &symbols,
            Span::Generated,
            &[TraitRef::new(
                Path::new("demo", "Show"),
                vec![Type::Integer],
            )],
        );

        assert!(!file_logger.is_ok());
        assert!(
            file_logger
                .iter()
                .any(|diagnostic| diagnostic.message == "Unresolved trait constraint")
        );
    }

    #[test]
    fn solve_predicates_with_assumptions_can_discharge_constraints() {
        let mut symbols = SymbolTable::new();
        symbols
            .insert_trait(TraitDef::new(Path::new("demo", "Show"), 1))
            .expect("trait insertion should succeed");

        let predicate = TraitRef::new(Path::new("demo", "Show"), vec![Type::Integer]);
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let mut inference_context = InferenceContext::new();

        solve_predicates_with_assumptions(
            &mut file_logger,
            &mut inference_context,
            &symbols,
            Span::Generated,
            std::slice::from_ref(&predicate),
            std::slice::from_ref(&predicate),
        );

        assert!(file_logger.is_ok());
    }

    #[test]
    fn trait_method_term_scheme_adds_self_predicate() {
        let scheme = Type::func(Type::v(0), Type::v(0)).scheme();
        let lifted = trait_method_term_scheme(&Path::new("demo", "Id"), 1, &scheme);

        assert_eq!(lifted.type_, scheme.type_);
        assert_eq!(
            lifted.predicates,
            vec![TraitRef::new(Path::new("demo", "Id"), vec![Type::v(0)])]
        );
    }
}
