use crate::types::Type;

use indexmap::IndexMap;

use super::*;

const INDENT_WIDTH: usize = 2;
const LINE_LIMIT: usize = 96;

impl std::fmt::Display for Module<Type> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let mut printer = Printer::new(Some(self.name.clone()));
        printer.module(self);
        write!(f, "{}", printer.finish())
    }
}

pub fn pretty_typed_term(term: &Term<Type>) -> String {
    let mut printer = Printer::new(None);
    printer.term(term);
    printer.finish()
}

pub fn pretty_typed_pattern(pattern: &Pattern<Type>) -> String {
    let mut printer = Printer::new(None);
    printer.line(printer.format_pattern_typed(pattern));
    printer.finish()
}

struct MatchArm<'a> {
    pattern: &'a Pattern<Type>,
    body: &'a Term<Type>,
}

struct MatchView<'a> {
    scrutinee: &'a Term<Type>,
    arms: Vec<MatchArm<'a>>,
}

struct Printer {
    indent: usize,
    output: String,
    module_name: Option<String>,
}

impl Printer {
    fn new(module_name: Option<String>) -> Self {
        Self {
            indent: 0,
            output: String::new(),
            module_name,
        }
    }

    fn finish(self) -> String {
        self.output
    }

    fn line(
        &mut self,
        text: impl AsRef<str>,
    ) {
        let text = text.as_ref();
        if !text.is_empty() {
            self.output.push_str(&" ".repeat(self.indent));
            self.output.push_str(text);
        }
        self.output.push('\n');
    }

    fn indented(
        &mut self,
        f: impl FnOnce(&mut Self),
    ) {
        self.indent += INDENT_WIDTH;
        f(self);
        self.indent = self.indent.saturating_sub(INDENT_WIDTH);
    }

    fn module(
        &mut self,
        module: &Module<Type>,
    ) {
        self.line(format!("module {} =", module.name));
        self.indented(|printer| {
            if module.statements.is_empty() {
                printer.line("<empty>");
                return;
            }
            for (index, statement) in module.statements.iter().enumerate() {
                if index > 0 {
                    printer.line("");
                }
                printer.statement(statement);
            }
        });
        self.line("end");
    }

    fn statement(
        &mut self,
        statement: &Statement<Type>,
    ) {
        match statement {
            Statement::Term(term) => self.term(term),
            Statement::Type {
                path,
                parameters,
                def,
            } => self.type_statement(path, parameters, def),
        }
    }

    fn type_statement(
        &mut self,
        path: &Path,
        parameters: &[Path],
        def: &TypeDef,
    ) {
        let params = if parameters.is_empty() {
            String::new()
        } else {
            let names = parameters
                .iter()
                .map(|param| self.format_path(param))
                .collect::<Vec<_>>()
                .join(" ");
            format!(" : {names}")
        };
        if let Some(def_inline) = self.format_type_def_inline(def) {
            let line = format!("type {}{params} = {def_inline}", self.format_path(path));
            if line.len() <= LINE_LIMIT {
                self.line(line);
                return;
            }
        }
        self.line(format!("type {}{params} =", self.format_path(path)));
        self.indented(|printer| printer.type_def(def));
    }

    fn term(
        &mut self,
        term: &Term<Type>,
    ) {
        if let Some(view) = match_view(term) {
            self.term_match(term, view);
            return;
        }
        if let Some(line) = self.format_term_inline(term) {
            self.line(line);
            return;
        }

        match &term.kind {
            TermKind::Let {
                assignee,
                scope,
                value,
                then,
                else_,
            } => self.term_let(term, assignee, *scope, value, then, else_),
            TermKind::Immediate(value) => {
                self.line(format!("{value} : {}", term.type_.pretty()));
            }
            TermKind::Identifier(path) => {
                self.line(format!(
                    "{} : {}",
                    self.format_path(path),
                    term.type_.pretty()
                ));
            }
            TermKind::Tuple(items) => self.term_tuple(term, items),
            TermKind::Struct(fields) => self.term_struct(term, fields),
            TermKind::Field { of, index } => self.term_field(term, of, index),
            TermKind::Function {
                parameter_name,
                parameter_type,
                body,
                ..
            } => self.term_function(term, parameter_name, parameter_type.as_ref(), body),
            TermKind::Call { callee, argument } => self.term_call(term, callee, argument),
            TermKind::Semicolon(left, right) => self.term_semicolon(term, left, right),
            TermKind::Unreachable => {
                self.line(format!("unreachable : {}", term.type_.pretty()));
            }
        }
    }

