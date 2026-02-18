use std::fs;
use std::path::Path;

use halcyon_lib::Logger;
use halcyon_lib::parse::ast::{
    self,
    AstNode,
    HasLeadingComments,
    HasName,
};
use halcyon_lib::parse::{
    SyntaxKind,
    SyntaxNode,
};

const INDENT: &str = "  ";

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!("Usage: halcyon-format <file>...");
        std::process::exit(2);
    }
    if let Err(err) = format_files(&args) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn format_files(paths: &[String]) -> std::io::Result<()> {
    for path in paths {
        format_file(Path::new(path))?;
    }
    Ok(())
}

fn format_file(path: &Path) -> std::io::Result<()> {
    let source = fs::read_to_string(path)?;
    let Some(formatted) = format_source(path, &source) else {
        return Ok(());
    };
    if formatted != source {
        fs::write(path, formatted)?;
    }
    Ok(())
}

fn format_source(
    path: &Path,
    source: &str,
) -> Option<String> {
    format_source_text(&path.display().to_string(), source)
}

fn format_source_text(
    name: &str,
    source: &str,
) -> Option<String> {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file(name, source.to_string());
    let tree = halcyon_lib::parse::parse(source, &mut file_logger);
    if !file_logger.is_ok() {
        logger.consume_file(file_logger);
        logger.print_logs();
        return None;
    }
    let tree = tree?;
    Some(format_source_file(tree.syntax()))
}

