use crate::parse_lossless::SyntaxKind;
use crate::parse_lossless::parser::Parser;

use super::{
    pattern,
    type_expr,
};

/// Parse any expression.
pub(crate) fn expr(p: &mut Parser<'_, '_>) {
    match p.current() {
        Some(SyntaxKind::LET_KW) => let_expr(p),
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
    loop {
        let Some(kind) = p.current() else {
            break;
        };

        // Field access: `.ident`
        if kind == SyntaxKind::DOT {
            if min_bp > FIELD_BP {
                break;
            }
            let m = p.start_node_at(checkpoint, SyntaxKind::FIELD_EXPR);
            p.bump(); // `.`
            p.expect(SyntaxKind::IDENT);
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
        SyntaxKind::MINUS | SyntaxKind::MINUS_DOT | SyntaxKind::NOT_KW => Some(20),
        _ => None,
    }
}

/// Returns (left binding power, right binding power) for infix operators.
fn infix_bp(kind: SyntaxKind) -> Option<(u8, u8)> {
    // Higher number = tighter binding.
    // Left-assoc: (N, N+1). Right-assoc: (N+1, N).
    match kind {
        SyntaxKind::SEMICOLON => Some((2, 3)),

        SyntaxKind::PIPE_ARROW => Some((4, 5)),

        SyntaxKind::OR_KW => Some((6, 7)),

        SyntaxKind::AND_KW => Some((8, 9)),

        SyntaxKind::DOUBLE_EQUAL
        | SyntaxKind::BANG_EQUAL
        | SyntaxKind::LESS
        | SyntaxKind::LESS_EQUAL
        | SyntaxKind::GREATER
        | SyntaxKind::GREATER_EQUAL => Some((10, 11)),

        SyntaxKind::COMPOSE_LEFT | SyntaxKind::COMPOSE_RIGHT | SyntaxKind::XOR_KW => {
            Some((12, 13))
        }

        SyntaxKind::PLUS | SyntaxKind::PLUS_DOT | SyntaxKind::MINUS | SyntaxKind::MINUS_DOT => {
            Some((14, 15))
        }

        SyntaxKind::STAR
        | SyntaxKind::STAR_DOT
        | SyntaxKind::SLASH
        | SyntaxKind::SLASH_DOT
        | SyntaxKind::PERCENT => Some((16, 17)),

        _ => None,
    }
}

/// Can this token start an atomic / primary expression?
fn can_start_atom(kind: SyntaxKind) -> bool {
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
                | SyntaxKind::REAL
                | SyntaxKind::STRING
                | SyntaxKind::GLYPH
                | SyntaxKind::TRUE_KW
                | SyntaxKind::FALSE_KW => {
                    let m = p.start_node(SyntaxKind::LITERAL);
                    p.bump();
                    p.finish_node(m);
                    true
                }

                // Identifier or path
                SyntaxKind::IDENT => {
                    // Check for path: Ident::Ident
                    if p.nth(1) == Some(SyntaxKind::DOUBLE_COLON) {
                        let m = p.start_node(SyntaxKind::PATH);
                        p.bump(); // first ident
                        p.bump(); // ::
                        p.expect(SyntaxKind::IDENT);
                        p.finish_node(m);
                    } else {
                        let m = p.start_node(SyntaxKind::IDENT_EXPR);
                        p.bump();
                        p.finish_node(m);
                    }
                    true
                }

                // Parenthesised expr, tuple, unit, or `(op)`
                SyntaxKind::L_PAREN => {
                    paren_or_tuple(p);
                    true
                }

                // Array literal
                SyntaxKind::L_SQUARE => {
                    array_expr(p);
                    true
                }

                // Struct literal
                SyntaxKind::L_BRACE => {
                    struct_expr(p);
                    true
                }

                _ => {
                    p.error_at_current("expected expression");
                    false
                }
            }
        }
    }
}

// ── Compound primaries ───────────────────────────────────────────────