    fn term_let(
        &mut self,
        term: &Term<Type>,
        assignee: &Pattern<Type>,
        scope: ScopeKind,
        value: &Term<Type>,
        then: &Term<Type>,
        else_: &Term<Type>,
    ) {
        if scope == ScopeKind::Global {
            let pattern = self.format_pattern_typed(assignee);
            let header = format!("let {pattern} =");
            if header.len() <= LINE_LIMIT {
                self.line(header);
            } else {
                self.line("let");
                self.indented(|printer| printer.line(format!("{pattern} =")));
            }
            self.indented(|printer| printer.term(value));
            return;
        }
        if scope == ScopeKind::Local
            && matches!(
                assignee.kind,
                PatternKind::Immediate(ImmediateValue::Boolean(true))
            )
            && !is_unreachable(else_)
        {
            let condition = self.format_term_inline_expr(value);
            if let Some(condition) = condition {
                let line = format!("if {condition}");
                if line.len() <= LINE_LIMIT {
                    self.line(line);
                } else {
                    self.line("if");
                    self.indented(|printer| printer.term(value));
                }
            } else {
                self.line("if");
                self.indented(|printer| printer.term(value));
            }
            self.line("then");
            self.indented(|printer| printer.term(then));
            self.line("else");
            self.indented(|printer| printer.term(else_));
            self.line(format!(": {}", term.type_.pretty()));
            return;
        }

        let pattern = self.format_pattern_typed(assignee);
        let header = format!("let {pattern} =");
        if header.len() <= LINE_LIMIT {
            self.line(header);
        } else {
            self.line("let");
            self.indented(|printer| printer.line(format!("{pattern} =")));
        }
        self.indented(|printer| printer.term(value));

        if scope == ScopeKind::Local && is_unreachable(else_) {
            self.line("in");
            self.indented(|printer| printer.term(then));
        } else {
            self.line("then");
            self.indented(|printer| printer.term(then));
            self.line("else");
            self.indented(|printer| printer.term(else_));
        }
        self.line(format!(": {}", term.type_.pretty()));
    }

    fn term_match(
        &mut self,
        term: &Term<Type>,
        view: MatchView<'_>,
    ) {
        let inline = self.format_term_inline_expr(view.scrutinee);
        if let Some(inline) = inline {
            let line = format!("match {inline} with");
            if line.len() <= LINE_LIMIT {
                self.line(line);
            } else {
                self.line("match");
                self.indented(|printer| printer.term(view.scrutinee));
                self.line("with");
            }
        } else {
            self.line("match");
            self.indented(|printer| printer.term(view.scrutinee));
            self.line("with");
        }
        self.indented(|printer| {
            for arm in view.arms {
                let pattern = printer.format_pattern_typed(arm.pattern);
                if let Some(body_inline) = printer.format_term_inline(arm.body) {
                    let line = format!("| {pattern} => {body_inline}");
                    if line.len() <= LINE_LIMIT {
                        printer.line(line);
                        continue;
                    }
                }
                printer.line(format!("| {pattern} =>"));
                printer.indented(|printer| printer.term(arm.body));
            }
        });
        self.line(format!(": {}", term.type_.pretty()));
    }

