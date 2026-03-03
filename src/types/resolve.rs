use std::collections::{
    HashMap,
    HashSet,
};
use std::convert::Infallible;

use indexmap::IndexMap;

use crate::ir::{
    Glob,
    ImplMethod,
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
    TypeError,
};
use super::instantiation::{
    instantiate_forall_strict,
    instantiate_predicates,
    leading_forall_count,
};
use super::type_expr::lower_type_expr;
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
            let definition = TypeDefinition {
                parameters: entry.parameters.len(),
                body: Type::Unit,
                kind: entry.kind,
            };
            symbols.insert_type(path.clone(), definition);
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
                typed_statements.push(Statement::Term(output.term));
            }
            Statement::Type {
                path,
                parameters,
                def,
                kind,
            } => {
                typed_statements.push(Statement::Type {
                    path,
                    parameters,
                    def,
                    kind,
                });
            }
            Statement::Trait {
                path,
                parameters,
                methods,
            } => {
                typed_statements.push(Statement::Trait {
                    path,
                    parameters,
                    methods,
                });
            }
            Statement::Impl {
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
                .process(trait_path, arguments, methods);
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

fn collect_type_entries(statements: &[Statement<()>]) -> IndexMap<Path, TypeDefEntry> {
    let mut entries = IndexMap::new();
    for statement in statements {
        let Statement::Type {
            path,
            parameters,
            def,
            kind,
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

fn type_definition_kind_from_decl_kind(kind: TypeDeclKind) -> TypeDefinitionKind {
    match kind {
        TypeDeclKind::Named => TypeDefinitionKind::Named,
        TypeDeclKind::Alias => TypeDefinitionKind::Alias,
    }
}

fn collect_term_definitions(statements: &[Statement<()>]) -> Vec<(Path, Span)> {
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

fn build_trait_definitions(
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

fn register_trait_definitions(
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

fn trait_method_term_scheme(
    trait_name: &Path,
    parameters: usize,
    method_scheme: &TypeScheme,
) -> TypeScheme {
    let mut predicates = method_scheme.predicates.clone();
    predicates.push(TraitRef::new(
        trait_name.clone(),
        type_vars_for_params(parameters),
    ));
    TypeScheme {
        predicates,
        type_: method_scheme.type_.clone(),
    }
}

struct ImplProcessingContext<'a> {
    logger: &'a mut FileLogger,
    ctx: &'a mut InferenceContext,
    env: &'a mut TypeEnv,
    symbols: &'a mut SymbolTable,
    schemes: &'a mut IndexMap<Path, TypeScheme>,
    type_entries: &'a IndexMap<Path, TypeDefEntry>,
    type_definitions: &'a IndexMap<Path, TypeDefinition>,
}

impl ImplProcessingContext<'_> {
    fn process(
        &mut self,
        trait_path: Path,
        arguments: Box<[TypeExpr]>,
        methods: Box<[ImplMethod<()>]>,
    ) -> (Statement<Type>, Vec<Term<Type>>) {
        let mut resolved_type_definitions = self.type_definitions.clone();
        let argument_types = arguments
            .iter()
            .map(|arg| {
                type_expr_to_type_in_def(
                    arg,
                    &HashMap::new(),
                    self.type_entries,
                    &mut resolved_type_definitions,
                    &mut Vec::new(),
                    self.logger,
                )
            })
            .collect::<Vec<_>>();

        let trait_definition = self.symbols.trait_defs().get(&trait_path).cloned();
        let mut typed_methods = Vec::new();
        let mut generated_terms = Vec::new();
        let mut method_map = IndexMap::new();

        for method in methods {
            let (mut typed_value, mut predicates) =
                match self.ctx.infer_term(self.env, &method.value, self.schemes) {
                    Ok(output) => (output.term, output.predicates),
                    Err(error) => {
                        log_type_error(self.logger, error);
                        (fallback_term(&method.value), Vec::new())
                    }
                };

            if let Some(definition) = trait_definition.as_ref()
                && let Some(method_scheme) = definition.methods.get(&method.trait_method)
            {
                let found = argument_types.len();
                if found == definition.parameters {
                    let expected = leading_forall_count(&method_scheme.type_);
                    if found > expected {
                        self.logger
                            .error("Invalid impl method type application")
                            .primary(
                                format!(
                                    "`{}` expects {expected} type arguments but got {found}.",
                                    method.trait_method
                                ),
                                method.span,
                            )
                            .done();
                    } else if let Some(instantiated) =
                        instantiate_method_scheme(method_scheme, &argument_types)
                    {
                        if let Err(error) = self
                            .ctx
                            .table_mut()
                            .unify(&typed_value.type_, &instantiated.type_)
                        {
                            log_type_error(
                                self.logger,
                                TypeError::Unification {
                                    error,
                                    span: method.span,
                                },
                            );
                        }
                        predicates.extend(instantiated.predicates);
                    } else {
                        log_trait_error(
                            self.logger,
                            method.span,
                            TraitError::InvalidInstance {
                                trait_name: trait_path.clone(),
                            },
                        );
                    }
                }
            }

            let normalized_type = self.ctx.table_mut().normalize(&typed_value.type_);
            typed_value.type_ = normalized_type.clone();
            solve_predicates(
                self.logger,
                self.ctx,
                self.symbols,
                method.span,
                &predicates,
            );

            let scheme =
                self.ctx
                    .generalize_with_predicates(&normalized_type, 0, predicates.clone());
            self.env.insert(method.impl_path.clone(), scheme.clone());
            self.schemes.insert(method.impl_path.clone(), scheme);

            method_map.insert(method.trait_method.clone(), method.impl_path.clone());
            generated_terms.push(build_impl_method_binding(
                method.impl_path.clone(),
                method.span,
                normalized_type,
                typed_value.clone(),
            ));
            typed_methods.push(ImplMethod {
                trait_method: method.trait_method,
                impl_path: method.impl_path,
                value: typed_value,
                span: method.span,
            });
        }

        let impl_span = typed_methods
            .first()
            .map(|method| method.span)
            .unwrap_or(Span::Generated);
        let trait_impl = TraitImpl {
            parameters: 0,
            head: TraitRef::new(trait_path.clone(), argument_types),
            predicates: Vec::new(),
            methods: method_map,
        };
        if let Err(error) = self.symbols.insert_impl(trait_impl) {
            log_trait_error(self.logger, impl_span, error);
        }

        (
            Statement::Impl {
                trait_path,
                arguments,
                methods: typed_methods.into_boxed_slice(),
            },
            generated_terms,
        )
    }
}

fn instantiate_method_scheme(
    scheme: &TypeScheme,
    arguments: &[Type],
) -> Option<TypeScheme> {
    if arguments.len() > leading_forall_count(&scheme.type_) {
        return None;
    }
    let type_ = instantiate_forall_strict(&scheme.type_, arguments)?;
    let predicates = instantiate_predicates(&scheme.predicates, arguments)?;
    Some(TypeScheme { predicates, type_ })
}

fn build_impl_method_binding(
    path: Path,
    span: Span,
    type_: Type,
    value: Term<Type>,
) -> Term<Type> {
    Term {
        comments: String::new(),
        kind: TermKind::Let {
            assignee: Pattern {
                comments: String::new(),
                kind: PatternKind::Identifier(path),
                span,
                type_,
            },
            scope: ScopeKind::Global,
            value: Box::new(value),
            then: Box::new(typed_unit_term()),
            else_: Box::new(typed_unreachable_term()),
        },
        span,
        type_: Type::Unit,
    }
}

fn typed_unit_term() -> Term<Type> {
    Term {
        comments: String::new(),
        kind: TermKind::Immediate(crate::ir::ImmediateValue::Unit),
        span: Span::Generated,
        type_: Type::Unit,
    }
}

fn typed_unreachable_term() -> Term<Type> {
    Term {
        comments: String::new(),
        kind: TermKind::Unreachable,
        span: Span::Generated,
        type_: Type::Unit,
    }
}

fn collect_constructor_definitions(
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

fn type_expr_to_type_in_def(
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
        lower_type_expr(expr, &mut |path, arguments, span| {
            Ok::<Type, Infallible>(self.lower_instantiation(path, arguments, span))
        })
        .unwrap_or_else(|never| match never {})
    }

    fn lower_instantiation(
        &mut self,
        path: &Path,
        arguments: Vec<Type>,
        span: Span,
    ) -> Type {
        if let Some(index) = self.param_map.get(path) {
            if !arguments.is_empty() {
                self.logger
                    .error("Type parameters cannot be applied")
                    .primary(
                        format!(
                            "`{}` is a type parameter but is applied to arguments.",
                            path
                        ),
                        span,
                    )
                    .done();
            }
            return Type::v(*index);
        }

        let definition = self.type_definitions.get(path).cloned().or_else(|| {
            self.entries.contains_key(path).then(|| {
                resolve_type_definition(
                    path,
                    self.entries,
                    self.type_definitions,
                    self.stack,
                    self.logger,
                )
            })
        });

        if let Some(definition) = definition {
            if definition.parameters != arguments.len() {
                self.logger
                    .error("Invalid type application")
                    .primary(
                        format!(
                            "`{}` expects {} type arguments but got {}.",
                            path,
                            definition.parameters,
                            arguments.len()
                        ),
                        span,
                    )
                    .done();
            }
            return match definition.kind {
                TypeDefinitionKind::Named => {
                    Type::Named {
                        name: path.clone(),
                        body: Box::new(definition.body),
                    }
                    .apply(arguments)
                }
                TypeDefinitionKind::Alias => {
                    instantiate_alias_type(&definition, &arguments).unwrap_or(definition.body)
                }
            };
        }

        Type::Named {
            name: path.clone(),
            body: Box::new(Type::Unit),
        }
        .apply(arguments)
    }
}

fn instantiate_alias_type(
    definition: &TypeDefinition,
    arguments: &[Type],
) -> Option<Type> {
    let argument_count = arguments.len().min(definition.parameters);
    instantiate_forall_strict(&definition.body, &arguments[..argument_count])
}

fn build_sum_constructors(
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
        .primary(format!("`{path}` is already defined."), span)
        .done();
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

fn predicate_is_ground(predicate: &TraitConstraint) -> bool {
    predicate.arguments.iter().all(type_is_ground)
}

fn type_is_ground(type_: &Type) -> bool {
    match type_ {
        Type::TypeVar(_) | Type::MetaVar(_) => false,
        _ => {
            let mut is_ground = true;
            for_each_child_type(type_, true, |child| {
                if is_ground && !type_is_ground(child) {
                    is_ground = false;
                }
            });
            is_ground
        }
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
                .primary(format!("`{path}` is not defined."), span)
                .done();
        }
        TraitError::DuplicateTrait(path) => {
            logger
                .error("Duplicate trait definition")
                .primary(format!("`{path}` is already defined."), span)
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
                        trait_name
                    ),
                    span,
                )
                .done();
        }
        TraitError::OverlappingInstance { trait_name, .. } => {
            logger
                .error("Overlapping trait instance")
                .primary(format!("Instances for `{trait_name}` overlap."), span)
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
                .primary(format!("Instance for `{trait_name}` is invalid."), span)
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
    error: TypeError,
) {
    match error {
        TypeError::UnknownIdentifier { path, span } => {
            logger
                .error("Unknown identifier")
                .primary(format!("`{path}` is not defined."), span)
                .done();
        }
        TypeError::UnknownConstructor { path, span } => {
            logger
                .error("Unknown constructor")
                .primary(format!("`{path}` is not defined."), span)
                .done();
        }
        TypeError::InvalidTypeApplication {
            name,
            expected,
            found,
            span,
        } => {
            logger
                .error("Invalid type application")
                .primary(
                    format!("`{name}` expects {expected} type arguments but got {found}."),
                    span,
                )
                .done();
        }
        TypeError::NotAFunction { type_, span } => {
            logger
                .error("Not a function")
                .primary(format!("`{type_}` is not callable."), span)
                .done();
        }
        TypeError::InvalidScheme { span } => {
            logger
                .error("Invalid type scheme")
                .primary("A type scheme could not be instantiated.", span)
                .done();
        }
        TypeError::Unification { error, span } => {
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
        TermKind::InlineWasm {
            asserted_type,
            definitions,
            instructions,
        } => {
            TermKind::InlineWasm {
                asserted_type: asserted_type.clone(),
                definitions: definitions.clone(),
                instructions: instructions.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiate_method_scheme_rejects_extra_type_arguments() {
        let scheme = Type::v(0).for_all(1).scheme();
        assert!(instantiate_method_scheme(&scheme, &[Type::Integer]).is_some());
        assert!(instantiate_method_scheme(&scheme, &[Type::Integer, Type::Boolean]).is_none());
    }
}
