use crate::parse::parser::Parser;

use super::SyntaxKind;

/// ```bnf
/// sexpr ::= "(" <sexpr_item>* ")"
/// sexpr_item ::= <sexpr>
///              | <sexpr_path>
///              | <sexpr_ident>
///              | <sexpr_field>
///              | <sexpr_string>
///              | <sexpr_integer>
///              | <sexpr_float>
///              | <sexpr_bool>
/// sexpr_path ::= "$" <ident> "::" <ident>
/// sexpr_ident ::= "$" <ident> | <ident>
/// sexpr_field ::= <ident> "." <ident>
/// sexpr_string ::= <string>
/// sexpr_integer ::= <integer>
/// sexpr_float ::= <real>
/// sexpr_bool ::= "true" | "false"
/// ```
pub fn parse(p: &mut Parser<'_, '_>) {
    if !p.at(SyntaxKind::L_PAREN) {
        return;
    }
    parse_list(p);
}

fn parse_list(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::SEXPR);
    p.expect(SyntaxKind::L_PAREN);
    while !p.at_end() && !p.at(SyntaxKind::R_PAREN) {
        parse_item(p);
    }
    p.expect(SyntaxKind::R_PAREN);
    p.finish_node(m);
}

fn parse_item(p: &mut Parser<'_, '_>) {
    match p.current() {
        Some(SyntaxKind::L_PAREN) => parse_list(p),
        Some(SyntaxKind::DOLLAR) => {
            if is_sexpr_path_start(p) {
                parse_path(p);
            } else {
                parse_ident(p);
            }
        }
        Some(SyntaxKind::STRING) => parse_string(p),
        Some(SyntaxKind::INTEGER) => parse_integer(p),
        Some(SyntaxKind::REAL) => parse_float(p),
        Some(SyntaxKind::TRUE_KW | SyntaxKind::FALSE_KW) => parse_bool(p),
        Some(kind) if is_ident_token(kind) => {
            if p.nth(1) == Some(SyntaxKind::DOT) && p.nth(2).is_some_and(is_ident_token) {
                parse_field(p);
            } else {
                parse_ident(p);
            }
        }
        _ => {
            p.error_and_bump("expected s-expression item");
        }
    }
}

fn parse_ident(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::IDENT);
    if p.eat(SyntaxKind::DOLLAR) {
        if p.current().is_some_and(is_ident_token) {
            p.bump();
        } else {
            p.error_at_current("expected identifier after `$`");
        }
    } else if p.current().is_some_and(is_ident_token) {
        p.bump();
    } else {
        p.error_at_current("expected identifier");
    }
    p.finish_node(m);
}

fn parse_path(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::PATH);
    p.expect(SyntaxKind::DOLLAR);
    expect_plain_ident_token(p);
    p.expect(SyntaxKind::DOUBLE_COLON);
    expect_plain_ident_token(p);
    p.finish_node(m);
}

fn parse_field(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::SEXPR_FIELD);
    expect_ident_token(p);
    p.expect(SyntaxKind::DOT);
    expect_ident_token(p);
    p.finish_node(m);
}

fn parse_string(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::STRING);
    p.bump();
    p.finish_node(m);
}

fn parse_integer(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::INTEGER);
    p.bump();
    p.finish_node(m);
}

fn parse_float(p: &mut Parser<'_, '_>) {
    let m = p.start_node(SyntaxKind::REAL);
    p.bump();
    p.finish_node(m);
}

fn parse_bool(p: &mut Parser<'_, '_>) {
    let m = p.start_node(p.current().unwrap_or(SyntaxKind::TRUE_KW));
    p.bump();
    p.finish_node(m);
}

fn expect_ident_token(p: &mut Parser<'_, '_>) {
    if p.current().is_some_and(is_ident_token) {
        p.bump();
    } else {
        p.error_at_current("expected identifier");
    }
}

fn expect_plain_ident_token(p: &mut Parser<'_, '_>) {
    if p.at(SyntaxKind::IDENT) {
        p.bump();
    } else {
        p.error_at_current("expected identifier");
    }
}

fn is_sexpr_path_start(p: &Parser<'_, '_>) -> bool {
    p.at(SyntaxKind::DOLLAR)
        && p.nth(1) == Some(SyntaxKind::IDENT)
        && p.nth(2) == Some(SyntaxKind::DOUBLE_COLON)
        && p.nth(3) == Some(SyntaxKind::IDENT)
}

fn is_ident_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::IDENT
            | SyntaxKind::MODULE_KW
            | SyntaxKind::IMPORT_KW
            | SyntaxKind::USE_KW
            | SyntaxKind::END_KW
            | SyntaxKind::MATCH_KW
            | SyntaxKind::WITH_KW
            | SyntaxKind::LET_KW
            | SyntaxKind::TYPE_KW
            | SyntaxKind::TRAIT_KW
            | SyntaxKind::IMPL_KW
            | SyntaxKind::DO_KW
            | SyntaxKind::OF_KW
            | SyntaxKind::IN_KW
            | SyntaxKind::IF_KW
            | SyntaxKind::THEN_KW
            | SyntaxKind::ELSE_KW
            | SyntaxKind::AND_KW
            | SyntaxKind::OR_KW
            | SyntaxKind::XOR_KW
            | SyntaxKind::NOT_KW
            | SyntaxKind::FN_KW
            | SyntaxKind::WASM_KW
    )
}