    fn term_function(
        &mut self,
        term: &Term<Type>,
        parameter_name: &Spanned<Path>,
        parameter_type: Option<&TypeExpr>,
        body: &Term<Type>,
    ) {
        let name = self.format_path(&parameter_name.inner);
        let param = match parameter_type {
            Some(type_expr) => {
                let ty = self.format_type_expr(type_expr);
                format!("({name} : {ty})")
            }
            None => name,
        };
        self.line(format!("fn {param} =>"));
        self.indented(|printer| printer.term(body));
        self.line(format!(": {}", term.type_.pretty()));
    }

    fn term_call(
        &mut self,
        term: &Term<Type>,
        callee: &Term<Type>,
        argument: &Term<Type>,
    ) {
        self.line("(");
        self.indented(|printer| {
            printer.term(callee);
            printer.term(argument);
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn term_tuple(
        &mut self,
        term: &Term<Type>,
        items: &[Term<Type>],
    ) {
        self.line("(");
        self.indented(|printer| {
            if items.is_empty() {
                printer.line("()");
                return;
            }
            for item in items.iter() {
                printer.term(item);
            }
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn term_struct(
        &mut self,
        term: &Term<Type>,
        fields: &IndexMap<Spanned<String>, Term<Type>>,
    ) {
        self.line("{");
        self.indented(|printer| {
            if fields.is_empty() {
                printer.line("{}");
                return;
            }
            for (name, value) in fields.iter() {
                if let Some(inline) = printer.format_term_inline(value) {
                    printer.line(format!("{name} = {inline}"));
                } else {
                    printer.line(format!("{name} ="));
                    printer.indented(|printer| printer.term(value));
                }
            }
        });
        self.line(format!("}} : {}", term.type_.pretty()));
    }

    fn term_field(
        &mut self,
        term: &Term<Type>,
        of: &Term<Type>,
        index: &Spanned<String>,
    ) {
        self.line("(");
        self.indented(|printer| printer.term(of));
        self.line(format!(").{index} : {}", term.type_.pretty()));
    }

    fn term_semicolon(
        &mut self,
        term: &Term<Type>,
        left: &Term<Type>,
        right: &Term<Type>,
    ) {
        self.line("(");
        self.indented(|printer| {
            if let Some(line) = printer.format_term_inline(left) {
                printer.line(format!("{line};"));
            } else {
                printer.term(left);
                printer.line(";");
            }
            printer.term(right);
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn type_def(
        &mut self,
        def: &TypeDef,
    ) {
        match def.kind() {
            TypeDefKind::Struct(fields) => {
                self.line("{");
                self.indented(|printer| {
                    if fields.is_empty() {
                        printer.line("{}");
                        return;
                    }
                    for (name, type_expr) in fields.iter() {
                        printer.line(format!("{name}: {}", printer.format_type_expr(type_expr)));
                    }
                });
                self.line("}");
            }
            TypeDefKind::Sum(variants) => {
                if variants.is_empty() {
                    self.line("|");
                    return;
                }
                for (name, type_expr) in variants.iter() {
                    let expr = self.format_type_expr(type_expr);
                    if expr == "()" {
                        self.line(format!("| {name}"));
                    } else {
                        self.line(format!("| {name} {expr}"));
                    }
                }
            }
            TypeDefKind::Expr(expr) => {
                self.line(self.format_type_expr(expr));
            }
        }
    }

    fn format_type_def_inline(
        &self,
        def: &TypeDef,
    ) -> Option<String> {
        match def.kind() {
            TypeDefKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, type_expr)| {
                        format!("{name}: {}", self.format_type_expr(type_expr))
                    })
                    .collect::<Vec<_>>();
                Some(format!("{{ {} }}", fields.join(", ")))
            }
            TypeDefKind::Sum(variants) => {
                if variants.is_empty() {
                    return Some("|".to_string());
                }
                let variants = variants
                    .iter()
                    .map(|(name, type_expr)| {
                        let expr = self.format_type_expr(type_expr);
                        if expr == "()" {
                            name.clone()
                        } else {
                            format!("{name} {expr}")
                        }
                    })
                    .collect::<Vec<_>>();
                Some(format!("| {}", variants.join(" | ")))
            }
            TypeDefKind::Expr(expr) => Some(self.format_type_expr(expr)),
        }
    }

    #[allow(clippy::missing_asserts_for_indexing)]
    fn format_type_expr(
        &self,
        expr: &TypeExpr,
    ) -> String {
        match &expr.kind {
            TypeExprKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format_type_expr(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
            TypeExprKind::Instantiation(path, args) => {
                let is_core = path.major == "core";
                if is_core && path.minor == "unit" && args.is_empty() {
                    return "()".to_string();
                }
                if is_core && path.minor == "array" && args.is_empty() {
                    return "[]".to_string();
                }
                if is_core && path.minor == "array" && args.len() == 1 {
                    let inner = self.format_type_expr(&args[0]);
                    return format!("[] {}", self.wrap_type_expr(&inner));
                }
                if is_core && path.minor == "function" && args.len() == 2 {
                    let param = self.wrap_type_expr(&self.format_type_expr(&args[0]));
                    let result = self.format_type_expr(&args[1]);
                    return format!("{param} -> {result}");
                }
                let name = self.format_path(path);
                if args.is_empty() {
                    return name;
                }
                let args = args
                    .iter()
                    .map(|arg| self.wrap_type_expr(&self.format_type_expr(arg)))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{name} {args}")
            }
        }
    }

    fn format_pattern_typed(
        &self,
        pattern: &Pattern<Type>,
    ) -> String {
        let raw = self.format_pattern(pattern);
        let wrapped = self.wrap_pattern(&raw);
        format!("{wrapped} : {}", pattern.type_.pretty())
    }

    fn format_pattern(
        &self,
        pattern: &Pattern<Type>,
    ) -> String {
        match &pattern.kind {
            PatternKind::Hole => "_".to_string(),
            PatternKind::Identifier(path) => self.format_path(path),
            PatternKind::ConstConstructor(path) => self.format_path(path),
            PatternKind::Constructor(path, payload) => {
                let payload = self.wrap_pattern(&self.format_pattern(payload));
                format!("{} of {payload}", self.format_path(path))
            }
            PatternKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format_pattern(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
            PatternKind::Array {
                starting,
                glob,
                ending,
            } => {
                let mut parts = Vec::new();
                parts.extend(starting.iter().map(|item| self.format_pattern(item)));
                match glob {
                    Glob::None => {}
                    Glob::Anonymous => parts.push("..".to_string()),
                    Glob::Named(path) => parts.push(format!("..{}", self.format_path(path))),
                }
                parts.extend(ending.iter().map(|item| self.format_pattern(item)));
                format!("[{}]", parts.join(", "))
            }
            PatternKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", self.format_pattern(value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {fields} }}")
            }
            PatternKind::Immediate(value) => value.to_string(),
            PatternKind::TypeHint(inner, type_expr) => {
                let inner = self.wrap_pattern(&self.format_pattern(inner));
                let type_expr = self.format_type_expr(type_expr);
                format!("{inner} : {type_expr}")
            }
        }
    }

    fn format_term_inline(
        &self,
        term: &Term<Type>,
    ) -> Option<String> {
        let expr = self.format_term_inline_expr(term)?;
        let line = format!("{expr} : {}", term.type_.pretty());
        (line.len() <= LINE_LIMIT).then_some(line)
    }

    fn format_term_wrapped_expr(
        &self,
        term: &Term<Type>,
    ) -> Option<String> {
        let expr = self.format_term_inline_expr(term)?;
        let expr = self.wrap_inline_expr(&expr);
        (expr.len() <= LINE_LIMIT).then_some(expr)
    }

    fn format_term_inline_expr(
        &self,
        term: &Term<Type>,
    ) -> Option<String> {
        match &term.kind {
            TermKind::Immediate(value) => Some(value.to_string()),
            TermKind::Identifier(path) => Some(self.format_path(path)),
            TermKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format_term_inline_expr(item))
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("({})", items.join(", ")))
            }
            TermKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| {
                        Some(format!("{name} = {}", self.format_term_inline_expr(value)?))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("{{ {} }}", fields.join(", ")))
            }
            TermKind::Field { of, index } => {
                let base = self.format_term_wrapped_expr(of)?;
                Some(format!("{base}.{index}"))
            }
            TermKind::Call { callee, argument } => {
                let callee = self.format_term_wrapped_expr(callee)?;
                let argument = self.format_term_wrapped_expr(argument)?;
                Some(format!("{callee} {argument}"))
            }
            TermKind::Semicolon(left, right) => {
                let left = self.format_term_wrapped_expr(left)?;
                let right = self.format_term_wrapped_expr(right)?;
                Some(format!("{left}; {right}"))
            }
            TermKind::Unreachable => Some("unreachable".to_string()),
            TermKind::Let { .. } | TermKind::Function { .. } => None,
        }
    }

    fn format_path(
        &self,
        path: &Path,
    ) -> String {
        match &self.module_name {
            Some(module) if module == &path.major => path.minor.clone(),
            _ => path.to_string(),
        }
    }

    fn wrap_pattern(
        &self,
        value: &str,
    ) -> String {
        if value.contains(' ') || value.contains(':') || value.contains('|') {
            format!("({value})")
        } else {
            value.to_string()
        }
    }

    fn wrap_type_expr(
        &self,
        value: &str,
    ) -> String {
        if value.contains(' ') || value.contains("->") || value.contains('|') {
            format!("({value})")
        } else {
            value.to_string()
        }
    }

    fn wrap_inline_expr(
        &self,
        expr: &str,
    ) -> String {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return "()".to_string();
        }
        if is_inline_atom(trimmed) {
            trimmed.to_string()
        } else {
            format!("({trimmed})")
        }
    }
}

fn is_unreachable(term: &Term<Type>) -> bool {
    matches!(term.kind, TermKind::Unreachable)
}

fn match_view<'a>(term: &'a Term<Type>) -> Option<MatchView<'a>> {
    let TermKind::Let {
        assignee,
        scope: ScopeKind::Local,
        value,
        then,
        else_,
    } = &term.kind
    else {
        return None;
    };
    let PatternKind::Identifier(scrutinee_path) = &assignee.kind else {
        return None;
    };
    if !is_unreachable(else_) {
        return None;
    }
    let mut arms = Vec::new();
    let mut current = then.as_ref();
    loop {
        let TermKind::Let {
            assignee,
            scope: ScopeKind::Local,
            value,
            then,
            else_,
        } = &current.kind
        else {
            return None;
        };
        let TermKind::Identifier(candidate) = &value.kind else {
            return None;
        };
        if candidate != scrutinee_path {
            return None;
        }
        arms.push(MatchArm {
            pattern: assignee,
            body: then,
        });
        if is_unreachable(else_) {
            break;
        }
        current = else_.as_ref();
    }
    if arms.len() <= 1 {
        return None;
    }
    Some(MatchView {
        scrutinee: value,
        arms,
    })
}

fn is_inline_atom(expr: &str) -> bool {
    if expr.starts_with('(') || expr.starts_with('{') || expr.starts_with('[') {
        return true;
    }
    if expr.contains(' ') || expr.contains(';') || expr.contains("=>") {
        return false;
    }
    expr.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '_' | ':' | '#' | '-' | '+' | '*' | '/' | '%' | '<' | '>' | '=' | '!'
            )
            || ch == '.'
            || ch == '\''
            || ch == '"'
    })
}