/// `"(" ")"` (unit), `"(" expr ")"` (paren), `"(" expr "," ... ")"` (tuple),
/// or `"(" op ")"` (operator-as-value).
fn paren_or_tuple(p: &mut Parser<'_, '_>) {
    // Unit: `()`
    if p.nth(1) == Some(SyntaxKind::R_PAREN) {
        let m = p.start_node(SyntaxKind::UNIT);
        p.bump(); // (
        p.bump(); // )
        p.finish_node(m);
        return;
    }

    let m = p.start_node(SyntaxKind::PAREN_EXPR);
    p.expect(SyntaxKind::L_PAREN);

    // Operator as value: `(+)`, `(not)`, etc.
    if is_operator(p) && p.nth(1) == Some(SyntaxKind::R_PAREN) {
        // Wrap the operator in OPERATOR_EXPR
        let om = p.start_node(SyntaxKind::OPERATOR_EXPR);
        p.bump(); // the operator token
        p.finish_node(om);
        p.expect(SyntaxKind::R_PAREN);
        p.finish_node(m);
        return;
    }

    // First expression
    expr(p);

    if p.at(SyntaxKind::COMMA) {
        // Tuple — re-tag the node kind by finishing and noting this is
        // a tuple.  Since we can't retag with rowan's builder API we
        // finish as PAREN_EXPR and note: the PAREN_EXPR with commas is
        // a tuple at the semantic level.  Actually let's use a
        // checkpoint approach to wrap properly.
        //
        // We already opened PAREN_EXPR which is fine for tuples too;
        // we'll reinterpret in the AST layer. For now PAREN_EXPR serves
        // for both grouping and tuple.
        while p.eat(SyntaxKind::COMMA) {
            if p.at(SyntaxKind::R_PAREN) {
                break; // trailing comma
            }
            expr(p);
        }
    }

    p.expect(SyntaxKind::R_PAREN);
    p.finish_node(m);
}

fn is_operator(p: &mut Parser<'_, '_>) -> bool {
    p.current().is_some_and(|k| {
        matches!(
            k,
            SyntaxKind::PLUS
                | SyntaxKind::PLUS_DOT
                | SyntaxKind::MINUS
                | SyntaxKind::MINUS_DOT
                | SyntaxKind::STAR
                | SyntaxKind::STAR_DOT
                | SyntaxKind::SLASH
                | SyntaxKind::SLASH_DOT
                | SyntaxKind::PERCENT
                | SyntaxKind::PIPE_ARROW
                | SyntaxKind::COMPOSE_LEFT
                | SyntaxKind::COMPOSE_RIGHT
                | SyntaxKind::DOUBLE_EQUAL
                | SyntaxKind::BANG_EQUAL
                | SyntaxKind::LESS
                | SyntaxKind::LESS_EQUAL
                | SyntaxKind::GREATER
                | SyntaxKind::GREATER_EQUAL
                | SyntaxKind::AND_KW
                | SyntaxKind::OR_KW
                | SyntaxKind::XOR_KW
                | SyntaxKind::NOT_KW
                | SyntaxKind::SEMICOLON
        )
    })
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
        p.expect(SyntaxKind::IDENT);
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
        p.expect(SyntaxKind::IDENT);
        p.expect(SyntaxKind::COLON);
        type_expr::type_expr(p);
        p.expect(SyntaxKind::R_PAREN);
        p.finish_node(m);
    } else if p.at(SyntaxKind::IDENT) {
        let m = p.start_node(SyntaxKind::PARAM);
        p.bump();
        p.finish_node(m);
    } else {
        p.error_at_current("expected parameter");
    }
}

/// `"fn" ("|" pattern "=>" expr)+`
fn fn_shorthand(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::FN_SHORTHAND_EXPR);
    p.expect(SyntaxKind::FN_KW);
    while p.at(SyntaxKind::PIPE) {
        match_arm(p);
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
    while p.at(SyntaxKind::PIPE) {
        match_arm(p);
    }
    p.finish_node(m);
}

/// `"|" pattern "=>" expr`
fn match_arm(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::MATCH_ARM);
    p.expect(SyntaxKind::PIPE);
    pattern::pattern(p);
    p.expect(SyntaxKind::DOUBLE_ARROW);
    expr(p);
    p.finish_node(m);
}
