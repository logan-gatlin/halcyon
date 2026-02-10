use std::io::Write;

use crate::{
    FileLogger,
    Span,
    Spanned,
    WithContext,
    WithSpan,
};
use multipeek::{
    MultiPeek,
    multipeek,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Base::Binary => write!(f, "binary"),
            Base::Octal => write!(f, "octal"),
            Base::Decimal => write!(f, "decimal"),
            Base::Hex => write!(f, "hex"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl TokenKind {
    pub fn category(&self) -> TokenCategory {
        use TokenCategory::*;
        use TokenKind::*;
        match self {
            LeftParen | LeftBrace | LeftSquare => BeginGrouping,
            RightParen | RightBrace | RightSquare => EndGrouping,
            Dot | DotDot | DoubleColon | Comma => Delimeter,
            Colon | Arrow | Semicolon | Plus | PlusDot | Minus | MinusDot | Slash | SlashDot
            | Star | StarDot | Percent | Apply | ComposeLeft | ComposeRight | BangEqual | Equal
            | DoubleEqual | Greater | GreaterEqual | Less | LessEqual => Operator,
            Identifier(_) | StringLiteral(_) | GlyphLiteral(_) | IntegerLiteral(..)
            | RealLiteral(_) => Literal,
            Module | Import | Use | End | Match | With | Let | Type | Do | Of | In | If | Then
            | Else | And | Or | Xor | Not | True | False | Fn | Pipe | FatArrow => Keyword,
            LineComment(_) | BlockComment(_) | DocComment(_) | Error => Extra,
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
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
        matches!(
            self,
            Self::GlyphLiteral(_)
                | Self::RealLiteral(_)
                | Self::IntegerLiteral(..)
                | Self::StringLiteral(_)
                | Self::True
                | Self::False
        )
    }
}

/// Broader categories of tokens.
/// See `crate::token::TokenKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenCategory {
    /// Language semantic tokens, not necessarily a word.
    /// Anything that is used by the parser to change parsing contexts is a keyword.
    Keyword,
    /// All symbols that are re-interpreted as a function, in addition to some type specific operators.
    Operator,
    /// Tokens that begin a grouping
    BeginGrouping,
    /// Tokens that end a grouping
    EndGrouping,
    /// Tokens that break up or modify other expressions without grouping them.
    Delimeter,
    /// Tokens that contain a literal value.
    /// Identifiers are included in this category.
    Literal,
    /// Tokens that do not factor into parsing
    Extra,
}

pub type Token = Spanned<TokenKind>;

struct Tokenizer<'a, I: Iterator<Item = char>> {
    iter: MultiPeek<I>,
    tokens: Vec<Token>,
    position: usize,
    logger: &'a mut FileLogger,
}

impl<'a, I: Iterator<Item = char>> Tokenizer<'a, I> {
    fn next(&mut self) -> Option<char> {
        let next = self.iter.next();
        if let Some(next) = next {
            self.position += next.len_utf8();
        }
        next
    }
    fn peek(&mut self) -> Option<char> {
        self.iter.peek().cloned()
    }
    fn peek_nth(
        &mut self,
        n: usize,
    ) -> Option<char> {
        self.iter.peek_nth(n).cloned()
    }
    fn push(
        &mut self,
        token: TokenKind,
        start: usize,
    ) {
        self.tokens.push(token.with_span(self.span(start)))
    }
    fn span(
        &self,
        start: usize,
    ) -> Span {
        self.logger.new_span(start, self.position - start)
    }
}

pub fn tokenize(
    input: impl IntoIterator<Item = char>,
    logger: &mut FileLogger,
) -> Vec<Token> {
    let mut iter = Tokenizer {
        iter: multipeek(input),
        tokens: vec![],
        position: 0,
        logger,
    };
    while let Some(current) = iter.next() {
        let start = iter.position - 1;
        // Skip whitespace
        if current.is_whitespace() {
            continue;
        }
        const DOC_COMMENT_START: &str = ">";
        // Parse multiline comment
        if let ('(', Some('*')) = (current, iter.peek())
            && iter.peek_nth(1) != Some(')')
        {
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
                let _ = write!(buffer, "{current}");
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
                let _ = write!(buffer, "{c}");
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
                if let Some(baked) = bake_string(start, iter.logger, &glyph) {
                    let chars = baked.chars().collect::<Vec<_>>();
                    if chars.len() != 1 {
                        let span = iter.span(start);
                        iter.logger
                            .error("Glyphs may only contain a single unicode character")
                            .primary(
                                format!("This string consists of {} characters", chars.len()),
                                span,
                            )
                            .done();
                        iter.push(TokenKind::GlyphLiteral('?'), start);
                    } else {
                        iter.push(TokenKind::GlyphLiteral(chars[0]), start);
                    }
                } else {
                    // Error reported during baking
                    iter.push(TokenKind::StringLiteral("".into()), start);
                }
            } else {
                let span = iter.logger.new_span(start, 1);
                iter.logger
                    .error("Missing closing single quote (\')")
                    .primary("Opening \' here is not closed", span)
                    .done();
                iter.push(TokenKind::Error, start);
            }
            continue;
        }

        // Parse string literal
        if current == '"' {
            if let Some(string) = parse_delimited(&mut iter, '"') {
                if let Some(baked) = bake_string(start, iter.logger, &string) {
                    iter.push(TokenKind::StringLiteral(baked), start);
                } else {
                    // Error reported during baking
                    iter.push(TokenKind::Error, start);
                }
            } else {
                let span = iter.logger.new_span(start, 1);
                iter.logger
                    .error("Missing closing double quote (\")")
                    .primary("Opening \" here is not closed", span)
                    .done();
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
                let _ = write!(buffer, "{current}");
                Base::Decimal
            };
            let is_digit = |c: char| {
                c == '_'
                    || match base {
                        Base::Binary => c == '0' || c == '1',
                        Base::Octal => ('0'..='7').contains(&c),
                        Base::Decimal => c.is_ascii_digit(),
                        Base::Hex => c.is_ascii_hexdigit(),
                    }
            };
            // Parse whole part
            while iter.peek().is_some_and(is_digit)
                && let Some(current) = iter.next()
            {
                let _ = write!(buffer, "{current}");
            }
            //No decimal or 'e', so integer literal
            if iter
                .peek()
                .is_none_or(|c| c.is_whitespace() || (c.is_ascii_punctuation() && c != '.'))
            {
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
                let _ = write!(buffer, ".");
                while let Some(next) = iter.peek()
                    && is_digit(next)
                {
                    iter.next();
                    let _ = write!(buffer, "{next}");
                }
            }
            // Parse exponent part
            if iter.peek().is_some_and(|c| c == 'e') {
                iter.next();
                let _ = write!(buffer, "e");
                if let Some(sign) = iter.peek()
                    && (sign == '+' || sign == '-')
                {
                    iter.next();
                    let _ = write!(buffer, "{sign}");
                }
                while let Some(next) = iter.peek()
                    && is_digit(next)
                {
                    iter.next();
                    let _ = write!(buffer, "{next}");
                }
            }
            // Finished parsing real
            if iter
                .peek()
                .is_none_or(|c| c.is_whitespace() || c.is_ascii_punctuation())
            {
                if base == Base::Decimal {
                    let str = String::from_utf8_lossy(&buffer)
                        .to_string()
                        .replace("_", "")
                        .to_ascii_lowercase();
                    let str = str.strip_prefix("0d").map(|s| s.to_string()).unwrap_or(str);
                    iter.push(TokenKind::RealLiteral(str), start);
                } else {
                    let span = iter.span(start);
                    iter.logger
                        .error("Real numbers must be written in decimal (base 10).")
                        .primary(
                            format!(
                                "This token was parsed as a {base} real, which is not allowed."
                            ),
                            span,
                        )
                        .done();
                    iter.push(TokenKind::RealLiteral("1.0".into()), start);
                }
                continue;
            }
            // Found erroneous character
            let next_char = iter.peek().unwrap_or_else(|| unreachable!());
            let span = iter.logger.new_span(iter.position + 1, 1);
            iter.logger
                .error("Illegal character in number.")
                .primary(
                    format!("The character {next_char} is not valid inside of a number."),
                    span,
                )
                .done();
            iter.push(TokenKind::Error, start);
            continue;
        }
        let is_ident_start =
            |c: char| (!c.is_ascii_punctuation() || c == '_') && !c.is_whitespace();
        let is_ident_continue =
            |c: char| (!c.is_ascii_punctuation() || c == '_' || c == '-') && !c.is_whitespace();
        if !is_ident_start(current) {
            let span = iter.logger.new_span(start, 1);
            iter.logger
                .error("Unexpected character")
                .primary(
                    format!(
                        "The character '{current}' ({}) is not a part of any token in the language",
                        unicode_names2::name(current)
                            .map(|n| n.to_string())
                            .unwrap_or("invalid UTF-8".to_string())
                    ),
                    span,
                )
                .done();
            iter.push(TokenKind::Error, start);
            continue;
        }
        // Parse identifier or keyword
        let mut buffer: Vec<u8> = vec![];
        let _ = write!(buffer, "{current}");
        while let Some(next) = iter.peek()
            && is_ident_continue(next)
        {
            // Don't consume a trailing hyphen
            if next == '-'
                && iter
                    .peek_nth(1)
                    .is_none_or(|c| !is_ident_continue(c) || c == '-')
            {
                break;
            }
            iter.next();
            let _ = write!(buffer, "{next}");
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

fn bake_string(
    mut start: usize,
    logger: &mut FileLogger,
    s: &str,
) -> Option<String> {
    let collect_hex_bytes = |arr: &[Option<char>]| {
        arr.iter()
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
        start += next.len_utf8();
        baked.push(if next == '\\' {
            let Some(next) = iter.next() else {
                let span = logger.new_span(start - 1, 2);
                logger
                    .error("Unknown escape sequence")
                    .primary(
                        "This sequence starts with a \\, but is not a recognized escape sequence.",
                        span
                    )
                    .done();
                return None;
            };
            start += next.len_utf8();
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
                    let chars = [iter.next(), iter.next()];
                    let length = chars.iter().fold(0, |acc, x| {
                        if let Some(x) = x {
                            acc + x.len_utf8()
                        } else {
                            acc
                        }
                    });
                    let bytes: Vec<_> = collect_hex_bytes(&chars);
                    if bytes.len() != 2 {
                        let span = logger.new_span(start - 1, 4);
                        logger.error("Unknown escape sequence")
                            .primary(
                                "This sequence starts with \\x, but is not followed by two hexadecimal digits.",
                                span
                            ).done();
                        return None;
                    }
                    start += length;
                    unsafe { char::from_u32_unchecked(bytes[0] << 4 | bytes[1]) }
                }
                'w' => {
                    let bytes =
                        collect_hex_bytes(&[iter.next(), iter.next(), iter.next(), iter.next()]);
                    if bytes.len() != 4 {
                        let span = logger.new_span(start - 1, 6);
                        logger.error("Unknown escape sequence")
                            .primary("This sequence starts with \\w, but is not followed by 4 hex digits.", span).done();
                        return None;
                    }
                    start += 4;
                    unsafe {
                        char::from_u32_unchecked(
                            bytes[0] << 12 | bytes[1] << 8 | bytes[2] << 4 | bytes[3],
                        )
                    }
                }
                c => {
                    let span = logger.new_span(start - 1, 2);
                    logger.error("Unknown escape sequence").primary(
                        format!("The \\{c} sequence here is not recognized."), span
                    ).done();
                    return None;
                }
            }
        } else {
            next
        })
    }
    Some(baked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<TokenKind> {
        let mut logger = FileLogger::new(0);
        let tokens = tokenize(input.chars(), &mut logger);
        assert!(logger.is_ok(), "Tokenizer produced errors: {:?}", logger);
        tokens.into_iter().map(|t| t.inner).collect()
    }

    #[test]
    fn test_symbols() {
        // Updated expectation: The test string includes symbols.
        // We need to match the exact TokenKind variants produced by the tokenizer.
        use TokenKind::*;
        let tokens =
            lex("() {} [] , : :: ; . .. + - / * +. -. /. *. % |> << >> -> => != = == > >= < <= |");
        let expected = vec![
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
            Minus,
            Slash,
            Star,
            PlusDot,
            MinusDot,
            SlashDot,
            StarDot,
            Percent,
            Apply,
            ComposeLeft,
            ComposeRight,
            Arrow,
            FatArrow,
            BangEqual,
            Equal,
            DoubleEqual,
            Greater,
            GreaterEqual,
            Less,
            LessEqual,
            Pipe,
        ];

        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        let expected_str: Vec<String> = expected.iter().map(|t| t.to_string()).collect();

        assert_eq!(tokens_str, expected_str);
    }

    #[test]
    fn test_keywords() {
        let tokens = lex(
            "module import use end match with let type do of in if then else and or xor not true false fn",
        );
        let expected = vec![
            TokenKind::Module,
            TokenKind::Import,
            TokenKind::Use,
            TokenKind::End,
            TokenKind::Match,
            TokenKind::With,
            TokenKind::Let,
            TokenKind::Type,
            TokenKind::Do,
            TokenKind::Of,
            TokenKind::In,
            TokenKind::If,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::And,
            TokenKind::Or,
            TokenKind::Xor,
            TokenKind::Not,
            TokenKind::True,
            TokenKind::False,
            TokenKind::Fn,
        ];
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        let expected_str: Vec<String> = expected.iter().map(|t| t.to_string()).collect();
        assert_eq!(tokens_str, expected_str);
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("foo bar_baz _qux");
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        assert_eq!(tokens_str, vec!["`foo`", "`bar_baz`", "`_qux`"]);
    }

    #[test]
    fn test_identifiers_with_hyphens() {
        let tokens = lex("foo-bar foo-");
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        assert_eq!(tokens_str, vec!["`foo-bar`", "`foo`", "-"]);
    }

    #[test]
    fn test_integers() {
        let tokens = lex("123 0xff 0o77 0b101");
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        assert_eq!(tokens_str, vec!["123", "0xff", "0o77", "0b101"]);
    }

    #[test]
    fn test_floats() {
        let tokens = lex("1.0 0.5 1e10 1.2e-5");
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        assert_eq!(tokens_str, vec!["1.0", "0.5", "1e10", "1.2e-5"]);
    }

    #[test]
    fn test_strings() {
        let tokens = lex(r#""hello" "world\n""#);
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        assert_eq!(
            tokens_str,
            vec![
                r#""hello""#,
                r#""world
""#
            ]
        );
        // Note: Display for StringLiteral uses write!(f, "\"{s}\"").
        // "world\n" as string contains a newline character.
    }

    #[test]
    fn test_glyphs() {
        let tokens = lex(r#"'a' '\n' '\x41'"#);
        let tokens_str: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        // '\n' char is newline.
        assert_eq!(tokens_str, vec!["'a'", "'\n'", "'A'"]);
    }

    #[test]
    fn test_comments() {
        let tokens = lex("1 -- comment\n2 (* block comment *) 3");

        match &tokens[1] {
            TokenKind::LineComment(s) => assert_eq!(s, " comment"),
            _ => panic!("Expected LineComment"),
        }
        match &tokens[3] {
            TokenKind::BlockComment(s) => assert_eq!(s, " block comment "),
            _ => panic!("Expected BlockComment"),
        }
    }

    #[test]
    fn test_doc_comments() {
        let tokens = lex("--> doc comment\n(*> block doc *)");

        match &tokens[0] {
            TokenKind::DocComment(s) => assert_eq!(s, " doc comment"),
            _ => panic!("Expected DocComment at 0"),
        }

        match &tokens[1] {
            TokenKind::DocComment(s) => assert_eq!(s, " block doc "),
            _ => panic!("Expected DocComment at 1, got {:?}", tokens[1]),
        }
    }
}
