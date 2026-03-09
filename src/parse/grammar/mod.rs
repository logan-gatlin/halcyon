mod expression;
mod module;
mod pattern;
mod sexpr;
mod type_expr;

use super::SyntaxKind;
use super::parser::Parser;

/// Parse a complete source file: top-level `import` and `module` items.
pub fn source_file(p: &mut Parser<'_, '_>) {
    let m = p.start_node_before_trivia(SyntaxKind::SOURCE_FILE);
    const TOP_LEVEL_RECOVERY: &[SyntaxKind] = &[SyntaxKind::IMPORT_KW, SyntaxKind::MODULE_KW];
    while !p.at_end() {
        if p.at(SyntaxKind::IMPORT_KW) {
            import_statement(p);
        } else if p.at(SyntaxKind::MODULE_KW) {
            module::module(p);
        } else {
            p.error_recover("expected `import` or `module`", TOP_LEVEL_RECOVERY);
        }
    }
    // Attach any trailing trivia to the root node.
    p.skip_trivia();
    p.finish_node(m);
}

fn import_statement(p: &mut Parser<'_, '_>) {
    let m = p.start_node_with_leading_comments(SyntaxKind::IMPORT_STATEMENT);
    p.expect(SyntaxKind::IMPORT_KW);

    if !p.at(SyntaxKind::STRING) {
        p.error_at_current("expected import path string literal");
    }

    while p.at(SyntaxKind::STRING) {
        p.bump();
        if !p.eat(SyntaxKind::COMMA) {
            break;
        }
        if !p.at(SyntaxKind::STRING) {
            p.error_at_current("expected import path string literal after `,`");
            break;
        }
    }

    p.finish_node(m);
}

// ── Common parsing logic ──────────────────────────────────────────────

pub(crate) fn can_start_identifier(p: &Parser<'_, '_>) -> bool {
    p.at(SyntaxKind::IDENT) || is_bracketed_operator_identifier_start(p)
}

pub(crate) fn can_start_path_or_ident(p: &Parser<'_, '_>) -> bool {
    can_start_identifier(p) || p.at(SyntaxKind::ROOT_KW)
}

pub(crate) fn is_bracketed_operator_identifier_start(p: &Parser<'_, '_>) -> bool {
    p.at(SyntaxKind::L_SQUARE)
        && p.nth(1).is_some_and(SyntaxKind::is_operator_token)
        && p.nth(2) == Some(SyntaxKind::R_SQUARE)
}

pub(crate) fn identifier(p: &mut Parser<'_, '_>) -> bool {
    if p.at(SyntaxKind::IDENT) {
        p.bump();
        return true;
    }
    if is_bracketed_operator_identifier_start(p) {
        p.bump();
        p.bump();
        p.bump();
        return true;
    }
    false
}

pub(crate) fn expect_identifier(p: &mut Parser<'_, '_>) {
    if !identifier(p) {
        p.error_at_current("expected identifier");
    }
}

/// Parse a simple identifier (bare or bracketed operator) or a qualified path.
pub(crate) fn path_or_ident(p: &mut Parser<'_, '_>) {
    let checkpoint = p.checkpoint();
    if p.eat(SyntaxKind::ROOT_KW) {
        let m = p.start_node_at(checkpoint, SyntaxKind::PATH);
        p.expect(SyntaxKind::DOUBLE_COLON);
        expect_identifier(p);
        while p.eat(SyntaxKind::DOUBLE_COLON) {
            expect_identifier(p);
        }
        p.finish_node(m);
        return;
    }

    if !identifier(p) {
        p.error_and_bump("expected identifier");
        return;
    }

    if p.at(SyntaxKind::DOUBLE_COLON) {
        let m = p.start_node_at(checkpoint, SyntaxKind::PATH);
        p.bump();
        expect_identifier(p);
        while p.eat(SyntaxKind::DOUBLE_COLON) {
            expect_identifier(p);
        }
        p.finish_node(m);
    } else {
        let m = p.start_node_at(checkpoint, SyntaxKind::IDENT_NODE);
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
