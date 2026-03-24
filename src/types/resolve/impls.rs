//! Trait implementation checking and registration during resolve.

use std::collections::{
    HashMap,
    HashSet,
};

use indexmap::IndexMap;

use crate::ir::{
    ImmediateValue,
    ImplMethod,
    ImplTypeDef,
    TypeExpr,
};
use crate::logging::WithContext;

use super::super::infer::{
    InferenceContext,
    TypeEnv,
    TypeError,
};
use super::super::instantiation::{
    instantiate_forall_strict,
    instantiate_predicates,
    leading_forall_count,
    peel_leading_foralls,
};
use super::super::kind::{
    KindError,
    SchemeKindError,
    constructor_kind,
    infer_scheme_kind,
};
use super::super::unify::UnificationTable;
use super::super::{
    normalize_parameter_kinds,
    split_applied_type,
};
use super::diagnostics::{
    log_trait_error,
    log_type_error,
};
use super::recovery::{
    fallback_term,
    normalize_term_types,
};
use super::traits::solve_predicates_with_assumptions;
use super::type_defs::type_expr_to_scheme_in_def;
use super::{
    FileLogger,
    Kind,
    Path,
    Pattern,
    PatternKind,
    PendingTypeDefinitionEntry,
    ScopeKind,
    Span,
    Statement,
    SymbolTable,
    Term,
    TermKind,
    TraitConstraint,
    TraitError,
    TraitImpl,
    TraitRef,
    Type,
    TypeDefinition,
    TypeDefinitionKind,
    TypeScheme,
};

/// Mutable dependencies required while checking one `impl` block.
pub(super) struct ImplProcessingContext<'a> {
    pub(super) module_name: &'a str,
    pub(super) logger: &'a mut FileLogger,
    pub(super) inference_context: &'a mut InferenceContext,
    pub(super) type_environment: &'a mut TypeEnv,
    pub(super) symbols: &'a mut SymbolTable,
    pub(super) schemes: &'a mut IndexMap<Path, TypeScheme>,
    pub(super) pending_type_definitions: &'a IndexMap<Path, PendingTypeDefinitionEntry>,
    pub(super) type_definitions: &'a IndexMap<Path, TypeDefinition>,
}

#[derive(Debug, Clone)]
struct ImplMethodIdentity {
    trait_item_name: String,
    canonical_trait_method: Path,
    impl_path: Path,
    span: Span,
}

