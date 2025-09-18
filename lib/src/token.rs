use std::io::Write;

use crate::Logger;
use crate::error;
use crate::lint::*;
use multipeek::{MultiPeek, multipeek};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sx::SXRepr)]
pub enum Base {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hex = 16,
}

impl Base {
    pub fn prefix(&self) -> &'static str {
        match self {
            Base::Binary => "0b",
            Base::Octal => "0o",
            Base::Decimal => "",
            Base::Hex => "0x",
        }
    }
}

impl std::fmt::Display for Base {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Base::Binary => write!(f, "binary"),
            Base::Octal => write!(f, "octal"),
            Base::Decimal => write!(f, "decimal"),
            Base::Hex => write!(f, "hex"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquare,
    RightSquare,

    Comma,
    Colon,
    DoubleColon,
    Semicolon,

    Dot,
    DotDot,
    Plus,
    PlusDot,
    Minus,
    MinusDot,
    Slash,
    SlashDot,
    Star,
    StarDot,
    Percent,
    Arrow,
    FatArrow,
    Apply,
    ComposeLeft,
    ComposeRight,

    BangEqual,
    Equal,
    DoubleEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Pipe,

    Identifier(String),
    StringLiteral(String),
    GlyphLiteral(char),
    IntegerLiteral(String, Base),
    RealLiteral(String),

    Module,
    Import,
    Use,
    End,
    Match,
    With,
    Let,
    Type,
    Do,
    Of,
    In,
    If,
    Then,
    Else,
    And,
    Or,
    Xor,
    Not,
    True,
    False,
    Fn,

    //Whitespace(String),
    LineComment(String),
    BlockComment(String),
    DocComment(String),
    Error,
}

impl PartialEq for TokenKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for TokenKind {}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        write!(
            f,
            "{}",
            match self {
                LeftParen => "(",
                RightParen => ")",
                LeftBrace => "{",
                RightBrace => "}",
                LeftSquare => "[",
                RightSquare => "]",
                Comma => ",",
                Colon => ":",
                DoubleColon => "::",
                Semicolon => ";",
                Dot => ".",
                DotDot => "..",
                Plus => "+",
                Minus => "-",
                Slash => "/",
                Star => "*",
                PlusDot => "+.",
                MinusDot => "-.",
                SlashDot => "/.",
                StarDot => "*.",
                Percent => "%",
                Apply => "|>",
                ComposeLeft => "<<",
                ComposeRight => ">>",
                Arrow => "->",
                FatArrow => "=>",
                BangEqual => "!=",
                Equal => "=",
                DoubleEqual => "==",
                Greater => ">",
                GreaterEqual => ">=",
                Less => "<",
                LessEqual => "<=",
                Pipe => "|",
                Identifier(id) => return write!(f, "`{id}`"),
                StringLiteral(s) => return write!(f, "\"{s}\""),
                GlyphLiteral(g) => return write!(f, "'{g}'"),
                IntegerLiteral(i, base) => return write!(f, "{}{i}", base.prefix()),
                RealLiteral(r) => return write!(f, "{r}"),
                Module => "module",
                Import => "import",
                Use => "use",
                Do => "do",
                End => "end",
                Match => "match",
                With => "with",
                Let => "let",
                In => "in",
                If => "if",
                Then => "then",
                Else => "else",
                And => "and",
                Or => "or",
                Xor => "xor",
                Not => "not",
                True => "true",
                False => "false",
                Fn => "fn",
                Type => "type",
                Of => "of",
                BlockComment(_) => "block comment",
                LineComment(_) => "line comment",
                DocComment(_) => "doc comment",
                Error => "[ERROR]",
            }
        )
    }
}

impl TokenKind {
    pub fn is_literal(&self) -> bool {
        match self {
            Self::GlyphLiteral(_)
            | Self::RealLiteral(_)
            | Self::IntegerLiteral(..)
            | Self::StringLiteral(_)
            | Self::True
            | Self::False => true,
            _ => false,
        }
    }
}

pub type Token = Spanned<TokenKind>;

