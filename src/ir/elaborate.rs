use indexmap::IndexMap;

use crate::hc_core::{
    core_impl_arguments,
    core_impl_path,
};
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
    TraitDef,
    Type,
    TypeScheme,
};
use crate::{
    Span,
    WithSpan,
};

#[derive(Debug, Clone)]
pub struct Specialization {
    pub method_path: Path,
    pub arguments: Vec<Type>,
    pub specialized_path: Path,
}

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

#[derive(Debug, Clone)]
struct TraitMethodInfo {
    trait_name: Option<Path>,
    parameters: usize,
}

struct ElaborationContext<'a> {
    symbols: &'a SymbolTable,
    scheme_env: &'a IndexMap<Path, TypeScheme>,
    module_name: &'a str,
    dict_types: IndexMap<Path, Type>,
    dict_salt: usize,
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
        }
    }
}

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
                Statement::Type {
                    path,
                    parameters,
                    def,
                } => {
                    Statement::Type {
                        path,
                        parameters,
                        def,
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
            let bindings = pattern_bindings(&assignee);
            if bindings.len() == 1 {
                let binding = bindings.first().cloned().unwrap_or_else(|| unreachable!());
                let predicates = context
                    .scheme_env
                    .get(&binding)
                    .and_then(|scheme| instantiate_predicates_for_scheme(scheme, &value.type_))
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
    if let Some(method_info) = method_info_for_path(context.symbols, &path) {
        if let Some(arguments) =
            specialize_method_arguments(&path, &type_, &method_info, context.symbols)
        {
            return Some(Term {
                comments,
                kind: TermKind::Identifier(core_impl_path(&path, &arguments)),
                span,
                type_,
            });
        }

        let scheme = context.scheme_env.get(&path)?;
        let predicates = instantiate_predicates_for_scheme(scheme, &type_)?;
        if let Some(field) = method_field_from_dict(&path, &method_info, &predicates, dict_env) {
            return Some(Term {
                comments,
                kind: field,
                span,
                type_,
            });
        }

        if is_compare_operator(&path) {
            let lambda = compare_lambda_from_dict(
                &path,
                &type_,
                &predicates,
                dict_env,
                context.symbols,
                context.module_name,
                &mut context.dict_salt,
            )?;
            return Some(lambda);
        }

        return None;
    }

    let scheme = context.scheme_env.get(&path)?;
    let args = dictionary_args_for_type(scheme, &type_, dict_env, context.symbols)?;
    if args.is_empty() {
        return None;
    }
    let callee = Term {
        comments,
        kind: TermKind::Identifier(path),
        span,
        type_: type_.clone(),
    };
    Some(args.into_iter().fold(callee, |current, arg| {
        let current_type = current.type_.clone();
        Term {
            comments: String::new(),
            kind: TermKind::Call {
                callee: current.into(),
                argument: arg.into(),
            },
            span: Span::Generated,
            type_: current_type,
        }
    }))
}

fn method_info_for_path(
    symbols: &SymbolTable,
    path: &Path,
) -> Option<TraitMethodInfo> {
    for (trait_path, def) in symbols.trait_defs() {
        if def.methods.contains_key(path) {
            return Some(TraitMethodInfo {
                trait_name: Some(trait_path.clone()),
                parameters: def.parameters,
            });
        }
    }
    let less_equal = Path::core("(<=)");
    let greater_equal = Path::core("(>=)");
    if path == &less_equal || path == &greater_equal {
        return Some(TraitMethodInfo {
            trait_name: None,
            parameters: 1,
        });
    }
    None
}

fn is_compare_operator(path: &Path) -> bool {
    path == &Path::core("(<=)") || path == &Path::core("(>=)")
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

fn method_field_from_dict(
    path: &Path,
    method_info: &TraitMethodInfo,
    predicates: &[TraitConstraint],
    dict_env: &DictEnv,
) -> Option<TermKind<Type>> {
    let trait_name = method_info.trait_name.as_ref()?;
    if predicates.len() != 1 {
        return None;
    }
    let predicate = predicates.first()?;
    if &predicate.trait_name != trait_name {
        return None;
    }
    let binding = find_dict_binding(dict_env, predicate)?;
    let dict_term = term_identifier(binding.path.clone(), binding.type_.clone());
    Some(TermKind::Field {
        of: dict_term.into(),
        index: path.minor.clone().with_span(Span::Generated),
    })
}

fn compare_lambda_from_dict(
    path: &Path,
    type_: &Type,
    predicates: &[TraitConstraint],
    dict_env: &DictEnv,
    symbols: &SymbolTable,
    module_name: &str,
    dict_salt: &mut usize,
) -> Option<Term<Type>> {
    let compare_pred = predicates
        .iter()
        .find(|pred| pred.trait_name.minor == "compare")?;
    let equal_pred = predicates
        .iter()
        .find(|pred| pred.trait_name.minor == "equal")?;
    let compare_dict = find_dict_binding(dict_env, compare_pred)?;
    let equal_dict = find_dict_binding(dict_env, equal_pred)?;

    let (left_type, right_type, result_type) = function_pair_types(type_)?;
    if result_type != Type::Boolean {
        return None;
    }

    let compare_method = match path.minor.as_str() {
        "(<=)" => Path::core("(<)"),
        "(>=)" => Path::core("(>)"),
        _ => return None,
    };

    let compare_method_type = method_type_for_predicate(symbols, compare_pred, &compare_method)?;
    let equal_method_path = Path::core("(==)");
    let equal_method_type = method_type_for_predicate(symbols, equal_pred, &equal_method_path)?;

    let left_path = temp_path(module_name, dict_salt, "left");
    let right_path = temp_path(module_name, dict_salt, "right");

    let compare_field = Term {
        comments: String::new(),
        kind: TermKind::Field {
            of: term_identifier(compare_dict.path.clone(), compare_dict.type_.clone()).into(),
            index: compare_method.minor.clone().with_span(Span::Generated),
        },
        span: Span::Generated,
        type_: compare_method_type,
    };

    let equal_field = Term {
        comments: String::new(),
        kind: TermKind::Field {
            of: term_identifier(equal_dict.path.clone(), equal_dict.type_.clone()).into(),
            index: equal_method_path.minor.clone().with_span(Span::Generated),
        },
        span: Span::Generated,
        type_: equal_method_type,
    };

    let left_term = term_identifier(left_path.clone(), left_type.clone());
    let right_term = term_identifier(right_path.clone(), right_type.clone());

    let compare_call = call2(compare_field, left_term.clone(), right_term.clone());
    let equal_call = call2(equal_field, left_term, right_term);

    let true_term = Term {
        comments: String::new(),
        kind: TermKind::Immediate(crate::ir::ImmediateValue::Boolean(true)),
        span: Span::Generated,
        type_: Type::Boolean,
    };

    let if_term = Term {
        comments: String::new(),
        kind: TermKind::Let {
            assignee: Pattern {
                comments: String::new(),
                kind: PatternKind::Immediate(crate::ir::ImmediateValue::Boolean(true)),
                span: Span::Generated,
                type_: Type::Boolean,
            },
            scope: ScopeKind::Local,
            value: compare_call.into(),
            then: true_term.into(),
            else_: equal_call.into(),
        },
        span: Span::Generated,
        type_: Type::Boolean,
    };

    let inner = Term {
        comments: String::new(),
        kind: TermKind::Function {
            parameter_name: right_path.with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: if_term.into(),
        },
        span: Span::Generated,
        type_: Type::func(right_type, Type::Boolean),
    };

    Some(Term {
        comments: String::new(),
        kind: TermKind::Function {
            parameter_name: left_path.with_span(Span::Generated),
            parameter_type: None,
            captures: [].into(),
            body: inner.into(),
        },
        span: Span::Generated,
        type_: type_.clone(),
    })
}

fn method_type_for_predicate(
    symbols: &SymbolTable,
    predicate: &TraitConstraint,
    method_path: &Path,
) -> Option<Type> {
    let def = symbols.trait_defs().get(&predicate.trait_name)?;
    let scheme = def.methods.get(method_path)?;
    substitute_type_vars(&scheme.type_, &predicate.arguments)
}

fn call2(
    callee: Term<Type>,
    left: Term<Type>,
    right: Term<Type>,
) -> Term<Type> {
    let inner_type = call_result_type(&callee.type_).unwrap_or(Type::Unit);
    let inner = Term {
        comments: String::new(),
        kind: TermKind::Call {
            callee: callee.into(),
            argument: left.into(),
        },
        span: Span::Generated,
        type_: inner_type,
    };
    let outer_type = call_result_type(&inner.type_).unwrap_or(Type::Unit);
    Term {
        comments: String::new(),
        kind: TermKind::Call {
            callee: inner.into(),
            argument: right.into(),
        },
        span: Span::Generated,
        type_: outer_type,
    }
}

fn call_result_type(type_: &Type) -> Option<Type> {
    match type_ {
        Type::Function(_, result) => Some(result.as_ref().clone()),
        _ => None,
    }
}

fn function_pair_types(type_: &Type) -> Option<(Type, Type, Type)> {
    let Type::Function(left, rest) = type_ else {
        return None;
    };
    let Type::Function(right, result) = rest.as_ref() else {
        return None;
    };
    Some((
        left.as_ref().clone(),
        right.as_ref().clone(),
        result.as_ref().clone(),
    ))
}

fn temp_path(
    module_name: &str,
    dict_salt: &mut usize,
    label: &str,
) -> Path {
    let path = Path::new(module_name, format!("[elab] {label} #{}", *dict_salt));
    *dict_salt += 1;
    path
}

fn specialize_method_arguments(
    method_path: &Path,
    concrete_type: &Type,
    method_info: &TraitMethodInfo,
    symbols: &SymbolTable,
) -> Option<Vec<Type>> {
    let scheme = symbols.terms().get(method_path)?;
    let (scheme_body, var_count) = peel_forall(&scheme.type_);
    if var_count < method_info.parameters {
        return None;
    }
    let mut bindings = vec![None; var_count];
    if !match_scheme_to_concrete(&scheme_body, concrete_type, &mut bindings) {
        return None;
    }
    let mut arguments = Vec::with_capacity(method_info.parameters);
    for index in 0..method_info.parameters {
        let binding = bindings.get(index).and_then(|binding| binding.clone())?;
        if !is_concrete_type(&binding) {
            return None;
        }
        arguments.push(binding);
    }

    if let Some(trait_name) = &method_info.trait_name {
        let predicate = TraitConstraint {
            trait_name: trait_name.clone(),
            arguments: arguments.clone(),
        };
        if let Some(selected_impl) = symbols.select_impl(&predicate).ok().flatten() {
            return Some(core_impl_arguments(&selected_impl.head.arguments));
        }
    }

    Some(arguments)
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
    let scheme = context.scheme_env.get(path).ok_or(callee.clone())?;
    let call_type = Type::func(argument.type_.clone(), result_type.clone());
    let args = dictionary_args_for_type(scheme, &call_type, dict_env, context.symbols)
        .ok_or(callee.clone())?;

    let callee_with_args = args.into_iter().fold(callee, |current, arg| {
        let current_type = current.type_.clone();
        Term {
            comments: String::new(),
            kind: TermKind::Call {
                callee: current.into(),
                argument: arg.into(),
            },
            span: Span::Generated,
            type_: current_type,
        }
    });

    Ok(callee_with_args)
}

fn dictionary_args_for_type(
    scheme: &TypeScheme,
    type_: &Type,
    dict_env: &DictEnv,
    symbols: &SymbolTable,
) -> Option<Vec<Term<Type>>> {
    if scheme.predicates.is_empty() {
        return Some(vec![]);
    }
    let (scheme_body, var_count) = peel_forall(&scheme.type_);
    let mut bindings = vec![None; var_count];
    if !match_scheme_to_type_relaxed(&scheme_body, type_, &mut bindings) {
        return None;
    }
    let predicates = instantiate_predicates(&scheme.predicates, &bindings)?;
    let predicates = sorted_predicates(&predicates);
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
    let def = symbols.trait_defs().get(&predicate.trait_name)?;
    let methods = ordered_trait_methods(def);
    let mut fields = IndexMap::new();
    for (method_path, scheme) in methods {
        let type_ = substitute_type_vars(&scheme.type_, &predicate.arguments)?;
        fields.insert(method_path.minor.clone(), type_);
    }
    Some(Type::Struct { fields })
}

fn dictionary_term_for_predicate(
    predicate: &TraitConstraint,
    symbols: &SymbolTable,
) -> Term<Type> {
    let Some(def) = symbols.trait_defs().get(&predicate.trait_name) else {
        return generated_term(
            TermKind::Struct(IndexMap::new()),
            Type::Struct {
                fields: Default::default(),
            },
        );
    };
    let methods = ordered_trait_methods(def);
    let selected_impl = symbols.select_impl(predicate).ok().flatten();
    let impl_arguments = selected_impl
        .as_ref()
        .map(|impl_| core_impl_arguments(&impl_.head.arguments));
    let mut fields = IndexMap::new();
    let mut field_types = IndexMap::new();
    for (method_path, scheme) in methods {
        let Some(method_type) = substitute_type_vars(&scheme.type_, &predicate.arguments) else {
            continue;
        };
        let specialized_path = match &impl_arguments {
            Some(arguments) => core_impl_path(&method_path, arguments),
            None => core_impl_path(&method_path, &predicate.arguments),
        };
        fields.insert(
            method_path.minor.clone().with_span(Span::Generated),
            Term {
                comments: String::new(),
                kind: TermKind::Identifier(specialized_path),
                span: Span::Generated,
                type_: method_type.clone(),
            },
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

fn ordered_trait_methods(def: &TraitDef) -> Vec<(Path, TypeScheme)> {
    let mut methods = def
        .methods
        .iter()
        .map(|(path, scheme)| (path.clone(), scheme.clone()))
        .collect::<Vec<_>>();
    methods.sort_by(|(left, _), (right, _)| method_key(left).cmp(&method_key(right)));
    methods
}

fn method_key(path: &Path) -> (String, String) {
    (path.major.clone(), path.minor.clone())
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

fn substitute_type_vars(
    type_: &Type,
    arguments: &[Type],
) -> Option<Type> {
    let mut current = type_.clone();
    for (index, arg) in arguments.iter().enumerate() {
        current = current.substitute_type_var(index as u32, arg)?;
    }
    Some(current)
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
        Type::StructConstraint { .. }
        | Type::MetaVar(_)
        | Type::RecVar(_)
        | Type::ForAll(_)
        | Type::Mu(_) => None,
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

fn pattern_bindings(pattern: &Pattern<Type>) -> Vec<Path> {
    match &pattern.kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => {
            Vec::new()
        }
        PatternKind::Identifier(path) => vec![path.clone()],
        PatternKind::Constructor(_, inner) => pattern_bindings(inner),
        PatternKind::Tuple(items) => items.iter().flat_map(pattern_bindings).collect(),
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let mut bindings = Vec::new();
            bindings.extend(starting.iter().flat_map(pattern_bindings));
            bindings.extend(ending.iter().flat_map(pattern_bindings));
            if let crate::ir::Glob::Named(path) = glob {
                bindings.push(path.clone());
            }
            bindings
        }
        PatternKind::Struct(fields) => fields.values().flat_map(pattern_bindings).collect(),
        PatternKind::TypeHint(inner, _) => pattern_bindings(inner),
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
        Type::StructConstraint { .. }
        | Type::MetaVar(_)
        | Type::RecVar(_)
        | Type::ForAll(_)
        | Type::Mu(_) => false,
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
                    if matches!(concrete, Type::MetaVar(_)) {
                        return false;
                    }
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
        Type::StructConstraint { .. }
        | Type::MetaVar(_)
        | Type::RecVar(_)
        | Type::ForAll(_)
        | Type::Mu(_) => false,
    }
}

fn match_scheme_to_concrete(
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
                    if !is_concrete_type(concrete) {
                        return false;
                    }
                    *slot = Some(concrete.clone());
                    true
                }
            }
        }
        _ => match_scheme_to_type(scheme, concrete, bindings),
    }
}

fn is_concrete_type(type_: &Type) -> bool {
    match type_ {
        Type::TypeVar(_)
        | Type::MetaVar(_)
        | Type::RecVar(_)
        | Type::ForAll(_)
        | Type::Mu(_)
        | Type::StructConstraint { .. } => false,
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
        let _ = compile_core_module(&mut symbols);

        let modules = parse::parse(source, &mut file_logger)
            .map(|m| m.modules())
            .unwrap_or_default()
            .into_iter()
            .flat_map(|m| crate::ir::module(m, &mut file_logger))
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

    fn find_global_binding<'a>(
        module: &'a Module<Type>,
        name: &str,
    ) -> Option<&'a Term<Type>> {
        module.statements.iter().find_map(|statement| {
            let Statement::Term(term) = statement else {
                return None;
            };
            let TermKind::Let {
                assignee,
                scope: ScopeKind::Global,
                value,
                ..
            } = &term.kind
            else {
                return None;
            };
            let PatternKind::Identifier(path) = &assignee.kind else {
                return None;
            };
            (path.minor == name).then(|| value.as_ref())
        })
    }

    fn term_contains_field(term: &Term<Type>) -> bool {
        match &term.kind {
            TermKind::Field { .. } => true,
            TermKind::Let {
                value, then, else_, ..
            } => {
                term_contains_field(value)
                    || term_contains_field(then)
                    || term_contains_field(else_)
            }
            TermKind::Tuple(items) => items.iter().any(term_contains_field),
            TermKind::Struct(fields) => fields.values().any(term_contains_field),
            TermKind::Function { body, .. } => term_contains_field(body),
            TermKind::Call { callee, argument } => {
                term_contains_field(callee) || term_contains_field(argument)
            }
            TermKind::Semicolon(left, right) => {
                term_contains_field(left) || term_contains_field(right)
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
            TermKind::Field { of, .. } => term_has_inline_dict_for(of, name),
            TermKind::Semicolon(left, right) => {
                term_has_inline_dict_for(left, name) || term_has_inline_dict_for(right, name)
            }
            _ => false,
        }
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
        assert!(term_contains_field(double));
    }

    #[test]
    fn inlines_dictionary_at_callsite() {
        let source = "module demo =\n\tlet double = fn x => x + x\n\tlet result = double 3\nend\n";
        let (elaborated, _symbols) = elaborate_source(source);
        let result = find_global_binding(&elaborated.module, "result").expect("result binding");
        assert!(term_has_inline_dict_for(result, "double"));
    }
}
