use crate::parse::SyntaxKind;
use crate::parse::parser::Parser;

use super::{
    can_start_identifier,
    expect_identifier,
    is_bracketed_operator_identifier_start,
    literal,
    paren_list,
    path_or_ident,
    pattern,
    sexpr,
    type_expr,
    use_target_path_or_ident,
};

/// Parse any expression.
pub(crate) fn expr(p: &mut Parser<'_, '_>) {
    match p.current() {
        Some(SyntaxKind::LET_KW) => let_expr(p),
        Some(SyntaxKind::USE_KW) => use_expr(p),
        Some(SyntaxKind::FN_KW) => fn_expr(p),
        Some(SyntaxKind::IF_KW) => if_expr(p),
        Some(SyntaxKind::MATCH_KW) => match_expr(p),
        _ => expr_bp(p, 0),
    }
}

/// Pratt parser for binary operators, function application, and field
/// access.
fn expr_bp(
    p: &mut Parser<'_, '_>,
    min_bp: u8,
) {
    let checkpoint = p.checkpoint();

    // Prefix: unary operators
    if let Some(kind) = p.current()
        && let Some(rbp) = prefix_bp(kind)
    {
        let m = p.start_node_at(checkpoint, SyntaxKind::UNARY_EXPR);
        p.bump(); // operator
        expr_bp(p, rbp);
        p.finish_node(m);
        return postfix_and_infix(p, checkpoint, min_bp);
    }

    // Atom
    if !primary(p) {
        return;
    }

    postfix_and_infix(p, checkpoint, min_bp);
}

fn postfix_and_infix(
    p: &mut Parser<'_, '_>,
    checkpoint: rowan::Checkpoint,
    min_bp: u8,
) {
    while let Some(kind) = p.current() {
        // Field access: `.ident`
        if kind == SyntaxKind::DOT {
            if min_bp > FIELD_BP {
                break;
            }
            let m = p.start_node_at(checkpoint, SyntaxKind::FIELD_EXPR);
            p.bump(); // `.`
            expect_identifier(p);
            p.finish_node(m);
            continue;
        }

        // Binary operator
        if let Some((lbp, rbp)) = infix_bp(kind) {
            if lbp < min_bp {
                break;
            }
            let m = p.start_node_at(checkpoint, SyntaxKind::BINARY_EXPR);
            p.bump(); // operator
            expr_bp(p, rbp);
            p.finish_node(m);
            continue;
        }

        // Function application (juxtaposition): if the next token could
        // start an atom and we're at high enough precedence.
        if min_bp <= CALL_BP && can_start_atom(kind) {
            let m = p.start_node_at(checkpoint, SyntaxKind::CALL_EXPR);
            // Parse exactly one argument atom (not a full expr_bp, to
            // avoid consuming binary operators as arguments).
            expr_bp(p, CALL_BP + 1);
            p.finish_node(m);
            continue;
        }

        break;
    }
}

const FIELD_BP: u8 = 34;
const CALL_BP: u8 = 24;

/// Returns the right binding power of a prefix (unary) operator.
fn prefix_bp(kind: SyntaxKind) -> Option<u8> {
    match kind {
        SyntaxKind::MINUS | SyntaxKind::NOT_KW => Some(20),
        _ => None,
    }
}

/// Returns (left binding power, right binding power) for infix operators.
fn infix_bp(kind: SyntaxKind) -> Option<(u8, u8)> {
    // Higher number = tighter binding.
    // Left-assoc: (N, N+1). Right-assoc: (N+1, N).
    match kind {
        SyntaxKind::SEMICOLON => Some((2, 3)),

        SyntaxKind::PIPE_ARROW | SyntaxKind::PLUS_ARROW | SyntaxKind::STAR_ARROW => Some((4, 5)),

        SyntaxKind::OR_KW => Some((6, 7)),

        SyntaxKind::AND_KW => Some((8, 9)),

        SyntaxKind::DOUBLE_EQUAL
        | SyntaxKind::BANG_EQUAL
        | SyntaxKind::LESS
        | SyntaxKind::LESS_EQUAL
        | SyntaxKind::GREATER
        | SyntaxKind::GREATER_EQUAL => Some((10, 11)),

        SyntaxKind::COMPOSE_LEFT | SyntaxKind::COMPOSE_RIGHT | SyntaxKind::XOR_KW => Some((12, 13)),

        SyntaxKind::PLUS | SyntaxKind::MINUS => Some((14, 15)),

        SyntaxKind::STAR | SyntaxKind::SLASH | SyntaxKind::MODULO_KW => Some((16, 17)),

        _ => None,
    }
}