struct Tokenizer<I: Iterator<Item = char>> {
    iter: MultiPeek<I>,
    tokens: Vec<Token>,
    position: usize,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
    fn next(&mut self) -> Option<char> {
        self.position += 1;
        self.iter.next()
    }
    fn peek(&mut self) -> Option<char> {
        self.iter.peek().cloned()
    }
    fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.iter.peek_nth(n).cloned()
    }

    fn push(&mut self, token: TokenKind, start: usize) {
        self.tokens.push(token.with_span(self.span(start)))
    }
    fn span(&self, start: usize) -> Span {
        Span {
            start: start,
            width: self.position - start,
        }
    }
}

pub fn tokenize(input: impl IntoIterator<Item = char>, logger: &mut Logger) -> Vec<Token> {
    let mut iter = Tokenizer {
        iter: multipeek(input),
        tokens: vec![],
        position: 0,
    };
    while let Some(current) = iter.next() {
        let start = iter.position - 1;
        // Skip whitespace
        if current.is_whitespace() {
            continue;
        }
        const DOC_COMMENT_START: &str = ">";
        // Parse multiline comment
        if let ('(', Some('*')) = (current, iter.peek()) {
            iter.next();
            let mut depth = 1;
            let mut buffer: Vec<u8> = vec![];
            while let Some(current) = iter.next() {
                if let ('(', Some('*')) = (current, iter.peek()) {
                    depth += 1;
                }
                if let ('*', Some(')')) = (current, iter.peek()) {
                    depth -= 1;
                }
                if depth == 0 {
                    iter.next();
                    break;
                }
                write!(buffer, "{current}").unwrap();
            }
            let content = String::from_utf8_lossy(&buffer).to_string();
            if let Some(content) = content.strip_prefix(DOC_COMMENT_START) {
                iter.push(TokenKind::DocComment(content.to_string()), start);
            } else {
                iter.push(TokenKind::BlockComment(content), start);
            }
            continue;
        }
        // Parse single line comment
        if let ('-', Some('-')) = (current, iter.peek()) {
            iter.next();
            let mut buffer: Vec<u8> = vec![];
            while let Some(c) = iter.next()
                && c != '\n'
            {
                write!(buffer, "{c}").unwrap();
            }
            let content = String::from_utf8_lossy(&buffer).to_string();
            if let Some(content) = content.strip_prefix(DOC_COMMENT_START) {
                iter.push(TokenKind::DocComment(content.to_string()), start);
            } else {
                iter.push(TokenKind::LineComment(content), start);
            }
            continue;
        }
        let next_char = iter.peek();
        // Parse single character tokens
        {
            use TokenKind::*;
            let not_next = move |c| Some(c) != next_char;
            let kind = match current {
                '(' => LeftParen,
                ')' => RightParen,
                '{' => LeftBrace,
                '}' => RightBrace,
                '[' => LeftSquare,
                ']' => RightSquare,
                ',' => Comma,
                ':' if not_next(':') => Colon,
                ';' => Semicolon,
                '|' if not_next('>') => Pipe,
                '.' if not_next('.') => Dot,
                '+' if not_next('.') => Plus,
                '-' if not_next('.') && not_next('>') && not_next('-') => Minus,
                '*' if not_next('.') => Star,
                '/' if not_next('.') => Slash,
                '%' => Percent,
                '=' if not_next('=') && not_next('>') => Equal,
                '<' if not_next('=') && not_next('<') => Less,
                '>' if not_next('=') && not_next('>') => Greater,
                _ => Error,
            };
            if kind != Error {
                iter.push(kind, start);
                continue;
            }
        }
        // Parse two character tokens
        let next_next_char = iter.peek_nth(1);
        if let Some(next_char) = next_char {
            use TokenKind::*;
            let not_next_next = move |c| Some(c) != next_next_char;
            let kind = match (current, next_char) {
                ('.', '.') if not_next_next('=') => DotDot,
                ('=', '=') => DoubleEqual,
                ('!', '=') => BangEqual,
                ('<', '=') => LessEqual,
                ('>', '=') => GreaterEqual,
                ('-', '>') => Arrow,
                ('=', '>') => FatArrow,
                (':', ':') => DoubleColon,
                ('|', '>') => Apply,
                ('<', '<') => ComposeLeft,
                ('>', '>') => ComposeRight,
                ('+', '.') => PlusDot,
                ('-', '.') => MinusDot,
                ('*', '.') => StarDot,
                ('/', '.') => SlashDot,
                _ => Error,
            };
            if kind != Error {
                iter.next();
                iter.push(kind, start);
                continue;
            }
        }
        // Parse glyph literal
        if current == '\'' {
            if let Some(glyph) = parse_delimited(&mut iter, '\'') {
                if let Some(baked) = bake_string(start, logger, &glyph) {
                    let chars = baked.chars().collect::<Vec<_>>();
                    if chars.len() != 1 {
                        error!(
                            logger,
                            iter.span(start),
                            "A glyph must contain a single unicode character"
                        );
                        iter.push(TokenKind::Error, start);
                    } else {
                        iter.push(TokenKind::GlyphLiteral(chars[0]), start);
                    }
                } else {
                    // Error reported during baking
                    iter.push(TokenKind::Error, start);
                }
            } else {
                error!(logger, iter.span(start), "Missing trailing single quote");
                iter.push(TokenKind::Error, start);
            }
            continue;
        }

        // Parse string literal
        if current == '"' {
            if let Some(string) = parse_delimited(&mut iter, '"') {
                if let Some(baked) = bake_string(start, logger, &string) {
                    iter.push(TokenKind::StringLiteral(baked), start);
                } else {
                    // Error reported during baking
                    iter.push(TokenKind::Error, start);
                }
            } else {
                error!(logger, iter.span(start), "Missing trailing double quote");
                iter.push(TokenKind::Error, start);
            }
            continue;
        }

        // Parse number
        if current.is_ascii_digit() {
            let mut buffer: Vec<u8> = vec![];
            // Parse base prefix
            let base = if let Some(next_char) = next_char
                && current == '0'
                && ['x', 'X', 'd', 'D', 'o', 'O', 'b', 'B'].contains(&next_char)
            {
                iter.next();
                match next_char.to_ascii_lowercase() {
                    'x' => Base::Hex,
                    'd' => Base::Decimal,
                    'o' => Base::Octal,
                    'b' => Base::Binary,
                    _ => unreachable!(),
                }
            } else {
                write!(buffer, "{current}").unwrap();
                Base::Decimal
            };
            let is_digit = |c: char| {
                c == '_'
                    || match base {
                        Base::Binary => c == '0' || c == '1',
                        Base::Octal => '0' <= c && c <= '7',
                        Base::Decimal => c.is_ascii_digit(),
                        Base::Hex => c.is_ascii_hexdigit(),
                    }
            };
            // Parse whole part
            while iter.peek().is_some_and(is_digit)
                && let Some(current) = iter.next()
            {
                write!(buffer, "{current}").unwrap();
            }
            //No decimal or 'e', so integer literal
            if iter.peek().is_none_or(char::is_whitespace) {
                let str = String::from_utf8_lossy(&buffer)
                    .to_string()
                    .replace("_", "")
                    .to_ascii_lowercase();
                let str = str.strip_prefix("0d").map(|s| s.to_string()).unwrap_or(str);
                iter.push(TokenKind::IntegerLiteral(str, base), start);
                continue;
            }
            // Parse decimal part
            if iter.peek().is_some_and(|c| c == '.') {
                iter.next();
                write!(buffer, ".").unwrap();
                while iter.peek().is_some_and(is_digit) {
                    write!(buffer, "{}", iter.next().unwrap()).unwrap();
                }
            }
            // Parse exponent part
            if iter.peek().is_some_and(|c| c == 'e') {
                iter.next();
                write!(buffer, "e").unwrap();
                while iter.peek().is_some_and(is_digit) {
                    write!(buffer, "{}", iter.next().unwrap()).unwrap();
                }
            }
            // Finished parsing real
            if iter.peek().is_none_or(char::is_whitespace) {
                if base == Base::Decimal {
                    let str = String::from_utf8_lossy(&buffer)
                        .to_string()
                        .replace("_", "")
                        .to_ascii_lowercase();
                    let str = str.strip_prefix("0d").map(|s| s.to_string()).unwrap_or(str);
                    iter.push(TokenKind::RealLiteral(str), start);
                } else {
                    error!(logger, iter.span(start), "Real numbers must be in base-10");
                    iter.push(TokenKind::Error, start);
                }
                continue;
            }
            // Found erroneous character
            error!(
                logger,
                iter.span(start),
                "Found an unexpected character while parsing this {base} number"
            );
            iter.push(TokenKind::Error, start);
            continue;
        }
        let is_ident = |c: char| (!c.is_ascii_punctuation() || c == '_') && !c.is_whitespace();
        if !is_ident(current) {
            error!(logger, iter.span(start), "Unexpected symbol");
            iter.push(TokenKind::Error, start);
            continue;
        }
        // Parse identifier or keyowrd
        let mut buffer: Vec<u8> = vec![];
        write!(buffer, "{current}").unwrap();
        while let Some(next) = iter.peek()
            && is_ident(next)
        {
            iter.next();
            write!(buffer, "{next}").unwrap();
        }
        let str = String::from_utf8_lossy(&buffer).to_string();
        use TokenKind::*;
        let token = match str.as_str() {
            "let" => Let,
            "do" => Do,
            "in" => In,
            "module" => Module,
            "import" => Import,
            "use" => Use,
            "of" => Of,
            "end" => End,
            "match" => Match,
            "with" => With,
            "if" => If,
            "then" => Then,
            "else" => Else,
            "and" => And,
            "or" => Or,
            "xor" => Xor,
            "not" => Not,
            "true" => True,
            "false" => False,
            "fn" => Fn,
            "type" => Type,
            _ => Identifier(str),
        };
        iter.push(token, start);
    }
    iter.tokens
}

