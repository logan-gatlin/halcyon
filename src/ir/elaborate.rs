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
};
use crate::types::{
    ResolvedModule,
    SymbolTable,
    TraitConstraint,
    Type,
    TypeScheme,
    ordered_trait_methods,
};
use crate::{
    Span,
    WithSpan,
};

#[derive(Debug, Clone)]
pub struct ElaborationResult {
    pub module: Module<Type>,
}

#[derive(Debug, Clone)]
struct DictBinding {
    path: Path,
    type_: Type,
}

#[derive(Debug, Clone)]
struct DictEntry {
    predicate: TraitConstraint,
    binding: DictBinding,
}

type DictEnv = Vec<DictEntry>;

struct ElaborationContext<'a> {
    symbols: &'a SymbolTable,
    scheme_env: &'a IndexMap<Path, TypeScheme>,
    module_name: &'a str,
    dict_types: IndexMap<Path, Type>,
    dict_salt: usize,
    grouped_binding_salt: usize,
    grouped_binding_predicates: IndexMap<Path, Vec<TraitConstraint>>,
}

impl<'a> ElaborationContext<'a> {
    fn new(
        symbols: &'a SymbolTable,
        scheme_env: &'a IndexMap<Path, TypeScheme>,
        module_name: &'a str,
    ) -> Self {
        Self {
            symbols,
            scheme_env,
            module_name,
            dict_types: IndexMap::new(),
            dict_salt: 0,
            grouped_binding_salt: 0,
            grouped_binding_predicates: IndexMap::new(),
        }
    }
}

