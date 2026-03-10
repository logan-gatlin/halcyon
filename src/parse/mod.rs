pub mod ast;
mod grammar;
pub mod lexer;
mod parser;

use self::ast::{
    AstNode,
    SourceFile,
};
use crate::FileLogger;

pub use lexer::tokenize;

/// All syntax kinds used in the lossless CST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    L_PAREN = 0,
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
    ARROW,
    DOUBLE_ARROW,
    PIPE_ARROW,
    COMPOSE_LEFT,
    COMPOSE_RIGHT,
    PIPE,
    DOLLAR,
    HASH,
    AT,
    TILDE,

    BANG_EQUAL,
    EQUAL,
    DOUBLE_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    IDENT,
    STRING,
    GLYPH,
    INTEGER,
    REAL,

    MODULE_KW,
    IMPORT_KW,
    USE_KW,
    AS_KW,
    END_KW,
    MATCH_KW,
    WITH_KW,
    LET_KW,
    TYPE_KW,
    TRAIT_KW,
    IMPL_KW,
    DO_KW,
    OF_KW,
    IN_KW,
    IF_KW,
    THEN_KW,
    ELSE_KW,
    AND_KW,
    OR_KW,
    XOR_KW,
    NOT_KW,
    TRUE_KW,
    FALSE_KW,
    FN_KW,
    WASM_KW,
    FOR_KW,
    WHERE_KW,
    ROOT_KW,

    WHITESPACE,
    LINE_COMMENT,
    BLOCK_COMMENT,

    TOKEN_ERROR,

    // Note: do not introduce new node kinds that duplicate existing token kinds.
    // The parser may reuse token kinds (IDENT, STRING, INTEGER, REAL) for nodes.
    SOURCE_FILE,
    MODULE,
    IMPORT_STATEMENT,
    USE_STATEMENT,

    LET_STATEMENT,
    TYPE_STATEMENT,
    TRAIT_STATEMENT,
    IMPL_STATEMENT,
    WASM_STATEMENT,

    TRAIT_METHOD_DECL,
    IMPL_METHOD_DEF,

    STRUCT_DEF,
    SUM_DEF,
    VARIANT,
    FIELD_DECL,
    STRUCT_SPREAD_DECL,
    TYPE_ALIAS_DEF,

    FUNCTION_TYPE,
    TYPE_APPLICATION,
    TUPLE_TYPE,
    ARRAY_TYPE,
    FORALL_TYPE,
    TYPE_CONSTRAINT,

    UNIT,

    LET_EXPR,
    USE_EXPR,
    FN_EXPR,
    FN_SHORTHAND_EXPR,
    IF_EXPR,
    MATCH_EXPR,
    MATCH_ARM,
    PARAM,
    BINARY_EXPR,
    UNARY_EXPR,
    CALL_EXPR,
    FIELD_EXPR,
    ARRAY_EXPR,
    STRUCT_EXPR,
    STRUCT_FIELD,
    LITERAL,
    IDENT_NODE,
    PATH,
    PAREN_EXPR,
    ARRAY_SPLAT,
    INLINE_WASM_EXPR,

    SEXPR,
    SEXPR_FIELD,

    PAT_TUPLE,
    PAT_ARRAY,
    PAT_STRUCT,
    PAT_CONSTRUCTOR,
    PAT_TYPE_HINT,
    PAT_REST,
    PAT_FIELD,

    ERROR,
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::WHITESPACE | Self::LINE_COMMENT | Self::BLOCK_COMMENT
        )
    }

    pub fn is_operator_token(self) -> bool {
        matches!(
            self,
            Self::PLUS
                | Self::MINUS
                | Self::STAR
                | Self::SLASH
                | Self::PERCENT
                | Self::PIPE_ARROW
                | Self::COMPOSE_LEFT
                | Self::COMPOSE_RIGHT
                | Self::DOUBLE_EQUAL
                | Self::BANG_EQUAL
                | Self::LESS
                | Self::LESS_EQUAL
                | Self::GREATER
                | Self::GREATER_EQUAL
                | Self::AND_KW
                | Self::OR_KW
                | Self::XOR_KW
                | Self::NOT_KW
                | Self::TILDE
                | Self::SEMICOLON
        )
    }
}

