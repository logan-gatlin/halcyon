use std::collections::HashSet;

use crate::logging::WithContext;
use indexmap::IndexMap;

use super::super::infer::InferenceContext;
use super::common::format_trait_ref;
use super::diagnostics::log_trait_error;
use super::type_defs::{
    param_index_map,
    type_expr_to_type_in_def,
    type_vars_for_params,
};
use super::{
    FileLogger,
    Path,
    Span,
    Statement,
    SymbolTable,
    TraitConstraint,
    TraitDef,
    TraitDefEntry,
    TypeDefEntry,
    TypeDefinition,
    TypeScheme,
};

pub(super) fn build_trait_definitions(
    statements: &[Statement<()>],
    type_entries: &IndexMap<Path, TypeDefEntry>,
    type_definitions: &IndexMap<Path, TypeDefinition>,
    logger: &mut FileLogger,
) -> Vec<TraitDefEntry> {
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
            let param_map = param_index_map(parameters);
            let span = method_decls
                .first()
                .map(|method| method.span)
                .unwrap_or(Span::Generated);
            let methods = method_decls
                .iter()
                .map(|method| {
                    let type_ = type_expr_to_type_in_def(
                        &method.type_expr,
                        &param_map,
                        type_entries,
                        &mut resolved_type_definitions,
                        &mut Vec::new(),
                        logger,
                    )
                    .for_all(parameters.len());
                    (method.path.clone(), TypeScheme::new(type_))
                })
                .collect();
            Some(TraitDefEntry {
                span,
                def: TraitDef {
                    name: path.clone(),
                    parameters: parameters.len(),
                    methods,
                },
            })
        })
        .collect()
}

pub(super) fn register_trait_definitions(
    symbols: &mut SymbolTable,
    entries: &[TraitDefEntry],
    logger: &mut FileLogger,
) {
    for entry in entries {
        match symbols.insert_trait(entry.def.clone()) {
            Ok(()) => {
                for (method_path, scheme) in entry.def.methods.iter() {
                    if symbols.terms().contains_key(method_path) {
                        continue;
                    }
                    symbols.insert_term(
                        method_path.clone(),
                        trait_method_term_scheme(&entry.def.name, entry.def.parameters, scheme),
                    );
                }
            }
            Err(error) => log_trait_error(logger, entry.span, error),
        }
    }
}

pub(super) fn solve_predicates(
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