impl ImplProcessingContext<'_> {
    /// Type-check and register a single impl declaration.
    #[tracing::instrument(level = "debug", skip_all, fields(trait_path = %trait_path))]
    pub(super) fn process(
        &mut self,
        comments: String,
        trait_path: Path,
        arguments: Box<[TypeExpr]>,
        associated_types: Box<[ImplTypeDef]>,
        methods: Box<[ImplMethod<()>]>,
    ) -> (Statement<Type>, Vec<Term<Type>>) {
        let _profile_total = crate::profiling::scope("resolve.impl.process.total");
        let mut resolved_type_definitions = {
            let _profile = crate::profiling::scope("resolve.impl.clone_type_definitions");
            self.type_definitions.clone()
        };
        let canonical_trait_path = self
            .symbols
            .canonical_trait_path(&trait_path)
            .unwrap_or_else(|| trait_path.clone());
        let trait_definition = self.symbols.trait_definition(&trait_path).cloned();
        let associated_types_for_statement = associated_types.clone();

        let raw_argument_schemes = {
            let _profile = crate::profiling::scope("resolve.impl.argument_schemes");
            arguments
                .iter()
                .map(|arg| {
                    type_expr_to_scheme_in_def(
                        arg,
                        &HashMap::new(),
                        self.pending_type_definitions,
                        &mut resolved_type_definitions,
                        &mut Vec::new(),
                        self.logger,
                    )
                })
                .collect::<Vec<_>>()
        };
        let argument_kinds = {
            let _profile = crate::profiling::scope("resolve.impl.argument_kinds");
            arguments
                .iter()
                .zip(raw_argument_schemes.iter())
                .map(|(argument_expr, argument_scheme)| {
                    match infer_scheme_kind(
                        argument_scheme,
                        0,
                        &|type_path| {
                            resolved_type_definitions.get(type_path).map(|definition| {
                                constructor_kind(definition.parameters, &definition.parameter_kinds)
                            })
                        },
                        &|trait_name| {
                            let canonical = self
                                .symbols
                                .canonical_trait_path(trait_name)
                                .unwrap_or_else(|| trait_name.clone());
                            self.symbols.trait_defs().get(&canonical).map(|definition| {
                                normalize_parameter_kinds(
                                    definition.parameter_kinds.clone(),
                                    definition.parameters,
                                )
                            })
                        },
                    ) {
                        Ok(inferred) => inferred.kind,
                        Err(error) => {
                            log_impl_argument_kind_error(
                                self.logger,
                                &canonical_trait_path,
                                argument_expr.span,
                                error,
                            );
                            Kind::Type
                        }
                    }
                })
                .collect::<Vec<_>>()
        };
        let peeled_arguments = raw_argument_schemes
            .iter()
            .map(|scheme| {
                let (forall_count, body) = peel_leading_foralls(&scheme.type_);
                (forall_count, body, scheme.predicates.clone())
            })
            .collect::<Vec<_>>();
        let impl_parameters = peeled_arguments
            .iter()
            .map(|(count, ..)| *count)
            .sum::<usize>();
        let mut argument_types = Vec::new();
        let mut impl_context_predicates = Vec::new();
        let mut parameter_offset = 0usize;
        for (forall_count, body, predicates) in peeled_arguments {
            argument_types.push(normalize_impl_head_argument(
                &body,
                forall_count,
                parameter_offset,
                impl_parameters,
            ));
            for predicate in normalize_impl_head_predicates(
                &predicates,
                forall_count,
                parameter_offset,
                impl_parameters,
            ) {
                if !impl_context_predicates.contains(&predicate) {
                    impl_context_predicates.push(predicate);
                }
            }
            parameter_offset += forall_count;
        }

        let mut trait_head_kinds_valid = true;
        if let Some(trait_definition) = trait_definition.as_ref()
            && trait_definition.parameters == argument_kinds.len()
        {
            let expected_kinds = normalize_parameter_kinds(
                trait_definition.parameter_kinds.clone(),
                trait_definition.parameters,
            );
            for ((argument_kind, expected_kind), argument_expr) in argument_kinds
                .iter()
                .zip(expected_kinds.iter())
                .zip(arguments.iter())
            {
                if argument_kind == expected_kind {
                    continue;
                }
                trait_head_kinds_valid = false;
                log_trait_error(
                    self.logger,
                    argument_expr.span,
                    TraitError::KindMismatch {
                        trait_name: canonical_trait_path.clone(),
                        expected: expected_kind.clone(),
                        found: argument_kind.clone(),
                    },
                );
            }
        }

        let orphan_rule_satisfied = canonical_trait_path.major == self.module_name
            || argument_types
                .iter()
                .any(|argument| type_contains_local_nominal_type(argument, self.module_name));
        if !orphan_rule_satisfied {
            self.logger
                .error("Invalid trait instance")
                .primary(
                    format!(
                        "`{canonical_trait_path}` and all impl-head types are defined outside module `{}`. Define the trait locally or use at least one local named type in the impl head.",
                        self.module_name
                    ),
                    arguments
                        .first()
                        .map(|argument| argument.span)
                        .unwrap_or(Span::Generated),
                )
                .done();
        }

        let mut associated_type_assignments = IndexMap::new();
        for associated_type in associated_types.into_iter() {
            let canonical_associated_type = canonical_trait_path.child(&associated_type.name.inner);
            let lowered = type_expr_to_scheme_in_def(
                &associated_type.type_expr,
                &HashMap::new(),
                self.pending_type_definitions,
                &mut resolved_type_definitions,
                &mut Vec::new(),
                self.logger,
            );
            if !lowered.predicates.is_empty() {
                self.logger
                    .error("Invalid impl associated type")
                    .primary(
                        "Associated type definitions in impls cannot declare trait constraints.",
                        associated_type.span,
                    )
                    .done();
            }
            let (forall_count, body) = peel_leading_foralls(&lowered.type_);
            if forall_count > impl_parameters {
                self.logger
                    .error("Invalid impl associated type")
                    .primary(
                        format!(
                            "`{}` introduces {forall_count} local type parameters, but this impl head only binds {impl_parameters} parameter(s).",
                            canonical_associated_type
                        ),
                        associated_type.span,
                    )
                    .done();
            }
            let normalized = normalize_impl_head_argument(&body, forall_count, 0, impl_parameters);
            associated_type_assignments.insert(canonical_associated_type, normalized);
        }

        let mut typed_methods = Vec::new();
        let mut generated_terms = Vec::new();
        let mut method_map = IndexMap::new();
        let impl_head_predicate =
            TraitRef::new(canonical_trait_path.clone(), argument_types.clone());
        let method_identities = methods
            .iter()
            .map(|method| {
                let trait_item_name = trait_item_name(&method.trait_method).to_string();
                ImplMethodIdentity {
                    trait_item_name: trait_item_name.clone(),
                    canonical_trait_method: canonical_trait_path.sibling(trait_item_name),
                    impl_path: method.impl_path.clone(),
                    span: method.span,
                }
            })
            .collect::<Vec<_>>();
        let unguarded_cycles = find_unguarded_impl_method_cycles(&methods, &method_identities);
        for cycle in unguarded_cycles.iter() {
            log_unguarded_impl_method_cycle(self.logger, &method_identities, cycle);
        }
        let methods_in_unguarded_cycle = unguarded_cycles
            .iter()
            .flat_map(|cycle| cycle.iter().copied())
            .collect::<HashSet<_>>();
        let has_unguarded_cycles = !methods_in_unguarded_cycle.is_empty();
        let recursive_impl_method_types = method_identities
            .iter()
            .map(|method| {
                (
                    method.impl_path.clone(),
                    self.inference_context.fresh_meta(),
                )
            })
            .collect::<HashMap<_, _>>();
        self.type_environment.extend(
            recursive_impl_method_types
                .iter()
                .map(|(path, type_)| (path.clone(), TypeScheme::new(type_.clone()))),
        );

        for (method_index, method) in methods.into_iter().enumerate() {
            let method_identity = &method_identities[method_index];
            let canonical_trait_method = method_identity.canonical_trait_method.clone();

            if methods_in_unguarded_cycle.contains(&method_index) {
                let typed_value = normalize_term_types(
                    fallback_term(&method.value),
                    self.inference_context.table_mut(),
                );
                let normalized_type = typed_value.type_.clone();
                let scheme = trait_definition
                    .as_ref()
                    .and_then(|trait_definition| {
                        (argument_types.len() == trait_definition.parameters)
                            .then_some(())
                            .and_then(|_| trait_definition.methods.get(&canonical_trait_method))
                            .and_then(|method_scheme| {
                                instantiate_method_scheme(method_scheme, &argument_types)
                            })
                            .map(|instantiated| {
                                let mut instantiated = substitute_impl_associated_types_in_scheme(
                                    instantiated,
                                    &impl_head_predicate,
                                    &associated_type_assignments,
                                );
                                for predicate in impl_context_predicates.iter().cloned() {
                                    if !instantiated.predicates.contains(&predicate) {
                                        instantiated.predicates.push(predicate);
                                    }
                                }
                                instantiated
                            })
                    })
                    .unwrap_or_else(|| TypeScheme::new(normalized_type.clone()));
                self.type_environment
                    .insert(method.impl_path.clone(), scheme.clone());
                self.schemes.insert(method.impl_path.clone(), scheme);

                method_map.insert(canonical_trait_method.clone(), method.impl_path.clone());
                generated_terms.push(build_impl_method_binding(
                    method.impl_path.clone(),
                    method.span,
                    normalized_type,
                    typed_value.clone(),
                ));
                typed_methods.push(ImplMethod {
                    trait_method: canonical_trait_method,
                    impl_path: method.impl_path,
                    value: typed_value,
                    span: method.span,
                });
                continue;
            }

            let mut method_declared_predicates = Vec::new();
            let inference_result = {
                let _profile = crate::profiling::scope("resolve.impl.method_infer_term");
                self.inference_context.infer_term(
                    self.type_environment,
                    &method.value,
                    self.schemes,
                )
            };
            let (mut typed_value, inferred_predicates) = match inference_result {
                Ok(output) => (output.term, output.predicates),
                Err(error) => {
                    log_type_error(self.logger, error);
                    (fallback_term(&method.value), Vec::new())
                }
            };

            if let Some(trait_definition) = trait_definition.as_ref()
                && let Some(method_scheme) = trait_definition.methods.get(&canonical_trait_method)
            {
                let found = argument_types.len();
                if found == trait_definition.parameters {
                    let expected = leading_forall_count(&method_scheme.type_);
                    if found > expected {
                        self.logger
                            .error("Invalid impl trait item type application")
                            .primary(
                                format!(
                                    "`{}` expects {expected} type arguments but got {found}.",
                                    canonical_trait_method
                                ),
                                method.span,
                            )
                            .done();
                    } else if let Some(instantiated) =
                        instantiate_method_scheme(method_scheme, &argument_types)
                    {
                        let instantiated = substitute_impl_associated_types_in_scheme(
                            instantiated,
                            &impl_head_predicate,
                            &associated_type_assignments,
                        );
                        tracing::debug!(
                            method = %canonical_trait_method,
                            instantiated = %instantiated.type_.pretty(),
                            "instantiated method scheme for impl check",
                        );
                        let expected_type = {
                            let _profile =
                                crate::profiling::scope("resolve.impl.normalize_expected_type");
                            normalize_alias_applications(
                                instantiate_forall_for_impl_check(
                                    self.inference_context,
                                    instantiated.type_.clone(),
                                    method.span,
                                ),
                                &resolved_type_definitions,
                            )
                        };
                        let value_type = {
                            let _profile =
                                crate::profiling::scope("resolve.impl.normalize_value_type");
                            let normalized = normalize_alias_applications(
                                instantiate_forall_for_impl_check(
                                    self.inference_context,
                                    typed_value.type_.clone(),
                                    method.span,
                                ),
                                &resolved_type_definitions,
                            );
                            substitute_impl_associated_types_in_type(
                                normalized,
                                &impl_head_predicate,
                                &associated_type_assignments,
                            )
                        };
                        tracing::debug!(
                            expected = %expected_type.pretty(),
                            inferred = %value_type.pretty(),
                            "impl method type check",
                        );
                        if let Err(error) = {
                            let _profile =
                                crate::profiling::scope("resolve.impl.unify_method_type");
                            self.inference_context
                                .table_mut()
                                .unify(&value_type, &expected_type)
                        } {
                            log_type_error(
                                self.logger,
                                TypeError::Unification {
                                    error,
                                    span: method.span,
                                    context: Some(
                                        "checking this impl method body against the trait method type",
                                    ),
                                },
                            );
                        }
                        for predicate in instantiated.predicates {
                            if !method_declared_predicates.contains(&predicate) {
                                method_declared_predicates.push(predicate);
                            }
                        }
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

            let mut solve_predicates = inferred_predicates.clone();
            for predicate in method_declared_predicates.iter().cloned() {
                if !solve_predicates.contains(&predicate) {
                    solve_predicates.push(predicate);
                }
            }
            for predicate in impl_context_predicates.iter().cloned() {
                if !solve_predicates.contains(&predicate) {
                    solve_predicates.push(predicate);
                }
            }

            let mut predicate_assumptions = method_declared_predicates.clone();
            for predicate in impl_context_predicates.iter().cloned() {
                if !predicate_assumptions.contains(&predicate) {
                    predicate_assumptions.push(predicate);
                }
            }
            if !predicate_assumptions.iter().any(|predicate| {
                predicate_matches_impl_head(self.symbols, predicate, &impl_head_predicate)
            }) {
                predicate_assumptions.push(impl_head_predicate.clone());
            }

            typed_value = normalize_term_types(typed_value, self.inference_context.table_mut());
            let normalized_type = typed_value.type_.clone();
            if let Some(recursive_method_type) = recursive_impl_method_types.get(&method.impl_path)
                && let Err(error) = self
                    .inference_context
                    .table_mut()
                    .unify(recursive_method_type, &normalized_type)
            {
                log_type_error(
                    self.logger,
                    TypeError::Unification {
                        error,
                        span: method.span,
                        context: Some(
                            "checking recursive impl method references against the method body type",
                        ),
                    },
                );
            }
            {
                let _profile = crate::profiling::scope("resolve.impl.solve_predicates");
                solve_predicates_with_assumptions(
                    self.logger,
                    self.inference_context,
                    self.symbols,
                    method.span,
                    &solve_predicates,
                    &predicate_assumptions,
                );
            }

            let mut scheme_predicates = inferred_predicates;
            scheme_predicates.retain(|predicate| {
                !predicate_matches_impl_head(self.symbols, predicate, &impl_head_predicate)
            });
            for predicate in method_declared_predicates {
                if !scheme_predicates.contains(&predicate) {
                    scheme_predicates.push(predicate);
                }
            }
            for predicate in impl_context_predicates.iter().cloned() {
                if !scheme_predicates.contains(&predicate) {
                    scheme_predicates.push(predicate);
                }
            }

            let scheme = self.inference_context.generalize_with_predicates(
                &normalized_type,
                0,
                scheme_predicates,
            );
            self.type_environment
                .insert(method.impl_path.clone(), scheme.clone());
            self.schemes.insert(method.impl_path.clone(), scheme);

            method_map.insert(canonical_trait_method.clone(), method.impl_path.clone());
            generated_terms.push(build_impl_method_binding(
                method.impl_path.clone(),
                method.span,
                normalized_type,
                typed_value.clone(),
            ));
            typed_methods.push(ImplMethod {
                trait_method: canonical_trait_method,
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
            parameters: impl_parameters,
            head: TraitRef::new(canonical_trait_path.clone(), argument_types),
            predicates: impl_context_predicates,
            associated_types: associated_type_assignments,
            methods: method_map,
        };
        if orphan_rule_satisfied
            && trait_head_kinds_valid
            && !has_unguarded_cycles
            && let Err(error) = self.symbols.insert_impl(trait_impl)
        {
            log_trait_error(
                self.logger,
                impl_error_span(&error, &typed_methods).unwrap_or(impl_span),
                error,
            );
        }

        (
            Statement::Impl {
                comments,
                trait_path,
                arguments,
                associated_types: associated_types_for_statement,
                methods: typed_methods.into_boxed_slice(),
            },
            generated_terms,
        )
    }
}

fn impl_error_span(
    error: &TraitError,
    typed_methods: &[ImplMethod<Type>],
) -> Option<Span> {
    match error {
        TraitError::InvalidInstanceItems { unknown_items, .. } => {
            unknown_items.iter().find_map(|method_path| {
                typed_methods
                    .iter()
                    .find(|method| method.trait_method == *method_path)
                    .map(|method| method.span)
            })
        }
        _ => None,
    }
}

/// Instantiate a trait-item scheme using impl-head arguments.
pub(super) fn instantiate_method_scheme(
    scheme: &TypeScheme,
    arguments: &[Type],
) -> Option<TypeScheme> {
    let count = leading_forall_count(&scheme.type_);
    if arguments.len() > count {
        return None;
    }
    let type_ = instantiate_forall_strict(&scheme.type_, arguments)?;
    let predicates = instantiate_predicates(&scheme.predicates, arguments)?;
    Some(TypeScheme { predicates, type_ })
}

fn substitute_impl_associated_types_in_scheme(
    scheme: TypeScheme,
    impl_head: &TraitRef,
    associated_types: &IndexMap<Path, Type>,
) -> TypeScheme {
    TypeScheme {
        type_: substitute_impl_associated_types_in_type(scheme.type_, impl_head, associated_types),
        predicates: scheme
            .predicates
            .into_iter()
            .map(|predicate| {
                TraitRef {
                    trait_name: predicate.trait_name,
                    arguments: predicate
                        .arguments
                        .into_iter()
                        .map(|argument| {
                            substitute_impl_associated_types_in_type(
                                argument,
                                impl_head,
                                associated_types,
                            )
                        })
                        .collect(),
                }
            })
            .collect(),
    }
}

fn substitute_impl_associated_types_in_type(
    type_: Type,
    impl_head: &TraitRef,
    associated_types: &IndexMap<Path, Type>,
) -> Type {
    let normalized = match type_ {
        Type::Unit
        | Type::Integer
        | Type::Natural
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_) => type_,
        Type::ForAll { name, body } => {
            Type::ForAll {
                name,
                body: Box::new(substitute_impl_associated_types_in_type(
                    *body,
                    impl_head,
                    associated_types,
                )),
            }
        }
        Type::Named { name, body } => Type::Named { name, body },
        Type::StructConstraint { fields, mode } => {
            Type::StructConstraint {
                fields: fields
                    .into_iter()
                    .map(|(name, field_type)| {
                        (
                            name,
                            substitute_impl_associated_types_in_type(
                                field_type,
                                impl_head,
                                associated_types,
                            ),
                        )
                    })
                    .collect(),
                mode,
            }
        }
        Type::Struct { fields } => {
            Type::Struct {
                fields: fields
                    .into_iter()
                    .map(|(name, field_type)| {
                        (
                            name,
                            substitute_impl_associated_types_in_type(
                                field_type,
                                impl_head,
                                associated_types,
                            ),
                        )
                    })
                    .collect(),
            }
        }
        Type::Array(inner) => {
            Type::Array(Box::new(substitute_impl_associated_types_in_type(
                *inner,
                impl_head,
                associated_types,
            )))
        }
        Type::Tuple(items) => {
            Type::Tuple(
                items
                    .into_iter()
                    .map(|item| {
                        substitute_impl_associated_types_in_type(item, impl_head, associated_types)
                    })
                    .collect(),
            )
        }
        Type::Sum { variants } => {
            Type::Sum {
                variants: variants
                    .into_iter()
                    .map(|(name, variant_type)| {
                        (
                            name,
                            substitute_impl_associated_types_in_type(
                                variant_type,
                                impl_head,
                                associated_types,
                            ),
                        )
                    })
                    .collect(),
            }
        }
        Type::Function(parameter, result) => {
            Type::func(
                substitute_impl_associated_types_in_type(*parameter, impl_head, associated_types),
                substitute_impl_associated_types_in_type(*result, impl_head, associated_types),
            )
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            let constructor =
                substitute_impl_associated_types_in_type(*constructor, impl_head, associated_types);
            let arguments = arguments
                .into_iter()
                .map(|argument| {
                    substitute_impl_associated_types_in_type(argument, impl_head, associated_types)
                })
                .collect::<Vec<_>>();
            if let Type::Named {
                name: associated_type,
                ..
            } = &constructor
                && arguments == impl_head.arguments
                && let Some(assigned_type) = associated_types.get(associated_type)
            {
                return assigned_type.clone();
            }
            constructor.apply(arguments)
        }
    };
    normalized
}

fn instantiate_forall_for_impl_check(
    inference_context: &mut InferenceContext,
    type_: Type,
    span: Span,
) -> Type {
    match type_ {
        forall @ Type::ForAll { .. } => {
            inference_context
                .instantiate(&TypeScheme::new(forall.clone()), span)
                .unwrap_or(forall)
        }
        other => other,
    }
}

fn normalize_alias_applications(
    type_: Type,
    type_definitions: &IndexMap<Path, TypeDefinition>,
) -> Type {
    let normalized = match type_ {
        Type::Unit
        | Type::Integer
        | Type::Natural
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_) => type_,
        Type::ForAll { name, body } => {
            Type::ForAll {
                name,
                body: Box::new(normalize_alias_applications(*body, type_definitions)),
            }
        }
        Type::Named { name, body } => Type::Named { name, body },
        Type::StructConstraint { fields, mode } => {
            Type::StructConstraint {
                fields: fields
                    .into_iter()
                    .map(|(name, field_type)| {
                        (
                            name,
                            normalize_alias_applications(field_type, type_definitions),
                        )
                    })
                    .collect(),
                mode,
            }
        }
        Type::Struct { fields } => {
            Type::Struct {
                fields: fields
                    .into_iter()
                    .map(|(name, field_type)| {
                        (
                            name,
                            normalize_alias_applications(field_type, type_definitions),
                        )
                    })
                    .collect(),
            }
        }
        Type::Array(inner) => {
            Type::Array(Box::new(normalize_alias_applications(
                *inner,
                type_definitions,
            )))
        }
        Type::Tuple(items) => {
            Type::Tuple(
                items
                    .into_iter()
                    .map(|item| normalize_alias_applications(item, type_definitions))
                    .collect(),
            )
        }
        Type::Sum { variants } => {
            Type::Sum {
                variants: variants
                    .into_iter()
                    .map(|(name, variant_type)| {
                        (
                            name,
                            normalize_alias_applications(variant_type, type_definitions),
                        )
                    })
                    .collect(),
            }
        }
        Type::Function(parameter, result) => {
            Type::func(
                normalize_alias_applications(*parameter, type_definitions),
                normalize_alias_applications(*result, type_definitions),
            )
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            Type::Apply {
                constructor: Box::new(normalize_alias_applications(*constructor, type_definitions)),
                arguments: arguments
                    .into_iter()
                    .map(|argument| normalize_alias_applications(argument, type_definitions))
                    .collect(),
            }
        }
    };
    normalize_alias_application_root(normalized, type_definitions)
}

fn normalize_alias_application_root(
    type_: Type,
    type_definitions: &IndexMap<Path, TypeDefinition>,
) -> Type {
    let (base, arguments) = split_applied_type(type_);
    let Type::Named { name, body } = base else {
        return apply_arguments(base, arguments);
    };
    let Some(definition) = type_definitions.get(&name) else {
        return apply_arguments(Type::Named { name, body }, arguments);
    };
    if definition.kind != TypeDefinitionKind::Alias || arguments.len() != definition.parameters {
        return apply_arguments(Type::Named { name, body }, arguments);
    }
    instantiate_forall_strict(&body, &arguments)
        .map(|expanded| normalize_alias_applications(expanded, type_definitions))
        .unwrap_or_else(|| apply_arguments(Type::Named { name, body }, arguments))
}

fn apply_arguments(
    constructor: Type,
    arguments: Vec<Type>,
) -> Type {
    constructor.apply(arguments)
}

/// Shift impl-head arguments so De Bruijn indices line up after peeling foralls.
fn normalize_impl_head_argument(
    body: &Type,
    forall_count: usize,
    parameter_offset: usize,
    total_parameters: usize,
) -> Type {
    let params_after = total_parameters.saturating_sub(parameter_offset + forall_count);
    body.shift_type_vars(params_after as i32, 0)
        .unwrap_or_else(|| body.clone())
}

fn normalize_impl_head_predicates(
    predicates: &[TraitConstraint],
    forall_count: usize,
    parameter_offset: usize,
    total_parameters: usize,
) -> Vec<TraitConstraint> {
    let params_after = total_parameters.saturating_sub(parameter_offset + forall_count);
    predicates
        .iter()
        .map(|predicate| {
            TraitRef {
                trait_name: predicate.trait_name.clone(),
                arguments: predicate
                    .arguments
                    .iter()
                    .map(|argument| {
                        argument
                            .shift_type_vars(params_after as i32, 0)
                            .unwrap_or_else(|| argument.clone())
                    })
                    .collect(),
            }
        })
        .collect()
}

fn predicate_matches_impl_head(
    symbols: &SymbolTable,
    predicate: &TraitConstraint,
    impl_head: &TraitRef,
) -> bool {
    let mut table = UnificationTable::default();
    let mut normalized_predicate = table.normalize_trait_ref(predicate);
    let mut normalized_head = table.normalize_trait_ref(impl_head);
    if let Some(canonical_trait_name) =
        symbols.canonical_trait_path(&normalized_predicate.trait_name)
    {
        normalized_predicate.trait_name = canonical_trait_name;
    }
    if let Some(canonical_trait_name) = symbols.canonical_trait_path(&normalized_head.trait_name) {
        normalized_head.trait_name = canonical_trait_name;
    }
    if normalized_predicate.trait_name != normalized_head.trait_name
        || normalized_predicate.arguments.len() != normalized_head.arguments.len()
    {
        return false;
    }
    normalized_predicate
        .arguments
        .iter()
        .zip(normalized_head.arguments.iter())
        .all(|(left, right)| table.unify(left, right).is_ok())
}

fn trait_item_name(path: &Path) -> &str {
    path.minor
        .rsplit_once(Path::DELIMETER)
        .map(|(_, name)| name)
        .unwrap_or(path.minor.as_str())
}

fn find_unguarded_impl_method_cycles(
    methods: &[ImplMethod<()>],
    method_identities: &[ImplMethodIdentity],
) -> Vec<Vec<usize>> {
    let impl_path_indices = method_identities
        .iter()
        .enumerate()
        .map(|(index, method)| (method.impl_path.clone(), index))
        .collect::<HashMap<_, _>>();
    let trait_method_indices = method_identities
        .iter()
        .enumerate()
        .map(|(index, method)| (method.canonical_trait_method.clone(), index))
        .collect::<HashMap<_, _>>();

    let adjacency = methods
        .iter()
        .map(|method| {
            let mut references = HashSet::new();
            if let TermKind::Identifier(path) = &method.value.kind
                && let Some(index) = trait_method_indices.get(path).copied()
            {
                references.insert(index);
            }
            collect_unguarded_impl_method_refs(
                &method.value,
                false,
                &impl_path_indices,
                &mut references,
            );
            let mut edges = references.into_iter().collect::<Vec<_>>();
            edges.sort_unstable();
            edges
        })
        .collect::<Vec<_>>();

    let mut cycles = strongly_connected_components(&adjacency)
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || component
                    .first()
                    .is_some_and(|index| adjacency[*index].contains(index))
        })
        .collect::<Vec<_>>();
    cycles.sort_by_key(|component| component.first().copied().unwrap_or(usize::MAX));
    cycles
}

