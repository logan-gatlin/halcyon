use super::{
    can_start_identifier,
    expect_identifier,
    expression,
    identifier,
    path_or_ident,
    pattern,
    sexpr,
    type_expr,
};
use crate::parse::SyntaxKind;
use crate::parse::parser::Parser;

/// Recovery set at the module-body level: we can resume parsing at any
/// statement-starting keyword or at `end`.
const STATEMENT_RECOVERY: &[SyntaxKind] = &[
    SyntaxKind::USE_KW,
    SyntaxKind::LET_KW,
    SyntaxKind::TYPE_KW,
    SyntaxKind::TRAIT_KW,
    SyntaxKind::IMPL_KW,
    SyntaxKind::END_KW,
    SyntaxKind::WASM_KW,
    SyntaxKind::MODULE_KW,
];

pub fn can_start_statement(p: &Parser<'_, '_>) -> bool {
    matches!(
        p.current(),
        Some(
            SyntaxKind::USE_KW
                | SyntaxKind::LET_KW
                | SyntaxKind::TYPE_KW
                | SyntaxKind::TRAIT_KW
                | SyntaxKind::IMPL_KW
                | SyntaxKind::WASM_KW
                | SyntaxKind::MODULE_KW
        )
    )
}

pub fn statement(p: &mut Parser<'_, '_>) {
    match p.current() {
        Some(SyntaxKind::USE_KW) => use_statement(p),
        Some(SyntaxKind::LET_KW) => let_statement(p),
        Some(SyntaxKind::TYPE_KW) => type_statement(p),
        Some(SyntaxKind::TRAIT_KW) => trait_statement(p),
        Some(SyntaxKind::IMPL_KW) => impl_statement(p),
        Some(SyntaxKind::WASM_KW) => inline_wasm(p),
        Some(SyntaxKind::MODULE_KW) => module(p),
        Some(SyntaxKind::IMPORT_KW) => {
            p.error_recover(
                "`import` statements are only allowed at source-file top level",
                STATEMENT_RECOVERY,
            )
        }
        _ => {
            p.error_recover(
                "expected `use`, `let`, `type`, `trait`, `impl`, `module`, `wasm`, or `end`",
                STATEMENT_RECOVERY,
            );
        }
    };
}

/// ```bnf
/// <module> ::= "module" <ident> "=" <statement>* "end"
/// ```
pub fn module(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::MODULE);
    p.expect(SyntaxKind::MODULE_KW);
    expect_identifier(p);
    p.expect(SyntaxKind::EQUAL);

    while !p.at(SyntaxKind::END_KW) && !p.at_end() {
        statement(p)
    }
    p.expect(SyntaxKind::END_KW);
    p.finish_node(m);
}

/// ```bnf
/// <let_statement> ::= "let" <pattern> "=" <expr>
///                   | "let" "|" <ident> "=" (<ident> | <path>)
/// ```
fn let_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::LET_STATEMENT);
    p.expect(SyntaxKind::LET_KW);
    if p.eat(SyntaxKind::PIPE) {
        expect_identifier(p);
        p.expect(SyntaxKind::EQUAL);
        path_or_ident(p);
        p.finish_node(m);
        return;
    }
    pattern::pattern(p);
    p.expect(SyntaxKind::EQUAL);
    expression::expr(p);
    p.finish_node(m);
}

/// ```bnf
/// <use_statement> ::= "use" (<ident> | <path>) ("as" <ident>)?
/// ```
fn use_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::USE_STATEMENT);
    p.expect(SyntaxKind::USE_KW);
    path_or_ident(p);
    if p.eat(SyntaxKind::AS_KW) {
        expect_identifier(p);
    }
    p.finish_node(m);
}