/// Can this token start an atomic / primary expression?
fn can_start_atom(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::IDENT
            | SyntaxKind::INTEGER
            | SyntaxKind::NATURAL
            | SyntaxKind::REAL
            | SyntaxKind::STRING
            | SyntaxKind::GLYPH
            | SyntaxKind::TRUE_KW
            | SyntaxKind::FALSE_KW
            | SyntaxKind::ROOT_KW
            | SyntaxKind::BUNDLE_KW
            | SyntaxKind::L_PAREN
            | SyntaxKind::L_SQUARE
            | SyntaxKind::L_BRACE
    )
}

/// Parse a primary (atomic) expression. Returns `false` if nothing was
/// parsed.
fn primary(p: &mut Parser<'_, '_>) -> bool {
    match p.current() {
        None => {
            p.error_at_current("expected expression");
            false
        }
        Some(kind) => {
            match kind {
                // Literals
                SyntaxKind::INTEGER
                | SyntaxKind::NATURAL
                | SyntaxKind::REAL
                | SyntaxKind::STRING
                | SyntaxKind::GLYPH
                | SyntaxKind::TRUE_KW
                | SyntaxKind::FALSE_KW => {
                    literal(p);
                    true
                }

                // Identifier or path
                SyntaxKind::IDENT | SyntaxKind::ROOT_KW | SyntaxKind::BUNDLE_KW => {
                    path_or_ident(p);
                    true
                }

                // Parenthesised expr, tuple, unit, or `(op)`
                SyntaxKind::L_PAREN => {
                    if p.nth(1) == Some(SyntaxKind::WASM_KW) {
                        inline_wasm_expr(p);
                    } else {
                        paren_or_tuple(p);
                    }
                    true
                }

                // Array literal
                SyntaxKind::L_SQUARE => {
                    if is_bracketed_operator_identifier_start(p) {
                        path_or_ident(p);
                    } else {
                        array_expr(p);
                    }
                    true
                }

                // Struct literal
                SyntaxKind::L_BRACE => {
                    struct_expr(p);
                    true
                }

                _ => {
                    p.error_and_bump("expected expression");
                    false
                }
            }
        }
    }
}

// ── Compound primaries ───────────────────────────────────────────────

/// `"(" ")"` (unit), `"(" expr ")"` (paren), `"(" expr "," ")"` (singleton tuple), or `"(" expr "," ... ")"` (tuple).
fn paren_or_tuple(p: &mut Parser<'_, '_>) {
    paren_list(p, SyntaxKind::UNIT, SyntaxKind::PAREN_EXPR, expr);
}

/// `("wasm" ":" <type_expr>) "=>" <sexpr>`
fn inline_wasm_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::INLINE_WASM_EXPR);
    p.expect(SyntaxKind::L_PAREN);
    p.expect(SyntaxKind::WASM_KW);
    p.expect(SyntaxKind::COLON);
    type_expr::type_expr(p);
    p.expect(SyntaxKind::R_PAREN);
    p.expect(SyntaxKind::DOUBLE_ARROW);
    sexpr::parse(p);
    p.finish_node(m);
}

/// `"[" (expr | ".." expr)* "]"`
fn array_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::ARRAY_EXPR);
    p.expect(SyntaxKind::L_SQUARE);
    while !p.at(SyntaxKind::R_SQUARE) && !p.at_end() {
        if p.at(SyntaxKind::DOT_DOT) {
            let sm = p.start_node(SyntaxKind::ARRAY_SPLAT);
            p.bump(); // ..
            expr(p);
            p.finish_node(sm);
        } else {
            expr(p);
        }
        if !p.at(SyntaxKind::R_SQUARE) {
            // Elements are whitespace-separated; no comma needed.
            // But if the user wrote a comma, eat it.
            p.eat(SyntaxKind::COMMA);
        }
    }
    p.expect(SyntaxKind::R_SQUARE);
    p.finish_node(m);
}

/// `"{" (ident ("=" | ":") expr)+ "}"`
fn struct_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::STRUCT_EXPR);
    p.expect(SyntaxKind::L_BRACE);
    while !p.at(SyntaxKind::R_BRACE) && !p.at_end() {
        let fm = p.start_node(SyntaxKind::STRUCT_FIELD);
        expect_identifier(p);
        if p.at(SyntaxKind::EQUAL) || p.at(SyntaxKind::COLON) {
            p.bump(); // `=` or `:`
            expr(p);
        }
        p.finish_node(fm);
        p.eat(SyntaxKind::COMMA);
    }
    p.expect(SyntaxKind::R_BRACE);
    p.finish_node(m);
}

