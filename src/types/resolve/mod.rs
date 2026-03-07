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
struct TypeDefEntry {
    kind: TypeDefinitionKind,
    parameters: Vec<Path>,
    def: crate::ir::TypeDef,
}

#[derive(Debug, Clone)]
struct TraitDefEntry {
    span: Span,
    def: TraitDef,
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub module: Module<Type>,
    pub schemes: IndexMap<Path, TypeScheme>,
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
    resolve_module_with_symbols_and_schemes(symbols, module, logger).module
}

pub fn resolve_module_with_symbols_and_schemes(
    symbols: &mut SymbolTable,
    module: Module<()>,
    logger: &mut FileLogger,
) -> ResolvedModule {
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
    for (path, entry) in type_entries.iter() {
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

    let trait_entries =
        build_trait_definitions(&statements, &type_entries, &type_definitions, logger);
    register_trait_definitions(symbols, &trait_entries, logger);

    let mut env = TypeEnv::new();
    env.extend(
        symbols
            .terms()
            .iter()
            .map(|(path, scheme)| (path.clone(), scheme.clone())),
    );
    let constructors = build_sum_constructors(&type_entries, &type_definitions, logger);
    env.extend(constructors);

    let mut ctx = InferenceContext::new();
    let mut schemes = IndexMap::new();
    ctx.set_type_definitions(
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
                let output = match ctx.infer_term(&mut env, &term, &mut schemes) {
                    Ok(output) => output,
                    Err(error) => {
                        log_type_error(logger, error);
                        typed_statements.push(Statement::Term(fallback_term(&term)));
                        continue;
                    }
                };

                solve_predicates(logger, &mut ctx, symbols, term.span, &output.predicates);
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
                solve_predicates(logger, &mut ctx, symbols, term.span, &grounded_predicates);
                typed_statements.push(Statement::Term(normalize_term_types(
                    output.term,
                    ctx.table_mut(),
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
                    logger,
                    ctx: &mut ctx,
                    env: &mut env,
                    symbols,
                    schemes: &mut schemes,
                    type_entries: &type_entries,
                    type_definitions: &type_definitions,
                }
                .process(comments, trait_path, arguments, methods);
                typed_statements.push(typed_impl);
                typed_statements.extend(generated_terms.into_iter().map(Statement::Term));
            }
            Statement::Wasm(sexpr) => typed_statements.push(Statement::Wasm(sexpr)),
        }
    }

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::Logger;
    use crate::ir::TypeExprKind;

    fn type_expr(kind: TypeExprKind) -> TypeExpr {
        TypeExpr {
            comments: String::new(),
            kind,
            span: Span::Generated,
        }
    }

    #[test]
    fn instantiate_method_scheme_rejects_extra_type_arguments() {
        let scheme = Type::v(0).for_all(1).scheme();
        assert!(impls::instantiate_method_scheme(&scheme, &[Type::Integer]).is_some());
        assert!(
            impls::instantiate_method_scheme(&scheme, &[Type::Integer, Type::Boolean]).is_none()
        );
    }

    #[test]
    fn type_expr_in_def_alias_arity_mismatch_recovers_without_partial_instantiation() {
        let pair = Path::new("test", "Pair");
        let mut type_definitions = [(
            pair.clone(),
            TypeDefinition {
                parameters: 1,
                body: Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1),
                kind: TypeDefinitionKind::Alias,
            },
        )]
        .into_iter()
        .collect::<IndexMap<_, _>>();
        let expression = type_expr(TypeExprKind::Instantiation(
            pair,
            [
                type_expr(TypeExprKind::Instantiation(
                    Path::core("integer"),
                    [].into(),
                )),
                type_expr(TypeExprKind::Instantiation(
                    Path::core("boolean"),
                    [].into(),
                )),
            ]
            .into(),
        ));

        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let lowered = type_defs::type_expr_to_type_in_def(
            &expression,
            &HashMap::new(),
            &IndexMap::new(),
            &mut type_definitions,
            &mut Vec::new(),
            &mut file_logger,
        );

        assert_eq!(
            lowered,
            Type::Tuple(vec![Type::v(0), Type::v(0)]).for_all(1)
        );
    }

    #[test]
    fn type_expr_in_def_applied_parameter_recovers_to_parameter_type() {
        let a = Path::new("test", "a");
        let expression = type_expr(TypeExprKind::Instantiation(
            a.clone(),
            [type_expr(TypeExprKind::Instantiation(
                Path::core("integer"),
                [].into(),
            ))]
            .into(),
        ));

        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let lowered = type_defs::type_expr_to_type_in_def(
            &expression,
            &[(a, 0)].into_iter().collect(),
            &IndexMap::new(),
            &mut IndexMap::new(),
            &mut Vec::new(),
            &mut file_logger,
        );

        assert_eq!(lowered, Type::v(0));
    }

    #[test]
    fn type_expr_in_def_placeholder_recovers_to_unit() {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", "");
        let lowered = type_defs::type_expr_to_type_in_def(
            &type_expr(TypeExprKind::Placeholder),
            &HashMap::new(),
            &IndexMap::new(),
            &mut IndexMap::new(),
            &mut Vec::new(),
            &mut file_logger,
        );

        assert_eq!(lowered, Type::Unit);
    }
}