fn collect_unguarded_impl_method_refs(
    term: &Term<()>,
    inside_function: bool,
    impl_path_indices: &HashMap<Path, usize>,
    references: &mut HashSet<usize>,
) {
    match &term.kind {
        TermKind::Identifier(path) => {
            if inside_function {
                return;
            }
            if let Some(index) = impl_path_indices.get(path).copied() {
                references.insert(index);
            }
        }
        TermKind::Let {
            value, then, else_, ..
        } => {
            collect_unguarded_impl_method_refs(
                value,
                inside_function,
                impl_path_indices,
                references,
            );
            collect_unguarded_impl_method_refs(
                then,
                inside_function,
                impl_path_indices,
                references,
            );
            collect_unguarded_impl_method_refs(
                else_,
                inside_function,
                impl_path_indices,
                references,
            );
        }
        TermKind::Tuple(items) => {
            for item in items {
                collect_unguarded_impl_method_refs(
                    item,
                    inside_function,
                    impl_path_indices,
                    references,
                );
            }
        }
        TermKind::Struct(fields) => {
            for value in fields.values() {
                collect_unguarded_impl_method_refs(
                    value,
                    inside_function,
                    impl_path_indices,
                    references,
                );
            }
        }
        TermKind::Field { of, .. } => {
            collect_unguarded_impl_method_refs(of, inside_function, impl_path_indices, references);
        }
        TermKind::Function { body, .. } => {
            collect_unguarded_impl_method_refs(body, true, impl_path_indices, references);
        }
        TermKind::Call { callee, argument } => {
            collect_unguarded_impl_method_refs(
                callee,
                inside_function,
                impl_path_indices,
                references,
            );
            collect_unguarded_impl_method_refs(
                argument,
                inside_function,
                impl_path_indices,
                references,
            );
        }
        TermKind::Semicolon(left, right) => {
            collect_unguarded_impl_method_refs(
                left,
                inside_function,
                impl_path_indices,
                references,
            );
            collect_unguarded_impl_method_refs(
                right,
                inside_function,
                impl_path_indices,
                references,
            );
        }
        TermKind::Immediate(_) | TermKind::InlineWasm { .. } | TermKind::Unreachable => {}
    }
}

