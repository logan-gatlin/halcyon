use super::{
    paren_list,
    path_or_ident,
};
use crate::parse::SyntaxKind;
use crate::parse::parser::Parser;

// ── Type definitions ─────────────────────────────────────────────────

/// Dispatch a type definition (right-hand side of `type Foo = ...`).
///
/// ```bnf
/// <type_def> ::= "{" field_decl+ "}"             -- record
///              | ("|" variant)+                  -- sum type
///              | <type_expr>                     -- alias
/// ```
pub(crate) fn type_def(p: &mut Parser<'_, '_>) {
    match p.current() {
        Some(SyntaxKind::L_BRACE) => struct_def(p),
        Some(SyntaxKind::PIPE) => sum_def(p),
        _ => type_alias_def(p),
    }
}

/// `"{" (ident ":" type_expr ","?)+ "}"`
fn struct_def(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::STRUCT_DEF);
    p.expect(SyntaxKind::L_BRACE);
    while !p.at(SyntaxKind::R_BRACE) && !p.at_end() {
        let fm = p.start_node(SyntaxKind::FIELD_DECL);
        p.expect(SyntaxKind::IDENT);
        p.expect(SyntaxKind::COLON);
        type_expr(p);
        p.finish_node(fm);
        p.eat(SyntaxKind::COMMA);
    }
    p.expect(SyntaxKind::R_BRACE);
    p.finish_node(m);
}

/// `("|" ident type_expr?)+`
fn sum_def(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::SUM_DEF);
    while p.at(SyntaxKind::PIPE) {
        let vm = p.start_node(SyntaxKind::VARIANT);
        p.bump(); // |
        p.expect(SyntaxKind::IDENT);
        // Optional payload type — if the next token could start a type
        // expression and is NOT `|` (which would be the next variant),
        // parse it as the variant's payload.
        if can_start_type_expr(p) {
            type_expr(p);
        }
        p.finish_node(vm);
    }
    p.finish_node(m);
}

/// A bare type expression used as an alias.
fn type_alias_def(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::TYPE_ALIAS_DEF);
    type_expr(p);
    p.finish_node(m);
}

// ── Type expressions ─────────────────────────────────────────────────

/// Parse a type expression with full precedence.
pub(crate) fn type_expr(p: &mut Parser<'_, '_>) {
    type_expr_bp(p, 0);
}

/// Pratt parser for type expressions.
///
/// Operators:
///   `->` (function type) — right-associative, low precedence
///   juxtaposition (type application) — left-associative, higher
fn type_expr_bp(
    p: &mut Parser<'_, '_>,
    min_bp: u8,
) {
    let checkpoint = p.checkpoint();

    if !type_primary(p) {
        return;
    }

    loop {
        let Some(kind) = p.current() else {
            break;
        };

        // `->` function type: right-associative
        if kind == SyntaxKind::ARROW {
            let (lbp, rbp) = (10, 9); // right-assoc: lbp > rbp
            if lbp < min_bp {
                break;
            }
            let m = p.start_node_at(checkpoint, SyntaxKind::FUNCTION_TYPE);
            p.bump(); // ->
            type_expr_bp(p, rbp);
            p.finish_node(m);
            continue;
        }

        // Type application by juxtaposition — collect all arguments
        if min_bp <= TYPE_APPLY_BP && can_start_type_atom(kind) {
            let m = p.start_node_at(checkpoint, SyntaxKind::TYPE_APPLICATION);
            while p.current().is_some_and(can_start_type_atom) {
                type_primary(p);
            }
            p.finish_node(m);
            continue;
        }

        break;
    }
}

const TYPE_APPLY_BP: u8 = 20;

fn can_start_type_expr(p: &mut Parser<'_, '_>) -> bool {
    p.current().is_some_and(can_start_type_atom)
}

fn can_start_type_atom(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::IDENT | SyntaxKind::L_PAREN | SyntaxKind::L_SQUARE
    )
}

/// Parse a type atom.
fn type_primary(p: &mut Parser<'_, '_>) -> bool {
    match p.current() {
        None => {
            p.error_at_current("expected type");
            false
        }
        Some(kind) => {
            match kind {
                SyntaxKind::IDENT => {
                    path_or_ident(p);
                    true
                }

                SyntaxKind::L_PAREN => {
                    // `()` unit type, `(type)` grouping, `(type, type, ...)` tuple type
                    paren_list(p, SyntaxKind::UNIT, SyntaxKind::TUPLE_TYPE, type_expr);
                    true
                }

                SyntaxKind::L_SQUARE => {
                    // `[]` — array type constructor
                    let m = p.start_node(SyntaxKind::ARRAY_TYPE);
                    p.bump(); // [
                    p.expect(SyntaxKind::R_SQUARE);
                    p.finish_node(m);
                    true
                }

                _ => {
                    p.error_and_bump("expected type");
                    false
                }
            }
        }
    }
}
