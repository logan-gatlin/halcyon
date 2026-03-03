use std::io::Write;

use super::SyntaxKind;
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

pub type LexToken = Spanned<SyntaxKind>;

struct Tokenizer<'a, I: Iterator<Item = char>> {
    iter: MultiPeek<I>,
    tokens: Vec<LexToken>,
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
        kind: SyntaxKind,
        start: usize,
    ) {
        self.tokens.push(kind.with_span(self.span(start)))
    }
    fn span(
        &self,
        start: usize,
    ) -> Span {
        Span::new(start, self.position - start)
    }
}

pub fn tokenize(
    input: impl IntoIterator<Item = char>,
    logger: &mut FileLogger,
) -> Vec<LexToken> {
    let mut iter = Tokenizer {
        iter: multipeek(input),
        tokens: vec![],
        position: 0,
        logger,
    };
    while let Some(current) = iter.next() {
        let start = iter.position - 1;
        if current.is_whitespace() {
            while iter.peek().is_some_and(|c| c.is_whitespace()) {
                iter.next();
            }
            iter.push(SyntaxKind::WHITESPACE, start);
            continue;
        }
        if let ('(', Some('*')) = (current, iter.peek())
            && iter.peek_nth(1) != Some(')')
        {
            iter.next();
            let mut depth = 1;
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
            }
            iter.push(SyntaxKind::BLOCK_COMMENT, start);
            continue;
        }
        if let ('-', Some('-')) = (current, iter.peek()) {
            iter.next();
            while let Some(c) = iter.next()
                && c != '\n'
            {}
            iter.push(SyntaxKind::LINE_COMMENT, start);
            continue;
        }
        let next_char = iter.peek();
        {
            use SyntaxKind::*;
            let not_next = move |c| Some(c) != next_char;
            let kind = match current {
                '(' => L_PAREN,
                ')' => R_PAREN,
                '{' => L_BRACE,
                '}' => R_BRACE,
                '[' => L_SQUARE,
                ']' => R_SQUARE,
                ',' => COMMA,
                '$' => DOLLAR,
                '#' => HASH,
                '@' => AT,
                '~' => TILDE,
                ':' if not_next(':') => COLON,
                ';' => SEMICOLON,
                '|' if not_next('>') => PIPE,
                '.' if not_next('.') => DOT,
                '+' if not_next('.') => PLUS,
                '-' if not_next('.') && not_next('>') && not_next('-') => MINUS,
                '*' if not_next('.') => STAR,
                '/' if not_next('.') => SLASH,
                '%' => PERCENT,
                '=' if not_next('=') && not_next('>') => EQUAL,
                '<' if not_next('=') && not_next('<') => LESS,
                '>' if not_next('=') && not_next('>') => GREATER,
                _ => TOKEN_ERROR,
            };
            if kind != TOKEN_ERROR {
                iter.push(kind, start);
                continue;
            }
        }
        let next_next_char = iter.peek_nth(1);
        if let Some(next_char) = next_char {
            use SyntaxKind::*;
            let not_next_next = move |c| Some(c) != next_next_char;
            let kind = match (current, next_char) {
                ('.', '.') if not_next_next('=') => DOT_DOT,
                ('=', '=') => DOUBLE_EQUAL,
                ('!', '=') => BANG_EQUAL,
                ('<', '=') => LESS_EQUAL,
                ('>', '=') => GREATER_EQUAL,
                ('-', '>') => ARROW,
                ('=', '>') => DOUBLE_ARROW,
                (':', ':') => DOUBLE_COLON,
                ('|', '>') => PIPE_ARROW,
                ('<', '<') => COMPOSE_LEFT,
                ('>', '>') => COMPOSE_RIGHT,
                _ => TOKEN_ERROR,
            };
            if kind != TOKEN_ERROR {
                iter.next();
                iter.push(kind, start);
                continue;
            }
        }
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
                        iter.push(SyntaxKind::GLYPH, start);
                    } else {
                        iter.push(SyntaxKind::GLYPH, start);
                    }
                } else {
                    iter.push(SyntaxKind::STRING, start);
                }
            } else {
                let span = Span::new(start, 1);
                iter.logger
                    .error("Missing closing single quote (\')")
                    .primary("Opening \' here is not closed", span)
                    .done();
                iter.push(SyntaxKind::TOKEN_ERROR, start);
            }
            continue;
        }

        if current == '"' {
            if let Some(string) = parse_delimited(&mut iter, '"') {
                if bake_string(start, iter.logger, &string).is_some() {
                    iter.push(SyntaxKind::STRING, start);
                } else {
                    iter.push(SyntaxKind::TOKEN_ERROR, start);
                }
            } else {
                let span = Span::new(start, 1);
                iter.logger
                    .error("Missing closing double quote (\")")
                    .primary("Opening \" here is not closed", span)
                    .done();
                iter.push(SyntaxKind::TOKEN_ERROR, start);
            }
            continue;
        }

        if current.is_ascii_digit() {
            let mut buffer: Vec<u8> = vec![];
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
            while iter.peek().is_some_and(is_digit)
                && let Some(current) = iter.next()
            {
                let _ = write!(buffer, "{current}");
            }
            if iter
                .peek()
                .is_none_or(|c| c.is_whitespace() || (c.is_ascii_punctuation() && c != '.'))
            {
                iter.push(SyntaxKind::INTEGER, start);
                continue;
            }
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
            if iter
                .peek()
                .is_none_or(|c| c.is_whitespace() || c.is_ascii_punctuation())
            {
                if base == Base::Decimal {
                    iter.push(SyntaxKind::REAL, start);
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
                    iter.push(SyntaxKind::REAL, start);
                }
                continue;
            }
            let next_char = iter.peek().unwrap_or_else(|| unreachable!());
            let span = Span::new(iter.position + 1, 1);
            iter.logger
                .error("Illegal character in number.")
                .primary(
                    format!("The character {next_char} is not valid inside of a number."),
                    span,
                )
                .done();
            iter.push(SyntaxKind::TOKEN_ERROR, start);
            continue;
        }
        let is_ident_start =
            |c: char| (!c.is_ascii_punctuation() || c == '_') && !c.is_whitespace();
        let is_ident_continue =
            |c: char| (!c.is_ascii_punctuation() || c == '_' || c == '-') && !c.is_whitespace();
        if !is_ident_start(current) {
            let span = Span::new(start, 1);
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
            iter.push(SyntaxKind::TOKEN_ERROR, start);
            continue;
        }
        let mut buffer: Vec<u8> = vec![];
        let _ = write!(buffer, "{current}");
        while let Some(next) = iter.peek()
            && is_ident_continue(next)
        {
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
        use SyntaxKind::*;
        let kind = match str.as_str() {
            "let" => LET_KW,
            "do" => DO_KW,
            "in" => IN_KW,
            "module" => MODULE_KW,
            "import" => IMPORT_KW,
            "use" => USE_KW,
            "of" => OF_KW,
            "end" => END_KW,
            "match" => MATCH_KW,
            "with" => WITH_KW,
            "if" => IF_KW,
            "then" => THEN_KW,
            "else" => ELSE_KW,
            "and" => AND_KW,
            "or" => OR_KW,
            "xor" => XOR_KW,
            "not" => NOT_KW,
            "true" => TRUE_KW,
            "false" => FALSE_KW,
            "fn" => FN_KW,
            "type" => TYPE_KW,
            "trait" => TRAIT_KW,
            "impl" => IMPL_KW,
            "wasm" => WASM_KW,
            _ => IDENT,
        };
        iter.push(kind, start);
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
                let span = Span::new(start - 1, 2);
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
                        let span = Span::new(start - 1, 4);
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
                        let span = Span::new(start - 1, 6);
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
                    let span = Span::new(start - 1, 2);
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
