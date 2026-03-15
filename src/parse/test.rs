#![allow(clippy::unwrap_used)]
use super::ast::{
    self,
    AstNode,
    HasLeadingComments,
    HasName,
};
use super::lexer::{
    decode_quoted_glyph_literal,
    decode_quoted_string_literal,
    tokenize,
};
use super::{
    SyntaxKind,
    parse,
};
use crate::logging::Logger;

// ═══════════════════════════════════════════════════════════════════════
// Lexer tests
// ═══════════════════════════════════════════════════════════════════════

fn lex(input: &str) -> Vec<SyntaxKind> {
    let mut logger = crate::logging::FileLogger::new(0);
    let tokens = tokenize(input.chars(), &mut logger);
    assert!(logger.is_ok(), "Tokenizer produced errors: {:?}", logger);
    tokens
        .into_iter()
        .map(|t| t.inner)
        .filter(|k| !k.is_trivia())
        .collect()
}

fn lex_with_text(input: &str) -> Vec<(SyntaxKind, &str)> {
    let mut logger = crate::logging::FileLogger::new(0);
    let tokens = tokenize(input.chars(), &mut logger);
    assert!(logger.is_ok(), "Tokenizer produced errors: {:?}", logger);
    tokens
        .into_iter()
        .filter(|t| !matches!(t.inner, SyntaxKind::WHITESPACE))
        .map(|t| {
            use crate::Span;
            let text = match t.span {
                Span::Source { start, width } => {
                    input.get(start..start + width).unwrap_or_default()
                }
                Span::Generated => "",
            };
            (t.inner, text)
        })
        .collect()
}

