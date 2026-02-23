pub mod ast;
mod grammar;
mod parser;

use self::ast::{
    AstNode,
    SourceFile,
};
use crate::token::{
    self,
    TokenKind,
};

/// All syntax kinds used in the lossless CST.
///
/// Token kinds (leaves) map 1:1 from the existing tokenizer.
/// Node kinds (interior) represent composite grammar constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    // ── Tokens (leaves) ──────────────────────────────────────────────
    // Delimiters
    L_PAREN = 0,
    R_PAREN,
    L_BRACE,
    R_BRACE,
    L_SQUARE,
    R_SQUARE,

    // Separators
    COMMA,
    COLON,
    DOUBLE_COLON,
    SEMICOLON,

    // Operators
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

    // Comparison
    BANG_EQUAL,
    EQUAL,
    DOUBLE_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals
    IDENT,
    STRING,
    GLYPH,
    INTEGER,
    REAL,

    // Keywords
    MODULE_KW,
    IMPORT_KW,
    USE_KW,
    END_KW,
    MATCH_KW,
    WITH_KW,
    LET_KW,
    TYPE_KW,
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

    // Trivia
    WHITESPACE,
    LINE_COMMENT,
    BLOCK_COMMENT,

    // Lexer error
    TOKEN_ERROR,

    // ── Nodes (interior) ─────────────────────────────────────────────
    SOURCE_FILE,
    MODULE,

    // Statements
    LET_STATEMENT,
    TYPE_STATEMENT,

    // Type definitions
    STRUCT_DEF,
    SUM_DEF,
    VARIANT,
    FIELD_DECL,
    TYPE_ALIAS_DEF,

    // Type expressions
    FUNCTION_TYPE,
    TYPE_APPLICATION,
    TUPLE_TYPE,
    ARRAY_TYPE,

    // Shared across expressions, patterns, and type expressions
    UNIT,

    // Value expressions (some nodes shared elsewhere)
    LET_EXPR,
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
    OPERATOR_EXPR,

    // Patterns
    PAT_TUPLE,
    PAT_ARRAY,
    PAT_STRUCT,
    PAT_CONSTRUCTOR,
    PAT_TYPE_HINT,
    PAT_REST,
    PAT_FIELD,

    // Error recovery
    ERROR,
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::WHITESPACE | Self::LINE_COMMENT | Self::BLOCK_COMMENT
        )
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// Convert a `TokenKind` from the existing tokenizer into a `SyntaxKind`.
impl From<&TokenKind> for SyntaxKind {
    fn from(tk: &TokenKind) -> Self {
        use TokenKind::*;
        match tk {
            LeftParen => Self::L_PAREN,
            RightParen => Self::R_PAREN,
            LeftBrace => Self::L_BRACE,
            RightBrace => Self::R_BRACE,
            LeftSquare => Self::L_SQUARE,
            RightSquare => Self::R_SQUARE,
            Comma => Self::COMMA,
            Colon => Self::COLON,
            DoubleColon => Self::DOUBLE_COLON,
            Semicolon => Self::SEMICOLON,
            Dot => Self::DOT,
            DotDot => Self::DOT_DOT,
            Plus => Self::PLUS,
            Minus => Self::MINUS,
            Slash => Self::SLASH,
            Star => Self::STAR,
            Percent => Self::PERCENT,
            Arrow => Self::ARROW,
            DoubleArrow => Self::DOUBLE_ARROW,
            Apply => Self::PIPE_ARROW,
            ComposeLeft => Self::COMPOSE_LEFT,
            ComposeRight => Self::COMPOSE_RIGHT,
            Pipe => Self::PIPE,
            BangEqual => Self::BANG_EQUAL,
            Equal => Self::EQUAL,
            DoubleEqual => Self::DOUBLE_EQUAL,
            Greater => Self::GREATER,
            GreaterEqual => Self::GREATER_EQUAL,
            Less => Self::LESS,
            LessEqual => Self::LESS_EQUAL,
            Identifier(_) => Self::IDENT,
            StringLiteral(_) => Self::STRING,
            GlyphLiteral(_) => Self::GLYPH,
            IntegerLiteral(..) => Self::INTEGER,
            RealLiteral(_) => Self::REAL,
            Module => Self::MODULE_KW,
            Import => Self::IMPORT_KW,
            Use => Self::USE_KW,
            End => Self::END_KW,
            Match => Self::MATCH_KW,
            With => Self::WITH_KW,
            Let => Self::LET_KW,
            Type => Self::TYPE_KW,
            Do => Self::DO_KW,
            Of => Self::OF_KW,
            In => Self::IN_KW,
            If => Self::IF_KW,
            Then => Self::THEN_KW,
            Else => Self::ELSE_KW,
            And => Self::AND_KW,
            Or => Self::OR_KW,
            Xor => Self::XOR_KW,
            Not => Self::NOT_KW,
            True => Self::TRUE_KW,
            False => Self::FALSE_KW,
            Fn => Self::FN_KW,
            Whitespace => Self::WHITESPACE,
            LineComment(_) => Self::LINE_COMMENT,
            BlockComment(_) => Self::BLOCK_COMMENT,
            Error => Self::TOKEN_ERROR,
        }
    }
}

// ── Rowan language definition ────────────────────────────────────────

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
        // SAFETY: SyntaxKind is repr(u16) and we checked the range.
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
    logger: &mut crate::FileLogger,
) -> Option<SourceFile> {
    let tokens = token::tokenize(source.chars(), logger);
    let mut p = parser::Parser::new(&tokens, source, logger);
    grammar::source_file(&mut p);
    SourceFile::cast(p.finish())
}

#[cfg(test)]
mod test;
