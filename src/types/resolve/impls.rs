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
use super::traits::solve_predicates;
use super::type_defs::type_expr_to_type_in_def;
use super::{
    FileLogger,
    Path,
    Pattern,
    PatternKind,
    ScopeKind,
    Span,
    Statement,
    SymbolTable,
    Term,
    TermKind,
    TraitError,
    TraitImpl,
    TraitRef,
    Type,
    TypeDefEntry,
    TypeDefinition,
    TypeScheme,
};

pub(super) struct ImplProcessingContext<'a> {
    pub(super) logger: &'a mut FileLogger,
    pub(super) ctx: &'a mut InferenceContext,
    pub(super) env: &'a mut TypeEnv,
    pub(super) symbols: &'a mut SymbolTable,
    pub(super) schemes: &'a mut IndexMap<Path, TypeScheme>,
    pub(super) type_entries: &'a IndexMap<Path, TypeDefEntry>,
    pub(super) type_definitions: &'a IndexMap<Path, TypeDefinition>,
}

impl ImplProcessingContext<'_> {
    pub(super) fn process(
        &mut self,
        comments: String,
        trait_path: Path,
        arguments: Box<[TypeExpr]>,
        methods: Box<[ImplMethod<()>]>,
    ) -> (Statement<Type>, Vec<Term<Type>>) {
        let mut resolved_type_definitions = self.type_definitions.clone();
        let raw_argument_types = arguments
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
        let peeled_arguments = raw_argument_types
            .iter()
            .map(peel_leading_foralls)
            .collect::<Vec<_>>();
        let impl_parameters = peeled_arguments
            .iter()
            .map(|(count, _)| *count)
            .sum::<usize>();
        let argument_types = peeled_arguments
            .iter()
            .scan(0usize, |parameter_offset, (forall_count, body)| {
                let normalized = normalize_impl_head_argument(
                    body,
                    *forall_count,
                    *parameter_offset,
                    impl_parameters,
                );
                *parameter_offset += *forall_count;
                Some(normalized)
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
                            self.ctx,
                            instantiated.type_.clone(),
                            method.span,
                        );
                        let value_type = instantiate_forall_for_impl_check(
                            self.ctx,
                            typed_value.type_.clone(),
                            method.span,
                        );
                        if let Err(error) = self.ctx.table_mut().unify(&value_type, &expected_type)
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

            typed_value = normalize_term_types(typed_value, self.ctx.table_mut());
            let normalized_type = typed_value.type_.clone();
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
            parameters: impl_parameters,
            head: TraitRef::new(trait_path.clone(), argument_types),
            predicates: Vec::new(),
            methods: method_map,
        };
        if let Err(error) = self.symbols.insert_impl(trait_impl) {
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
    ctx: &mut InferenceContext,
    type_: Type,
    span: Span,
) -> Type {
    match type_ {
        forall @ Type::ForAll(_) => {
            ctx.instantiate(&TypeScheme::new(forall.clone()), span)
                .unwrap_or(forall)
        }
        other => other,
    }
}

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
        kind: TermKind::Immediate(ImmediateValue::Unit),
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