#[tracing::instrument(skip_all, fields(module = %resolved.module.name))]
pub fn elaborate_module(
    resolved: ResolvedModule,
    symbols: &SymbolTable,
) -> ElaborationResult {
    let scheme_env = build_scheme_env(symbols, &resolved.schemes);
    let module_name = resolved.module.name.clone();
    let mut context = ElaborationContext::new(symbols, &scheme_env, &module_name);
    let statements = Vec::from(resolved.module.statements)
        .into_iter()
        .map(|statement| {
            match statement {
                Statement::Term(term) => {
                    let term = elaborate_term(&mut context, term, &Vec::new());
                    Statement::Term(fix_dict_captures(term, &context.dict_types))
                }
                Statement::ConstructorAlias {
                    comments,
                    path,
                    target,
                    span,
                } => {
                    Statement::ConstructorAlias {
                        comments,
                        path,
                        target,
                        span,
                    }
                }
                Statement::Type {
                    comments,
                    path,
                    parameters,
                    def,
                    kind,
                } => {
                    Statement::Type {
                        comments,
                        path,
                        parameters,
                        def,
                        kind,
                    }
                }
                Statement::Trait {
                    comments,
                    path,
                    parameters,
                    methods,
                } => {
                    Statement::Trait {
                        comments,
                        path,
                        parameters,
                        methods,
                    }
                }
                Statement::TraitAlias {
                    comments,
                    path,
                    target,
                } => {
                    Statement::TraitAlias {
                        comments,
                        path,
                        target,
                    }
                }
                Statement::Impl {
                    comments,
                    trait_path,
                    arguments,
                    methods,
                } => {
                    Statement::Impl {
                        comments,
                        trait_path,
                        arguments,
                        methods,
                    }
                }
                Statement::Wasm(sexpr) => Statement::Wasm(sexpr),
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    ElaborationResult {
        module: Module {
            name: resolved.module.name,
            statements,
        },
    }
}

fn build_scheme_env(
    symbols: &SymbolTable,
    schemes: &IndexMap<Path, TypeScheme>,
) -> IndexMap<Path, TypeScheme> {
    let mut env = IndexMap::new();
    env.extend(
        symbols
            .terms()
            .iter()
            .map(|(path, scheme)| (path.clone(), scheme.clone())),
    );
    env.extend(
        schemes
            .iter()
            .map(|(path, scheme)| (path.clone(), scheme.clone())),
    );
    env
}

fn elaborate_term(
    context: &mut ElaborationContext<'_>,
    term: Term<Type>,
    dict_env: &DictEnv,
) -> Term<Type> {
    let Term {
        comments,
        kind,
        span,
        type_,
    } = term;

    if let TermKind::Identifier(ref path) = kind
        && let Some(term) = elaborate_identifier(
            context,
            path.clone(),
            comments.clone(),
            span,
            type_.clone(),
            dict_env,
        )
    {
        return term;
    }

    let kind = match kind {
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            let binding_entries = pattern_binding_entries(&assignee);
            if binding_entries.len() > 1
                && grouped_binding_has_non_concrete_predicates(context, &binding_entries)
            {
                let rewritten = if scope == ScopeKind::Global {
                    rewrite_top_level_grouped_binding_let(
                        context,
                        GroupedBindingRewriteInput {
                            comments: comments.clone(),
                            assignee,
                            value,
                            then,
                            else_,
                            span,
                            type_: type_.clone(),
                        },
                        &binding_entries,
                    )
                } else {
                    rewrite_local_grouped_binding_let(
                        context,
                        GroupedBindingRewriteInput {
                            comments: comments.clone(),
                            assignee,
                            value,
                            then,
                            else_,
                            span,
                            type_: type_.clone(),
                        },
                        &binding_entries,
                    )
                };
                let predicate_overrides =
                    grouped_binding_predicate_overrides(context, &binding_entries);
                let overridden_paths = predicate_overrides.keys().cloned().collect::<Vec<_>>();
                context
                    .grouped_binding_predicates
                    .extend(predicate_overrides);
                let elaborated = elaborate_term(context, rewritten, dict_env);
                for path in overridden_paths {
                    context.grouped_binding_predicates.shift_remove(&path);
                }
                return elaborated;
            }

            if binding_entries.len() == 1 {
                let (binding, binding_type) = binding_entries
                    .first()
                    .cloned()
                    .unwrap_or_else(|| unreachable!());
                let predicates = context
                    .grouped_binding_predicates
                    .get(&binding)
                    .cloned()
                    .or_else(|| {
                        let scheme = context.scheme_env.get(&binding)?;
                        instantiate_predicates_for_scheme(scheme, &binding_type)
                    })
                    .unwrap_or_default();
                let has_non_concrete = predicates
                    .iter()
                    .any(|predicate| !predicate_is_concrete(predicate));
                if has_non_concrete {
                    let sorted_predicates = sorted_predicates(&predicates);
                    let dict_params = build_dict_params(
                        &sorted_predicates,
                        context.module_name,
                        context.symbols,
                        &mut context.dict_types,
                        &mut context.dict_salt,
                    );
                    let mut value_dict_env = dict_env.clone();
                    value_dict_env.extend(dict_params.iter().cloned());
                    let value = elaborate_term(context, *value, &value_dict_env);
                    let wrapped_value = wrap_with_dict_params(value, &dict_params);
                    let assignee =
                        rewrite_single_binding_pattern_type(assignee, wrapped_value.type_.clone());
                    let then = elaborate_term(context, *then, dict_env);
                    let else_ = elaborate_term(context, *else_, dict_env);
                    return Term {
                        comments,
                        kind: TermKind::Let {
                            assignee,
                            scope,
                            value: wrapped_value.into(),
                            then: then.into(),
                            else_: else_.into(),
                        },
                        span,
                        type_,
                    };
                }
            }
            TermKind::Let {
                assignee,
                scope,
                value: elaborate_term(context, *value, dict_env).into(),
                then: elaborate_term(context, *then, dict_env).into(),
                else_: elaborate_term(context, *else_, dict_env).into(),
            }
        }
        TermKind::Tuple(items) => {
            TermKind::Tuple(
                items
                    .into_iter()
                    .map(|item| elaborate_term(context, item, dict_env))
                    .collect(),
            )
        }
        TermKind::Struct(fields) => {
            TermKind::Struct(
                fields
                    .into_iter()
                    .map(|(name, value)| (name, elaborate_term(context, value, dict_env)))
                    .collect(),
            )
        }
        TermKind::Field { of, index } => {
            TermKind::Field {
                of: elaborate_term(context, *of, dict_env).into(),
                index,
            }
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            TermKind::Function {
                parameter_name,
                parameter_type,
                captures,
                body: elaborate_term(context, *body, dict_env).into(),
            }
        }
        TermKind::Call { callee, argument } => {
            let callee = elaborate_term(context, *callee, dict_env);
            let argument = elaborate_term(context, *argument, dict_env);
            let callee = apply_dictionary_args(context, callee, &argument, &type_, dict_env)
                .unwrap_or_else(|callee| callee);
            TermKind::Call {
                callee: callee.into(),
                argument: argument.into(),
            }
        }
        TermKind::Semicolon(left, right) => {
            TermKind::Semicolon(
                elaborate_term(context, *left, dict_env).into(),
                elaborate_term(context, *right, dict_env).into(),
            )
        }
        other => other,
    };

    Term {
        comments,
        kind,
        span,
        type_,
    }
}

fn elaborate_identifier(
    context: &mut ElaborationContext<'_>,
    path: Path,
    comments: String,
    span: Span,
    type_: Type,
    dict_env: &DictEnv,
) -> Option<Term<Type>> {
    let is_trait_item = is_trait_item_path(context, &path);
    let scheme = context.scheme_env.get(&path)?;
    let args = dictionary_args_for_type(scheme, &type_, dict_env, context.symbols, is_trait_item)?;
    if args.is_empty() {
        return None;
    }

    if is_trait_item
        && args.len() == 1
        && let Some(dict) = args.first().cloned()
    {
        return Some(Term {
            comments,
            kind: TermKind::Field {
                of: Box::new(dict),
                index: path.minor.clone().with_span(Span::Generated),
            },
            span,
            type_,
        });
    }

    let callee = Term {
        comments,
        kind: TermKind::Identifier(path),
        span,
        type_,
    };
    Some(apply_explicit_arguments(callee, args))
}

fn find_dict_binding<'a>(
    dict_env: &'a [DictEntry],
    predicate: &TraitConstraint,
) -> Option<&'a DictBinding> {
    if let Some(entry) = dict_env.iter().find(|entry| entry.predicate == *predicate) {
        return Some(&entry.binding);
    }
    let mut matches = dict_env
        .iter()
        .filter(|entry| entry.predicate.trait_name == predicate.trait_name)
        .map(|entry| &entry.binding);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn wrap_with_dict_params(
    term: Term<Type>,
    dict_params: &[DictEntry],
) -> Term<Type> {
    dict_params.iter().rev().fold(term, |inner, entry| {
        let binding = &entry.binding;
        let function_type = Type::func(binding.type_.clone(), inner.type_.clone());
        Term {
            comments: String::new(),
            kind: TermKind::Function {
                parameter_name: binding.path.clone().with_span(Span::Generated),
                parameter_type: None,
                captures: [].into(),
                body: inner.into(),
            },
            span: Span::Generated,
            type_: function_type,
        }
    })
}

fn rewrite_single_binding_pattern_type(
    pattern: Pattern<Type>,
    new_type: Type,
) -> Pattern<Type> {
    let Pattern {
        comments,
        kind,
        span,
        type_,
    } = pattern;
    match kind {
        PatternKind::Identifier(path) => {
            Pattern {
                comments,
                kind: PatternKind::Identifier(path),
                span,
                type_: new_type,
            }
        }
        PatternKind::TypeHint(inner, type_expr) => {
            Pattern {
                comments,
                kind: PatternKind::TypeHint(
                    Box::new(rewrite_single_binding_pattern_type(
                        *inner,
                        new_type.clone(),
                    )),
                    type_expr,
                ),
                span,
                type_,
            }
        }
        _ => {
            Pattern {
                comments,
                kind,
                span,
                type_,
            }
        }
    }
}

fn grouped_binding_has_non_concrete_predicates(
    context: &ElaborationContext<'_>,
    binding_entries: &[(Path, Type)],
) -> bool {
    binding_entries.iter().any(|(binding, _)| {
        context.scheme_env.get(binding).is_some_and(|scheme| {
            scheme
                .predicates
                .iter()
                .any(|predicate| !predicate_is_concrete(predicate))
        })
    })
}

fn grouped_binding_predicate_overrides(
    context: &ElaborationContext<'_>,
    binding_entries: &[(Path, Type)],
) -> IndexMap<Path, Vec<TraitConstraint>> {
    let mut overrides = IndexMap::new();
    for (binding_path, _) in binding_entries {
        let Some(scheme) = context.scheme_env.get(binding_path) else {
            continue;
        };
        if scheme.predicates.is_empty() {
            continue;
        }
        let (_, var_count) = peel_forall(&scheme.type_);
        let mut merged_bindings = vec![None; var_count];
        for (other_path, other_type) in binding_entries {
            let Some(other_scheme) = context.scheme_env.get(other_path) else {
                continue;
            };
            let (other_body, other_var_count) = peel_forall(&other_scheme.type_);
            if other_var_count != var_count {
                continue;
            }
            let mut local_bindings = vec![None; var_count];
            if !match_scheme_to_type(&other_body, other_type, &mut local_bindings) {
                continue;
            }
            for (slot, binding) in merged_bindings.iter_mut().zip(local_bindings.into_iter()) {
                if slot.is_none() {
                    *slot = binding;
                }
            }
        }
        if let Some(predicates) = instantiate_predicates(&scheme.predicates, &merged_bindings) {
            overrides.insert(binding_path.clone(), predicates);
        }
    }
    overrides
}

struct GroupedBindingRewriteInput {
    comments: String,
    assignee: Pattern<Type>,
    value: Box<Term<Type>>,
    then: Box<Term<Type>>,
    else_: Box<Term<Type>>,
    span: Span,
    type_: Type,
}

fn rewrite_top_level_grouped_binding_let(
    context: &mut ElaborationContext<'_>,
    rewrite: GroupedBindingRewriteInput,
    binding_entries: &[(Path, Type)],
) -> Term<Type> {
    let GroupedBindingRewriteInput {
        comments,
        assignee,
        value,
        then,
        else_,
        span,
        type_,
    } = rewrite;
    let grouped_else = (*else_).clone();
    let grouped_value = *value;
    let mut chain = *then;

    for (binding_path, binding_type) in binding_entries.iter().rev() {
        let binding_value = extraction_value_for_grouped_binding_from_value(
            context,
            &assignee,
            binding_path,
            binding_type,
            grouped_value.clone(),
        );
        chain = generated_term(
            TermKind::Let {
                assignee: generated_identifier_pattern(binding_path.clone(), binding_type.clone()),
                scope: ScopeKind::Global,
                value: binding_value.into(),
                then: chain.into(),
                else_: grouped_else.clone().into(),
            },
            type_.clone(),
        );
    }

    Term {
        comments,
        span,
        type_,
        ..chain
    }
}

fn rewrite_local_grouped_binding_let(
    context: &mut ElaborationContext<'_>,
    rewrite: GroupedBindingRewriteInput,
    binding_entries: &[(Path, Type)],
) -> Term<Type> {
    let GroupedBindingRewriteInput {
        comments,
        assignee,
        value,
        then,
        else_,
        span,
        type_,
    } = rewrite;
    let grouped_value = *value;
    let grouped_else = *else_;
    let grouped_then = *then;
    let grouped_scrutinee =
        grouped_binding_temp_path(context.module_name, &mut context.grouped_binding_salt);
    let grouped_scrutinee_type = assignee.type_.clone();

    let chain = grouped_binding_chain(
        context,
        GroupedBindingChainInput {
            assignee: &assignee,
            grouped_scrutinee: &grouped_scrutinee,
            grouped_scrutinee_type: &grouped_scrutinee_type,
            then: grouped_then,
            else_: &grouped_else,
            result_type: &type_,
            binding_entries,
        },
    );

    let guard = generated_term(
        TermKind::Let {
            assignee: rewrite_pattern_for_match_guard(assignee),
            scope: ScopeKind::Local,
            value: term_identifier(grouped_scrutinee.clone(), grouped_scrutinee_type.clone())
                .into(),
            then: chain.into(),
            else_: grouped_else.into(),
        },
        type_.clone(),
    );

    let rewritten = generated_term(
        TermKind::Let {
            assignee: generated_identifier_pattern(grouped_scrutinee, grouped_scrutinee_type),
            scope: ScopeKind::Local,
            value: grouped_value.into(),
            then: guard.into(),
            else_: generated_term(TermKind::Unreachable, type_.clone()).into(),
        },
        type_.clone(),
    );

    Term {
        comments,
        span,
        type_,
        ..rewritten
    }
}

struct GroupedBindingChainInput<'a> {
    assignee: &'a Pattern<Type>,
    grouped_scrutinee: &'a Path,
    grouped_scrutinee_type: &'a Type,
    then: Term<Type>,
    else_: &'a Term<Type>,
    result_type: &'a Type,
    binding_entries: &'a [(Path, Type)],
}