fn strongly_connected_components(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    fn dfs_post_order(
        node: usize,
        adjacency: &[Vec<usize>],
        visited: &mut [bool],
        order: &mut Vec<usize>,
    ) {
        visited[node] = true;
        for edge in adjacency[node].iter().copied() {
            if edge < adjacency.len() && !visited[edge] {
                dfs_post_order(edge, adjacency, visited, order);
            }
        }
        order.push(node);
    }

    fn dfs_component(
        node: usize,
        adjacency: &[Vec<usize>],
        visited: &mut [bool],
        component: &mut Vec<usize>,
    ) {
        visited[node] = true;
        component.push(node);
        for edge in adjacency[node].iter().copied() {
            if edge < adjacency.len() && !visited[edge] {
                dfs_component(edge, adjacency, visited, component);
            }
        }
    }

    let mut visited = vec![false; adjacency.len()];
    let mut order = Vec::with_capacity(adjacency.len());
    for node in 0..adjacency.len() {
        if visited[node] {
            continue;
        }
        dfs_post_order(node, adjacency, &mut visited, &mut order);
    }

    let mut reversed = vec![Vec::new(); adjacency.len()];
    for (from, edges) in adjacency.iter().enumerate() {
        for edge in edges {
            if *edge < adjacency.len() {
                reversed[*edge].push(from);
            }
        }
    }

    visited.fill(false);
    let mut components = Vec::new();
    while let Some(node) = order.pop() {
        if visited[node] {
            continue;
        }
        let mut component = Vec::new();
        dfs_component(node, &reversed, &mut visited, &mut component);
        component.sort_unstable();
        components.push(component);
    }

    components
}

