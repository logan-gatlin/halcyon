use crate::parse::SyntaxKind;
use crate::parse::parser::Parser;

use super::{
    literal,
    paren_list,
    path_or_ident,
    type_expr,
};

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

pub(crate) fn can_start_pattern(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::IDENT
            | SyntaxKind::INTEGER
            | SyntaxKind::REAL
            | SyntaxKind::STRING
            | SyntaxKind::GLYPH
            | SyntaxKind::TRUE_KW
            | SyntaxKind::FALSE_KW
            | SyntaxKind::L_PAREN
            | SyntaxKind::L_SQUARE
            | SyntaxKind::L_BRACE
    )
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
                    literal(p);
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
                    p.error_and_bump("expected pattern");
                }
            }
        }
    }
}

/// Identifier, path (`Module::Ctor`), or constructor (`Ctor of pat`).
fn ident_or_constructor(p: &mut Parser<'_, '_>) {
    let checkpoint = p.checkpoint();
    path_or_ident(p, SyntaxKind::IDENT_NODE, SyntaxKind::PATH);

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
    paren_list(p, SyntaxKind::UNIT, SyntaxKind::PAT_TUPLE, pattern);
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