fn grouped_binding_chain(
    context: &mut ElaborationContext<'_>,
    chain: GroupedBindingChainInput<'_>,
) -> Term<Type> {
    let GroupedBindingChainInput {
        assignee,
        grouped_scrutinee,
        grouped_scrutinee_type,
        then,
        else_,
        result_type,
        binding_entries,
    } = chain;
    let mut chain = then;
    for (binding_path, binding_type) in binding_entries.iter().rev() {
        let binding_value = extraction_value_for_grouped_binding_from_scrutinee(
            context,
            assignee,
            binding_path,
            binding_type,
            grouped_scrutinee,
            grouped_scrutinee_type,
        );
        chain = generated_term(
            TermKind::Let {
                assignee: generated_identifier_pattern(binding_path.clone(), binding_type.clone()),
                scope: ScopeKind::Local,
                value: binding_value.into(),
                then: chain.into(),
                else_: else_.clone().into(),
            },
            result_type.clone(),
        );
    }
    chain
}

fn extraction_value_for_grouped_binding_from_scrutinee(
    context: &mut ElaborationContext<'_>,
    assignee: &Pattern<Type>,
    target_binding: &Path,
    target_type: &Type,
    grouped_scrutinee: &Path,
    grouped_scrutinee_type: &Type,
) -> Term<Type> {
    let extracted_binding =
        grouped_binding_temp_path(context.module_name, &mut context.grouped_binding_salt);
    let extraction_pattern =
        rewrite_pattern_for_target_binding(assignee.clone(), target_binding, &extracted_binding);
    generated_term(
        TermKind::Let {
            assignee: extraction_pattern,
            scope: ScopeKind::Local,
            value: term_identifier(grouped_scrutinee.clone(), grouped_scrutinee_type.clone())
                .into(),
            then: term_identifier(extracted_binding, target_type.clone()).into(),
            else_: generated_term(TermKind::Unreachable, target_type.clone()).into(),
        },
        target_type.clone(),
    )
}

fn extraction_value_for_grouped_binding_from_value(
    context: &mut ElaborationContext<'_>,
    assignee: &Pattern<Type>,
    target_binding: &Path,
    target_type: &Type,
    value: Term<Type>,
) -> Term<Type> {
    let extracted_binding =
        grouped_binding_temp_path(context.module_name, &mut context.grouped_binding_salt);
    let extraction_pattern =
        rewrite_pattern_for_target_binding(assignee.clone(), target_binding, &extracted_binding);
    generated_term(
        TermKind::Let {
            assignee: extraction_pattern,
            scope: ScopeKind::Local,
            value: value.into(),
            then: term_identifier(extracted_binding, target_type.clone()).into(),
            else_: generated_term(TermKind::Unreachable, target_type.clone()).into(),
        },
        target_type.clone(),
    )
}

fn rewrite_pattern_for_match_guard(pattern: Pattern<Type>) -> Pattern<Type> {
    let Pattern {
        comments,
        kind,
        span,
        type_,
    } = pattern;
    let kind = match kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => kind,
        PatternKind::Identifier(_) => PatternKind::Hole,
        PatternKind::Constructor(path, inner) => {
            PatternKind::Constructor(path, Box::new(rewrite_pattern_for_match_guard(*inner)))
        }
        PatternKind::Tuple(items) => {
            PatternKind::Tuple(
                items
                    .into_iter()
                    .map(rewrite_pattern_for_match_guard)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            PatternKind::Array {
                starting: starting
                    .into_iter()
                    .map(rewrite_pattern_for_match_guard)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                glob: match glob {
                    crate::ir::Glob::Named(_) => crate::ir::Glob::Anonymous,
                    other => other,
                },
                ending: ending
                    .into_iter()
                    .map(rewrite_pattern_for_match_guard)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            }
        }
        PatternKind::Struct(fields) => {
            PatternKind::Struct(
                fields
                    .into_iter()
                    .map(|(name, field_pattern)| {
                        (name, rewrite_pattern_for_match_guard(field_pattern))
                    })
                    .collect(),
            )
        }
        PatternKind::TypeHint(inner, type_expr) => {
            PatternKind::TypeHint(Box::new(rewrite_pattern_for_match_guard(*inner)), type_expr)
        }
    };

    Pattern {
        comments,
        kind,
        span,
        type_,
    }
}

fn grouped_binding_temp_path(
    module_name: &str,
    grouped_binding_salt: &mut usize,
) -> Path {
    let path = Path::new(
        module_name,
        format!("[group binding] #{}", *grouped_binding_salt),
    );
    *grouped_binding_salt += 1;
    path
}

fn generated_identifier_pattern(
    path: Path,
    type_: Type,
) -> Pattern<Type> {
    Pattern {
        comments: String::new(),
        kind: PatternKind::Identifier(path),
        span: Span::Generated,
        type_,
    }
}