fn log_unguarded_impl_method_cycle(
    logger: &mut FileLogger,
    method_identities: &[ImplMethodIdentity],
    cycle: &[usize],
) {
    let Some((&first, others)) = cycle.split_first() else {
        return;
    };
    let cycle_text = format_unguarded_impl_method_cycle(method_identities, cycle);
    let mut builder = logger.error("Invalid circular impl definition").primary(
        format!(
            "`{}` is part of unguarded circular definition `{cycle_text}`.",
            method_identities[first].trait_item_name
        ),
        method_identities[first].span,
    );
    for index in others.iter().copied() {
        builder = builder.secondary(
            format!(
                "`{}` also participates in this cycle.",
                method_identities[index].trait_item_name
            ),
            method_identities[index].span,
        );
    }
    builder
        .note("Recursive impl definitions must be function-guarded, for example `let f = fn x => ...`.")
        .done();
}

fn format_unguarded_impl_method_cycle(
    method_identities: &[ImplMethodIdentity],
    cycle: &[usize],
) -> String {
    let mut names = cycle
        .iter()
        .map(|index| method_identities[*index].trait_item_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    if let Some(first) = names.first().cloned() {
        names.push(first);
    }
    names.join(" -> ")
}

fn type_contains_local_nominal_type(
    type_: &Type,
    module_name: &str,
) -> bool {
    match type_ {
        Type::Named { name, .. } => name.major == module_name,
        Type::Array(inner) | Type::ForAll { body: inner, .. } => {
            type_contains_local_nominal_type(inner, module_name)
        }
        Type::Tuple(items) => {
            items
                .iter()
                .any(|item| type_contains_local_nominal_type(item, module_name))
        }
        Type::Struct { fields } => {
            fields
                .values()
                .any(|item| type_contains_local_nominal_type(item, module_name))
        }
        Type::Sum { variants } => {
            variants
                .values()
                .any(|item| type_contains_local_nominal_type(item, module_name))
        }
        Type::Function(parameter, result) => {
            type_contains_local_nominal_type(parameter, module_name)
                || type_contains_local_nominal_type(result, module_name)
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            type_contains_local_nominal_type(constructor, module_name)
                || arguments
                    .iter()
                    .any(|item| type_contains_local_nominal_type(item, module_name))
        }
        Type::StructConstraint { fields, .. } => {
            fields
                .values()
                .any(|item| type_contains_local_nominal_type(item, module_name))
        }
        Type::Unit
        | Type::Integer
        | Type::Natural
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_) => false,
    }
}

