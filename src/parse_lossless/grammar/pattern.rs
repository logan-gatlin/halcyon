use crate::parse_lossless::parser::Parser;
use crate::parse_lossless::SyntaxKind;

use super::type_expr;

/// Parse a pattern, optionally followed by `: type` for a type hint.
///
/// ```bnf
/// <pattern> ::= <pattern_primary> (":" <type_expr>)?
/// ```
pub(crate) fn pattern(p: &mut Parser<'_, '_>) {
    let checkpoint = p.checkpoint();
    pattern_primary(p);

    // Optional type annotation
    if p.at(SyntaxKind::COLON) {
        let m = p.start_node_at(checkpoint, SyntaxKind::PAT_TYPE_HINT);
        p.bump(); // :
        type_expr::type_expr(p);
        p.finish_node(m);
    }
}

/// Parse the core of a pattern without type hints.
fn pattern_primary(p: &mut Parser<'_, '_>) {
    match p.current() {
        None => {
            p.error_at_current("expected pattern");
        }
        Some(kind) => {
            match kind {
                // Literals
                SyntaxKind::INTEGER
                | SyntaxKind::REAL
                | SyntaxKind::STRING
                | SyntaxKind::GLYPH
                | SyntaxKind::TRUE_KW
                | SyntaxKind::FALSE_KW => {
                    let m = p.start_node(SyntaxKind::LITERAL);
                    p.bump();
                    p.finish_node(m);
                }

                // Identifier, path, or constructor
                SyntaxKind::IDENT => ident_or_constructor(p),

                // Tuple or grouping
                SyntaxKind::L_PAREN => paren_pattern(p),

                // Array pattern
                SyntaxKind::L_SQUARE => array_pattern(p),

                // Struct pattern
                SyntaxKind::L_BRACE => struct_pattern(p),

                _ => {
                    p.error_at_current("expected pattern");
                }
            }
        }
    }
}

/// Identifier, path (`Module::Ctor`), or constructor (`Ctor of pat`).
fn ident_or_constructor(p: &mut Parser<'_, '_>) {
    let checkpoint = p.checkpoint();

    let is_path = p.nth(1) == Some(SyntaxKind::DOUBLE_COLON);

    if is_path {
        let m = p.start_node(SyntaxKind::PATH);
        p.bump(); // module
        p.bump(); // ::
        p.expect(SyntaxKind::IDENT);
        p.finish_node(m);
    } else {
        let m = p.start_node(SyntaxKind::IDENT_EXPR);
        p.bump(); // ident
        p.finish_node(m);
    }

    // `of` pattern — constructor application
    if p.at(SyntaxKind::OF_KW) {
        let m = p.start_node_at(checkpoint, SyntaxKind::PAT_CONSTRUCTOR);
        p.bump(); // of
        pattern(p);
        p.finish_node(m);
    }
}

/// `"(" pattern ("," pattern)* ")"` or `"(" ")"` (unit pattern).
fn paren_pattern(p: &mut Parser<'_, '_>) {
    if p.nth(1) == Some(SyntaxKind::R_PAREN) {
        // Unit pattern
        let m = p.start_node(SyntaxKind::UNIT);
        p.bump(); // (
        p.bump(); // )
        p.finish_node(m);
        return;
    }

    let m = p.start_node(SyntaxKind::PAT_TUPLE);
    p.bump(); // (
    pattern(p);
    while p.eat(SyntaxKind::COMMA) {
        if p.at(SyntaxKind::R_PAREN) {
            break;
        }
        pattern(p);
    }
    p.expect(SyntaxKind::R_PAREN);
    p.finish_node(m);
}

/// `"[" (pattern | ".." ident?)* "]"`
fn array_pattern(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::PAT_ARRAY);
    p.bump(); // [
    while !p.at(SyntaxKind::R_SQUARE) && !p.at_end() {
        if p.at(SyntaxKind::DOT_DOT) {
            let rm = p.start_node(SyntaxKind::PAT_REST);
            p.bump(); // ..
                      // Optional binding name
            if p.at(SyntaxKind::IDENT) {
                p.bump();
            }
            p.finish_node(rm);
        } else {
            pattern(p);
        }
        p.eat(SyntaxKind::COMMA);
    }
    p.expect(SyntaxKind::R_SQUARE);
    p.finish_node(m);
}

/// `"{" (ident ("=" pattern)?)+ "}"`
fn struct_pattern(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::PAT_STRUCT);
    p.bump(); // {
    while !p.at(SyntaxKind::R_BRACE) && !p.at_end() {
        let fm = p.start_node(SyntaxKind::PAT_FIELD);
        p.expect(SyntaxKind::IDENT);
        if p.eat(SyntaxKind::EQUAL) {
            pattern(p);
        }
        p.finish_node(fm);
        p.eat(SyntaxKind::COMMA);
    }
    p.expect(SyntaxKind::R_BRACE);
    p.finish_node(m);
}