#[test]
fn lex_symbols() {
    use SyntaxKind::*;
    let tokens = lex("() {} [] , : :: ; . .. + - / * % |> << >> -> => != = == > >= < <= | ~");
    let expected = vec![
        L_PAREN,
        R_PAREN,
        L_BRACE,
        R_BRACE,
        L_SQUARE,
        R_SQUARE,
        COMMA,
        COLON,
        DOUBLE_COLON,
        SEMICOLON,
        DOT,
        DOT_DOT,
        PLUS,
        MINUS,
        SLASH,
        STAR,
        PERCENT,
        PIPE_ARROW,
        COMPOSE_LEFT,
        COMPOSE_RIGHT,
        ARROW,
        DOUBLE_ARROW,
        BANG_EQUAL,
        EQUAL,
        DOUBLE_EQUAL,
        GREATER,
        GREATER_EQUAL,
        LESS,
        LESS_EQUAL,
        PIPE,
        TILDE,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn lex_keywords() {
    use SyntaxKind::*;
    let tokens = lex(
        "module bundle import use as end match with let type trait impl do of in if then else and or xor not true false fn wasm for where root",
    );
    let expected = vec![
        MODULE_KW, BUNDLE_KW, IMPORT_KW, USE_KW, AS_KW, END_KW, MATCH_KW, WITH_KW, LET_KW, TYPE_KW,
        TRAIT_KW, IMPL_KW, DO_KW, OF_KW, IN_KW, IF_KW, THEN_KW, ELSE_KW, AND_KW, OR_KW, XOR_KW,
        NOT_KW, TRUE_KW, FALSE_KW, FN_KW, WASM_KW, FOR_KW, WHERE_KW, ROOT_KW,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn lex_identifiers() {
    let tokens = lex_with_text("foo bar_baz _qux");
    assert_eq!(
        tokens.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
        vec!["foo", "bar_baz", "_qux"]
    );
}

#[test]
fn lex_identifiers_with_hyphens() {
    let tokens = lex_with_text("foo-bar foo-");
    assert_eq!(
        tokens.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
        vec!["foo-bar", "foo", "-"]
    );
}

#[test]
fn lex_integers() {
    let tokens = lex_with_text("123 0xff 0o77 0b101");
    assert_eq!(
        tokens.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
        vec!["123", "0xff", "0o77", "0b101"]
    );
    assert!(tokens.iter().all(|(k, _)| *k == SyntaxKind::INTEGER));
}

#[test]
fn lex_floats() {
    let tokens = lex_with_text("1.0 0.5 1e10 1.2e-5");
    assert_eq!(
        tokens.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
        vec!["1.0", "0.5", "1e10", "1.2e-5"]
    );
    assert!(tokens.iter().all(|(k, _)| *k == SyntaxKind::REAL));
}

#[test]
fn lex_strings() {
    let tokens = lex_with_text(r#""hello" "world""#);
    assert_eq!(
        tokens.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
        vec![r#""hello""#, r#""world""#]
    );
}

#[test]
fn decode_string_literal_all_escape_sequences() {
    let cases = [
        (r#""\n""#, "\n"),
        (r#""\r""#, "\r"),
        (r#""\t""#, "\t"),
        (r#""\b""#, "\x08"),
        (r#""\\""#, "\\"),
        (r#""\0""#, "\0"),
        (r#""\"""#, "\""),
        (r#""\'""#, "'"),
        (r#""\x41""#, "A"),
        (r#""\w03BB""#, "\u{03BB}"),
    ];

    for (literal, expected) in cases {
        assert_eq!(
            decode_quoted_string_literal(literal).as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn lex_glyphs() {
    let tokens = lex_with_text(r#"'a' 'b' 'c'"#);
    assert_eq!(
        tokens.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
        vec!["'a'", "'b'", "'c'"]
    );
}

#[test]
fn decode_glyph_literal_all_escape_sequences() {
    let cases = [
        (r#"'\n'"#, '\n'),
        (r#"'\r'"#, '\r'),
        (r#"'\t'"#, '\t'),
        (r#"'\b'"#, '\x08'),
        (r#"'\\'"#, '\\'),
        (r#"'\0'"#, '\0'),
        (r#"'\"'"#, '"'),
        (r#"'\''"#, '\''),
        (r#"'\x41'"#, 'A'),
        (r#"'\w03BB'"#, '\u{03BB}'),
    ];

    for (literal, expected) in cases {
        assert_eq!(decode_quoted_glyph_literal(literal), Some(expected));
    }
}

#[test]
fn lex_comments() {
    let tokens = lex_with_text("1 -- comment\n2 (* block comment *) 3");
    assert_eq!(tokens[0].0, SyntaxKind::INTEGER);
    assert_eq!(tokens[0].1, "1");
    assert_eq!(tokens[1].0, SyntaxKind::LINE_COMMENT);
    assert_eq!(tokens[1].1, "-- comment\n");
    assert_eq!(tokens[2].0, SyntaxKind::INTEGER);
    assert_eq!(tokens[2].1, "2");
    assert_eq!(tokens[3].0, SyntaxKind::BLOCK_COMMENT);
    assert_eq!(tokens[3].1, "(* block comment *)");
    assert_eq!(tokens[4].0, SyntaxKind::INTEGER);
    assert_eq!(tokens[4].1, "3");
}

// ═══════════════════════════════════════════════════════════════════════
// Parser tests
// ═══════════════════════════════════════════════════════════════════════

/// Helper: parse source and return the debug-printed syntax tree.
fn parse_to_string(source: &str) -> String {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let tree = parse(source, &mut file_logger).expect("root should be SourceFile");
    format!("{:#?}", tree.syntax())
}

/// Helper: parse source and check that it has errors.
fn assert_has_errors(source: &str) {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let _ = parse(source, &mut file_logger);
    assert!(!file_logger.is_ok(), "Expected parse errors");
}

/// Helper: parse source and check that it has no errors.
fn assert_no_errors(source: &str) {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let _ = parse(source, &mut file_logger);
    assert!(file_logger.is_ok(), "Expected parse errors to be empty");
}

/// Concatenating all tokens of the tree should reproduce the original
/// source exactly.
fn assert_round_trip(source: &str) {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let tree = parse(source, &mut file_logger).expect("root should be SourceFile");
    let recovered: String = tree.syntax().text().to_string();
    assert_eq!(
        recovered, source,
        "Round-trip failed.\nExpected:\n{source}\nGot:\n{recovered}"
    );
}

// ── Round-trip tests ─────────────────────────────────────────────────

#[test]
fn round_trip_empty() {
    assert_round_trip("");
}

#[test]
fn round_trip_simple_module() {
    assert_round_trip("module Foo =\n  let x = 1\nend\n");
}

#[test]
fn round_trip_with_comments() {
    assert_round_trip("-- hello\nmodule Foo =\n  -- a comment\n  let x = 1\nend\n");
}

#[test]
fn round_trip_complex() {
    let src = "module Math =\n  let add = fn x y => x + y\n  let apply = fn f x => f x\nend\n";
    assert_round_trip(src);
}

#[test]
fn round_trip_type_def() {
    let src = "module T =\n  type option: a = | Some a | None\nend\n";
    assert_round_trip(src);
}

#[test]
fn round_trip_match() {
    let src = "module M =\n  let f = fn x => match x with\n    | 0 => 1\n    | n => n\nend\n";
    assert_round_trip(src);
}

#[test]
fn round_trip_if_expr() {
    let src = "module M =\n  let f = if true then 1 else 0\nend\n";
    assert_round_trip(src);
}

#[test]
fn round_trip_struct() {
    let src = "module M =\n  type point = { x: int, y: int }\nend\n";
    assert_round_trip(src);
}

#[test]
fn round_trip_struct_spread() {
    let src = "module M =\n  type circle = { radius: int, ..point }\nend\n";
    assert_round_trip(src);
}

#[test]
fn round_trip_array() {
    let src = "module M =\n  let xs = [1 2 3]\nend\n";
    assert_round_trip(src);
}

// ── Structure tests ──────────────────────────────────────────────────

#[test]
fn parse_empty_source() {
    let tree_str = parse_to_string("");
    assert!(
        tree_str.contains("SOURCE_FILE"),
        "Root should be SOURCE_FILE"
    );
}

#[test]
fn parse_module_structure() {
    let tree_str = parse_to_string("module Foo = end");
    assert!(tree_str.contains("MODULE"), "Should contain MODULE node");
    assert!(
        tree_str.contains("MODULE_KW"),
        "Should contain module keyword"
    );
    assert!(tree_str.contains("END_KW"), "Should contain end keyword");
    assert!(tree_str.contains("Foo"), "Should contain module name");
}

#[test]
fn parse_nested_module_statement() {
    assert_no_errors("module Foo =\n  module Bar =\n    let x = 1\n  end\nend");
}

#[test]
fn parse_top_level_import_statement() {
    let tree_str = parse_to_string("import \"./dep.hc\"\nmodule Main = end");
    assert!(
        tree_str.contains("IMPORT_STATEMENT"),
        "Should contain IMPORT_STATEMENT"
    );
    assert!(
        tree_str.contains("STRING"),
        "Should contain import path token"
    );
}

#[test]
fn parse_top_level_import_multiple_paths() {
    assert_no_errors("import \"./a.hc\", \"./b.hc\"\nmodule Main = end");
}

#[test]
fn parse_import_inside_module_is_error() {
    assert_no_errors("module Main =\n  import \"./dep.hc\"\nend");
}

#[test]
fn parse_use_inside_module() {
    assert_no_errors("module Main =\n  use core\n  let x = default\nend");
}

#[test]
fn parse_use_with_alias_inside_module() {
    assert_no_errors("module Main =\n  use core as c\n  let x = c::default\nend");
}

#[test]
fn parse_use_expr() {
    assert_no_errors("module Main =\n  let x = use core in default\nend");
}

#[test]
fn parse_use_expr_with_alias() {
    assert_no_errors("module Main =\n  let x = use core as c in c::default\nend");
}

#[test]
fn parse_top_level_use_is_error() {
    assert_no_errors("use core\nmodule Main = end");
}

#[test]
fn parse_top_level_statement_is_error() {
    assert_no_errors("let x = 1");
}

#[test]
fn parse_top_level_end_is_error() {
    assert_has_errors("end");
}

#[test]
fn parse_let_statement() {
    let tree_str = parse_to_string("module M =\n  let x = 42\nend");
    assert!(
        tree_str.contains("LET_STATEMENT"),
        "Should contain LET_STATEMENT"
    );
    assert!(tree_str.contains("LITERAL"), "Should contain LITERAL node");
}

#[test]
fn parse_binary_expr() {
    let tree_str = parse_to_string("module M =\n  let x = 1 + 2\nend");
    assert!(
        tree_str.contains("BINARY_EXPR"),
        "Should contain BINARY_EXPR"
    );
}

#[test]
fn parse_fn_expr() {
    let tree_str = parse_to_string("module M =\n  let f = fn x => x\nend");
    assert!(tree_str.contains("FN_EXPR"), "Should contain FN_EXPR");
    assert!(tree_str.contains("PARAM"), "Should contain PARAM");
}

#[test]
fn parse_type_sum() {
    let tree_str = parse_to_string("module M =\n  type bool = | True | False\nend");
    assert!(tree_str.contains("SUM_DEF"), "Should contain SUM_DEF");
    assert!(tree_str.contains("VARIANT"), "Should contain VARIANT");
}

#[test]
fn parse_trait_statement() {
    let tree_str = parse_to_string(
        "module M =\n  trait Eq : a =\n    let eq : a -> a -> core::Boolean\n  end\nend",
    );
    assert!(
        tree_str.contains("TRAIT_STATEMENT"),
        "Should contain TRAIT_STATEMENT"
    );
    assert!(
        tree_str.contains("TRAIT_METHOD_DECL"),
        "Should contain TRAIT_METHOD_DECL"
    );
}

#[test]
fn parse_trait_alias_statement() {
    let tree_str = parse_to_string("module M =\n  trait ~Eq = core::Equal\nend");
    assert!(
        tree_str.contains("TRAIT_STATEMENT"),
        "Should contain TRAIT_STATEMENT"
    );
    assert!(
        !tree_str.contains("TRAIT_METHOD_DECL"),
        "Trait alias should not contain TRAIT_METHOD_DECL"
    );
}

#[test]
fn parse_impl_statement() {
    let tree_str = parse_to_string(
        "module M =\n  impl Eq core::Integer =\n    let eq = fn x y => x == y\n  end\nend",
    );
    assert!(
        tree_str.contains("IMPL_STATEMENT"),
        "Should contain IMPL_STATEMENT"
    );
    assert!(
        tree_str.contains("IMPL_METHOD_DEF"),
        "Should contain IMPL_METHOD_DEF"
    );
}

#[test]
fn parse_impl_statement_multiple_comma_args() {
    assert_no_errors(
        "module M =\n  impl Pair core::Integer, core::String =\n    let show = fn p => p\n  end\nend",
    );
}

#[test]
fn parse_impl_statement_with_forall_argument() {
    assert_no_errors("module M =\n  impl Id for a in a =\n    let id = fn x => x\n  end\nend");
}

#[test]
fn parse_error_recovery() {
    assert_has_errors("module M = !!! end");
}

#[test]
fn parse_error_recovery_invalid_expr_token() {
    let source = "module M =\n  let x = )\n  let y = 1\nend";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let tree = parse(source, &mut file_logger).expect("root should be SourceFile");
    assert!(!file_logger.is_ok(), "Should have errors");
    let statements = tree.modules()[0].statements();
    assert_eq!(statements.len(), 2, "Should recover to next statement");
}

#[test]
fn parse_error_recovery_invalid_pattern_token() {
    let source = "module M =\n  let ) = 1\n  let y = 2\nend";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let tree = parse(source, &mut file_logger).expect("root should be SourceFile");
    assert!(!file_logger.is_ok(), "Should have errors");
    let statements = tree.modules()[0].statements();
    assert_eq!(statements.len(), 2, "Should recover to next statement");
}

#[test]
fn parse_error_recovery_invalid_type_token() {
    let source = "module M =\n  type t = )\n  type u = int\nend";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let tree = parse(source, &mut file_logger).expect("root should be SourceFile");
    assert!(!file_logger.is_ok(), "Should have errors");
    let statements = tree.modules()[0].statements();
    assert_eq!(statements.len(), 2, "Should recover to next statement");
}

#[test]
fn error_recovery_preserves_tree() {
    // Even with errors, the tree should still round-trip
    let source = "module M = !!! end";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let tree = parse(source, &mut file_logger).expect("root should be SourceFile");
    assert!(!file_logger.is_ok(), "Should have errors");
    // The tree should still be rooted at SOURCE_FILE
    assert_eq!(tree.syntax().kind(), SyntaxKind::SOURCE_FILE);
}

#[test]
fn parse_function_call() {
    let tree_str = parse_to_string("module M =\n  let x = f 42\nend");
    assert!(tree_str.contains("CALL_EXPR"), "Should contain CALL_EXPR");
}

#[test]
fn parse_if_expr() {
    let tree_str = parse_to_string("module M =\n  let x = if true then 1 else 0\nend");
    assert!(tree_str.contains("IF_EXPR"), "Should contain IF_EXPR");
}

#[test]
fn parse_match_expr() {
    let tree_str =
        parse_to_string("module M =\n  let x = match y with\n    | 0 => 1\n    | n => n\nend");
    assert!(tree_str.contains("MATCH_EXPR"), "Should contain MATCH_EXPR");
    assert!(tree_str.contains("MATCH_ARM"), "Should contain MATCH_ARM");
}

#[test]
fn parse_match_expr_without_pipe() {
    assert_no_errors("module M =\n  let x = match y with\n    0 => 1\n    | n => n\nend");
}

#[test]
fn parse_path_expr() {
    let tree_str = parse_to_string("module M =\n  let x = Foo::bar\nend");
    assert!(tree_str.contains("PATH"), "Should contain PATH");
}

#[test]
fn parse_field_access() {
    let tree_str = parse_to_string("module M =\n  let x = r.field\nend");
    assert!(tree_str.contains("FIELD_EXPR"), "Should contain FIELD_EXPR");
}

#[test]
fn parse_polymorphic_type() {
    let tree_str = parse_to_string("module M =\n  type list: a = | Cons (a, list a) | Nil\nend");
    assert!(
        tree_str.contains("TYPE_STATEMENT"),
        "Should contain TYPE_STATEMENT"
    );
    assert!(tree_str.contains("SUM_DEF"), "Should contain SUM_DEF");
}

#[test]
fn parse_struct_def() {
    let tree_str = parse_to_string("module M =\n  type point = { x: int, y: int }\nend");
    assert!(tree_str.contains("STRUCT_DEF"), "Should contain STRUCT_DEF");
    assert!(tree_str.contains("FIELD_DECL"), "Should contain FIELD_DECL");
}

#[test]
fn parse_struct_def_with_spread() {
    let tree_str = parse_to_string("module M =\n  type circle = { radius: int, ..point }\nend");
    assert!(tree_str.contains("STRUCT_DEF"), "Should contain STRUCT_DEF");
    assert!(
        tree_str.contains("STRUCT_SPREAD_DECL"),
        "Should contain STRUCT_SPREAD_DECL"
    );
}

#[test]
fn parse_pattern_type_hint() {
    let tree_str = parse_to_string("module M =\n  let (x: int) = 42\nend");
    assert!(
        tree_str.contains("PAT_TYPE_HINT"),
        "Should contain PAT_TYPE_HINT"
    );
}

#[test]
fn parse_constructor_pattern_juxtaposition() {
    let tree_str = parse_to_string(
        "module M =\n  let x = match opt with\n    | Some value => value\n    | None => 0\nend",
    );
    assert!(
        tree_str.contains("PAT_CONSTRUCTOR"),
        "Should contain PAT_CONSTRUCTOR"
    );
}

#[test]
fn parse_constructor_pattern_with_of_is_error() {
    assert_has_errors(
        "module M =\n  let x = match opt with\n    | Some of value => value\n    | None => 0\nend",
    );
}

#[test]
fn parse_let_in_expr() {
    let tree_str = parse_to_string("module M =\n  let x = let y = 1 in y + 1\nend");
    assert!(tree_str.contains("LET_EXPR"), "Should contain LET_EXPR");
}

#[test]
fn parse_fn_shorthand() {
    let tree_str = parse_to_string("module M =\n  let f = fn\n    | 0 => 1\n    | n => n\nend");
    assert!(
        tree_str.contains("FN_SHORTHAND_EXPR"),
        "Should contain FN_SHORTHAND_EXPR"
    );
}

#[test]
fn parse_unary_expr() {
    let tree_str = parse_to_string("module M =\n  let x = -1\nend");
    assert!(tree_str.contains("UNARY_EXPR"), "Should contain UNARY_EXPR");
}

#[test]
fn parse_parenthesized_operator_as_value_is_error() {
    assert_has_errors("module M =\n  let x = (+) 1 2\nend");
}

#[test]
fn parse_struct_literal() {
    let tree_str = parse_to_string("module M =\n  let p = { x = 1, y = 2 }\nend");
    assert!(
        tree_str.contains("STRUCT_EXPR"),
        "Should contain STRUCT_EXPR"
    );
    assert!(
        tree_str.contains("STRUCT_FIELD"),
        "Should contain STRUCT_FIELD"
    );
}

#[test]
fn parse_array_expr() {
    let tree_str = parse_to_string("module M =\n  let xs = [1 2 3]\nend");
    assert!(tree_str.contains("ARRAY_EXPR"), "Should contain ARRAY_EXPR");
}

// ═══════════════════════════════════════════════════════════════════════
// Typed AST wrapper tests
// ═══════════════════════════════════════════════════════════════════════

/// Helper: parse source and return a typed SourceFile.
fn parse_source_file(source: &str) -> ast::SourceFile {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    parse(source, &mut file_logger).expect("root should be SourceFile")
}

#[test]
fn ast_source_file_modules() {
    let sf = parse_source_file("module A = end\nmodule B = end");
    let mods = sf.modules();
    assert_eq!(mods.len(), 2, "Should have 2 modules");
    assert_eq!(mods[0].name_text().as_deref(), Some("A"));
    assert_eq!(mods[1].name_text().as_deref(), Some("B"));
}

#[test]
fn ast_source_file_items_preserve_order() {
    let sf =
        parse_source_file("import \"./a.hc\"\nmodule A = end\nimport \"./b.hc\"\nmodule B = end");
    let items = sf.items();
    assert_eq!(items.len(), 4, "Should have 4 top-level items");
    assert!(matches!(items[0], ast::Statement::Import(_)));
    assert!(matches!(items[1], ast::Statement::Module(_)));
    assert!(matches!(items[2], ast::Statement::Import(_)));
    assert!(matches!(items[3], ast::Statement::Module(_)));
}

#[test]
fn ast_import_statement_paths() {
    let sf = parse_source_file("import \"./a.hc\", \"./b.hc\"\nmodule Main = end");
    let imports = sf.imports();
    assert_eq!(imports.len(), 1, "Should have one import statement");
    let literals = imports[0].path_literals();
    assert_eq!(literals.len(), 2, "Should have two import path literals");
    assert_eq!(literals[0].inner, "\"./a.hc\"");
    assert_eq!(literals[1].inner, "\"./b.hc\"");
}

#[test]
fn ast_module_statements() {
    let sf = parse_source_file("module M =\n  let x = 1\n  type t = int\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    assert_eq!(stmts.len(), 2, "Should have 2 statements");
    assert!(
        matches!(stmts[0], ast::Statement::Let(_)),
        "First should be Let"
    );
    assert!(
        matches!(stmts[1], ast::Statement::Type(_)),
        "Second should be Type"
    );
}

#[test]
fn ast_use_statement_accessors() {
    let sf = parse_source_file("module M =\n  use root::core\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    assert_eq!(stmts.len(), 1, "Should have 1 statement");
    let ast::Statement::Use(ref use_stmt) = stmts[0] else {
        panic!("expected use statement");
    };
    let target = use_stmt.target().expect("use should have target");
    let ast::PathOrIdent::Path(path) = target else {
        panic!("expected path target");
    };
    assert!(path.is_rooted(), "use target should be rooted");
    assert_eq!(path.segments(), vec!["core"]);
}

#[test]
fn ast_use_statement_bundle_rooted_accessors() {
    let sf = parse_source_file("module M =\n  use bundle::core\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    assert_eq!(stmts.len(), 1, "Should have 1 statement");
    let ast::Statement::Use(ref use_stmt) = stmts[0] else {
        panic!("expected use statement");
    };
    let target = use_stmt.target().expect("use should have target");
    let ast::PathOrIdent::Path(path) = target else {
        panic!("expected path target");
    };
    assert!(
        path.is_bundle_rooted(),
        "use target should be bundle-rooted"
    );
    assert_eq!(path.segments(), vec!["core"]);
}

#[test]
fn ast_use_statement_alias_accessor() {
    let sf = parse_source_file("module M =\n  use core as c\nend");
    let m = &sf.modules()[0];
    let ast::Statement::Use(ref use_stmt) = m.statements()[0] else {
        panic!("expected use statement");
    };
    assert_eq!(
        use_stmt
            .alias_name_spanned()
            .map(|alias| alias.inner)
            .as_deref(),
        Some("c")
    );
}

#[test]
fn ast_use_expr_accessors() {
    let sf = parse_source_file("module M =\n  let x = use core as c in c::default\nend");
    let ast::Statement::Let(ref let_stmt) = sf.modules()[0].statements()[0] else {
        panic!("expected let statement");
    };
    let ast::Expr::Use(ref use_expr) = let_stmt.value().expect("let should have value") else {
        panic!("expected use expression");
    };
    let ast::PathOrIdent::Ident(target) = use_expr.target().expect("use should have target") else {
        panic!("expected ident use target");
    };
    assert_eq!(target.name_text().as_deref(), Some("core"));
    assert_eq!(
        use_expr
            .alias_name_spanned()
            .map(|alias| alias.inner)
            .as_deref(),
        Some("c")
    );
    assert!(use_expr.body().is_some(), "use expression should have body");
}

#[test]
fn ast_module_can_contain_nested_module_statements() {
    let sf = parse_source_file("module M =\n  module N =\n    let x = 1\n  end\nend");
    let outer = &sf.modules()[0];
    let stmts = outer.statements();
    assert_eq!(stmts.len(), 1, "Should have one statement");
    let ast::Statement::Module(ref inner) = stmts[0] else {
        panic!("expected nested module statement");
    };
    assert_eq!(inner.name_text().as_deref(), Some("N"));
    assert_eq!(inner.statements().len(), 1);
}

#[test]
fn ast_wasm_keyword_head_is_ident_atom() {
    let sf = parse_source_file("module M =\n  wasm => ((type integer i64))\nend");
    let ast::Statement::Wasm(ref wasm_stmt) = sf.modules()[0].statements()[0] else {
        panic!("expected wasm statement");
    };
    let Some(ast::SexprItem::List(list)) = wasm_stmt
        .sexpr()
        .and_then(|sexpr| sexpr.items().into_iter().next())
    else {
        panic!("expected wasm declaration list");
    };
    let Some(ast::SexprItem::Atom(ast::SexprAtom::Ident(token))) = list.items().into_iter().next()
    else {
        panic!("expected keyword as ident atom");
    };
    assert_eq!(token.text(), "type");
}

#[test]
fn ast_wasm_symbol_keyword_is_symbol_ident_atom() {
    let sf = parse_source_file("module M =\n  wasm => ((global $type i64))\nend");
    let ast::Statement::Wasm(ref wasm_stmt) = sf.modules()[0].statements()[0] else {
        panic!("expected wasm statement");
    };
    let Some(ast::SexprItem::List(list)) = wasm_stmt
        .sexpr()
        .and_then(|sexpr| sexpr.items().into_iter().next())
    else {
        panic!("expected wasm declaration list");
    };
    let Some(ast::SexprItem::Atom(ast::SexprAtom::SymbolIdent(token))) =
        list.items().into_iter().nth(1)
    else {
        panic!("expected `$` keyword as symbol ident atom");
    };
    assert_eq!(token.text(), "type");
}

#[test]
fn parse_wasm_requires_fat_arrow() {
    assert_has_errors("module M =\n  wasm ((func $foo))\nend");
    assert_no_errors("module M =\n  wasm => ((func $foo))\nend");
}

#[test]
fn ast_inline_wasm_expr_has_type_and_instructions() {
    let sf = parse_source_file("module M =\n  let x = (wasm : core::Integer) => (get x)\nend");
    let ast::Statement::Let(ref let_stmt) = sf.modules()[0].statements()[0] else {
        panic!("expected let statement");
    };
    let ast::Expr::InlineWasm(ref inline_wasm) = let_stmt.value().expect("should have value")
    else {
        panic!("expected inline wasm expression");
    };
    assert!(
        inline_wasm.asserted_type().is_some(),
        "expected asserted type"
    );
    let Some(ast::SexprItem::Atom(ast::SexprAtom::Ident(token))) = inline_wasm
        .instructions()
        .and_then(|sexpr| sexpr.items().into_iter().next())
    else {
        panic!("expected first instruction token");
    };
    assert_eq!(token.text(), "get");
}

#[test]
fn ast_let_statement_accessors() {
    let sf = parse_source_file("module M =\n  let x = 42\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    let ast::Statement::Let(ref let_stmt) = stmts[0] else {
        panic!("expected let statement");
    };
    // Pattern should be an identifier
    let pat = let_stmt.pattern().expect("should have pattern");
    assert!(
        matches!(pat, ast::Pattern::Ident(_)),
        "pattern should be ident"
    );
    if let ast::Pattern::Ident(ref id) = pat {
        assert_eq!(id.name_text().as_deref(), Some("x"));
    }
    // Value should be a literal
    let val = let_stmt.value().expect("should have value");
    assert!(
        matches!(val, ast::Expr::Literal(_)),
        "value should be literal"
    );
}

#[test]
fn ast_type_statement_accessors() {
    let sf = parse_source_file("module M =\n  type point = { x: int, y: int }\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    let ast::Statement::Type(ref type_stmt) = stmts[0] else {
        panic!("expected type statement");
    };
    assert_eq!(type_stmt.name_text().as_deref(), Some("point"));
    let td = type_stmt.type_def().expect("should have type def");
    assert!(
        matches!(td, ast::TypeDef::Struct(_)),
        "should be struct def"
    );
    if let ast::TypeDef::Struct(ref sd) = td {
        let fields = sd.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name_text().as_deref(), Some("x"));
        assert_eq!(fields[1].name_text().as_deref(), Some("y"));
        // Each field should have a type
        assert!(fields[0].ty().is_some(), "field x should have a type");
        assert!(fields[1].ty().is_some(), "field y should have a type");
    }
}

#[test]
fn ast_struct_type_members_include_spreads() {
    let sf = parse_source_file("module M =\n  type circle = { radius: int, ..point }\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    let ast::Statement::Type(ref type_stmt) = stmts[0] else {
        panic!("expected type statement");
    };
    let ast::TypeDef::Struct(ref sd) = type_stmt.type_def().expect("should have type def") else {
        panic!("expected struct type def");
    };

    let members = sd.members();
    assert_eq!(members.len(), 2);
    match &members[0] {
        ast::StructTypeMemberDecl::Field(field) => {
            assert_eq!(field.name_text().as_deref(), Some("radius"));
            assert!(field.ty().is_some());
        }
        ast::StructTypeMemberDecl::Spread(_) => panic!("first member should be a field"),
    }
    match &members[1] {
        ast::StructTypeMemberDecl::Spread(spread) => {
            assert!(spread.ty().is_some());
        }
        ast::StructTypeMemberDecl::Field(_) => panic!("second member should be a spread"),
    }
    assert_eq!(sd.fields().len(), 1);
    assert_eq!(sd.spreads().len(), 1);
}

#[test]
fn ast_type_statement_alias_marker() {
    let sf = parse_source_file("module M =\n  type ~pair: a b = (a, b)\nend");
    let m = &sf.modules()[0];
    let ast::Statement::Type(ref type_stmt) = m.statements()[0] else {
        panic!("expected type statement");
    };
    assert!(type_stmt.is_alias());
    assert_eq!(type_stmt.name_text().as_deref(), Some("pair"));
    let params = type_stmt.type_params();
    assert_eq!(params.len(), 2);
    let ast::TypeDef::Alias(_) = type_stmt.type_def().expect("should have type def") else {
        panic!("expected alias rhs");
    };
}

#[test]
fn parse_tilde_type_alias_rejects_struct_rhs() {
    assert_has_errors("module M =\n  type ~point = { x: int }\nend");
}

#[test]
fn ast_trait_statement_accessors() {
    let sf = parse_source_file(
        "module M =\n  trait Eq : a =\n    let eq : a -> a -> core::Boolean\n  end\nend",
    );
    let m = &sf.modules()[0];
    let ast::Statement::Trait(ref trait_stmt) = m.statements()[0] else {
        panic!("expected trait statement");
    };
    assert_eq!(trait_stmt.name_text().as_deref(), Some("Eq"));
    let params = trait_stmt.trait_params();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].inner, "a");
    let methods = trait_stmt.methods();
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name_text().as_deref(), Some("eq"));
    assert!(methods[0].ty().is_some());
}

#[test]
fn ast_trait_alias_statement_accessors() {
    let sf = parse_source_file("module M =\n  trait ~Eq = core::Equal\nend");
    let m = &sf.modules()[0];
    let ast::Statement::Trait(ref trait_stmt) = m.statements()[0] else {
        panic!("expected trait statement");
    };
    assert!(trait_stmt.is_alias());
    let Some(ast::PathOrIdent::Path(alias_target)) = trait_stmt.alias_target() else {
        panic!("expected path alias target");
    };
    assert_eq!(alias_target.segments(), vec!["core", "Equal"]);
}

#[test]
fn ast_impl_statement_accessors() {
    let sf = parse_source_file(
        "module M =\n  impl Eq core::Integer =\n    let eq = fn x y => x == y\n  end\nend",
    );
    let m = &sf.modules()[0];
    let ast::Statement::Impl(ref impl_stmt) = m.statements()[0] else {
        panic!("expected impl statement");
    };
    let Some(head) = impl_stmt.trait_name() else {
        panic!("expected impl trait head");
    };
    assert_eq!(head.name_text().as_deref(), Some("Eq"));
    assert_eq!(impl_stmt.type_args().len(), 1);
    let methods = impl_stmt.methods();
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name_text().as_deref(), Some("eq"));
    assert!(methods[0].value().is_some());
}

#[test]
fn ast_sum_type() {
    let sf = parse_source_file("module M =\n  type color = | Red | Green | Blue\nend");
    let m = &sf.modules()[0];
    let stmts = m.statements();
    let ast::Statement::Type(ref ts) = stmts[0] else {
        panic!("expected type");
    };
    let ast::TypeDef::Sum(ref sum) = ts.type_def().unwrap() else {
        panic!("expected sum");
    };
    let variants = sum.variants();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0].name_text().as_deref(), Some("Red"));
    assert_eq!(variants[1].name_text().as_deref(), Some("Green"));
    assert_eq!(variants[2].name_text().as_deref(), Some("Blue"));
    // No payload types on these variants
    assert!(variants[0].payload_type().is_none());
}

#[test]
fn ast_sum_with_payload() {
    let sf = parse_source_file("module M =\n  type option: a = | Some a | None\nend");
    let m = &sf.modules()[0];
    let ast::Statement::Type(ref ts) = m.statements()[0] else {
        panic!("expected type");
    };
    // Type params are now on TypeStatement
    let params = ts.type_params();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].inner.clone(), "a");
    let ast::TypeDef::Sum(ref sum) = ts.type_def().unwrap() else {
        panic!("expected sum");
    };
    assert_eq!(sum.variants().len(), 2);
    assert!(
        sum.variants()[0].payload_type().is_some(),
        "Some has payload"
    );
    assert!(
        sum.variants()[1].payload_type().is_none(),
        "None has no payload"
    );
}

#[test]
fn ast_binary_expr() {
    let sf = parse_source_file("module M =\n  let x = 1 + 2\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Binary(ref bin) = ls.value().unwrap() else {
        panic!("expected binary");
    };
    assert!(bin.lhs().is_some(), "should have lhs");
    assert!(bin.rhs().is_some(), "should have rhs");
    let op = bin.op_token().expect("should have op");
    assert_eq!(op.text(), "+");
}

#[test]
fn ast_unary_expr() {
    let sf = parse_source_file("module M =\n  let x = -1\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Unary(ref un) = ls.value().unwrap() else {
        panic!("expected unary");
    };
    let op = un.op_token().expect("should have op");
    assert_eq!(op.text(), "-");
    assert!(un.operand().is_some(), "should have operand");
}

#[test]
fn ast_fn_expr() {
    let sf = parse_source_file("module M =\n  let f = fn x y => x + y\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Fn(ref fn_e) = ls.value().unwrap() else {
        panic!("expected fn");
    };
    let params = fn_e.params();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name_text().as_deref(), Some("x"));
    assert_eq!(params[1].name_text().as_deref(), Some("y"));
    assert!(fn_e.body().is_some(), "should have body");
}

#[test]
fn ast_fn_typed_param() {
    let sf = parse_source_file("module M =\n  let f = fn (x: int) => x\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Fn(ref fn_e) = ls.value().unwrap() else {
        panic!("expected fn");
    };
    let params = fn_e.params();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name_text().as_deref(), Some("x"));
    assert!(
        params[0].ty().is_some(),
        "param should have type annotation"
    );
}

#[test]
fn ast_fn_shorthand() {
    let sf = parse_source_file("module M =\n  let f = fn\n    | 0 => 1\n    | n => n\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::FnShorthand(ref fns) = ls.value().unwrap() else {
        panic!("expected fn shorthand");
    };
    assert_eq!(fns.arms().len(), 2);
}

#[test]
fn ast_if_expr() {
    let sf = parse_source_file("module M =\n  let x = if true then 1 else 0\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::If(ref if_e) = ls.value().unwrap() else {
        panic!("expected if");
    };
    assert!(if_e.condition().is_some());
    assert!(if_e.then_branch().is_some());
    assert!(if_e.else_branch().is_some());
}

#[test]
fn ast_match_expr() {
    let sf =
        parse_source_file("module M =\n  let x = match y with\n    | 0 => 1\n    | n => n\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Match(ref m) = ls.value().unwrap() else {
        panic!("expected match");
    };
    assert!(m.scrutinee().is_some());
    let arms = m.arms();
    assert_eq!(arms.len(), 2);
    assert!(arms[0].pattern().is_some());
    assert!(arms[0].body().is_some());
}

#[test]
fn ast_call_expr() {
    let sf = parse_source_file("module M =\n  let x = f 42\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Call(ref call) = ls.value().unwrap() else {
        panic!("expected call");
    };
    assert!(call.callee().is_some());
    assert!(call.arg().is_some());
}

#[test]
fn ast_field_expr() {
    let sf = parse_source_file("module M =\n  let x = r.field\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Field(ref fe) = ls.value().unwrap() else {
        panic!("expected field");
    };
    assert!(fe.base().is_some());
    let field_tok = fe.field_token().expect("should have field token");
    assert_eq!(field_tok.text(), "field");
}

#[test]
fn ast_field_expr_with_bracketed_operator_name() {
    assert_no_errors("module M =\n  let x = r.[ + ]\nend");
    let sf = parse_source_file("module M =\n  let x = r.[ + ]\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Field(ref fe) = ls.value().unwrap() else {
        panic!("expected field");
    };
    assert_eq!(
        fe.field_name_spanned().map(|name| name.inner).as_deref(),
        Some("[+]")
    );
}

#[test]
fn ast_path_expr() {
    let sf = parse_source_file("module M =\n  let x = Foo::bar\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Path(ref pe) = ls.value().unwrap() else {
        panic!("expected path");
    };
    assert_eq!(pe.segments().len(), 2);
    assert_eq!(pe.qualifier().as_deref(), Some("Foo"));
    assert_eq!(pe.name_text().as_deref(), Some("bar"));
}

#[test]
fn ast_path_expr_with_multiple_segments() {
    let sf = parse_source_file("module M =\n  let x = Foo::Bar::baz\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Path(ref pe) = ls.value().unwrap() else {
        panic!("expected path");
    };
    assert_eq!(pe.segments(), vec!["Foo", "Bar", "baz"]);
    assert_eq!(pe.qualifier().as_deref(), Some("Foo::Bar"));
    assert_eq!(pe.name_text().as_deref(), Some("baz"));
    assert!(!pe.is_rooted(), "unrooted path should not be rooted");
}

#[test]
fn ast_rooted_path_expr() {
    let sf = parse_source_file("module M =\n  let x = root::Foo::bar\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Path(ref pe) = ls.value().unwrap() else {
        panic!("expected path");
    };
    assert!(pe.is_rooted(), "root-prefixed path should be rooted");
    assert_eq!(pe.segments(), vec!["Foo", "bar"]);
    assert_eq!(pe.qualifier().as_deref(), Some("Foo"));
    assert_eq!(pe.name_text().as_deref(), Some("bar"));
}

#[test]
fn ast_bundle_rooted_path_expr() {
    let sf = parse_source_file("module M =\n  let x = bundle::Foo::bar\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Path(ref pe) = ls.value().unwrap() else {
        panic!("expected path");
    };
    assert!(
        pe.is_bundle_rooted(),
        "bundle-prefixed path should be bundle-rooted"
    );
    assert!(!pe.is_rooted(), "bundle-prefixed path should not be root::");
    assert_eq!(pe.segments(), vec!["Foo", "bar"]);
    assert_eq!(pe.qualifier().as_deref(), Some("Foo"));
    assert_eq!(pe.name_text().as_deref(), Some("bar"));
}

#[test]
fn ast_bracket_operator_ident_expr() {
    assert_no_errors("module M =\n  let x = [+]\nend");
    let sf = parse_source_file("module M =\n  let x = [+]\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Ident(ref ident) = ls.value().unwrap() else {
        panic!("expected ident");
    };
    assert_eq!(ident.name_text().as_deref(), Some("[+]"));
}

#[test]
fn ast_bracket_tilde_operator_ident_expr() {
    assert_no_errors("module M =\n  let x = [~]\nend");
    let sf = parse_source_file("module M =\n  let x = [~]\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Ident(ref ident) = ls.value().unwrap() else {
        panic!("expected ident");
    };
    assert_eq!(ident.name_text().as_deref(), Some("[~]"));
}

#[test]
fn ast_bracket_operator_path_expr() {
    assert_no_errors("module M =\n  let x = core::[+]\nend");
    let sf = parse_source_file("module M =\n  let x = core::[+]\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Path(ref pe) = ls.value().unwrap() else {
        panic!("expected path");
    };
    assert_eq!(pe.qualifier().as_deref(), Some("core"));
    assert_eq!(pe.name_text().as_deref(), Some("[+]"));
}

#[test]
fn ast_bracket_operator_name_is_canonicalized() {
    assert_no_errors("module M =\n  let [ + ] = fn a b => a + b\nend");
    let sf = parse_source_file("module M =\n  let [ + ] = fn a b => a + b\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Pattern::Ident(ref ident) = ls.pattern().unwrap() else {
        panic!("expected ident pattern");
    };
    assert_eq!(ident.name_text().as_deref(), Some("[+]"));
}

#[test]
fn ast_bracket_operator_module_name() {
    assert_no_errors("module [ + ] =\nend");
    let sf = parse_source_file("module [ + ] =\nend");
    assert_eq!(sf.modules()[0].name_text().as_deref(), Some("[+]"));
}

#[test]
fn ast_simple_ident() {
    let sf = parse_source_file("module M =\n  let x = foo\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Ident(ref ident) = ls.value().unwrap() else {
        panic!("expected ident");
    };
    assert_eq!(ident.name_text().as_deref(), Some("foo"));
}

#[test]
fn ast_let_in_expr() {
    let sf = parse_source_file("module M =\n  let x = let y = 1 in y + 1\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Let(ref le) = ls.value().unwrap() else {
        panic!("expected let expr");
    };
    assert!(le.pattern().is_some());
    assert!(le.value().is_some());
    assert!(le.body().is_some());
}

#[test]
fn ast_unit_expr() {
    let sf = parse_source_file("module M =\n  let x = ()\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    assert!(
        matches!(ls.value().unwrap(), ast::Expr::Unit(_)),
        "should be unit"
    );
}

#[test]
fn ast_paren_expr_tuple() {
    let sf = parse_source_file("module M =\n  let x = (1, 2, 3)\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Paren(ref pe) = ls.value().unwrap() else {
        panic!("expected paren");
    };
    assert!(pe.is_tuple(), "should be tuple");
    assert_eq!(pe.inner_exprs().len(), 3);
}

#[test]
fn ast_paren_expr_singleton_tuple_with_trailing_comma() {
    let sf = parse_source_file("module M =\n  let x = (1,)\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Paren(ref pe) = ls.value().unwrap() else {
        panic!("expected paren");
    };
    assert!(pe.is_tuple(), "should be singleton tuple");
    assert_eq!(pe.inner_exprs().len(), 1);
}

#[test]
fn ast_struct_expr() {
    let sf = parse_source_file("module M =\n  let p = { x = 1, y = 2 }\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Struct(ref se) = ls.value().unwrap() else {
        panic!("expected struct");
    };
    let fields = se.fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name_text().as_deref(), Some("x"));
    assert!(fields[0].value().is_some());
    assert_eq!(fields[1].name_text().as_deref(), Some("y"));
}

#[test]
fn ast_array_expr() {
    // Use comma-separated elements to avoid juxtaposition being
    // parsed as function application.
    let sf = parse_source_file("module M =\n  let xs = [1, 2, 3]\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Array(ref ae) = ls.value().unwrap() else {
        panic!("expected array");
    };
    assert_eq!(ae.exprs().len(), 3);
}

#[test]
fn ast_pattern_type_hint() {
    let sf = parse_source_file("module M =\n  let (x: int) = 42\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let pat = ls.pattern().expect("should have pattern");
    // The outer pattern should be a tuple wrapping a type hint
    // Actually: `(x: int)` parses as PAT_TUPLE containing PAT_TYPE_HINT
    match pat {
        ast::Pattern::Tuple(ref tup) => {
            let inner = &tup.patterns()[0];
            assert!(
                matches!(inner, ast::Pattern::TypeHint(_)),
                "inner should be type hint, got: {inner:?}"
            );
            if let ast::Pattern::TypeHint(th) = inner {
                assert!(th.pattern().is_some());
                assert!(th.ty().is_some());
            }
        }
        ast::Pattern::TypeHint(ref th) => {
            assert!(th.pattern().is_some());
            assert!(th.ty().is_some());
        }
        other => panic!("expected tuple or type hint, got: {other:?}"),
    }
}

#[test]
fn ast_constructor_pattern_payload() {
    let sf = parse_source_file(
        "module M =\n  let x = match opt with\n    | Some value => value\n    | None => 0\nend",
    );
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Expr::Match(ref match_expr) = ls.value().unwrap() else {
        panic!("expected match");
    };
    let first_arm = &match_expr.arms()[0];
    let ast::Pattern::Constructor(constructor) = first_arm.pattern().unwrap() else {
        panic!("expected constructor pattern");
    };
    let ast::PathOrIdent::Ident(head) = constructor.head().unwrap() else {
        panic!("expected constructor head identifier");
    };
    assert_eq!(head.name_text().as_deref(), Some("Some"));
    let ast::Pattern::Ident(payload) = constructor.payload().unwrap() else {
        panic!("expected identifier payload");
    };
    assert_eq!(payload.name_text().as_deref(), Some("value"));
}

#[test]
fn ast_function_type() {
    let sf = parse_source_file("module M =\n  type f = int -> int\nend");
    let ast::Statement::Type(ref ts) = sf.modules()[0].statements()[0] else {
        panic!("expected type");
    };
    let ast::TypeDef::Alias(ref alias) = ts.type_def().unwrap() else {
        panic!("expected alias");
    };
    let te = alias.type_expr().unwrap();
    let ast::TypeExpr::Function(ref ft) = te else {
        panic!("expected function type, got: {te:?}");
    };
    assert!(ft.param_type().is_some());
    assert!(ft.return_type().is_some());
}

#[test]
fn ast_type_application() {
    let sf = parse_source_file("module M =\n  type xs = list int\nend");
    let ast::Statement::Type(ref ts) = sf.modules()[0].statements()[0] else {
        panic!("expected type");
    };
    let ast::TypeDef::Alias(ref alias) = ts.type_def().unwrap() else {
        panic!("expected alias");
    };
    let te = alias.type_expr().unwrap();
    let ast::TypeExpr::Application(ref app) = te else {
        panic!("expected type application, got: {te:?}");
    };
    assert!(app.base().is_some());
    assert_eq!(app.args().len(), 1);
}

#[test]
fn ast_type_application_multiple_args() {
    let sf = parse_source_file("module M =\n  type xs = map int string\nend");
    let ast::Statement::Type(ref ts) = sf.modules()[0].statements()[0] else {
        panic!("expected type");
    };
    let ast::TypeDef::Alias(ref alias) = ts.type_def().unwrap() else {
        panic!("expected alias");
    };
    let te = alias.type_expr().unwrap();
    let ast::TypeExpr::Application(ref app) = te else {
        panic!("expected type application, got: {te:?}");
    };
    assert!(app.base().is_some());
    assert_eq!(app.args().len(), 2);
}

#[test]
fn ast_type_singleton_tuple_with_trailing_comma() {
    let sf = parse_source_file("module M =\n  type one = (int,)\nend");
    let ast::Statement::Type(ref ts) = sf.modules()[0].statements()[0] else {
        panic!("expected type");
    };
    let ast::TypeDef::Alias(ref alias) = ts.type_def().unwrap() else {
        panic!("expected alias");
    };
    let te = alias.type_expr().unwrap();
    let ast::TypeExpr::Tuple(ref tuple) = te else {
        panic!("expected tuple type, got: {te:?}");
    };
    assert!(tuple.is_tuple(), "should be singleton tuple type");
    assert_eq!(tuple.fields().len(), 1);
}

#[test]
fn ast_missing_children_return_none() {
    // Parse something with errors — accessors should return None
    // for missing parts rather than panicking
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", "module M = let = end");
    let tree = parse("module M = let = end", &mut file_logger).expect("root should be SourceFile");
    assert!(!file_logger.is_ok(), "Should have errors");
    let m = &tree.modules()[0];
    // We should be able to navigate without panicking
    let stmts = m.statements();
    // The let statement may have None for its value due to errors
    if let Some(ast::Statement::Let(ls)) = stmts.first() {
        // These should not panic even with parse errors
        let _ = ls.pattern();
        let _ = ls.value();
    }
}

#[test]
fn ast_cast_wrong_kind_returns_none() {
    let sf = parse_source_file("module M = end");
    // Try to cast a MODULE node as a LetStatement — should fail
    let m = &sf.modules()[0];
    let result = ast::LetStatement::cast(m.syntax().clone());
    assert!(
        result.is_none(),
        "casting MODULE as LetStatement should be None"
    );
}

#[test]
fn ast_expr_enum_dispatch() {
    let sf = parse_source_file(
        "module M =\n  let a = 1\n  let b = fn x => x\n  let c = if true then 1 else 0\nend",
    );
    let stmts = sf.modules()[0].statements();
    let values: Vec<ast::Expr> = stmts
        .iter()
        .filter_map(|s| {
            match s {
                ast::Statement::Let(ls) => ls.value(),
                _ => None,
            }
        })
        .collect();
    assert_eq!(values.len(), 3);
    assert!(matches!(values[0], ast::Expr::Literal(_)));
    assert!(matches!(values[1], ast::Expr::Fn(_)));
    assert!(matches!(values[2], ast::Expr::If(_)));
}

#[test]
fn ast_pattern_enum_dispatch() {
    let sf =
        parse_source_file("module M =\n  let x = 1\n  let (a, b) = foo\n  let [h, ..t] = xs\nend");
    let stmts = sf.modules()[0].statements();
    let patterns: Vec<ast::Pattern> = stmts
        .iter()
        .filter_map(|s| {
            match s {
                ast::Statement::Let(ls) => ls.pattern(),
                _ => None,
            }
        })
        .collect();
    assert_eq!(patterns.len(), 3);
    assert!(matches!(patterns[0], ast::Pattern::Ident(_)));
    assert!(matches!(patterns[1], ast::Pattern::Tuple(_)));
    assert!(matches!(patterns[2], ast::Pattern::Array(_)));
}

#[test]
fn ast_pattern_singleton_tuple_with_trailing_comma() {
    let sf = parse_source_file("module M =\n  let (x,) = (1,)\nend");
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    let ast::Pattern::Tuple(ref tuple) = ls.pattern().expect("should have pattern") else {
        panic!("expected tuple pattern");
    };
    assert!(tuple.is_tuple(), "should be singleton tuple pattern");
    assert_eq!(tuple.patterns().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Leading comment attachment tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn leading_comment_attached_to_let() {
    let src = "module M =\n  -- a comment\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    assert_eq!(stmts.len(), 1);
    let comments = stmts[0].leading_comments();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text().trim_end(), "-- a comment");
}

#[test]
fn leading_comment_attached_to_type() {
    let src = "module M =\n  -- type docs\n  type t = int\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    assert_eq!(stmts.len(), 1);
    let comments = stmts[0].leading_comments();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text().trim_end(), "-- type docs");
}

#[test]
fn multiple_leading_comments() {
    let src = "module M =\n  -- line 1\n  -- line 2\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let comments = stmts[0].leading_comments();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].text().trim_end(), "-- line 1");
    assert_eq!(comments[1].text().trim_end(), "-- line 2");
}

#[test]
fn blank_line_separates_comment_from_statement() {
    let src = "module M =\n  -- detached\n\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let comments = stmts[0].leading_comments();
    assert_eq!(
        comments.len(),
        0,
        "Comment separated by blank line should not attach"
    );
}

#[test]
fn blank_line_with_spaces_separates_comment_from_statement() {
    let src = "module M =\n  -- detached\n  \n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let comments = stmts[0].leading_comments();
    assert_eq!(
        comments.len(),
        0,
        "Comment separated by blank line with whitespace should not attach"
    );
}

#[test]
fn comment_before_blank_line_detaches_but_after_attaches() {
    let src = "module M =\n  -- detached\n\n  -- attached\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let comments = stmts[0].leading_comments();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text().trim_end(), "-- attached");
}

#[test]
fn no_comments_means_empty() {
    let src = "module M =\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let comments = stmts[0].leading_comments();
    assert!(comments.is_empty());
}

#[test]
fn each_statement_gets_own_comments() {
    let src = "module M =\n  -- about x\n  let x = 1\n  -- about y\n  let y = 2\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    assert_eq!(stmts.len(), 2);
    let c0 = stmts[0].leading_comments();
    let c1 = stmts[1].leading_comments();
    assert_eq!(c0.len(), 1);
    assert_eq!(c0[0].text().trim_end(), "-- about x");
    assert_eq!(c1.len(), 1);
    assert_eq!(c1[0].text().trim_end(), "-- about y");
}

#[test]
fn block_comment_attaches() {
    let src = "module M =\n  (* block *)\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let comments = stmts[0].leading_comments();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].kind(), SyntaxKind::BLOCK_COMMENT);
}

#[test]
fn leading_comments_round_trip() {
    let src = "module M =\n  -- a\n  -- b\n  let x = 1\nend\n";
    assert_round_trip(src);
}

// ═══════════════════════════════════════════════════════════════════════
// Operator precedence tests
// ═══════════════════════════════════════════════════════════════════════

/// Helper: parse a let-value expression and return it.
fn parse_expr(expr_src: &str) -> ast::Expr {
    let src = format!("module M =\n  let x = {expr_src}\nend");
    let sf = parse_source_file(&src);
    let ast::Statement::Let(ref ls) = sf.modules()[0].statements()[0] else {
        panic!("expected let");
    };
    ls.value().expect("should have value")
}

/// Helper: assert the root expression is a BinaryExpr with the given operator.
fn assert_root_op(
    expr_src: &str,
    expected_op: &str,
) {
    let expr = parse_expr(expr_src);
    let ast::Expr::Binary(ref bin) = expr else {
        panic!("expected binary expr for `{expr_src}`, got: {expr:?}");
    };
    let op = bin.op_token().expect("should have op");
    assert_eq!(
        op.text(),
        expected_op,
        "for `{expr_src}`: root op should be `{expected_op}`, got `{}`",
        op.text()
    );
}

// ── Multiplicative binds tighter than additive ───────────────────────

#[test]
fn precedence_mul_before_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3), root op = +
    assert_root_op("1 + 2 * 3", "+");
}

#[test]
fn precedence_div_before_sub() {
    assert_root_op("1 - 2 / 3", "-");
}

#[test]
fn precedence_mod_before_add() {
    assert_root_op("1 + 2 % 3", "+");
}

// ── Additive binds tighter than comparison ───────────────────────────

#[test]
fn precedence_add_before_eq() {
    // 1 + 2 == 3 should parse as (1 + 2) == 3, root op = ==
    assert_root_op("1 + 2 == 3", "==");
}

#[test]
fn precedence_sub_before_less() {
    assert_root_op("a - b < c", "<");
}

#[test]
fn precedence_add_before_neq() {
    assert_root_op("1 + 2 != 3", "!=");
}

// ── Comparison binds tighter than logical operators ──────────────────
// BP ordering: or(6) < and(8) < comparison(10)

#[test]
fn precedence_comparison_before_and() {
    // a == b and c == d => (a == b) and (c == d), root = and
    assert_root_op("a == b and c == d", "and");
}

#[test]
fn precedence_comparison_before_or() {
    // a < b or c > d => (a < b) or (c > d), root = or
    assert_root_op("a < b or c > d", "or");
}

// ── AND binds tighter than OR (standard boolean algebra) ─────────────

#[test]
fn precedence_and_above_or() {
    // a or b and c or d => a or (b and c) or d, root = or
    assert_root_op("a or b and c or d", "or");
}

#[test]
fn precedence_and_above_or_complex() {
    // a or b and c => a or (b and c), root = or
    assert_root_op("a or b and c", "or");
}

// ── Semicolon is lowest precedence ───────────────────────────────────

#[test]
fn precedence_semicolon_lowest() {
    assert_root_op("a + b ; c * d", ";");
}

#[test]
fn precedence_semicolon_below_and() {
    assert_root_op("a and b ; c or d", ";");
}

// ── Left associativity ──────────────────────────────────────────────

#[test]
fn left_assoc_addition() {
    // 1 + 2 + 3 should parse as (1 + 2) + 3
    // root = +, root.lhs = binary(+), root.rhs = 3
    let expr = parse_expr("1 + 2 + 3");
    let ast::Expr::Binary(ref bin) = expr else {
        panic!("expected binary");
    };
    assert_eq!(bin.op_token().unwrap().text(), "+");
    // lhs should also be a binary +
    let ast::Expr::Binary(ref lhs) = bin.lhs().unwrap() else {
        panic!("lhs should be binary");
    };
    assert_eq!(lhs.op_token().unwrap().text(), "+");
    // rhs should be a literal 3
    assert!(matches!(bin.rhs().unwrap(), ast::Expr::Literal(_)));
}

#[test]
fn left_assoc_multiplication() {
    // 2 * 3 * 4 => (2 * 3) * 4
    let expr = parse_expr("2 * 3 * 4");
    let ast::Expr::Binary(ref bin) = expr else {
        panic!("expected binary");
    };
    assert_eq!(bin.op_token().unwrap().text(), "*");
    assert!(matches!(bin.lhs().unwrap(), ast::Expr::Binary(_)));
    assert!(matches!(bin.rhs().unwrap(), ast::Expr::Literal(_)));
}

#[test]
fn left_assoc_mixed_add_sub() {
    // 1 - 2 + 3 => (1 - 2) + 3
    let expr = parse_expr("1 - 2 + 3");
    let ast::Expr::Binary(ref bin) = expr else {
        panic!("expected binary");
    };
    assert_eq!(bin.op_token().unwrap().text(), "+");
    let ast::Expr::Binary(ref lhs) = bin.lhs().unwrap() else {
        panic!("lhs should be binary");
    };
    assert_eq!(lhs.op_token().unwrap().text(), "-");
}

// ── Function application vs arithmetic ──────────────────────────────
// Call BP=24 is above additive BP=14 and multiplicative BP=16,
// so application binds tighter than arithmetic (standard ML).

#[test]
fn precedence_call_above_add() {
    // f x + g y => (f x) + (g y), root = +
    assert_root_op("f x + g y", "+");
}

#[test]
fn precedence_call_above_mul() {
    // f x * g y => (f x) * (g y), root = *
    assert_root_op("f x * g y", "*");
}

#[test]
fn precedence_call_above_comparison() {
    // f x == g y => (f x) == (g y), root = ==
    assert_root_op("f x == g y", "==");
}

// ── Unary minus vs binary operators ─────────────────────────────────
// Unary prefix right BP=20, above multiplicative BP=16.

#[test]
fn precedence_unary_minus_above_add() {
    // -a + b => (-a) + b, root = +
    assert_root_op("-a + b", "+");
}

#[test]
fn precedence_unary_minus_above_mul() {
    // -a * b => (-a) * b, root = *
    // Unary right BP=20 > mul left BP=16, so mul wins at the infix
    // level after the unary operand is consumed.
    assert_root_op("-a * b", "*");
}

// ── Field access binds tightest ─────────────────────────────────────

#[test]
fn precedence_field_above_call() {
    // f r.x parses as f (r.x) — a call whose arg is a field expr
    let expr = parse_expr("f r.x");
    let ast::Expr::Call(ref call) = expr else {
        panic!("expected call, got: {expr:?}");
    };
    let arg = call.arg().unwrap();
    assert!(
        matches!(arg, ast::Expr::Field(_)),
        "arg should be field expr, got: {arg:?}"
    );
}

#[test]
fn precedence_field_above_arithmetic() {
    // a.x + b.y => (a.x) + (b.y), root = +
    assert_root_op("a.x + b.y", "+");
}

// ── Pipe operator precedence ────────────────────────────────────────

#[test]
fn precedence_pipe_below_addition() {
    // a + b |> f => (a + b) |> f, root = |>
    assert_root_op("a + b |> f", "|>");
}

#[test]
fn precedence_pipe_below_or() {
    // a or b |> f => (a or b) |> f, root = |>
    assert_root_op("a or b |> f", "|>");
}

#[test]
fn precedence_pipe_above_semicolon() {
    // a |> f ; b => (a |> f) ; b, root = ;
    assert_root_op("a |> f ; b", ";");
}

// ── Composition precedence ──────────────────────────────────────────

#[test]
fn precedence_compose_above_comparison() {
    // f << g == h => (f << g) == h, root = ==
    assert_root_op("f << g == h", "==");
}

#[test]
fn precedence_compose_below_addition() {
    // a + b << c + d => (a + b) << (c + d), root = <<
    assert_root_op("a + b << c + d", "<<");
}

// ── Complex mixed expressions ───────────────────────────────────────

#[test]
fn precedence_complex_mixed() {
    // a + b * c == d and e => root = and
    // because: a + (b * c) == d, then (a + (b * c) == d) and e
    assert_root_op("a + b * c == d and e", "and");
}

#[test]
fn precedence_complex_chain() {
    // a * b + c * d => (a * b) + (c * d), root = +
    assert_root_op("a * b + c * d", "+");
}

// ── Round-trip for precedence expressions ───────────────────────────

#[test]
fn round_trip_precedence_exprs() {
    assert_round_trip(
        "module M =\n  let x = 1 + 2 * 3\n  let y = a == b and c != d\n  let z = f x + g y\nend\n",
    );
}

#[test]
fn leading_comment_text_concatenated() {
    let src = "module M =\n  -- line 1\n  -- line 2\n  let x = 1\nend";
    let sf = parse_source_file(src);
    let stmts = sf.modules()[0].statements();
    let text = stmts[0].leading_comment_text();
    assert_eq!(text, "-- line 1\n-- line 2");
}

// ═══════════════════════════════════════════════════════════════════════
// Forall type annotation tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_forall_type_annotation() {
    let tree_str = parse_to_string("module M =\n  let (id: for a in a -> a) = fn x => x\nend");
    assert!(
        tree_str.contains("FORALL_TYPE"),
        "Should contain FORALL_TYPE"
    );
}

#[test]
fn round_trip_forall_type_annotation() {
    assert_round_trip("module M =\n  let (id: for a in a -> a) = fn x => x\nend\n");
}

#[test]
fn round_trip_forall_type_annotation_multi_params() {
    assert_round_trip("module M =\n  let (f: for a b in a -> b -> a) = fn x y => x\nend\n");
}

#[test]
fn parse_forall_type_annotation_no_errors() {
    assert_no_errors("module M =\n  let (id: for a in a -> a) = fn x => x\nend");
}

#[test]
fn ast_forall_type_accessors() {
    let sf = parse_source_file("module M =\n  type f = for a b in a -> b where Eq a, Show b\nend");
    let ast::Statement::Type(ref ts) = sf.modules()[0].statements()[0] else {
        panic!("expected type statement");
    };
    let ast::TypeDef::Alias(ref alias) = ts.type_def().unwrap() else {
        panic!("expected alias");
    };
    let te = alias.type_expr().unwrap();
    let ast::TypeExpr::ForAll(ref fa) = te else {
        panic!("expected forall type, got: {te:?}");
    };
    let params = fa.params();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name_text().as_deref(), Some("a"));
    assert_eq!(params[1].name_text().as_deref(), Some("b"));
    assert_eq!(fa.constraints().len(), 2);
    assert!(fa.body().is_some(), "should have body type");
}