fn log_impl_argument_kind_error(
    logger: &mut FileLogger,
    trait_name: &Path,
    span: Span,
    error: SchemeKindError,
) {
    match error {
        SchemeKindError::Kind(kind_error) => {
            let message = match kind_error {
                KindError::Mismatch { left, right } => {
                    format!(
                        "Trait argument for `{trait_name}` has incompatible kinds `{left}` and `{right}`."
                    )
                }
                KindError::Occurs { in_kind, .. } => {
                    format!("Trait argument for `{trait_name}` has recursive kind `{in_kind}`.")
                }
            };
            logger
                .error("Invalid trait argument kind")
                .primary(message, span)
                .done();
        }
        SchemeKindError::PredicateArityMismatch {
            trait_name,
            expected,
            found,
        } => {
            logger
                .error("Invalid trait constraint application")
                .primary(
                    format!("`{trait_name}` expects {expected} type arguments but got {found}."),
                    span,
                )
                .done();
        }
        SchemeKindError::PredicateKindMismatch {
            trait_name,
            expected,
            found,
        } => {
            logger
                .error("Invalid trait constraint kind")
                .primary(
                    format!(
                        "`{trait_name}` expects kind `{expected}` but this argument has kind `{found}`."
                    ),
                    span,
                )
                .done();
        }
    }
}

