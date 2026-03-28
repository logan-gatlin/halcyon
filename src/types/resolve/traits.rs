//! Trait-definition construction and predicate solving for resolve.

use std::collections::HashSet;

use crate::logging::WithContext;
use indexmap::IndexMap;

use super::super::infer::InferenceContext;
use super::super::instantiation::leading_forall_count;
use super::super::kind::{
    KindError,
    SchemeKindError,
    constructor_kind,
    infer_scheme_kind,
};
use super::super::{
    Type,
    normalize_parameter_kinds,
};
use super::common::format_trait_ref;
use super::diagnostics::{
    log_scheme_predicate_kind_error,
    log_trait_error,
};
use super::type_defs::{
    param_index_map,
    type_expr_to_scheme_in_def,
};
use super::{
    FileLogger,
    Kind,
    Path,
    PendingTraitAliasEntry,
    PendingTraitDefinitionEntry,
    PendingTypeDefinitionEntry,
    Span,
    Statement,
    SymbolTable,
    TraitDef,
    TraitRef,
    TypeDefinition,
    TypeScheme,
};

/// Build trait definitions from syntax-level statements.
#[tracing::instrument(level = "debug", skip_all)]
pub(super) fn build_trait_definitions(
    statements: &[Statement<()>],
    pending_type_definitions: &IndexMap<Path, PendingTypeDefinitionEntry>,
    type_definitions: &IndexMap<Path, TypeDefinition>,
    trait_definitions: &IndexMap<Path, TraitDef>,
    logger: &mut FileLogger,
) -> Vec<PendingTraitDefinitionEntry> {
    let mut resolved_type_definitions = type_definitions.clone();
    let mut known_trait_kinds = trait_definitions
        .iter()
        .map(|(path, definition)| {
            (
                path.clone(),
                normalize_parameter_kinds(
                    definition.parameter_kinds.clone(),
                    definition.parameters,
                ),
            )
        })
        .collect::<IndexMap<_, _>>();
    let mut seen_traits = HashSet::new();
    statements
        .iter()
        .filter_map(|statement| {
            let Statement::Trait {
                path,
                parameters,
                associated_types,
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
            let mut inferred_parameter_kinds = vec![None; parameters.len()];
            let mut methods = IndexMap::new();
            for method in method_decls.iter() {
                let mut scheme = type_expr_to_scheme_in_def(
                    &method.type_expr,
                    &trait_parameter_indices,
                    pending_type_definitions,
                    &mut resolved_type_definitions,
                    &mut Vec::new(),
                    logger,
                );
                scheme.type_ = scheme.type_.for_all(parameters.len());
                match infer_scheme_kind(
                    &scheme,
                    parameters.len(),
                    &|type_path| {
                        resolved_type_definitions.get(type_path).map(|definition| {
                            constructor_kind(definition.parameters, &definition.parameter_kinds)
                        })
                    },
                    &|trait_name| known_trait_kinds.get(trait_name).cloned(),
                ) {
                    Ok(inferred) => {
                        if inferred.kind != Kind::Type {
                            logger
                                .error("Invalid trait method kind")
                                .primary(
                                    format!(
                                        "`{}` resolves to kind `{}` but trait methods must resolve to `Type`.",
                                        method.path, inferred.kind
                                    ),
                                    method.span,
                                )
                                .done();
                        }
                        merge_parameter_kinds(
                            &mut inferred_parameter_kinds,
                            &inferred.parameter_kinds,
                            path,
                            method.span,
                            logger,
                        );
                    }
                    Err(error) => {
                        log_trait_scheme_kind_error(logger, path, &method.path, method.span, error);
                    }
                }
                methods.insert(method.path.clone(), scheme);
            }
            let parameter_kinds = inferred_parameter_kinds
                .into_iter()
                .map(|kind| kind.unwrap_or(Kind::Type))
                .collect::<Vec<_>>();
            known_trait_kinds.insert(path.clone(), parameter_kinds.clone());
            let associated_types = associated_types
                .iter()
                .map(|associated_type| (associated_type.path.clone(), Kind::Type))
                .collect();
            Some(PendingTraitDefinitionEntry {
                span,
                trait_definition: TraitDef {
                    name: path.clone(),
                    parameters: parameters.len(),
                    parameter_kinds,
                    associated_types,
                    methods,
                },
            })
        })
        .collect()
}

fn merge_parameter_kinds(
    inferred: &mut [Option<Kind>],
    current: &[Kind],
    trait_path: &Path,
    span: Span,
    logger: &mut FileLogger,
) {
    for (slot, inferred_kind) in inferred.iter_mut().zip(current.iter()) {
        if let Some(existing) = slot {
            if existing != inferred_kind {
                logger
                    .error("Inconsistent trait parameter kinds")
                    .primary(
                        format!(
                            "`{trait_path}` infers conflicting kinds `{}` and `{}` for the same trait parameter.",
                            existing,
                            inferred_kind
                        ),
                        span,
                    )
                    .done();
            }
            continue;
        }
        *slot = Some(inferred_kind.clone());
    }
}

fn log_trait_scheme_kind_error(
    logger: &mut FileLogger,
    trait_path: &Path,
    method_path: &Path,
    span: Span,
    error: SchemeKindError,
) {
    if log_scheme_predicate_kind_error(logger, span, &error) {
        return;
    }

    let SchemeKindError::Kind(kind_error) = error else {
        return;
    };
    let message = match kind_error {
        KindError::Mismatch { left, right } => {
            format!(
                "`{method_path}` in trait `{trait_path}` has incompatible kinds `{left}` and `{right}`."
            )
        }
        KindError::Occurs { in_kind, .. } => {
            format!("`{method_path}` in trait `{trait_path}` has recursive kind `{in_kind}`.")
        }
    };
    logger
        .error("Invalid trait method kind")
        .primary(message, span)
        .done();
}

/// Register trait definitions and publish trait method schemes into term symbols.
#[tracing::instrument(level = "debug", skip_all, fields(count = entries.len()))]
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
#[tracing::instrument(level = "debug", skip_all, fields(predicate_count = predicates.len()))]
pub(super) fn solve_predicates(
    logger: &mut FileLogger,
    inference_context: &mut InferenceContext,
    symbols: &SymbolTable,
    span: Span,
    predicates: &[TraitRef],
) {
    solve_predicates_with_assumptions(logger, inference_context, symbols, span, predicates, &[]);
}

/// Attempt to solve predicates under additional assumed constraints.
#[tracing::instrument(level = "debug", skip_all, fields(predicate_count = predicates.len(), assumption_count = assumptions.len()))]
pub(super) fn solve_predicates_with_assumptions(
    logger: &mut FileLogger,
    inference_context: &mut InferenceContext,
    symbols: &SymbolTable,
    span: Span,
    predicates: &[TraitRef],
    assumptions: &[TraitRef],
) {
    if predicates.is_empty() {
        return;
    }
    for predicate in predicates {
        let normalized = inference_context.table_mut().normalize_trait_ref(predicate);
        tracing::debug!(
            raw = %format_trait_ref(predicate),
            normalized = %format_trait_ref(&normalized),
            "solving predicate",
        );
    }
    match symbols.resolve_predicates_with_assumptions(
        inference_context.table_mut(),
        predicates,
        assumptions,
    ) {
        Ok(unresolved) => {
            for predicate in unresolved.iter() {
                tracing::debug!(
                    predicate = %format_trait_ref(predicate),
                    "unresolved predicate",
                );
            }
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
        Err(error) => {
            tracing::debug!("predicate resolution failed with error");
            log_trait_error(logger, span, error);
        }
    }
}

/// Convert a trait-item scheme into a globally callable term scheme.
///
/// The method scheme already has both the trait-parameter ForAlls (outermost)
/// and the method's own ForAlls (inner). In the innermost body, the trait
/// parameter TypeVars sit above the method parameters: at indices
/// `method_forall_count .. method_forall_count + trait_params - 1`.
fn trait_method_term_scheme(
    trait_name: &Path,
    parameters: usize,
    method_scheme: &TypeScheme,
) -> TypeScheme {
    let total_foralls = leading_forall_count(&method_scheme.type_);
    let method_foralls = total_foralls.saturating_sub(parameters);
    let mut predicates = method_scheme.predicates.clone();
    let trait_param_type_vars = (0..parameters)
        .map(|i| Type::v((method_foralls + parameters - 1 - i) as u32))
        .collect();
    predicates.push(super::TraitRef::new(
        trait_name.clone(),
        trait_param_type_vars,
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
            .and_then(|module| {
                ir::lower_module(module, &mut file_logger, ir::LoweringOptions::default())
                    .map(|lowered| lowered.module)
            })
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
            &IndexMap::new(),
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
        assert_eq!(trait_def.parameter_kinds, vec![Kind::Type]);
    }

    #[test]
    fn build_trait_definitions_infers_higher_kinded_parameters() {
        let statements = parse_statements(
            "module demo =\n  trait Monad : m =\n    let map : for a b in (a -> b) -> m a -> m b\n  end\nend\n",
        );
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");

        let built = build_trait_definitions(
            &statements,
            &IndexMap::new(),
            &core_like_type_definitions(),
            &IndexMap::new(),
            &mut file_logger,
        );

        assert_eq!(built.len(), 1);
        let trait_def = &built[0].trait_definition;
        assert_eq!(trait_def.name, Path::new("demo", "Monad"));
        assert_eq!(
            trait_def.parameter_kinds,
            vec![Kind::arrow(Kind::Type, Kind::Type)]
        );
    }

    #[test]
    fn build_trait_definitions_keep_map_as_two_argument_function() {
        let statements = parse_statements(
            "module demo =\n  trait Monad : m =\n    let map : for a b in (a -> b) -> m a -> m b\n  end\nend\n",
        );
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");

        let built = build_trait_definitions(
            &statements,
            &IndexMap::new(),
            &core_like_type_definitions(),
            &IndexMap::new(),
            &mut file_logger,
        );

        assert_eq!(built.len(), 1);
        let trait_def = &built[0].trait_definition;
        let method_scheme = trait_def
            .methods
            .get(&Path::new("demo", "map"))
            .expect("method scheme should exist");

        let mut foralls = 0usize;
        let mut body = &method_scheme.type_;
        while let Type::ForAll { body: next, .. } = body {
            foralls += 1;
            body = next;
        }

        assert_eq!(foralls, 3);
        let Type::Function(_, result) = body else {
            panic!("expected map to lower as a function type");
        };
        assert!(matches!(**result, Type::Function(_, _)));
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
            &IndexMap::new(),
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

    /// When a trait method has its own ForAll binders (e.g. `map : for a b in ...`),
    /// the predicate's TypeVar for the trait parameter must point at the OUTERMOST
    /// ForAll, not the innermost.
    ///
    /// For `trait Monad : m` with `map : for a b in (a -> b) -> m a -> m b`,
    /// the full scheme is `for m in for a in for b in (...)`. In the innermost
    /// body, `m = TypeVar(2)`, `a = TypeVar(1)`, `b = TypeVar(0)`.
    /// The predicate must be `Monad TypeVar(2)`.
    #[test]
    fn trait_method_term_scheme_predicate_uses_outermost_type_var_for_hkt() {
        // Simulate: trait Monad : m, method map : for a b in (a -> b) -> m a -> m b
        // After build_trait_definitions the scheme type has 3 ForAlls:
        //   for m in for a in for b in ((a -> b) -> (m a -> m b))
        // In the body: m = TypeVar(2), a = TypeVar(1), b = TypeVar(0)
        let body = Type::func(
            Type::func(Type::v(1), Type::v(0)),
            Type::func(
                Type::v(2).apply(vec![Type::v(1)]),
                Type::v(2).apply(vec![Type::v(0)]),
            ),
        );
        let scheme = body.for_all(3).scheme(); // 3 ForAlls: m, a, b

        let lifted = trait_method_term_scheme(&Path::new("demo", "Monad"), 1, &scheme);

        // The predicate must refer to TypeVar(2) = outermost ForAll = m
        assert_eq!(
            lifted.predicates,
            vec![TraitRef::new(Path::new("demo", "Monad"), vec![Type::v(2)])]
        );
    }

    /// Regression: with `instantiate_scheme`'s reversed-metas fix, the
    /// predicate's TypeVar(2) must map to the same meta as the type body's
    /// `m` after instantiation.
    #[test]
    fn hkt_trait_method_predicate_resolves_to_constructor_after_instantiation() {
        use crate::types::infer::InferenceContext;

        // Scheme: for m a b in ((a -> b) -> m a -> m b) where Monad m
        // m = TypeVar(2), a = TypeVar(1), b = TypeVar(0)
        let option = Type::Named {
            name: Path::new("core", "Option"),
            body: Box::new(Type::Unit),
        };
        let body = Type::func(
            Type::func(Type::v(1), Type::v(0)),
            Type::func(
                Type::v(2).apply(vec![Type::v(1)]),
                Type::v(2).apply(vec![Type::v(0)]),
            ),
        );
        let scheme_type = body.for_all(3);
        let predicates = vec![TraitRef::new(Path::new("demo", "Monad"), vec![Type::v(2)])];
        let scheme = TypeScheme {
            type_: scheme_type,
            predicates,
        };

        let mut ctx = InferenceContext::new();
        let instance = ctx
            .instantiate_scheme(&scheme, Span::Generated)
            .expect("instantiation should succeed");

        // instance.type_ is (v_a -> v_b) -> (v_m v_a -> v_m v_b)
        // Extract (v_a -> v_b) from the outer function parameter
        let Type::Function(param_fn, result_fn) = &instance.type_ else {
            panic!("expected function type");
        };
        // Unify (v_a -> v_b) with (Integer -> Integer)
        ctx.table_mut()
            .unify(param_fn, &Type::func(Type::Integer, Type::Integer))
            .expect("argument unification should succeed");

        // Extract (v_m v_a) from the inner function parameter
        let Type::Function(m_a, _) = result_fn.as_ref() else {
            panic!("expected inner function type");
        };
        // Unify (v_m v_a) with (Option Integer)
        ctx.table_mut()
            .unify(m_a, &option.clone().apply(vec![Type::Integer]))
            .expect("constructor unification should succeed");

        // The predicate `Monad v_m` should now normalize to `Monad Option`
        let normalized = ctx.table_mut().normalize_trait_ref(&instance.predicates[0]);
        assert_eq!(
            normalized,
            TraitRef::new(Path::new("demo", "Monad"), vec![option]),
            "predicate must resolve to the type constructor, not a type argument",
        );
    }
}
