use super::{
    expression,
    pattern,
    type_expr,
};
use crate::parse_lossless::parser::Parser;
use crate::parse_lossless::SyntaxKind;

/// Recovery set at the module-body level: we can resume parsing at any
/// statement-starting keyword or at `end`.
const STATEMENT_RECOVERY: &[SyntaxKind] = &[
    SyntaxKind::LET_KW,
    SyntaxKind::TYPE_KW,
    SyntaxKind::END_KW,
    SyntaxKind::MODULE_KW,
];

pub fn statement(p: &mut Parser<'_, '_>) {
    match p.current() {
        Some(SyntaxKind::LET_KW) => let_statement(p),
        Some(SyntaxKind::TYPE_KW) => type_statement(p),
        _ => {
            if p.at(SyntaxKind::MODULE_KW) {
                p.error_recover("nested modules are not supported", &[SyntaxKind::END_KW]);
                p.bump()
            }
            p.error_recover("expected `let`, `type`, or `end`", STATEMENT_RECOVERY);
        }
    };
}

/// ```bnf
/// <module> ::= "module" <ident> "=" <statement>* "end"
/// ```
pub fn module(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::MODULE);
    p.expect(SyntaxKind::MODULE_KW);
    p.expect(SyntaxKind::IDENT);
    p.expect(SyntaxKind::EQUAL);

    while !p.at(SyntaxKind::END_KW) && !p.at_end() {
        statement(p)
    }
    p.expect(SyntaxKind::END_KW);
    p.finish_node(m);
}

/// ```bnf
/// <let_statement> ::= "let" <pattern> "=" <expr>
/// ```
fn let_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::LET_STATEMENT);
    p.expect(SyntaxKind::LET_KW);
    pattern::pattern(p);
    p.expect(SyntaxKind::EQUAL);
    expression::expr(p);
    p.finish_node(m);
}

/// ```bnf
/// <type_statement> ::= "type" <ident> (":" <ident>+)? "=" <type_def>
/// ```
fn type_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::TYPE_STATEMENT);
    p.expect(SyntaxKind::TYPE_KW);
    p.expect(SyntaxKind::IDENT);
    // Optional type parameters: `: a b c`
    if p.eat(SyntaxKind::COLON) {
        while p.at(SyntaxKind::IDENT) {
            p.bump();
        }
    }
    p.expect(SyntaxKind::EQUAL);
    type_expr::type_def(p);
    p.finish_node(m);
}