/// Build a generated top-level binding that materializes an impl item.
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

/// Typed `()` fallback used in generated impl bindings.
fn typed_unit_term() -> Term<Type> {
    Term {
        comments: String::new(),
        kind: TermKind::Immediate(ImmediateValue::Unit),
        span: Span::Generated,
        type_: Type::Unit,
    }
}

/// Typed unreachable fallback used in generated impl bindings.
fn typed_unreachable_term() -> Term<Type> {
    Term {
        comments: String::new(),
        kind: TermKind::Unreachable,
        span: Span::Generated,
        type_: Type::Unit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WithSpan;

    #[test]
    fn instantiate_method_scheme_handles_partial_exact_and_excess_args() {
        let scheme = TypeScheme {
            predicates: vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])],
            type_: Type::func(Type::v(0), Type::v(0)).for_all(1),
        };

        let partial = instantiate_method_scheme(&scheme, &[]).expect("partial should succeed");
        assert_eq!(partial.type_, scheme.type_);

        let exact = instantiate_method_scheme(&scheme, &[Type::Integer])
            .expect("exact instantiation should succeed");
        assert_eq!(exact.type_, Type::func(Type::Integer, Type::Integer));
        assert_eq!(
            exact.predicates,
            vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::Integer])]
        );

        assert!(instantiate_method_scheme(&scheme, &[Type::Integer, Type::Boolean]).is_none());
    }

    #[test]
    fn impl_error_span_points_at_unknown_impl_item() {
        let unknown_method = Path::new("demo", "flatmap");
        let methods = vec![
            ImplMethod {
                trait_method: Path::new("demo", "new"),
                impl_path: Path::new("demo", "impl_new"),
                value: typed_unit_term(),
                span: Span::new(1, 3),
            },
            ImplMethod {
                trait_method: unknown_method.clone(),
                impl_path: Path::new("demo", "impl_flatmap"),
                value: typed_unit_term(),
                span: Span::new(10, 7),
            },
        ];

        let error = TraitError::InvalidInstanceItems {
            trait_name: Path::new("demo", "Monad"),
            unknown_items: vec![unknown_method],
            missing_items: vec![Path::new("demo", "flat_map")],
            unknown_associated_types: Vec::new(),
            missing_associated_types: Vec::new(),
        };

        assert_eq!(impl_error_span(&error, &methods), Some(Span::new(10, 7)));
    }

    #[test]
    fn instantiate_method_scheme_preserves_curried_shape_after_impl_head_substitution() {
        let m_applied_to_b = Type::v(2).apply(vec![Type::v(0)]);
        let m_applied_to_a = Type::v(2).apply(vec![Type::v(1)]);
        let scheme = TypeScheme {
            predicates: Vec::new(),
            type_: Type::func(
                Type::func(Type::v(1), m_applied_to_b.clone()),
                Type::func(m_applied_to_a.clone(), m_applied_to_b),
            )
            .for_all(3),
        };

        let option_constructor = Type::Named {
            name: Path::new("core", "Option"),
            body: Box::new(Type::Unit),
        };
        let partially_instantiated =
            instantiate_method_scheme(&scheme, std::slice::from_ref(&option_constructor))
                .expect("impl-head argument should instantiate outer forall");

        let mut inference_context = InferenceContext::new();
        let fully_instantiated = instantiate_forall_for_impl_check(
            &mut inference_context,
            partially_instantiated.type_,
            Span::Generated,
        );

        let Type::Function(_, result) = fully_instantiated else {
            panic!("expected flat_map method type to stay curried");
        };
        assert!(matches!(*result, Type::Function(_, _)));
    }

    #[test]
    fn normalize_impl_head_argument_and_predicates_shift_indices() {
        let argument = normalize_impl_head_argument(&Type::func(Type::v(1), Type::v(0)), 1, 0, 2);
        assert_eq!(argument, Type::func(Type::v(2), Type::v(1)));

        let predicates = normalize_impl_head_predicates(
            &[TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::Tuple(vec![Type::v(1), Type::v(0)])],
            )],
            1,
            0,
            2,
        );
        assert_eq!(
            predicates,
            vec![TraitRef::new(
                Path::new("demo", "Eq"),
                vec![Type::Tuple(vec![Type::v(2), Type::v(1)])],
            )]
        );
    }

    #[test]
    fn type_contains_local_nominal_type_searches_nested_types() {
        let local = Type::Named {
            name: Path::new("demo", "Token"),
            body: Box::new(Type::Unit),
        };
        let nested = Type::func(
            Type::Tuple(vec![Type::Integer, local.clone()]),
            Type::Array(Box::new(Type::Boolean)),
        );

        assert!(type_contains_local_nominal_type(&local, "demo"));
        assert!(type_contains_local_nominal_type(&nested, "demo"));
        assert!(!type_contains_local_nominal_type(&nested, "other"));
    }

    #[test]
    fn normalize_alias_applications_expands_fully_applied_aliases() {
        let pair = Path::new("demo", "Pair");
        let pair_body = Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2);
        let definitions = [(
            pair.clone(),
            TypeDefinition {
                parameters: 2,
                parameter_kinds: vec![Kind::Type, Kind::Type],
                body: pair_body.clone(),
                kind: TypeDefinitionKind::Alias,
            },
        )]
        .into_iter()
        .collect::<IndexMap<_, _>>();

        let applied = Type::Named {
            name: pair,
            body: Box::new(pair_body),
        }
        .apply(vec![Type::Integer, Type::Boolean]);

        assert_eq!(
            normalize_alias_applications(applied, &definitions),
            Type::Tuple(vec![Type::Integer, Type::Boolean])
        );
    }

    #[test]
    fn normalize_alias_applications_preserves_partial_alias_constructors() {
        let pair = Path::new("demo", "Pair");
        let pair_body = Type::Tuple(vec![Type::v(1), Type::v(0)]).for_all(2);
        let definitions = [(
            pair.clone(),
            TypeDefinition {
                parameters: 2,
                parameter_kinds: vec![Kind::Type, Kind::Type],
                body: pair_body.clone(),
                kind: TypeDefinitionKind::Alias,
            },
        )]
        .into_iter()
        .collect::<IndexMap<_, _>>();

        let partial = Type::Named {
            name: pair.clone(),
            body: Box::new(pair_body),
        }
        .apply(vec![Type::Integer]);

        assert_eq!(
            normalize_alias_applications(partial.clone(), &definitions),
            partial
        );
    }

    #[test]
    fn generated_impl_method_binding_has_expected_shape() {
        let path = Path::new("demo", "impl_eq");
        let value = Term {
            comments: String::new(),
            kind: TermKind::Immediate(ImmediateValue::Integer(1)),
            span: Span::Generated,
            type_: Type::Integer,
        };

        let binding =
            build_impl_method_binding(path.clone(), Span::Generated, Type::Integer, value);

        assert_eq!(binding.type_, Type::Unit);
        let TermKind::Let {
            assignee,
            scope,
            then,
            else_,
            ..
        } = binding.kind
        else {
            panic!("expected generated let binding");
        };
        assert_eq!(scope, ScopeKind::Global);
        assert!(matches!(
            assignee.kind,
            PatternKind::Identifier(name) if name == path
        ));
        assert!(matches!(
            then.kind,
            TermKind::Immediate(ImmediateValue::Unit)
        ));
        assert!(matches!(else_.kind, TermKind::Unreachable));
    }

    #[test]
    fn typed_fallback_terms_are_unit_typed() {
        let unit = typed_unit_term();
        let unreachable = typed_unreachable_term();

        assert!(matches!(
            unit.kind,
            TermKind::Immediate(ImmediateValue::Unit)
        ));
        assert_eq!(unit.type_, Type::Unit);
        assert!(matches!(unreachable.kind, TermKind::Unreachable));
        assert_eq!(unreachable.type_, Type::Unit);
    }

    fn untyped_term(kind: TermKind<()>) -> Term<()> {
        Term {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    #[test]
    fn unguarded_impl_cycles_detect_self_reference() {
        let method = ImplMethod {
            trait_method: Path::new("demo", "f"),
            impl_path: Path::new("demo", "f#0"),
            value: untyped_term(TermKind::Identifier(Path::new("demo", "f#0"))),
            span: Span::new(1, 1),
        };
        let identities = vec![ImplMethodIdentity {
            trait_item_name: "f".to_string(),
            canonical_trait_method: Path::new("demo", "f"),
            impl_path: Path::new("demo", "f#0"),
            span: Span::new(1, 1),
        }];

        let cycles = find_unguarded_impl_method_cycles(&[method], &identities);

        assert_eq!(cycles, vec![vec![0]]);
    }

    #[test]
    fn unguarded_impl_cycles_ignore_function_guarded_self_calls() {
        let method = ImplMethod {
            trait_method: Path::new("demo", "f"),
            impl_path: Path::new("demo", "f#0"),
            value: untyped_term(TermKind::Function {
                parameter_name: Path::new("demo", "x").with_span(Span::Generated),
                parameter_type: None,
                captures: [(Path::new("demo", "f#0"), ())].into(),
                body: Box::new(untyped_term(TermKind::Identifier(Path::new("demo", "f#0")))),
            }),
            span: Span::new(1, 1),
        };
        let identities = vec![ImplMethodIdentity {
            trait_item_name: "f".to_string(),
            canonical_trait_method: Path::new("demo", "f"),
            impl_path: Path::new("demo", "f#0"),
            span: Span::new(1, 1),
        }];

        let cycles = find_unguarded_impl_method_cycles(&[method], &identities);

        assert!(cycles.is_empty());
    }

    #[test]
    fn unguarded_impl_cycles_include_trait_method_edges() {
        let methods = [
            ImplMethod {
                trait_method: Path::new("demo", "f"),
                impl_path: Path::new("demo", "f#0"),
                value: untyped_term(TermKind::Identifier(Path::new("demo", "g"))),
                span: Span::new(1, 1),
            },
            ImplMethod {
                trait_method: Path::new("demo", "g"),
                impl_path: Path::new("demo", "g#1"),
                value: untyped_term(TermKind::Identifier(Path::new("demo", "f#0"))),
                span: Span::new(2, 1),
            },
        ];
        let identities = vec![
            ImplMethodIdentity {
                trait_item_name: "f".to_string(),
                canonical_trait_method: Path::new("demo", "f"),
                impl_path: Path::new("demo", "f#0"),
                span: Span::new(1, 1),
            },
            ImplMethodIdentity {
                trait_item_name: "g".to_string(),
                canonical_trait_method: Path::new("demo", "g"),
                impl_path: Path::new("demo", "g#1"),
                span: Span::new(2, 1),
            },
        ];

        let cycles = find_unguarded_impl_method_cycles(&methods, &identities);

        assert_eq!(cycles, vec![vec![0, 1]]);
    }
}