impl std::fmt::Display for SyntaxKind {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        use SyntaxKind::*;
        write!(
            f,
            "{}",
            match self {
                L_PAREN => "(",
                R_PAREN => ")",
                L_BRACE => "{",
                R_BRACE => "}",
                L_SQUARE => "[",
                R_SQUARE => "]",
                COMMA => ",",
                COLON => ":",
                DOLLAR => "$",
                AT => "@",
                HASH => "#",
                TILDE => "~",
                DOUBLE_COLON => "::",
                SEMICOLON => ";",
                DOT => ".",
                DOT_DOT => "..",
                PLUS => "+",
                MINUS => "-",
                SLASH => "/",
                STAR => "*",
                PERCENT => "%",
                PIPE_ARROW => "|>",
                COMPOSE_LEFT => "<<",
                COMPOSE_RIGHT => ">>",
                ARROW => "->",
                DOUBLE_ARROW => "=>",
                BANG_EQUAL => "!=",
                EQUAL => "=",
                DOUBLE_EQUAL => "==",
                GREATER => ">",
                GREATER_EQUAL => ">=",
                LESS => "<",
                LESS_EQUAL => "<=",
                PIPE => "|",
                IDENT => "identifier",
                STRING => "string",
                GLYPH => "glyph",
                INTEGER => "integer",
                REAL => "real",
                MODULE_KW => "module",
                IMPORT_KW => "import",
                USE_KW => "use",
                AS_KW => "as",
                END_KW => "end",
                MATCH_KW => "match",
                WITH_KW => "with",
                LET_KW => "let",
                TYPE_KW => "type",
                TRAIT_KW => "trait",
                IMPL_KW => "impl",
                DO_KW => "do",
                OF_KW => "of",
                IN_KW => "in",
                IF_KW => "if",
                THEN_KW => "then",
                ELSE_KW => "else",
                AND_KW => "and",
                OR_KW => "or",
                XOR_KW => "xor",
                NOT_KW => "not",
                TRUE_KW => "true",
                FALSE_KW => "false",
                FN_KW => "fn",
                WASM_KW => "wasm",
                FOR_KW => "for",
                WHERE_KW => "where",
                ROOT_KW => "root",
                WHITESPACE => "whitespace",
                LINE_COMMENT => "line comment",
                BLOCK_COMMENT => "block comment",
                TOKEN_ERROR => "[ERROR]",
                SOURCE_FILE => "source file",
                MODULE => "module",
                IMPORT_STATEMENT => "import statement",
                USE_STATEMENT => "use statement",
                LET_STATEMENT => "let statement",
                TYPE_STATEMENT => "type statement",
                TRAIT_STATEMENT => "trait statement",
                IMPL_STATEMENT => "impl statement",
                WASM_STATEMENT => "wasm statement",
                TRAIT_METHOD_DECL => "trait method declaration",
                IMPL_METHOD_DEF => "impl method definition",
                STRUCT_DEF => "struct definition",
                SUM_DEF => "sum definition",
                VARIANT => "variant",
                FIELD_DECL => "field declaration",
                STRUCT_SPREAD_DECL => "struct spread declaration",
                TYPE_ALIAS_DEF => "type alias",
                FUNCTION_TYPE => "function type",
                TYPE_APPLICATION => "type application",
                TUPLE_TYPE => "tuple type",
                ARRAY_TYPE => "array type",
                FORALL_TYPE => "forall type",
                TYPE_CONSTRAINT => "type constraint",
                UNIT => "()",
                LET_EXPR => "let expression",
                USE_EXPR => "use expression",
                FN_EXPR => "function expression",
                FN_SHORTHAND_EXPR => "function shorthand",
                IF_EXPR => "if expression",
                MATCH_EXPR => "match expression",
                MATCH_ARM => "match arm",
                PARAM => "parameter",
                BINARY_EXPR => "binary expression",
                UNARY_EXPR => "unary expression",
                CALL_EXPR => "call expression",
                FIELD_EXPR => "field access",
                ARRAY_EXPR => "array expression",
                STRUCT_EXPR => "struct expression",
                STRUCT_FIELD => "struct field",
                LITERAL => "literal",
                IDENT_NODE => "identifier",
                PATH => "path",
                PAREN_EXPR => "parenthesized expression",
                ARRAY_SPLAT => "array splat",
                INLINE_WASM_EXPR => "inline wasm expression",
                SEXPR => "s-expression",
                SEXPR_FIELD => "s-expression field",
                PAT_TUPLE => "tuple pattern",
                PAT_ARRAY => "array pattern",
                PAT_STRUCT => "struct pattern",
                PAT_CONSTRUCTOR => "constructor pattern",
                PAT_TYPE_HINT => "type hint pattern",
                PAT_REST => "rest pattern",
                PAT_FIELD => "pattern field",
                ERROR => "error",
            }
        )
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HalcyonLanguage {}

impl rowan::Language for HalcyonLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        assert!(
            raw.0 <= SyntaxKind::ERROR as u16,
            "SyntaxKind out of range: {}",
            raw.0
        );
        unsafe { std::mem::transmute(raw.0) }
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

pub type SyntaxNode = rowan::SyntaxNode<HalcyonLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<HalcyonLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<HalcyonLanguage>;

pub fn parse(
    source: &str,
    logger: &mut FileLogger,
) -> Option<SourceFile> {
    let tokens = lexer::tokenize(source.chars(), logger);
    let mut p = parser::Parser::new(&tokens, source, logger);
    grammar::source_file(&mut p);
    SourceFile::cast(p.finish())
}

#[cfg(test)]
mod test;