fn parse_delimited(
    iter: &mut Tokenizer<impl Iterator<Item = char>>,
    terminator: char,
) -> Option<String> {
    let mut buffer = String::new();
    let mut escape = false;
    loop {
        let c = match iter.next() {
            Some(c) if c == terminator && !escape => {
                break;
            }
            Some(c) => {
                if c == '\\' {
                    escape = !escape;
                } else {
                    escape = false;
                }
                c
            }
            None => return None,
        };
        buffer.push(c)
    }
    Some(buffer)
}

fn bake_string(mut start: usize, logger: &mut Logger, s: &str) -> Option<String> {
    let collect_hex_bytes = |arr: &[Option<char>]| {
        arr.into_iter()
            .flatten()
            .map(|c| c.to_ascii_lowercase())
            .flat_map(|c| {
                if c.is_ascii_digit() {
                    Some(c as u32 - '0' as u32)
                } else if ('a'..='f').contains(&c) {
                    Some(c as u32 - 'a' as u32 + 10)
                } else {
                    None
                }
            })
            .collect()
    };
    let mut baked = String::with_capacity(s.len());
    let mut iter = s.chars();
    while let Some(next) = iter.next() {
        start += 1;
        baked.push(if next == '\\' {
            let Some(next) = iter.next() else {
                error!(
                    logger,
                    Span {
                        start: start - 1,
                        width: 2
                    },
                    "Expecting an escape sequence here"
                );
                return None;
            };
            start += 1;
            match next {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'b' => '\x08',
                '\\' => '\\',
                '0' => '\0',
                '"' => '"',
                '\'' => '\'',
                'x' => {
                    let bytes: Vec<_> = collect_hex_bytes(&[iter.next(), iter.next()]);
                    if bytes.len() != 2 {
                        error!(
                            logger,
                            Span {
                                start: start - 1,
                                width: 4
                            },
                            "The \\xXX escape sequence requires 2 hex digits"
                        );
                        return None;
                    }
                    start += 2;
                    unsafe { char::from_u32_unchecked(bytes[0] << 8 | bytes[1]) }
                }
                'w' => {
                    let bytes =
                        collect_hex_bytes(&[iter.next(), iter.next(), iter.next(), iter.next()]);
                    if bytes.len() != 4 {
                        error!(
                            logger,
                            Span {
                                start: start - 1,
                                width: 6
                            },
                            "The \\wXXXX escape sequence requires 4 hex digits"
                        );
                        return None;
                    }
                    start += 4;
                    unsafe {
                        char::from_u32_unchecked(
                            bytes[0] << 24 | bytes[1] << 16 | bytes[2] << 8 | bytes[3],
                        )
                    }
                }
                c => {
                    error!(
                        logger,
                        Span {
                            start: start - 1,
                            width: 2
                        },
                        "Invalid escape sequence \"\\{c}\" "
                    );
                    return None;
                }
            }
        } else {
            next
        })
    }
    Some(baked)
}
