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

        let orphan_rule_satisfied = trait_path.major == self.module_name
            || argument_types
                .iter()
                .any(|argument| type_contains_local_nominal_type(argument, self.module_name));
        if !orphan_rule_satisfied {
            self.logger
                .error("Invalid trait instance")
                .primary(
                    format!(
                        "`{trait_path}` and all impl-head types are defined outside module `{}`. Define the trait locally or use at least one local named type in the impl head.",
                        self.module_name
                    ),
                    arguments
                        .first()
                        .map(|argument| argument.span)
                        .unwrap_or(Span::Generated),
                )
                .done();
        }

        let trait_definition = self.symbols.trait_defs().get(&trait_path).cloned();
        let mut typed_methods = Vec::new();
        let mut generated_terms = Vec::new();
        let mut method_map = IndexMap::new();

        for method in methods {
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
                && let Some(method_scheme) = trait_definition.methods.get(&method.trait_method)
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
                                    method.trait_method
                                ),
                                method.span,
                            )
                            .done();
                    } else if let Some(instantiated) =
                        instantiate_method_scheme(method_scheme, &argument_types)
                    {
                        let expected_type = instantiate_forall_for_impl_check(
                            self.inference_context,
                            instantiated.type_.clone(),
                            method.span,
                        );
                        let value_type = instantiate_forall_for_impl_check(
                            self.inference_context,
                            typed_value.type_.clone(),
                            method.span,
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

            for predicate in impl_context_predicates.iter().cloned() {
                if !predicates.contains(&predicate) {
                    predicates.push(predicate);
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
                &impl_context_predicates,
            );

            let scheme = self.inference_context.generalize_with_predicates(
                &normalized_type,
                0,
                predicates.clone(),
            );
            self.type_environment
                .insert(method.impl_path.clone(), scheme.clone());
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
            parameters: impl_parameters,
            head: TraitRef::new(trait_path.clone(), argument_types),
            predicates: impl_context_predicates,
            methods: method_map,
        };
        if orphan_rule_satisfied && let Err(error) = self.symbols.insert_impl(trait_impl) {
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
    if arguments.len() > leading_forall_count(&scheme.type_) {
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