// ── Keyword expressions ──────────────────────────────────────────────

/// `"let" pattern "=" expr "in" expr`
fn let_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::LET_EXPR);
    p.expect(SyntaxKind::LET_KW);
    pattern::pattern(p);
    p.expect(SyntaxKind::EQUAL);
    expr(p);
    p.expect(SyntaxKind::IN_KW);
    expr(p);
    p.finish_node(m);
}

/// `"use" (<ident> | <path>) ("as" <ident>)? "in" expr`
fn use_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::USE_EXPR);
    p.expect(SyntaxKind::USE_KW);
    use_target_path_or_ident(p);
    if p.eat(SyntaxKind::AS_KW) {
        expect_identifier(p);
    }
    p.expect(SyntaxKind::IN_KW);
    expr(p);
    p.finish_node(m);
}

/// `"fn" params "=>" expr` or `"fn" match_arm+`
fn fn_expr(p: &mut Parser<'_, '_>) {
    // Peek after `fn` to decide which form
    if p.nth(1) == Some(SyntaxKind::PIPE) {
        fn_shorthand(p);
    } else {
        fn_long(p);
    }
}

/// `"fn" param+ "=>" expr`
fn fn_long(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::FN_EXPR);
    p.expect(SyntaxKind::FN_KW);

    // Parse parameters until we see `=>`
    while !p.at_any(&[
        SyntaxKind::DOUBLE_ARROW,
        SyntaxKind::ARROW,
        SyntaxKind::EQUAL,
    ]) && !p.at_end()
    {
        param(p);
    }
    p.expect(SyntaxKind::DOUBLE_ARROW);
    expr(p);
    p.finish_node(m);
}

/// A function parameter: either a bare identifier or `"(" ident ":" type ")"`.
fn param(p: &mut Parser<'_, '_>) {
    if p.at(SyntaxKind::L_PAREN) {
        let m = p.start_node(SyntaxKind::PARAM);
        p.bump(); // (
        expect_identifier(p);
        p.expect(SyntaxKind::COLON);
        type_expr::type_expr(p);
        p.expect(SyntaxKind::R_PAREN);
        p.finish_node(m);
    } else if can_start_identifier(p) {
        let m = p.start_node(SyntaxKind::PARAM);
        expect_identifier(p);
        p.finish_node(m);
    } else {
        p.error_and_bump("expected parameter");
    }
}

/// `"fn" ("|" pattern "=>" expr)+`
fn fn_shorthand(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::FN_SHORTHAND_EXPR);
    p.expect(SyntaxKind::FN_KW);
    let arm_indent = p.current_column();
    while next_match_arm_has_expected_indent(p, arm_indent) {
        match_arm(p, true);
    }
    p.finish_node(m);
}

/// `"if" expr "then" expr "else" expr`
fn if_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::IF_EXPR);
    p.expect(SyntaxKind::IF_KW);
    expr(p);
    p.expect(SyntaxKind::THEN_KW);
    expr(p);
    p.expect(SyntaxKind::ELSE_KW);
    expr(p);
    p.finish_node(m);
}

/// `"match" expr "with" match_arm+`
fn match_expr(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::MATCH_EXPR);
    p.expect(SyntaxKind::MATCH_KW);
    expr(p);
    p.expect(SyntaxKind::WITH_KW);
    if p.at(SyntaxKind::PIPE) {
        let arm_indent = p.current_column();
        while next_match_arm_has_expected_indent(p, arm_indent) {
            match_arm(p, true);
        }
    } else if p.current().is_some_and(pattern::can_start_pattern) {
        let arm_indent = p.current_column();
        match_arm(p, false);
        while next_match_arm_has_expected_indent(p, arm_indent) {
            match_arm(p, true);
        }
    } else {
        p.error_at_current("expected match arm");
    }
    p.finish_node(m);
}

fn next_match_arm_has_expected_indent(
    p: &Parser<'_, '_>,
    expected_indent: Option<usize>,
) -> bool {
    if !p.at(SyntaxKind::PIPE) {
        return false;
    }
    match (expected_indent, p.current_column()) {
        (Some(expected), Some(actual)) => actual == expected,
        _ => true,
    }
}

/// `"|" pattern "=>" expr`
fn match_arm(
    p: &mut Parser<'_, '_>,
    has_pipe: bool,
) {
    let m = p.start_node(SyntaxKind::MATCH_ARM);
    if has_pipe {
        p.expect(SyntaxKind::PIPE);
    }
    pattern::pattern(p);
    p.expect(SyntaxKind::DOUBLE_ARROW);
    expr(p);
    p.finish_node(m);
}
