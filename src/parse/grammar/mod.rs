mod expression;
mod module;
mod pattern;
mod type_expr;

use super::SyntaxKind;
use super::parser::Parser;

/// Parse a complete source file: zero or more module declarations.
pub fn source_file(p: &mut Parser<'_, '_>) {
    let m = p.start_node_before_trivia(SyntaxKind::SOURCE_FILE);
    while !p.at_end() {
        if p.at(SyntaxKind::MODULE_KW) {
            module::module(p);
        } else {
            module::statement(p);
        }
    }
    // Attach any trailing trivia to the root node.
    p.skip_trivia();
    p.finish_node(m);
}

// ── Common parsing logic ──────────────────────────────────────────────

/// Parse a simple `IDENT` or qualified `Module::Name` path.
///
/// Uses `ident_kind` for simple identifiers and `path_kind` for qualified paths.
pub(crate) fn path_or_ident(p: &mut Parser<'_, '_>) {
    if p.nth(1) == Some(SyntaxKind::DOUBLE_COLON) {
        let m = p.start_node(SyntaxKind::PATH);
        p.bump(); // first ident
        p.bump(); // ::
        p.expect(SyntaxKind::IDENT);
        p.finish_node(m);
    } else {
        let m = p.start_node(SyntaxKind::IDENT_NODE);
        p.bump();
        p.finish_node(m);
    }
}

/// Parse a literal (integer, real, string, glyph, boolean).
pub(crate) fn literal(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::LITERAL);
    p.bump();
    p.finish_node(m);
}

/// Parse a parenthesized list: `()` (unit) or `(item, ...)` (tuple/grouping).
pub(crate) fn paren_list(
    p: &mut Parser<'_, '_>,
    unit_kind: SyntaxKind,
    list_kind: SyntaxKind,
    mut item: impl FnMut(&mut Parser<'_, '_>),
) {
    // Unit: `()`
    if p.nth(1) == Some(SyntaxKind::R_PAREN) {
        let m = p.start_node(unit_kind);
        p.bump(); // (
        p.bump(); // )
        p.finish_node(m);
        return;
    }

    let m = p.start_node(list_kind);
    p.expect(SyntaxKind::L_PAREN);
    item(p);
    while p.eat(SyntaxKind::COMMA) {
        if p.at(SyntaxKind::R_PAREN) {
            break; // trailing comma
        }
        item(p);
    }
    p.expect(SyntaxKind::R_PAREN);
    p.finish_node(m);
}
