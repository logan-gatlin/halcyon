use crate::types::Type;

use indexmap::IndexMap;

use super::*;

impl Module<()> {
    pub fn pretty(&self) -> String {
        pretty_module(self, &|_| None)
    }
}

impl Module<Type> {
    pub fn pretty(&self) -> String {
        pretty_module(self, &|type_| Some(type_.to_string()))
    }
}

fn pretty_module<T>(
    module: &Module<T>,
    type_of: &impl Fn(&T) -> Option<String>,
) -> String {
    let statements = module
        .statements
        .iter()
        .map(|statement| pretty_statement(statement, type_of))
        .collect::<Vec<_>>();
    if statements.is_empty() {
        return format!("module {}", module.name);
    }
    format!(
        "module {} {{\n{}\n}}",
        module.name,
        indent(&statements.join("\n"))
    )
}

fn pretty_statement<T>(
    statement: &Statement<T>,
    type_of: &impl Fn(&T) -> Option<String>,
) -> String {
    match statement {
        Statement::Term(term) => pretty_term(term, type_of),
        Statement::Type {
            path,
            parameters,
            def,
        } => {
            let params = if parameters.is_empty() {
                String::new()
            } else {
                format!(
                    " {}",
                    parameters
                        .iter()
                        .map(format_path)
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };
            format!(
                "type {}{} = {}",
                format_path(path),
                params,
                pretty_type_def(def)
            )
        }
    }
}

fn pretty_type_def(def: &TypeDef) -> String {
    match def.kind() {
        TypeDefKind::Struct(fields) => {
            format!("struct {{{}}}", pretty_field_types(fields))
        }
        TypeDefKind::Sum(variants) => {
            format!("sum {{{}}}", pretty_field_types(variants))
        }
        TypeDefKind::Expr(expr) => pretty_type_expr(expr),
    }
}

fn pretty_field_types(fields: &IndexMap<String, TypeExpr>) -> String {
    fields
        .iter()
        .map(|(name, type_expr)| format!("{}: {}", name, pretty_type_expr(type_expr)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn pretty_type_expr(expr: &TypeExpr) -> String {
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            let items = items
                .iter()
                .map(pretty_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({items})")
        }
        TypeExprKind::Instantiation(path, args) => {
            if args.is_empty() {
                format_path(path)
            } else {
                let args = args
                    .iter()
                    .map(pretty_type_expr)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", format_path(path), args)
            }
        }
    }
}

fn pretty_term<T>(
    term: &Term<T>,
    type_of: &impl Fn(&T) -> Option<String>,
) -> String {
    let base = match &term.kind {
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            let scope = match scope {
                ScopeKind::Local => "local",
                ScopeKind::Global => "global",
            };
            format!(
                "let[{scope}] {} = {} then {} else {}",
                pretty_pattern(assignee, type_of),
                pretty_term(value, type_of),
                pretty_term(then, type_of),
                pretty_term(else_, type_of)
            )
        }
        TermKind::Immediate(value) => value.to_string(),
        TermKind::Identifier(path) => format_path(path),
        TermKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| pretty_term(item, type_of))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({items})")
        }
        TermKind::Struct(fields) => {
            let fields = fields
                .iter()
                .map(|(name, value)| format!("{} = {}", name.inner, pretty_term(value, type_of)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{fields}}}")
        }
        TermKind::Field { of, index } => {
            format!("{}.{}", pretty_term(of, type_of), index.inner)
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            let param = format_path(&parameter_name.inner);
            let param = match parameter_type {
                Some(type_expr) => format!("{param}: {}", pretty_type_expr(type_expr)),
                None => param,
            };
            let captures = pretty_captures(captures, type_of);
            format!("fn {param} => {}{captures}", pretty_term(body, type_of))
        }
        TermKind::Call { callee, argument } => {
            format!(
                "call {} {}",
                pretty_term(callee, type_of),
                pretty_term(argument, type_of)
            )
        }
        TermKind::Semicolon(left, right) => {
            format!(
                "{} ; {}",
                pretty_term(left, type_of),
                pretty_term(right, type_of)
            )
        }
        TermKind::Unreachable => "unreachable".to_string(),
    };
    annotate(base, &term.type_, type_of)
}

fn pretty_captures<T>(
    captures: &[(Path, T)],
    type_of: &impl Fn(&T) -> Option<String>,
) -> String {
    if captures.is_empty() {
        return String::new();
    }
    let captures = captures
        .iter()
        .map(|(path, type_)| annotate(format_path(path), type_, type_of))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" captures [{captures}]")
}

fn pretty_pattern<T>(
    pattern: &Pattern<T>,
    type_of: &impl Fn(&T) -> Option<String>,
) -> String {
    let base = match &pattern.kind {
        PatternKind::Hole => "_".to_string(),
        PatternKind::Identifier(path) => format_path(path),
        PatternKind::ConstConstructor(path) => format_path(path),
        PatternKind::Constructor(path, payload) => {
            format!("{} {}", format_path(path), pretty_pattern(payload, type_of))
        }
        PatternKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| pretty_pattern(item, type_of))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({items})")
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let mut items = Vec::new();
            items.extend(starting.iter().map(|item| pretty_pattern(item, type_of)));
            match glob {
                Glob::None => {}
                Glob::Anonymous => items.push("..".to_string()),
                Glob::Named(path) => items.push(format!("..{}", format_path(path))),
            }
            items.extend(ending.iter().map(|item| pretty_pattern(item, type_of)));
            format!("[{}]", items.join(", "))
        }
        PatternKind::Struct(fields) => {
            let fields = fields
                .iter()
                .map(|(name, value)| format!("{} = {}", name.inner, pretty_pattern(value, type_of)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{fields}}}")
        }
        PatternKind::Immediate(value) => value.to_string(),
        PatternKind::TypeHint(inner, type_expr) => {
            format!(
                "{} : {}",
                pretty_pattern(inner, type_of),
                pretty_type_expr(type_expr)
            )
        }
    };
    annotate(base, &pattern.type_, type_of)
}

fn annotate<T>(
    text: String,
    type_: &T,
    type_of: &impl Fn(&T) -> Option<String>,
) -> String {
    match type_of(type_) {
        Some(type_) => format!("{text} : {type_}"),
        None => text,
    }
}

fn format_path(path: &Path) -> String {
    if path.major.is_empty() {
        path.minor.clone()
    } else {
        format!("{}::{}", path.major, path.minor)
    }
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