fn rewrite_pattern_for_target_binding(
    pattern: Pattern<Type>,
    target_binding: &Path,
    replacement_binding: &Path,
) -> Pattern<Type> {
    let Pattern {
        comments,
        kind,
        span,
        type_,
    } = pattern;
    let kind = match kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => kind,
        PatternKind::Identifier(path) => {
            if path == *target_binding {
                PatternKind::Identifier(replacement_binding.clone())
            } else {
                PatternKind::Hole
            }
        }
        PatternKind::Constructor(path, inner) => {
            PatternKind::Constructor(
                path,
                Box::new(rewrite_pattern_for_target_binding(
                    *inner,
                    target_binding,
                    replacement_binding,
                )),
            )
        }
        PatternKind::Tuple(items) => {
            PatternKind::Tuple(
                items
                    .into_iter()
                    .map(|item| {
                        rewrite_pattern_for_target_binding(
                            item,
                            target_binding,
                            replacement_binding,
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            PatternKind::Array {
                starting: starting
                    .into_iter()
                    .map(|item| {
                        rewrite_pattern_for_target_binding(
                            item,
                            target_binding,
                            replacement_binding,
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                glob: match glob {
                    crate::ir::Glob::Named(path) if path == *target_binding => {
                        crate::ir::Glob::Named(replacement_binding.clone())
                    }
                    crate::ir::Glob::Named(_) => crate::ir::Glob::Anonymous,
                    other => other,
                },
                ending: ending
                    .into_iter()
                    .map(|item| {
                        rewrite_pattern_for_target_binding(
                            item,
                            target_binding,
                            replacement_binding,
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            }
        }
        PatternKind::Struct(fields) => {
            PatternKind::Struct(
                fields
                    .into_iter()
                    .map(|(name, pattern)| {
                        (
                            name,
                            rewrite_pattern_for_target_binding(
                                pattern,
                                target_binding,
                                replacement_binding,
                            ),
                        )
                    })
                    .collect(),
            )
        }
        PatternKind::TypeHint(inner, type_expr) => {
            PatternKind::TypeHint(
                Box::new(rewrite_pattern_for_target_binding(
                    *inner,
                    target_binding,
                    replacement_binding,
                )),
                type_expr,
            )
        }
    };

    Pattern {
        comments,
        kind,
        span,
        type_,
    }
}

fn apply_dictionary_args(
    context: &mut ElaborationContext<'_>,
    callee: Term<Type>,
    argument: &Term<Type>,
    result_type: &Type,
    dict_env: &DictEnv,
) -> Result<Term<Type>, Term<Type>> {
    let TermKind::Identifier(path) = &callee.kind else {
        return Err(callee);
    };
    let is_trait_item = is_trait_item_path(context, path);
    let scheme = context.scheme_env.get(path).ok_or(callee.clone())?;
    let call_type = Type::func(argument.type_.clone(), result_type.clone());
    let args =
        dictionary_args_for_type(scheme, &call_type, dict_env, context.symbols, is_trait_item)
            .ok_or(callee.clone())?;
    Ok(apply_explicit_arguments(callee, args))
}

fn is_trait_item_path(
    context: &ElaborationContext<'_>,
    path: &Path,
) -> bool {
    context
        .symbols
        .trait_defs()
        .values()
        .any(|def| def.methods.contains_key(path))
}

fn apply_explicit_arguments(
    mut callee: Term<Type>,
    arguments: Vec<Term<Type>>,
) -> Term<Type> {
    if arguments.is_empty() {
        return callee;
    }

    callee.type_ = arguments
        .iter()
        .rev()
        .fold(callee.type_.clone(), |result, argument| {
            Type::func(argument.type_.clone(), result)
        });

    arguments.into_iter().fold(callee, |current, argument| {
        let result_type = match &current.type_ {
            Type::Function(_, result) => (**result).clone(),
            other => other.clone(),
        };
        Term {
            comments: String::new(),
            kind: TermKind::Call {
                callee: current.into(),
                argument: argument.into(),
            },
            span: Span::Generated,
            type_: result_type,
        }
    })
}

fn dictionary_args_for_type(
    scheme: &TypeScheme,
    type_: &Type,
    dict_env: &DictEnv,
    symbols: &SymbolTable,
    include_concrete_predicates: bool,
) -> Option<Vec<Term<Type>>> {
    if scheme.predicates.is_empty() {
        return Some(vec![]);
    }
    let (scheme_body, var_count) = peel_forall(&scheme.type_);
    let mut bindings = vec![None; var_count];
    if !match_scheme_to_type_relaxed(&scheme_body, type_, &mut bindings) {
        tracing::debug!("match_scheme_to_type_relaxed failed");
        return None;
    }
    let predicates = scheme
        .predicates
        .iter()
        .filter(|predicate| include_concrete_predicates || !predicate_is_concrete(predicate))
        .map(|predicate| substitute_type_vars_in_trait_ref(predicate, &bindings))
        .collect::<Option<Vec<_>>>()?;
    let args = dictionary_args_for_predicates(&predicates, dict_env, symbols);
    tracing::debug!(arg_count = ?args.as_ref().map(|v| v.len()), "dictionary_args_for_type");
    args
}

fn dictionary_args_for_predicates(
    predicates: &[TraitConstraint],
    dict_env: &DictEnv,
    symbols: &SymbolTable,
) -> Option<Vec<Term<Type>>> {
    let predicates = sorted_predicates(predicates);
    let mut args = Vec::new();
    for predicate in predicates {
        if let Some(binding) = find_dict_binding(dict_env, &predicate) {
            args.push(term_identifier(binding.path.clone(), binding.type_.clone()));
        } else if predicate_is_concrete(&predicate) {
            args.push(dictionary_term_for_predicate(&predicate, symbols));
        } else {
            return None;
        }
    }
    Some(args)
}

fn build_dict_params(
    predicates: &[TraitConstraint],
    module_name: &str,
    symbols: &SymbolTable,
    dict_types: &mut IndexMap<Path, Type>,
    dict_salt: &mut usize,
) -> Vec<DictEntry> {
    predicates
        .iter()
        .map(|predicate| {
            let type_ = dictionary_type_for_predicate(predicate, symbols).unwrap_or(Type::Struct {
                fields: Default::default(),
            });
            let path = dict_param_path(module_name, predicate, dict_salt);
            dict_types.insert(path.clone(), type_.clone());
            DictEntry {
                predicate: predicate.clone(),
                binding: DictBinding { path, type_ },
            }
        })
        .collect()
}

fn dict_param_path(
    module_name: &str,
    predicate: &TraitConstraint,
    dict_salt: &mut usize,
) -> Path {
    let key = predicate_key(predicate);
    let path = Path::new(module_name, format!("[dict] {key} #{}", *dict_salt));
    *dict_salt += 1;
    path
}

fn dictionary_type_for_predicate(
    predicate: &TraitConstraint,
    symbols: &SymbolTable,
) -> Option<Type> {
    let def = symbols.trait_definition(&predicate.trait_name)?;
    let methods = ordered_trait_methods(def);
    let mut fields = IndexMap::new();
    for (method_path, scheme) in methods {
        let type_ = scheme.type_.clone();
        fields.insert(method_path.minor.clone(), type_);
    }
    Some(Type::Struct { fields })
}

fn dictionary_term_for_predicate(
    predicate: &TraitConstraint,
    symbols: &SymbolTable,
) -> Term<Type> {
    let Some(def) = symbols.trait_definition(&predicate.trait_name) else {
        return generated_term(
            TermKind::Struct(IndexMap::new()),
            Type::Struct {
                fields: Default::default(),
            },
        );
    };
    let methods = ordered_trait_methods(def);
    let empty_dict_env: DictEnv = Vec::new();
    let mut fields = IndexMap::new();
    let mut field_types = IndexMap::new();
    for (method_path, scheme) in methods {
        let method_type = scheme.type_.clone();
        let specialization = symbols
            .resolve_method_specialization(&method_path, &predicate.arguments)
            .ok()
            .flatten();
        let specialized_path = specialization
            .as_ref()
            .map(|specialization| specialization.impl_method_path.clone())
            .unwrap_or_else(|| method_path.clone());
        let mut field_term = Term {
            comments: String::new(),
            kind: TermKind::Identifier(specialized_path),
            span: Span::Generated,
            type_: method_type.clone(),
        };
        if let Some(specialization) = specialization
            && let Some(dict_args) =
                dictionary_args_for_predicates(&specialization.predicates, &empty_dict_env, symbols)
        {
            field_term = apply_explicit_arguments(field_term, dict_args);
        }
        fields.insert(
            method_path.minor.clone().with_span(Span::Generated),
            field_term,
        );
        field_types.insert(method_path.minor, method_type);
    }
    Term {
        comments: String::new(),
        kind: TermKind::Struct(fields),
        span: Span::Generated,
        type_: Type::Struct {
            fields: field_types,
        },
    }
}

fn predicate_key(predicate: &TraitConstraint) -> String {
    let args = predicate
        .arguments
        .iter()
        .map(type_key)
        .collect::<Vec<_>>()
        .join("_");
    if args.is_empty() {
        format!(
            "{}::{}",
            predicate.trait_name.major, predicate.trait_name.minor
        )
    } else {
        format!(
            "{}::{} {}",
            predicate.trait_name.major, predicate.trait_name.minor, args
        )
    }
}

fn sorted_predicates(predicates: &[TraitConstraint]) -> Vec<TraitConstraint> {
    let mut preds = predicates.to_vec();
    preds.sort_by_key(predicate_key);
    preds
}

fn predicate_is_concrete(predicate: &TraitConstraint) -> bool {
    predicate.arguments.iter().all(is_concrete_type)
}

fn instantiate_predicates_for_scheme(
    scheme: &TypeScheme,
    concrete: &Type,
) -> Option<Vec<TraitConstraint>> {
    if scheme.predicates.is_empty() {
        return None;
    }
    let (scheme_body, var_count) = peel_forall(&scheme.type_);
    let mut bindings = vec![None; var_count];
    if !match_scheme_to_type(&scheme_body, concrete, &mut bindings) {
        return None;
    }
    instantiate_predicates(&scheme.predicates, &bindings)
}

fn instantiate_predicates(
    predicates: &[TraitConstraint],
    bindings: &[Option<Type>],
) -> Option<Vec<TraitConstraint>> {
    predicates
        .iter()
        .map(|predicate| substitute_type_vars_in_trait_ref(predicate, bindings))
        .collect()
}

fn substitute_type_vars_in_trait_ref(
    trait_ref: &TraitConstraint,
    bindings: &[Option<Type>],
) -> Option<TraitConstraint> {
    let arguments = trait_ref
        .arguments
        .iter()
        .map(|arg| substitute_type_vars_in_type(arg, bindings))
        .collect::<Option<Vec<_>>>()?;
    Some(TraitConstraint {
        trait_name: trait_ref.trait_name.clone(),
        arguments,
    })
}

fn substitute_type_vars_in_type(
    type_: &Type,
    bindings: &[Option<Type>],
) -> Option<Type> {
    match type_ {
        Type::TypeVar(index) => {
            let binding = bindings.get(*index as usize)?;
            binding.clone()
        }
        Type::Array(inner) => {
            substitute_type_vars_in_type(inner, bindings).map(|inner| Type::Array(Box::new(inner)))
        }
        Type::Tuple(items) => {
            items
                .iter()
                .map(|item| substitute_type_vars_in_type(item, bindings))
                .collect::<Option<Vec<_>>>()
                .map(Type::Tuple)
        }
        Type::Struct { fields } => {
            fields
                .iter()
                .map(|(name, type_)| {
                    substitute_type_vars_in_type(type_, bindings).map(|type_| (name.clone(), type_))
                })
                .collect::<Option<IndexMap<_, _>>>()
                .map(|fields| Type::Struct { fields })
        }
        Type::Sum { variants } => {
            variants
                .iter()
                .map(|(name, type_)| {
                    substitute_type_vars_in_type(type_, bindings).map(|type_| (name.clone(), type_))
                })
                .collect::<Option<IndexMap<_, _>>>()
                .map(|variants| Type::Sum { variants })
        }
        Type::Function(parameter, result) => {
            let parameter = substitute_type_vars_in_type(parameter, bindings)?;
            let result = substitute_type_vars_in_type(result, bindings)?;
            Some(Type::func(parameter, result))
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            let constructor = substitute_type_vars_in_type(constructor, bindings)?;
            let arguments = arguments
                .iter()
                .map(|arg| substitute_type_vars_in_type(arg, bindings))
                .collect::<Option<Vec<_>>>()?;
            Some(Type::Apply {
                constructor: Box::new(constructor),
                arguments,
            })
        }
        Type::StructConstraint { .. } | Type::MetaVar(_) | Type::ForAll(_) => None,
        other => Some(other.clone()),
    }
}

fn term_identifier(
    path: Path,
    type_: Type,
) -> Term<Type> {
    Term {
        comments: String::new(),
        kind: TermKind::Identifier(path),
        span: Span::Generated,
        type_,
    }
}

fn generated_term(
    kind: TermKind<Type>,
    type_: Type,
) -> Term<Type> {
    Term {
        comments: String::new(),
        kind,
        span: Span::Generated,
        type_,
    }
}

fn pattern_binding_entries(pattern: &Pattern<Type>) -> Vec<(Path, Type)> {
    match &pattern.kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => {
            Vec::new()
        }
        PatternKind::Identifier(path) => vec![(path.clone(), pattern.type_.clone())],
        PatternKind::Constructor(_, inner) => pattern_binding_entries(inner),
        PatternKind::Tuple(items) => items.iter().flat_map(pattern_binding_entries).collect(),
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let mut bindings = Vec::new();
            bindings.extend(starting.iter().flat_map(pattern_binding_entries));
            bindings.extend(ending.iter().flat_map(pattern_binding_entries));
            if let crate::ir::Glob::Named(path) = glob {
                bindings.push((path.clone(), pattern.type_.clone()));
            }
            bindings
        }
        PatternKind::Struct(fields) => fields.values().flat_map(pattern_binding_entries).collect(),
        PatternKind::TypeHint(inner, _) => pattern_binding_entries(inner),
    }
}

fn fix_dict_captures(
    term: Term<Type>,
    dict_types: &IndexMap<Path, Type>,
) -> Term<Type> {
    let Term {
        comments,
        kind,
        span,
        type_,
    } = term;
    let kind = match kind {
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            let body = fix_dict_captures(*body, dict_types);
            let mut capture_map = IndexMap::new();
            for (path, type_) in captures.into_iter() {
                capture_map.insert(path, type_);
            }
            let mut dict_uses = collect_dict_uses(&body, dict_types);
            dict_uses.shift_remove(&parameter_name.inner);
            let mut new_dicts = dict_uses
                .into_iter()
                .filter(|(path, _)| !capture_map.contains_key(path))
                .collect::<Vec<_>>();
            new_dicts.sort_by(|(left, _), (right, _)| path_key(left).cmp(&path_key(right)));
            for (path, type_) in new_dicts {
                capture_map.insert(path, type_);
            }
            TermKind::Function {
                parameter_name,
                parameter_type,
                captures: capture_map
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                body: body.into(),
            }
        }
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            TermKind::Let {
                assignee,
                scope,
                value: fix_dict_captures(*value, dict_types).into(),
                then: fix_dict_captures(*then, dict_types).into(),
                else_: fix_dict_captures(*else_, dict_types).into(),
            }
        }
        TermKind::Tuple(items) => {
            TermKind::Tuple(
                items
                    .into_iter()
                    .map(|item| fix_dict_captures(item, dict_types))
                    .collect(),
            )
        }
        TermKind::Struct(fields) => {
            TermKind::Struct(
                fields
                    .into_iter()
                    .map(|(name, value)| (name, fix_dict_captures(value, dict_types)))
                    .collect(),
            )
        }
        TermKind::Field { of, index } => {
            TermKind::Field {
                of: fix_dict_captures(*of, dict_types).into(),
                index,
            }
        }
        TermKind::Call { callee, argument } => {
            TermKind::Call {
                callee: fix_dict_captures(*callee, dict_types).into(),
                argument: fix_dict_captures(*argument, dict_types).into(),
            }
        }
        TermKind::Semicolon(left, right) => {
            TermKind::Semicolon(
                fix_dict_captures(*left, dict_types).into(),
                fix_dict_captures(*right, dict_types).into(),
            )
        }
        other => other,
    };
    Term {
        comments,
        kind,
        span,
        type_,
    }
}

fn collect_dict_uses(
    term: &Term<Type>,
    dict_types: &IndexMap<Path, Type>,
) -> IndexMap<Path, Type> {
    match &term.kind {
        TermKind::Identifier(path) => {
            dict_types
                .get(path)
                .map(|type_| IndexMap::from([(path.clone(), type_.clone())]))
                .unwrap_or_default()
        }
        TermKind::Let {
            value, then, else_, ..
        } => {
            let mut uses = collect_dict_uses(value, dict_types);
            uses.extend(collect_dict_uses(then, dict_types));
            uses.extend(collect_dict_uses(else_, dict_types));
            uses
        }
        TermKind::Tuple(items) => {
            items
                .iter()
                .flat_map(|item| collect_dict_uses(item, dict_types))
                .collect()
        }
        TermKind::Struct(fields) => {
            fields
                .values()
                .flat_map(|value| collect_dict_uses(value, dict_types))
                .collect()
        }
        TermKind::Field { of, .. } => collect_dict_uses(of, dict_types),
        TermKind::Call { callee, argument } => {
            let mut uses = collect_dict_uses(callee, dict_types);
            uses.extend(collect_dict_uses(argument, dict_types));
            uses
        }
        TermKind::Semicolon(left, right) => {
            let mut uses = collect_dict_uses(left, dict_types);
            uses.extend(collect_dict_uses(right, dict_types));
            uses
        }
        TermKind::Function { captures, .. } => {
            captures
                .iter()
                .filter_map(|(path, type_)| {
                    dict_types.get(path).map(|_| (path.clone(), type_.clone()))
                })
                .collect()
        }
        _ => IndexMap::new(),
    }
}

fn path_key(path: &Path) -> String {
    format!("{path}")
}

fn peel_forall(type_: &Type) -> (Type, usize) {
    let mut current = type_.clone();
    let mut count = 0;
    loop {
        match current {
            Type::ForAll(body) => {
                count += 1;
                current = *body;
            }
            other => return (other, count),
        }
    }
}

fn match_scheme_to_type(
    scheme: &Type,
    concrete: &Type,
    bindings: &mut [Option<Type>],
) -> bool {
    match scheme {
        Type::TypeVar(index) => {
            let Some(slot) = bindings.get_mut(*index as usize) else {
                return false;
            };
            match slot {
                Some(existing) => existing == concrete,
                None => {
                    *slot = Some(concrete.clone());
                    true
                }
            }
        }
        Type::Unit => matches!(concrete, Type::Unit),
        Type::Integer => matches!(concrete, Type::Integer),
        Type::Real => matches!(concrete, Type::Real),
        Type::Boolean => matches!(concrete, Type::Boolean),
        Type::String => matches!(concrete, Type::String),
        Type::Glyph => matches!(concrete, Type::Glyph),
        Type::Array(inner) => {
            let Type::Array(other) = concrete else {
                return false;
            };
            match_scheme_to_type(inner, other, bindings)
        }
        Type::Tuple(items) => {
            let Type::Tuple(other) = concrete else {
                return false;
            };
            if items.len() != other.len() {
                return false;
            }
            items
                .iter()
                .zip(other.iter())
                .all(|(left, right)| match_scheme_to_type(left, right, bindings))
        }
        Type::Struct { fields } => {
            let Type::Struct { fields: other } = concrete else {
                return false;
            };
            if fields.len() != other.len() {
                return false;
            }
            fields
                .iter()
                .zip(other.iter())
                .all(|((ln, lt), (rn, rt))| ln == rn && match_scheme_to_type(lt, rt, bindings))
        }
        Type::Sum { variants } => {
            let Type::Sum { variants: other } = concrete else {
                return false;
            };
            if variants.len() != other.len() {
                return false;
            }
            variants
                .iter()
                .zip(other.iter())
                .all(|((ln, lt), (rn, rt))| ln == rn && match_scheme_to_type(lt, rt, bindings))
        }
        Type::Function(parameter, result) => {
            let Type::Function(other_parameter, other_result) = concrete else {
                return false;
            };
            match_scheme_to_type(parameter, other_parameter, bindings)
                && match_scheme_to_type(result, other_result, bindings)
        }
        Type::Named { name, .. } => {
            let Type::Named { name: other, .. } = concrete else {
                return false;
            };
            name == other
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            let Type::Apply {
                constructor: other_constructor,
                arguments: other_arguments,
            } = concrete
            else {
                return false;
            };
            if arguments.len() != other_arguments.len() {
                return false;
            }
            match_scheme_to_type(constructor, other_constructor, bindings)
                && arguments
                    .iter()
                    .zip(other_arguments.iter())
                    .all(|(left, right)| match_scheme_to_type(left, right, bindings))
        }
        Type::StructConstraint { .. } | Type::MetaVar(_) | Type::ForAll(_) => false,
    }
}

fn match_scheme_to_type_relaxed(
    scheme: &Type,
    concrete: &Type,
    bindings: &mut [Option<Type>],
) -> bool {
    match scheme {
        Type::TypeVar(index) => {
            let Some(slot) = bindings.get_mut(*index as usize) else {
                return false;
            };
            match slot {
                Some(existing) => {
                    match concrete {
                        Type::MetaVar(_) => true,
                        _ => existing == concrete,
                    }
                }
                None => {
                    *slot = Some(concrete.clone());
                    true
                }
            }
        }
        Type::Unit => matches!(concrete, Type::Unit),
        Type::Integer => matches!(concrete, Type::Integer),
        Type::Real => matches!(concrete, Type::Real),
        Type::Boolean => matches!(concrete, Type::Boolean),
        Type::String => matches!(concrete, Type::String),
        Type::Glyph => matches!(concrete, Type::Glyph),
        Type::Array(inner) => {
            let Type::Array(other) = concrete else {
                return false;
            };
            match_scheme_to_type_relaxed(inner, other, bindings)
        }
        Type::Tuple(items) => {
            let Type::Tuple(other) = concrete else {
                return false;
            };
            if items.len() != other.len() {
                return false;
            }
            items
                .iter()
                .zip(other.iter())
                .all(|(left, right)| match_scheme_to_type_relaxed(left, right, bindings))
        }
        Type::Struct { fields } => {
            let Type::Struct { fields: other } = concrete else {
                return false;
            };
            if fields.len() != other.len() {
                return false;
            }
            fields.iter().zip(other.iter()).all(|((ln, lt), (rn, rt))| {
                ln == rn && match_scheme_to_type_relaxed(lt, rt, bindings)
            })
        }
        Type::Sum { variants } => {
            let Type::Sum { variants: other } = concrete else {
                return false;
            };
            if variants.len() != other.len() {
                return false;
            }
            variants
                .iter()
                .zip(other.iter())
                .all(|((ln, lt), (rn, rt))| {
                    ln == rn && match_scheme_to_type_relaxed(lt, rt, bindings)
                })
        }
        Type::Function(parameter, result) => {
            let Type::Function(other_parameter, other_result) = concrete else {
                return false;
            };
            match_scheme_to_type_relaxed(parameter, other_parameter, bindings)
                && match_scheme_to_type_relaxed(result, other_result, bindings)
        }
        Type::Named { name, .. } => {
            let Type::Named { name: other, .. } = concrete else {
                return false;
            };
            name == other
        }
        Type::Apply {
            constructor,
            arguments,
        } => {
            let Type::Apply {
                constructor: other_constructor,
                arguments: other_arguments,
            } = concrete
            else {
                return false;
            };
            if arguments.len() != other_arguments.len() {
                return false;
            }
            match_scheme_to_type_relaxed(constructor, other_constructor, bindings)
                && arguments
                    .iter()
                    .zip(other_arguments.iter())
                    .all(|(left, right)| match_scheme_to_type_relaxed(left, right, bindings))
        }
        Type::StructConstraint { .. } | Type::MetaVar(_) | Type::ForAll(_) => false,
    }
}

fn is_concrete_type(type_: &Type) -> bool {
    match type_ {
        Type::TypeVar(_) | Type::MetaVar(_) | Type::ForAll(_) | Type::StructConstraint { .. } => {
            false
        }
        Type::Array(inner) => is_concrete_type(inner),
        Type::Tuple(items) => items.iter().all(is_concrete_type),
        Type::Struct { fields } => fields.values().all(is_concrete_type),
        Type::Sum { variants } => variants.values().all(is_concrete_type),
        Type::Function(parameter, result) => {
            is_concrete_type(parameter) && is_concrete_type(result)
        }
        Type::Named { body, .. } => is_concrete_type(body),
        Type::Apply {
            constructor,
            arguments,
        } => is_concrete_type(constructor) && arguments.iter().all(is_concrete_type),
        _ => true,
    }
}

fn type_key(type_: &Type) -> String {
    type_
        .pretty()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::hc_core::compile_core_module;
    use crate::ir::ScopeKind;
    use crate::types::resolve_module_with_symbols_and_schemes;
    use crate::{
        Logger,
        parse,
    };

    fn elaborate_source(source: &str) -> (ElaborationResult, SymbolTable) {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", source);
        let mut symbols = SymbolTable::new();
        let _ = compile_core_module(&mut symbols, &mut Logger::new());
        let mut prelude = Vec::new();
        prelude.extend(
            symbols
                .terms()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Term)),
        );
        prelude.extend(
            symbols
                .type_definitions()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Type)),
        );
        prelude.extend(
            symbols
                .trait_defs()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Trait)),
        );
        prelude.extend(
            symbols
                .trait_aliases()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Trait)),
        );

        let modules = parse::parse(source, &mut file_logger)
            .map(|m| m.modules())
            .unwrap_or_default()
            .into_iter()
            .flat_map(|m| crate::ir::module_with_prelude(m, &mut file_logger, &prelude))
            .collect::<Vec<_>>();

        let resolved_modules = modules
            .into_iter()
            .map(|m| resolve_module_with_symbols_and_schemes(&mut symbols, m, &mut file_logger))
            .collect::<Vec<_>>();
        let mut results = resolved_modules
            .into_iter()
            .map(|m| elaborate_module(m, &symbols))
            .collect::<Vec<_>>();

        logger.consume_file(file_logger);
        logger.print_logs();
        assert!(logger.is_ok());

        (results.pop().unwrap(), symbols)
    }

    fn resolve_source(source: &str) -> (ResolvedModule, SymbolTable) {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("test.hc", source);
        let mut symbols = SymbolTable::new();
        let _ = compile_core_module(&mut symbols, &mut Logger::new());
        let mut prelude = Vec::new();
        prelude.extend(
            symbols
                .terms()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Term)),
        );
        prelude.extend(
            symbols
                .type_definitions()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Type)),
        );
        prelude.extend(
            symbols
                .trait_defs()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Trait)),
        );
        prelude.extend(
            symbols
                .trait_aliases()
                .keys()
                .cloned()
                .map(|path| (path, crate::ir::NameSpace::Trait)),
        );

        let modules = parse::parse(source, &mut file_logger)
            .map(|m| m.modules())
            .unwrap_or_default()
            .into_iter()
            .flat_map(|m| crate::ir::module_with_prelude(m, &mut file_logger, &prelude))
            .collect::<Vec<_>>();

        let mut resolved_modules = modules
            .into_iter()
            .map(|m| resolve_module_with_symbols_and_schemes(&mut symbols, m, &mut file_logger))
            .collect::<Vec<_>>();

        logger.consume_file(file_logger);
        logger.print_logs();
        assert!(logger.is_ok());

        (resolved_modules.pop().unwrap(), symbols)
    }

    fn find_global_binding<'a>(
        module: &'a Module<Type>,
        name: &str,
    ) -> Option<&'a Term<Type>> {
        fn binding_name(pattern: &Pattern<Type>) -> Option<&Path> {
            match &pattern.kind {
                PatternKind::Identifier(path) => Some(path),
                PatternKind::TypeHint(inner, _) => binding_name(inner),
                _ => None,
            }
        }

        fn find_global_binding_in_term<'a>(
            term: &'a Term<Type>,
            name: &str,
        ) -> Option<&'a Term<Type>> {
            match &term.kind {
                TermKind::Let {
                    assignee,
                    scope: ScopeKind::Global,
                    value,
                    then,
                    else_,
                } => {
                    binding_name(assignee)
                        .is_some_and(|path| path.minor == name)
                        .then(|| value.as_ref())
                        .or_else(|| find_global_binding_in_term(value, name))
                        .or_else(|| find_global_binding_in_term(then, name))
                        .or_else(|| find_global_binding_in_term(else_, name))
                }
                TermKind::Let {
                    value, then, else_, ..
                } => {
                    find_global_binding_in_term(value, name)
                        .or_else(|| find_global_binding_in_term(then, name))
                        .or_else(|| find_global_binding_in_term(else_, name))
                }
                TermKind::Tuple(items) => {
                    items
                        .iter()
                        .find_map(|item| find_global_binding_in_term(item, name))
                }
                TermKind::Struct(fields) => {
                    fields
                        .values()
                        .find_map(|item| find_global_binding_in_term(item, name))
                }
                TermKind::Field { of, .. } => find_global_binding_in_term(of, name),
                TermKind::Function { body, .. } => find_global_binding_in_term(body, name),
                TermKind::Call { callee, argument } => {
                    find_global_binding_in_term(callee, name)
                        .or_else(|| find_global_binding_in_term(argument, name))
                }
                TermKind::Semicolon(left, right) => {
                    find_global_binding_in_term(left, name)
                        .or_else(|| find_global_binding_in_term(right, name))
                }
                _ => None,
            }
        }

        module.statements.iter().find_map(|statement| {
            let Statement::Term(term) = statement else {
                return None;
            };
            find_global_binding_in_term(term, name)
        })
    }

    fn term_has_global_binding(
        term: &Term<Type>,
        name: &str,
    ) -> bool {
        fn binding_name(pattern: &Pattern<Type>) -> Option<&Path> {
            match &pattern.kind {
                PatternKind::Identifier(path) => Some(path),
                PatternKind::TypeHint(inner, _) => binding_name(inner),
                _ => None,
            }
        }

        match &term.kind {
            TermKind::Let {
                assignee,
                scope: ScopeKind::Global,
                value,
                then,
                else_,
            } => {
                binding_name(assignee).is_some_and(|path| path.minor == name)
                    || term_has_global_binding(value, name)
                    || term_has_global_binding(then, name)
                    || term_has_global_binding(else_, name)
            }
            TermKind::Let {
                value, then, else_, ..
            } => {
                term_has_global_binding(value, name)
                    || term_has_global_binding(then, name)
                    || term_has_global_binding(else_, name)
            }
            TermKind::Tuple(items) => items.iter().any(|item| term_has_global_binding(item, name)),
            TermKind::Struct(fields) => {
                fields
                    .values()
                    .any(|item| term_has_global_binding(item, name))
            }
            TermKind::Field { of, .. } => term_has_global_binding(of, name),
            TermKind::Function { body, .. } => term_has_global_binding(body, name),
            TermKind::Call { callee, argument } => {
                term_has_global_binding(callee, name) || term_has_global_binding(argument, name)
            }
            TermKind::Semicolon(left, right) => {
                term_has_global_binding(left, name) || term_has_global_binding(right, name)
            }
            _ => false,
        }
    }

    fn module_has_global_binding(
        module: &Module<Type>,
        name: &str,
    ) -> bool {
        module.statements.iter().any(|statement| {
            let Statement::Term(term) = statement else {
                return false;
            };
            term_has_global_binding(term, name)
        })
    }

    fn term_has_local_wrapped_binding(
        term: &Term<Type>,
        binding_prefix: &str,
    ) -> bool {
        fn binding_name(pattern: &Pattern<Type>) -> Option<&Path> {
            match &pattern.kind {
                PatternKind::Identifier(path) => Some(path),
                PatternKind::TypeHint(inner, _) => binding_name(inner),
                _ => None,
            }
        }

        match &term.kind {
            TermKind::Let {
                assignee,
                scope: ScopeKind::Local,
                value,
                then,
                else_,
            } => {
                let is_wrapped_binding = binding_name(assignee).is_some_and(|path| {
                    path.minor.starts_with(binding_prefix)
                        && matches!(
                            &value.kind,
                            TermKind::Function { parameter_name, .. }
                                if parameter_name.inner.minor.starts_with("[dict]")
                        )
                });
                is_wrapped_binding
                    || term_has_local_wrapped_binding(value, binding_prefix)
                    || term_has_local_wrapped_binding(then, binding_prefix)
                    || term_has_local_wrapped_binding(else_, binding_prefix)
            }
            TermKind::Let {
                value, then, else_, ..
            } => {
                term_has_local_wrapped_binding(value, binding_prefix)
                    || term_has_local_wrapped_binding(then, binding_prefix)
                    || term_has_local_wrapped_binding(else_, binding_prefix)
            }
            TermKind::Tuple(items) => {
                items
                    .iter()
                    .any(|item| term_has_local_wrapped_binding(item, binding_prefix))
            }
            TermKind::Struct(fields) => {
                fields
                    .values()
                    .any(|item| term_has_local_wrapped_binding(item, binding_prefix))
            }
            TermKind::Field { of, .. } => term_has_local_wrapped_binding(of, binding_prefix),
            TermKind::Function { body, .. } => term_has_local_wrapped_binding(body, binding_prefix),
            TermKind::Call { callee, argument } => {
                term_has_local_wrapped_binding(callee, binding_prefix)
                    || term_has_local_wrapped_binding(argument, binding_prefix)
            }
            TermKind::Semicolon(left, right) => {
                term_has_local_wrapped_binding(left, binding_prefix)
                    || term_has_local_wrapped_binding(right, binding_prefix)
            }
            _ => false,
        }
    }

    fn term_has_refutable_group_guard_with_fallback(term: &Term<Type>) -> bool {
        fn pattern_contains_integer(
            pattern: &Pattern<Type>,
            expected: i64,
        ) -> bool {
            match &pattern.kind {
                PatternKind::Immediate(crate::ir::ImmediateValue::Integer(value)) => {
                    *value == expected
                }
                PatternKind::Constructor(_, inner) => pattern_contains_integer(inner, expected),
                PatternKind::Tuple(items) => {
                    items
                        .iter()
                        .any(|item| pattern_contains_integer(item, expected))
                }
                PatternKind::Array {
                    starting, ending, ..
                } => {
                    starting
                        .iter()
                        .any(|item| pattern_contains_integer(item, expected))
                        || ending
                            .iter()
                            .any(|item| pattern_contains_integer(item, expected))
                }
                PatternKind::Struct(fields) => {
                    fields
                        .values()
                        .any(|item| pattern_contains_integer(item, expected))
                }
                PatternKind::TypeHint(inner, _) => pattern_contains_integer(inner, expected),
                _ => false,
            }
        }

        match &term.kind {
            TermKind::Let {
                assignee,
                scope: ScopeKind::Local,
                value,
                then,
                else_,
            } => {
                let is_guard = pattern_binding_entries(assignee).is_empty()
                    && pattern_contains_integer(assignee, 0)
                    && !matches!(else_.kind, TermKind::Unreachable);
                is_guard
                    || term_has_refutable_group_guard_with_fallback(value)
                    || term_has_refutable_group_guard_with_fallback(then)
                    || term_has_refutable_group_guard_with_fallback(else_)
            }
            TermKind::Let {
                value, then, else_, ..
            } => {
                term_has_refutable_group_guard_with_fallback(value)
                    || term_has_refutable_group_guard_with_fallback(then)
                    || term_has_refutable_group_guard_with_fallback(else_)
            }
            TermKind::Tuple(items) => {
                items
                    .iter()
                    .any(term_has_refutable_group_guard_with_fallback)
            }
            TermKind::Struct(fields) => {
                fields
                    .values()
                    .any(term_has_refutable_group_guard_with_fallback)
            }
            TermKind::Field { of, .. } => term_has_refutable_group_guard_with_fallback(of),
            TermKind::Function { body, .. } => term_has_refutable_group_guard_with_fallback(body),
            TermKind::Call { callee, argument } => {
                term_has_refutable_group_guard_with_fallback(callee)
                    || term_has_refutable_group_guard_with_fallback(argument)
            }
            TermKind::Semicolon(left, right) => {
                term_has_refutable_group_guard_with_fallback(left)
                    || term_has_refutable_group_guard_with_fallback(right)
            }
            _ => false,
        }
    }

    fn term_has_inline_dict_for(
        term: &Term<Type>,
        name: &str,
    ) -> bool {
        match &term.kind {
            TermKind::Call { callee, argument } => {
                let matches = matches!(argument.kind, TermKind::Struct(_))
                    && match &callee.kind {
                        TermKind::Identifier(path) => path.minor == name,
                        TermKind::Call { callee: inner, .. } => {
                            matches!(inner.kind, TermKind::Identifier(ref path) if path.minor == name)
                        }
                        _ => false,
                    };
                matches
                    || term_has_inline_dict_for(callee, name)
                    || term_has_inline_dict_for(argument, name)
            }
            TermKind::Let {
                value, then, else_, ..
            } => {
                term_has_inline_dict_for(value, name)
                    || term_has_inline_dict_for(then, name)
                    || term_has_inline_dict_for(else_, name)
            }
            TermKind::Tuple(items) => {
                items
                    .iter()
                    .any(|item| term_has_inline_dict_for(item, name))
            }
            TermKind::Struct(fields) => {
                fields
                    .values()
                    .any(|item| term_has_inline_dict_for(item, name))
            }
            TermKind::Function { body, .. } => term_has_inline_dict_for(body, name),
            TermKind::Field { of, index } => {
                (index.inner == name && matches!(of.kind, TermKind::Struct(_)))
                    || term_has_inline_dict_for(of, name)
            }
            TermKind::Semicolon(left, right) => {
                term_has_inline_dict_for(left, name) || term_has_inline_dict_for(right, name)
            }
            _ => false,
        }
    }

    fn inline_dict_field_count_for(
        term: &Term<Type>,
        name: &str,
    ) -> Option<usize> {
        match &term.kind {
            TermKind::Call { callee, argument } => {
                let is_target = match &callee.kind {
                    TermKind::Identifier(path) => path.minor == name,
                    TermKind::Call { callee: inner, .. } => {
                        matches!(inner.kind, TermKind::Identifier(ref path) if path.minor == name)
                    }
                    _ => false,
                };

                if is_target && let TermKind::Struct(fields) = &argument.kind {
                    return Some(fields.len());
                }

                inline_dict_field_count_for(callee, name)
                    .or_else(|| inline_dict_field_count_for(argument, name))
            }
            TermKind::Let {
                value, then, else_, ..
            } => {
                inline_dict_field_count_for(value, name)
                    .or_else(|| inline_dict_field_count_for(then, name))
                    .or_else(|| inline_dict_field_count_for(else_, name))
            }
            TermKind::Tuple(items) => {
                items
                    .iter()
                    .find_map(|item| inline_dict_field_count_for(item, name))
            }
            TermKind::Struct(fields) => {
                fields
                    .values()
                    .find_map(|item| inline_dict_field_count_for(item, name))
            }
            TermKind::Function { body, .. } => inline_dict_field_count_for(body, name),
            TermKind::Field { of, index } => {
                if index.inner == name
                    && let TermKind::Struct(fields) = &of.kind
                {
                    return Some(fields.len());
                }
                inline_dict_field_count_for(of, name)
            }
            TermKind::Semicolon(left, right) => {
                inline_dict_field_count_for(left, name)
                    .or_else(|| inline_dict_field_count_for(right, name))
            }
            _ => None,
        }
    }

    #[test]
    fn resolve_stage_has_no_dictionary_rewrite() {
        let source = "module demo =\n\tlet double = fn x => x + x\n\tlet result = double 3\nend\n";
        let (resolved, _symbols) = resolve_source(source);

        let double = find_global_binding(&resolved.module, "double").expect("double binding");
        let TermKind::Function { parameter_name, .. } = &double.kind else {
            panic!("expected function");
        };
        assert!(!parameter_name.inner.minor.starts_with("[dict]"));

        let result = find_global_binding(&resolved.module, "result").expect("result binding");
        assert!(!term_has_inline_dict_for(result, "double"));
    }

    #[test]
    fn adds_dictionary_params_for_polymorphic_function() {
        let source = "module demo =\n\tlet double = fn x => x + x\n\tlet result = double 3\nend\n";
        let (elaborated, _symbols) = elaborate_source(source);
        let double = find_global_binding(&elaborated.module, "double").expect("double binding");
        let TermKind::Function { parameter_name, .. } = &double.kind else {
            panic!("expected function");
        };
        assert!(parameter_name.inner.minor.starts_with("[dict]"));

        let Type::Function(parameter, _) = &double.type_ else {
            panic!("expected dictionary function type");
        };
        let Type::Struct { fields } = parameter.as_ref() else {
            panic!("expected dictionary parameter to be a struct");
        };
        assert!(!fields.is_empty());
    }

    #[test]
    fn inlines_dictionary_at_callsite() {
        let source = "module demo =\n\tlet double = fn x => x + x\n\tlet result = double 3\nend\n";
        let (elaborated, symbols) = elaborate_source(source);
        let _double_scheme = symbols
            .terms()
            .get(&crate::ir::Path::new("demo", "double"))
            .expect("scheme");
        let result = find_global_binding(&elaborated.module, "result").expect("result binding");
        let _double = find_global_binding(&elaborated.module, "double").expect("double binding");
        assert!(term_has_inline_dict_for(result, "double"));
        assert_eq!(inline_dict_field_count_for(result, "double"), Some(1));
    }

    #[test]
    fn elaborates_associated_constant_dictionary_argument() {
        let source = "module demo =\n\ttrait DefaultValue : a =\n\t\tlet default : a\n\tend\n\timpl DefaultValue core::Integer =\n\t\tlet default = 7\n\tend\n\tlet value : core::Integer = default\nend\n";
        let (elaborated, _symbols) = elaborate_source(source);
        let value = find_global_binding(&elaborated.module, "value").expect("value binding");
        assert!(term_has_inline_dict_for(value, "default"));
        assert_eq!(inline_dict_field_count_for(value, "default"), Some(1));
    }

    #[test]
    fn rewrites_top_level_grouped_polymorphic_destructuring() {
        let source = "module demo =\n\tlet default_pair = (core::default, core::default)\n\tlet (a, b) = default_pair\nend\n";
        let (elaborated, _symbols) = elaborate_source(source);
        assert!(module_has_global_binding(&elaborated.module, "a"));
        assert!(module_has_global_binding(&elaborated.module, "b"));

        let a = find_global_binding(&elaborated.module, "a").expect("a binding");
        let TermKind::Function { parameter_name, .. } = &a.kind else {
            panic!("expected dictionary-wrapper function for grouped binding");
        };
        assert!(parameter_name.inner.minor.starts_with("[dict]"));
    }

    #[test]
    fn rewrites_local_grouped_refutable_polymorphic_destructuring() {
        let source = "module demo =\n\tlet result : core::Integer =\n\t\tmatch (core::default, core::default, 1) with\n\t\t| (left_local, right_local, 0) => left_local\n\t\t| _ => core::default\nend\n";
        let (elaborated, _symbols) = elaborate_source(source);
        let result = find_global_binding(&elaborated.module, "result").expect("result binding");
        assert!(term_has_local_wrapped_binding(result, "left_local#"));
        assert!(term_has_local_wrapped_binding(result, "right_local#"));
        assert!(term_has_refutable_group_guard_with_fallback(result));
    }

    #[test]
    fn does_not_inline_concrete_dictionary_for_non_trait_identifier() {
        let source = "module demo =\n\tlet append_newline = fn value => value + \"\\n\"\n\tlet alias = append_newline\nend\n";
        let (elaborated, _symbols) = elaborate_source(source);
        let alias = find_global_binding(&elaborated.module, "alias").expect("alias binding");
        assert!(!term_has_inline_dict_for(alias, "append_newline"));
    }
}