/// ```bnf
/// <type_statement> ::= "type" ("~")? <ident> (":" <ident>+)? "=" (<type_def> | <type_expr>)
/// ```
fn type_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::TYPE_STATEMENT);
    p.expect(SyntaxKind::TYPE_KW);
    let is_alias = p.eat(SyntaxKind::TILDE);
    expect_identifier(p);
    // Optional type parameters: `: a b c`
    if p.eat(SyntaxKind::COLON) {
        while can_start_identifier(p) {
            identifier(p);
        }
    }
    p.expect(SyntaxKind::EQUAL);
    if is_alias {
        let alias = p.start_node(SyntaxKind::TYPE_ALIAS_DEF);
        type_expr::type_expr(p);
        p.finish_node(alias);
    } else {
        type_expr::type_def(p);
    }
    p.finish_node(m);
}

/// ```bnf
/// <trait_statement> ::= "trait" "~"? <ident> (":" <ident>+)? "=" (<trait_method_decl>* "end" | (<ident> | <path>))
/// <trait_method_decl> ::= "let" <ident> ":" <type_expr>
/// ```
fn trait_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::TRAIT_STATEMENT);
    p.expect(SyntaxKind::TRAIT_KW);
    let is_alias = p.eat(SyntaxKind::TILDE);
    expect_identifier(p);
    if is_alias {
        p.expect(SyntaxKind::EQUAL);
        path_or_ident(p);
        p.finish_node(m);
        return;
    }
    if p.eat(SyntaxKind::COLON) {
        if !can_start_identifier(p) {
            p.error_at_current("expected trait type parameter");
        }
        while can_start_identifier(p) {
            identifier(p);
            p.eat(SyntaxKind::COMMA);
        }
    }
    p.expect(SyntaxKind::EQUAL);
    while p.at(SyntaxKind::LET_KW) {
        trait_method_decl(p);
    }
    p.expect(SyntaxKind::END_KW);
    p.finish_node(m);
}

fn trait_method_decl(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::TRAIT_METHOD_DECL);
    p.expect(SyntaxKind::LET_KW);
    expect_identifier(p);
    p.expect(SyntaxKind::COLON);
    type_expr::type_expr(p);
    p.finish_node(m);
}

/// ```bnf
/// <impl_statement> ::= "impl" (<ident> | <path>) <type_expr> ("," <type_expr>)* "=" <impl_method_def>* "end"
/// <impl_method_def> ::= "let" <ident> "=" <expr>
/// ```
fn impl_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::IMPL_STATEMENT);
    p.expect(SyntaxKind::IMPL_KW);
    path_or_ident(p);
    if !can_start_impl_argument(p) {
        p.error_at_current("expected at least one impl type argument");
    }
    while can_start_impl_argument(p) {
        type_expr::type_expr(p);
        if !p.eat(SyntaxKind::COMMA) {
            break;
        }
        if p.at(SyntaxKind::EQUAL) {
            break;
        }
    }
    p.expect(SyntaxKind::EQUAL);
    while p.at(SyntaxKind::LET_KW) {
        impl_method_def(p);
    }
    p.expect(SyntaxKind::END_KW);
    p.finish_node(m);
}

fn impl_method_def(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::IMPL_METHOD_DEF);
    p.expect(SyntaxKind::LET_KW);
    expect_identifier(p);
    p.expect(SyntaxKind::EQUAL);
    expression::expr(p);
    p.finish_node(m);
}

fn can_start_impl_argument(p: &Parser<'_, '_>) -> bool {
    matches!(
        p.current(),
        Some(
            SyntaxKind::IDENT
                | SyntaxKind::ROOT_KW
                | SyntaxKind::BUNDLE_KW
                | SyntaxKind::L_PAREN
                | SyntaxKind::L_SQUARE
                | SyntaxKind::FOR_KW
        )
    )
}

/// ```bnf
/// <inline_wasm> ::= "wasm" "=>" <sexpr>
/// ```
fn inline_wasm(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::WASM_STATEMENT);
    p.expect(SyntaxKind::WASM_KW);
    p.expect(SyntaxKind::DOUBLE_ARROW);
    sexpr::parse(p);
    p.finish_node(m);
}
