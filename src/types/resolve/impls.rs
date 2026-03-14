//! Trait implementation checking and registration during resolve.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::ir::{
    ImmediateValue,
    ImplMethod,
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

impl ImplProcessingContext<'_> {
    /// Type-check and register a single impl declaration.
    pub(super) fn process(
        &mut self,
        comments: String,
        trait_path: Path,
        arguments: Box<[TypeExpr]>,
        methods: Box<[ImplMethod<()>]>,
    ) -> (Statement<Type>, Vec<Term<Type>>) {
        let mut resolved_type_definitions = self.type_definitions.clone();
        let canonical_trait_path = self
            .symbols
            .canonical_trait_path(&trait_path)
            .unwrap_or_else(|| trait_path.clone());
        let trait_definition = self.symbols.trait_definition(&trait_path).cloned();

        let raw_argument_schemes = arguments
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
            .collect::<Vec<_>>();
        let argument_kinds = arguments
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
                        self.symbols
                            .trait_defs()
                            .get(&canonical)
                            .map(|definition| {
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
            .collect::<Vec<_>>();
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

        let mut typed_methods = Vec::new();
        let mut generated_terms = Vec::new();
        let mut method_map = IndexMap::new();

        for method in methods {
            let mut method_predicate_assumptions = Vec::new();
            let method_name = method
                .trait_method
                .minor
                .rsplit_once(Path::DELIMETER)
                .map(|(_, name)| name)
                .unwrap_or_else(|| method.trait_method.minor.as_str());
            let canonical_trait_method = canonical_trait_path.sibling(method_name);
            let (mut typed_value, mut predicates) = match self.inference_context.infer_term(
                self.type_environment,
                &method.value,
                self.schemes,
            ) {
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
                        let expected_type = normalize_alias_applications(
                            instantiate_forall_for_impl_check(
                                self.inference_context,
                                instantiated.type_.clone(),
                                method.span,
                            ),
                            &resolved_type_definitions,
                        );
                        let value_type = normalize_alias_applications(
                            instantiate_forall_for_impl_check(
                                self.inference_context,
                                typed_value.type_.clone(),
                                method.span,
                            ),
                            &resolved_type_definitions,
                        );
                        if let Err(error) = self
                            .inference_context
                            .table_mut()
                            .unify(&value_type, &expected_type)
                        {
                            log_type_error(
                                self.logger,
                                TypeError::Unification {
                                    error,
                                    span: method.span,
                                },
                            );
                        }
                        for predicate in instantiated.predicates {
                            if !method_predicate_assumptions.contains(&predicate) {
                                method_predicate_assumptions.push(predicate.clone());
                            }
                            if !predicates.contains(&predicate) {
                                predicates.push(predicate);
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

            for predicate in impl_context_predicates.iter().cloned() {
                if !predicates.contains(&predicate) {
                    predicates.push(predicate);
                }
            }

            let mut predicate_assumptions = method_predicate_assumptions.clone();
            for predicate in impl_context_predicates.iter().cloned() {
                if !predicate_assumptions.contains(&predicate) {
                    predicate_assumptions.push(predicate);
                }
            }

            typed_value = normalize_term_types(typed_value, self.inference_context.table_mut());
            let normalized_type = typed_value.type_.clone();
            solve_predicates_with_assumptions(
                self.logger,
                self.inference_context,
                self.symbols,
                method.span,
                &predicates,
                &predicate_assumptions,
            );

            let scheme = self.inference_context.generalize_with_predicates(
                &normalized_type,
                0,
                predicates.clone(),
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
            methods: method_map,
        };
        if orphan_rule_satisfied && trait_head_kinds_valid
            && let Err(error) = self.symbols.insert_impl(trait_impl)
        {
            log_trait_error(self.logger, impl_span, error);
        }

        (
            Statement::Impl {
                comments,
                trait_path,
                arguments,
                methods: typed_methods.into_boxed_slice(),
            },
            generated_terms,
        )
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

fn instantiate_forall_for_impl_check(
    inference_context: &mut InferenceContext,
    type_: Type,
    span: Span,
) -> Type {
    match type_ {
        forall @ Type::ForAll(_) => {
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
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_) => type_,
        Type::ForAll(body) => {
            Type::ForAll(Box::new(normalize_alias_applications(*body, type_definitions)))
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
            Type::Array(Box::new(normalize_alias_applications(*inner, type_definitions)))
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
    if definition.kind != TypeDefinitionKind::Alias
        || arguments.len() != definition.parameters
    {
        return apply_arguments(Type::Named { name, body }, arguments);
    }
    instantiate_forall_strict(&body, &arguments)
        .map(|expanded| normalize_alias_applications(expanded, type_definitions))
        .unwrap_or_else(|| apply_arguments(Type::Named { name, body }, arguments))
}

fn split_applied_type(type_: Type) -> (Type, Vec<Type>) {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } => {
            let (base, mut constructor_arguments) = split_applied_type(*constructor);
            constructor_arguments.extend(arguments);
            (base, constructor_arguments)
        }
        other => (other, Vec::new()),
    }
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

fn type_contains_local_nominal_type(
    type_: &Type,
    module_name: &str,
) -> bool {
    match type_ {
        Type::Named { name, .. } => name.major == module_name,
        Type::Array(inner) | Type::ForAll(inner) => {
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
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_) => false,
    }
}

fn normalize_parameter_kinds(
    mut kinds: Vec<Kind>,
    parameter_count: usize,
) -> Vec<Kind> {
    if kinds.len() < parameter_count {
        kinds.extend(std::iter::repeat_n(Kind::Type, parameter_count - kinds.len()));
    }
    kinds.truncate(parameter_count);
    kinds
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
                    format!(
                        "Trait argument for `{trait_name}` has recursive kind `{in_kind}`."
                    )
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
                    format!(
                        "`{trait_name}` expects {expected} type arguments but got {found}."
                    ),
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
        let partially_instantiated = instantiate_method_scheme(
            &scheme,
            std::slice::from_ref(&option_constructor),
        )
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

        assert_eq!(normalize_alias_applications(partial.clone(), &definitions), partial);
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
}