fn format_source_file(root: &SyntaxNode) -> String {
    let items = source_items(root);
    let mut lines = Vec::new();
    let mut previous_block = false;
    for item in items {
        let (item_lines, is_block) = match item {
            SourceItem::Module(module) => (format_module(&module).lines, true),
            SourceItem::Statement(statement) => (format_statement(&statement).lines, true),
            SourceItem::Comment(comment_lines) => (comment_lines, false),
        };
        if previous_block && is_block && !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(item_lines);
        previous_block = is_block;
    }
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[derive(Clone)]
enum SourceItem {
    Module(ast::Module),
    Statement(ast::Statement),
    Comment(Vec<String>),
}

fn source_items(root: &SyntaxNode) -> Vec<SourceItem> {
    root.children_with_tokens()
        .filter_map(|element| {
            if let Some(node) = element.clone().into_node() {
                ast::Module::cast(node.clone())
                    .map(SourceItem::Module)
                    .or_else(|| ast::Statement::cast(node).map(SourceItem::Statement))
            } else if let Some(token) = element.into_token()
                && is_comment_kind(token.kind())
            {
                Some(SourceItem::Comment(comment_token_lines(&token)))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone, Debug)]
struct Formatted {
    lines: Vec<String>,
}

impl Formatted {
    fn single(line: impl Into<String>) -> Self {
        Self {
            lines: vec![line.into()],
        }
    }

    fn from_lines(lines: Vec<String>) -> Self {
        Self { lines }
    }

    fn is_multiline(&self) -> bool {
        self.lines.len() > 1
    }

    fn indent(
        &self,
        level: usize,
    ) -> Self {
        let prefix = INDENT.repeat(level);
        let lines = self
            .lines
            .iter()
            .map(|line| format!("{prefix}{line}"))
            .collect();
        Self { lines }
    }

    fn flatten(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn format_module(module: &ast::Module) -> Formatted {
    let name = module.name_text().unwrap_or_default();
    let mut lines = vec![format!("module {name} =")];
    let items = module_items(module);
    for item in items {
        match item {
            ModuleItem::Statement(statement) => {
                lines.extend(format_statement(&statement).indent(1).lines);
            }
            ModuleItem::Comment(comment_lines) => {
                let indented = comment_lines
                    .iter()
                    .map(|line| format!("{}{}", INDENT, line))
                    .collect::<Vec<_>>();
                lines.extend(indented);
            }
        }
    }
    lines.push("end".to_string());
    Formatted::from_lines(lines)
}

#[derive(Clone)]
enum ModuleItem {
    Statement(ast::Statement),
    Comment(Vec<String>),
}

fn module_items(module: &ast::Module) -> Vec<ModuleItem> {
    module
        .syntax()
        .children_with_tokens()
        .filter_map(|element| {
            if let Some(node) = element.clone().into_node() {
                ast::Statement::cast(node).map(ModuleItem::Statement)
            } else if let Some(token) = element.into_token()
                && is_comment_kind(token.kind())
            {
                Some(ModuleItem::Comment(comment_token_lines(&token)))
            } else {
                None
            }
        })
        .collect()
}

fn format_statement(statement: &ast::Statement) -> Formatted {
    let mut lines = Vec::new();
    lines.extend(format_leading_comments(statement));
    let stmt_lines = match statement {
        ast::Statement::Let(let_statement) => format_let_statement(let_statement),
        ast::Statement::Type(type_statement) => format_type_statement(type_statement),
    };
    lines.extend(stmt_lines.lines);
    Formatted::from_lines(lines)
}

fn format_leading_comments(statement: &ast::Statement) -> Vec<String> {
    statement
        .leading_comments()
        .iter()
        .flat_map(comment_token_lines)
        .collect()
}

fn is_comment_kind(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT)
}

fn comment_token_lines(token: &halcyon_lib::parse::SyntaxToken) -> Vec<String> {
    token
        .text()
        .trim_end()
        .lines()
        .map(str::to_string)
        .collect()
}

fn format_let_statement(let_statement: &ast::LetStatement) -> Formatted {
    let pattern = let_statement
        .pattern()
        .map(|pat| format_pattern(&pat))
        .unwrap_or_default();
    let value = let_statement
        .value()
        .map(|expr| format_expr(&expr))
        .unwrap_or_else(|| Formatted::single(String::new()));
    if value.is_multiline() {
        let mut lines = vec![format!("let {pattern} =")];
        lines.extend(value.indent(1).lines);
        Formatted::from_lines(lines)
    } else {
        Formatted::single(format!("let {pattern} = {}", value.flatten()))
    }
}

fn format_type_statement(type_statement: &ast::TypeStatement) -> Formatted {
    let name = type_statement.name_text().unwrap_or_default();
    let params = type_statement
        .type_params()
        .iter()
        .map(|param| param.inner.clone())
        .collect::<Vec<_>>();
    let head = if params.is_empty() {
        format!("type {name}")
    } else {
        format!("type {name}: {}", params.join(" "))
    };
    let body = type_statement
        .type_def()
        .map(|def| format_type_def(&def))
        .unwrap_or_default();
    Formatted::single(format!("{head} = {body}"))
}

fn format_type_def(type_def: &ast::TypeDef) -> String {
    match type_def {
        ast::TypeDef::Struct(def) => format_struct_def(def),
        ast::TypeDef::Sum(def) => format_sum_def(def),
        ast::TypeDef::Alias(def) => {
            def.type_expr()
                .map(|ty| format_type_expr(&ty))
                .unwrap_or_default()
        }
    }
}

fn format_struct_def(def: &ast::StructDef) -> String {
    let fields = def
        .fields()
        .iter()
        .map(|field| {
            let name = field.name_text().unwrap_or_default();
            let ty = field
                .ty()
                .map(|ty| format_type_expr(&ty))
                .unwrap_or_default();
            format!("{name}: {ty}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {fields} }}")
}

fn format_sum_def(def: &ast::SumDef) -> String {
    def.variants()
        .iter()
        .map(format_variant)
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_variant(variant: &ast::Variant) -> String {
    let name = variant.name_text().unwrap_or_default();
    let payload = variant.payload_type().map(|ty| format_type_expr(&ty));
    match payload {
        Some(payload) => format!("| {name} {payload}"),
        None => format!("| {name}"),
    }
}

const TYPE_PREC_LOWEST: u8 = 0;
const TYPE_PREC_ARROW: u8 = 10;
const TYPE_PREC_APPLY: u8 = 20;
const TYPE_PREC_ATOM: u8 = 30;

fn type_expr_precedence(type_expr: &ast::TypeExpr) -> u8 {
    match type_expr {
        ast::TypeExpr::Function(_) => TYPE_PREC_ARROW,
        ast::TypeExpr::Application(_) => TYPE_PREC_APPLY,
        ast::TypeExpr::Tuple(_)
        | ast::TypeExpr::Array(_)
        | ast::TypeExpr::Unit(_)
        | ast::TypeExpr::Path(_) => TYPE_PREC_ATOM,
    }
}

fn format_type_expr(type_expr: &ast::TypeExpr) -> String {
    format_type_expr_prec(type_expr, TYPE_PREC_LOWEST)
}

fn format_type_expr_prec(
    type_expr: &ast::TypeExpr,
    parent_prec: u8,
) -> String {
    let prec = type_expr_precedence(type_expr);
    let inner = match type_expr {
        ast::TypeExpr::Function(ft) => {
            let param = ft
                .param_type()
                .map(|ty| format_type_expr_prec(&ty, TYPE_PREC_ARROW + 1))
                .unwrap_or_default();
            let ret = ft
                .return_type()
                .map(|ty| format_type_expr_prec(&ty, TYPE_PREC_ARROW))
                .unwrap_or_default();
            format!("{param} -> {ret}")
        }
        ast::TypeExpr::Application(app) => {
            let base = app
                .base()
                .map(|ty| format_type_expr_prec(&ty, TYPE_PREC_APPLY))
                .unwrap_or_default();
            let args = app
                .args()
                .iter()
                .map(|ty| format_type_expr_prec(ty, TYPE_PREC_APPLY + 1))
                .collect::<Vec<_>>()
                .join(" ");
            if args.is_empty() {
                base
            } else {
                format!("{base} {args}")
            }
        }
        ast::TypeExpr::Tuple(tuple) => {
            let fields = tuple
                .fields()
                .iter()
                .map(|ty| format_type_expr_prec(ty, TYPE_PREC_LOWEST))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({fields})")
        }
        ast::TypeExpr::Array(_) => "[]".to_string(),
        ast::TypeExpr::Unit(_) => "()".to_string(),
        ast::TypeExpr::Path(path) => format_path(path),
    };
    if prec < parent_prec {
        format!("({inner})")
    } else {
        inner
    }
}

const EXPR_PREC_LOWEST: u8 = 0;
const EXPR_PREC_SEMICOLON: u8 = 2;
const EXPR_PREC_PIPE: u8 = 4;
const EXPR_PREC_OR: u8 = 6;
const EXPR_PREC_AND: u8 = 8;
const EXPR_PREC_COMPARE: u8 = 10;
const EXPR_PREC_COMPOSE: u8 = 12;
const EXPR_PREC_ADD: u8 = 14;
const EXPR_PREC_MUL: u8 = 16;
const EXPR_PREC_UNARY: u8 = 20;
const EXPR_PREC_CALL: u8 = 24;
const EXPR_PREC_FIELD: u8 = 34;
const EXPR_PREC_ATOM: u8 = 40;

fn binary_op_precedence(kind: SyntaxKind) -> u8 {
    match kind {
        SyntaxKind::SEMICOLON => EXPR_PREC_SEMICOLON,
        SyntaxKind::PIPE_ARROW => EXPR_PREC_PIPE,
        SyntaxKind::OR_KW => EXPR_PREC_OR,
        SyntaxKind::AND_KW => EXPR_PREC_AND,
        SyntaxKind::DOUBLE_EQUAL
        | SyntaxKind::BANG_EQUAL
        | SyntaxKind::LESS
        | SyntaxKind::LESS_EQUAL
        | SyntaxKind::GREATER
        | SyntaxKind::GREATER_EQUAL => EXPR_PREC_COMPARE,
        SyntaxKind::COMPOSE_LEFT | SyntaxKind::COMPOSE_RIGHT | SyntaxKind::XOR_KW => {
            EXPR_PREC_COMPOSE
        }
        SyntaxKind::PLUS | SyntaxKind::PLUS_DOT | SyntaxKind::MINUS | SyntaxKind::MINUS_DOT => {
            EXPR_PREC_ADD
        }
        SyntaxKind::STAR
        | SyntaxKind::STAR_DOT
        | SyntaxKind::SLASH
        | SyntaxKind::SLASH_DOT
        | SyntaxKind::PERCENT => EXPR_PREC_MUL,
        _ => EXPR_PREC_LOWEST,
    }
}

fn expr_precedence(expr: &ast::Expr) -> u8 {
    match expr {
        ast::Expr::Let(_)
        | ast::Expr::Fn(_)
        | ast::Expr::FnShorthand(_)
        | ast::Expr::If(_)
        | ast::Expr::Match(_) => EXPR_PREC_LOWEST,
        ast::Expr::Binary(binary) => {
            binary
                .op_token()
                .map(|tok| binary_op_precedence(tok.kind()))
                .unwrap_or(EXPR_PREC_LOWEST)
        }
        ast::Expr::Unary(_) => EXPR_PREC_UNARY,
        ast::Expr::Call(_) => EXPR_PREC_CALL,
        ast::Expr::Field(_) => EXPR_PREC_FIELD,
        ast::Expr::Paren(_)
        | ast::Expr::Array(_)
        | ast::Expr::Struct(_)
        | ast::Expr::Literal(_)
        | ast::Expr::Unit(_)
        | ast::Expr::Ident(_)
        | ast::Expr::Path(_)
        | ast::Expr::Operator(_) => EXPR_PREC_ATOM,
    }
}

fn format_expr(expr: &ast::Expr) -> Formatted {
    if contains_comment(expr.syntax()) {
        return format_node_text(expr.syntax());
    }
    match expr {
        ast::Expr::Let(expr) => format_let_expr(expr),
        ast::Expr::Fn(expr) => format_fn_expr(expr),
        ast::Expr::FnShorthand(expr) => format_fn_shorthand_expr(expr),
        ast::Expr::If(expr) => format_if_expr(expr),
        ast::Expr::Match(expr) => format_match_expr(expr),
        _ => Formatted::single(format_expr_inline(expr, EXPR_PREC_LOWEST)),
    }
}

fn format_expr_inline(
    expr: &ast::Expr,
    parent_prec: u8,
) -> String {
    let prec = expr_precedence(expr);
    let inner = match expr {
        ast::Expr::Let(expr) => format_let_expr(expr).flatten(),
        ast::Expr::Fn(expr) => format_fn_expr(expr).flatten(),
        ast::Expr::FnShorthand(expr) => format_fn_shorthand_expr(expr).flatten(),
        ast::Expr::If(expr) => format_if_expr(expr).flatten(),
        ast::Expr::Match(expr) => format_match_expr(expr).flatten(),
        ast::Expr::Binary(expr) => format_binary_expr(expr),
        ast::Expr::Unary(expr) => format_unary_expr(expr),
        ast::Expr::Call(expr) => format_call_expr(expr),
        ast::Expr::Field(expr) => format_field_expr(expr),
        ast::Expr::Paren(expr) => format_paren_expr(expr),
        ast::Expr::Array(expr) => format_array_expr(expr),
        ast::Expr::Struct(expr) => format_struct_expr(expr),
        ast::Expr::Literal(lit) => format_literal(lit),
        ast::Expr::Unit(_) => "()".to_string(),
        ast::Expr::Ident(ident) => format_ident(ident),
        ast::Expr::Path(path) => format_path(path),
        ast::Expr::Operator(op) => format_operator_expr(op),
    };
    if prec < parent_prec {
        format!("({inner})")
    } else {
        inner
    }
}

fn contains_comment(node: &SyntaxNode) -> bool {
    node.descendants_with_tokens().any(|element| {
        element
            .into_token()
            .is_some_and(|token| is_comment_kind(token.kind()))
    })
}

fn format_node_text(node: &SyntaxNode) -> Formatted {
    let text = node.text().to_string();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Formatted::single(String::new());
    }
    Formatted::from_lines(trimmed.lines().map(str::to_string).collect())
}

fn format_let_expr(expr: &ast::LetExpr) -> Formatted {
    let pattern = expr
        .pattern()
        .map(|pat| format_pattern(&pat))
        .unwrap_or_default();
    let value = expr
        .value()
        .map(|value| format_expr(&value))
        .unwrap_or_else(|| Formatted::single(String::new()));
    let body = expr
        .body()
        .map(|body| format_expr(&body))
        .unwrap_or_else(|| Formatted::single(String::new()));
    let mut lines = Vec::new();
    if value.is_multiline() {
        lines.push(format!("let {pattern} ="));
        lines.extend(value.indent(1).lines);
    } else {
        lines.push(format!("let {pattern} = {}", value.flatten()));
    }
    if body.is_multiline() {
        lines.push("in".to_string());
        lines.extend(body.indent(1).lines);
    } else {
        lines.push(format!("in {}", body.flatten()));
    }
    Formatted::from_lines(lines)
}

fn format_fn_expr(expr: &ast::FnExpr) -> Formatted {
    let params = expr
        .params()
        .iter()
        .map(format_param)
        .collect::<Vec<_>>()
        .join(" ");
    let head = if params.is_empty() {
        "fn".to_string()
    } else {
        format!("fn {params}")
    };
    let body = expr
        .body()
        .map(|body| format_expr(&body))
        .unwrap_or_else(|| Formatted::single(String::new()));
    if body.is_multiline() {
        let mut lines = vec![format!("{head} =>")];
        lines.extend(body.indent(1).lines);
        Formatted::from_lines(lines)
    } else {
        Formatted::single(format!("{head} => {}", body.flatten()))
    }
}

fn format_fn_shorthand_expr(expr: &ast::FnShorthandExpr) -> Formatted {
    let mut lines = vec!["fn".to_string()];
    for arm in expr.arms() {
        lines.extend(format_match_arm(&arm).indent(1).lines);
    }
    Formatted::from_lines(lines)
}

fn format_if_expr(expr: &ast::IfExpr) -> Formatted {
    let condition = expr
        .condition()
        .map(|cond| format_expr_inline(&cond, EXPR_PREC_LOWEST))
        .unwrap_or_default();
    let then_branch = expr
        .then_branch()
        .map(|then_branch| format_expr(&then_branch))
        .unwrap_or_else(|| Formatted::single(String::new()));
    let else_branch = expr
        .else_branch()
        .map(|else_branch| format_expr(&else_branch))
        .unwrap_or_else(|| Formatted::single(String::new()));
    if !then_branch.is_multiline() && !else_branch.is_multiline() {
        return Formatted::single(format!(
            "if {condition} then {} else {}",
            then_branch.flatten(),
            else_branch.flatten()
        ));
    }
    let mut lines = vec![format!("if {condition} then")];
    lines.extend(then_branch.indent(1).lines);
    lines.push("else".to_string());
    lines.extend(else_branch.indent(1).lines);
    Formatted::from_lines(lines)
}

fn format_match_expr(expr: &ast::MatchExpr) -> Formatted {
    let scrutinee = expr
        .scrutinee()
        .map(|scrutinee| format_expr_inline(&scrutinee, EXPR_PREC_LOWEST))
        .unwrap_or_default();
    let mut lines = vec![format!("match {scrutinee} with")];
    for arm in expr.arms() {
        lines.extend(format_match_arm(&arm).indent(1).lines);
    }
    Formatted::from_lines(lines)
}

fn format_match_arm(arm: &ast::MatchArm) -> Formatted {
    let pattern = arm
        .pattern()
        .map(|pat| format_pattern(&pat))
        .unwrap_or_default();
    let body = arm
        .body()
        .map(|body| format_expr(&body))
        .unwrap_or_else(|| Formatted::single(String::new()));
    if body.is_multiline() {
        let mut lines = vec![format!("| {pattern} =>")];
        lines.extend(body.indent(1).lines);
        Formatted::from_lines(lines)
    } else {
        Formatted::single(format!("| {pattern} => {}", body.flatten()))
    }
}

fn format_binary_expr(expr: &ast::BinaryExpr) -> String {
    let op_kind = expr.op_token().map(|tok| tok.kind());
    let prec = op_kind
        .map(binary_op_precedence)
        .unwrap_or(EXPR_PREC_LOWEST);
    let left = expr
        .lhs()
        .map(|lhs| format_expr_inline(&lhs, prec))
        .unwrap_or_default();
    let op = expr
        .op_token()
        .map(|tok| tok.text().to_string())
        .unwrap_or_default();
    let right = expr
        .rhs()
        .map(|rhs| format_expr_inline(&rhs, prec.saturating_add(1)))
        .unwrap_or_default();
    format!("{left} {op} {right}")
}

fn format_unary_expr(expr: &ast::UnaryExpr) -> String {
    let op = expr
        .op_token()
        .map(|tok| tok.text().to_string())
        .unwrap_or_default();
    let operand = expr
        .operand()
        .map(|operand| format_expr_inline(&operand, EXPR_PREC_UNARY))
        .unwrap_or_default();
    if op == "not" {
        format!("{op} {operand}")
    } else {
        format!("{op}{operand}")
    }
}

fn format_call_expr(expr: &ast::CallExpr) -> String {
    let callee = expr
        .callee()
        .map(|callee| format_expr_inline(&callee, EXPR_PREC_CALL))
        .unwrap_or_default();
    let arg = expr
        .arg()
        .map(|arg| format_expr_inline(&arg, EXPR_PREC_CALL.saturating_add(1)))
        .unwrap_or_default();
    format!("{callee} {arg}")
}

fn format_field_expr(expr: &ast::FieldExpr) -> String {
    let base = expr
        .base()
        .map(|base| format_expr_inline(&base, EXPR_PREC_FIELD))
        .unwrap_or_default();
    let field = expr
        .field_token()
        .map(|tok| tok.text().to_string())
        .unwrap_or_default();
    format!("{base}.{field}")
}

fn format_paren_expr(expr: &ast::ParenExpr) -> String {
    if let Some(op) = expr.operator() {
        return format!("({})", format_operator_expr(&op));
    }
    let inner = expr
        .inner_exprs()
        .iter()
        .map(|expr| format_expr_inline(expr, EXPR_PREC_LOWEST))
        .collect::<Vec<_>>()
        .join(", ");
    format!("({inner})")
}

fn format_array_expr(expr: &ast::ArrayExpr) -> String {
    let items = expr
        .syntax()
        .children()
        .filter_map(|node| {
            ast::ArraySplat::cast(node.clone())
                .map(ArrayItem::Splat)
                .or_else(|| ast::Expr::cast(node).map(ArrayItem::Expr))
        })
        .map(|item| {
            match item {
                ArrayItem::Expr(expr) => format_expr_inline(&expr, EXPR_PREC_LOWEST),
                ArrayItem::Splat(splat) => {
                    let value = splat
                        .expr()
                        .map(|expr| format_expr_inline(&expr, EXPR_PREC_LOWEST))
                        .unwrap_or_default();
                    format!("..{value}")
                }
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

#[derive(Clone)]
enum ArrayItem {
    Expr(ast::Expr),
    Splat(ast::ArraySplat),
}

fn format_struct_expr(expr: &ast::StructExpr) -> String {
    let fields = expr
        .fields()
        .iter()
        .map(|field| {
            let name = field.name_text().unwrap_or_default();
            let value = field
                .value()
                .map(|value| format_expr_inline(&value, EXPR_PREC_LOWEST));
            match value {
                Some(value) => format!("{name} = {value}"),
                None => name,
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {fields} }}")
}

fn format_literal(literal: &ast::Literal) -> String {
    literal
        .token()
        .map(|tok| tok.text().to_string())
        .unwrap_or_default()
}

fn format_ident(ident: &ast::Ident) -> String {
    ident.name_text().unwrap_or_default()
}

fn format_operator_expr(expr: &ast::OperatorExpr) -> String {
    expr.op_token()
        .map(|tok| tok.text().to_string())
        .unwrap_or_default()
}

fn format_pattern(pattern: &ast::Pattern) -> String {
    match pattern {
        ast::Pattern::Ident(ident) => format_ident(ident),
        ast::Pattern::Literal(literal) => format_literal(literal),
        ast::Pattern::Unit(_) => "()".to_string(),
        ast::Pattern::Tuple(tuple) => {
            let patterns = tuple
                .patterns()
                .iter()
                .map(format_pattern)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({patterns})")
        }
        ast::Pattern::Array(array) => format_pattern_array(array),
        ast::Pattern::Struct(struct_) => format_pattern_struct(struct_),
        ast::Pattern::Constructor(constructor) => format_pattern_constructor(constructor),
        ast::Pattern::TypeHint(type_hint) => {
            let inner = type_hint
                .pattern()
                .map(|pat| format_pattern(&pat))
                .unwrap_or_default();
            let ty = type_hint
                .ty()
                .map(|ty| format_type_expr(&ty))
                .unwrap_or_default();
            format!("{inner}: {ty}")
        }
        ast::Pattern::Path(path) => format_path(path),
    }
}

fn format_pattern_array(array: &ast::PatArray) -> String {
    let items = array
        .syntax()
        .children()
        .filter_map(|node| {
            ast::PatRest::cast(node.clone())
                .map(PatternArrayItem::Rest)
                .or_else(|| ast::Pattern::cast(node).map(PatternArrayItem::Pattern))
        })
        .map(|item| {
            match item {
                PatternArrayItem::Pattern(pattern) => format_pattern(&pattern),
                PatternArrayItem::Rest(rest) => {
                    let binding = rest
                        .binding_token()
                        .map(|tok| tok.text().to_string())
                        .unwrap_or_default();
                    if binding.is_empty() {
                        "..".to_string()
                    } else {
                        format!("..{binding}")
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

#[derive(Clone)]
enum PatternArrayItem {
    Pattern(ast::Pattern),
    Rest(ast::PatRest),
}

fn format_pattern_struct(struct_: &ast::PatStruct) -> String {
    let fields = struct_
        .fields()
        .iter()
        .map(|field| {
            let name = field.name_text().unwrap_or_default();
            let pattern = field.pattern().map(|pat| format_pattern(&pat));
            match pattern {
                Some(pattern) => format!("{name} = {pattern}"),
                None => name,
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {fields} }}")
}

fn format_pattern_constructor(constructor: &ast::PatConstructor) -> String {
    let head = constructor
        .head()
        .map(format_path_or_ident)
        .unwrap_or_default();
    let payload = constructor.payload().map(|pat| format_pattern(&pat));
    match payload {
        Some(payload) => format!("{head} of {payload}"),
        None => head,
    }
}

fn format_path_or_ident(head: ast::PathOrIdent) -> String {
    match head {
        ast::PathOrIdent::Ident(ident) => format_ident(&ident),
        ast::PathOrIdent::Path(path) => format_path(&path),
    }
}

fn format_param(param: &ast::Param) -> String {
    let name = param.name_text().unwrap_or_default();
    match param.ty() {
        Some(ty) => format!("({name}: {})", format_type_expr(&ty)),
        None => name,
    }
}

fn format_path(path: &ast::Path) -> String {
    path.segments()
        .iter()
        .map(|seg| seg.text().to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod test;
