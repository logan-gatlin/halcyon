use crate::Span;
use crate::lint::*;
use multipeek::{MultiPeek, multipeek};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
  Binary = 2,
  Octal = 8,
  Decimal = 10,
  Hex = 16,
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
  DoubleSemicolon,

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
  Tilda,
  Arrow,
  FatArrow,
  Apply,

  Bang,
  BangEqual,
  Question,
  QuestionEqual,
  Equal,
  DoubleEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  At,

  Pipe,
  Ampersand,
  Carrot,
  Hash,

  DotDotEqual,

  Identifier(String),
  StringLiteral(String),
  GlyphLiteral(char),
  IntegerLiteral(String, Base),
  RealLiteral(String),

  Module,
  End,
  Match,
  Let,
  Type,
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

  Whitespace(String),
  SmallComment(String),
  BigComment(String),

  Idk,
  EOF,
}

impl PartialEq for TokenKind {
  fn eq(&self, other: &Self) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(other)
  }
}

impl Eq for TokenKind {
}

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
        DoubleSemicolon => ";;",
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
        Tilda => "~",
        Apply => "|>",
        Hash => "#",
        At => "@",
        Arrow => "->",
        FatArrow => "=>",
        Bang => "!",
        BangEqual => "!=",
        Question => "?",
        QuestionEqual => "?=",
        Equal => "=",
        DoubleEqual => "==",
        Greater => ">",
        GreaterEqual => ">=",
        Less => "<",
        LessEqual => "<=",
        Pipe => "|",
        Ampersand => "&",
        Carrot => "^",
        DotDotEqual => "..=",
        Identifier(_) => "identifier",
        StringLiteral(_) => "string literal",
        GlyphLiteral(_) => "glyph literal",
        IntegerLiteral(_, _) => "integer literal",
        RealLiteral(_) => "float literal",
        Module => "module",
        End => "end",
        Match => "match",
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
        Whitespace(_) => "whitespace",
        BigComment(_) | SmallComment(_) => "comment",
        Idk => "unknown symbol",
        EOF => "EOF",
      }
    )
  }
}

#[derive(Clone, Debug)]
pub struct Token(pub TokenKind, pub Span);

fn t(tk: TokenKind, sp: Span) -> Result<Token> {
  Ok(Token(tk, sp))
}

pub fn tokenize(chars: impl IntoIterator<Item = char>) -> Result<Vec<Token>> {
  let c = chars.into_iter();
  Tokenizer::new(c).try_collect()
}

struct Tokenizer<I: Iterator<Item = char>> {
  iter: MultiPeek<I>,
  index: usize,
  ended: bool,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
  pub fn new(iter: I) -> Self {
    Self {
      iter: multipeek(iter),
      index: 0,
      ended: false,
    }
  }

  fn next_char(&mut self) -> Option<char> {
    match self.iter.next() {
      Some(c) => {
        self.index += 1;
        Some(c)
      },
      _ => None,
    }
  }

  fn delimited(&mut self, terminator: char) -> Option<String> {
    let mut buffer = String::new();
    let mut escape = false;
    loop {
      let c = match self.next_char() {
        Some(c) if c == terminator && !escape => {
          break;
        },
        Some(c) => {
          if c == '\\' {
            escape = !escape;
          } else {
            escape = false;
          }
          c
        },
        None => return None,
      };
      buffer.push(c)
    }
    Some(buffer)
  }

  fn peek(&mut self, n: usize) -> Option<char> {
    self.iter.peek_nth(n).cloned()
  }

  fn _next(&mut self) -> Result<Token> {
    use TokenKind::*;
    let mut position = Span {
      start: self.index,
      width: 0,
    };
    if self.ended == true {
      return t(EOF, position);
    }
    let current = match self.next_char() {
      Some(std::char::REPLACEMENT_CHARACTER) => {
        return Err(lint(TokenLint::InvalidInput, position, []));
      },
      Some(c) => c,
      None => {
        self.ended = true;
        return t(EOF, position);
      },
    };
    // Parse whitespace
    if current.is_whitespace() {
      let mut buffer = String::from(current);
      while let Some(c) = self.peek(0) {
        if !c.is_whitespace() {
          break;
        }
        _ = self.next_char();
        buffer.push(c.clone());
      }
      position.width = buffer.chars().count();
      return t(Whitespace(buffer), position);
    }
    // Parse multiline comments
    if let ('/', Some('*')) = (current, self.peek(0)) {
      let _ = self.next_char();
      let mut comment_level = 1;
      let mut buffer = String::new();
      while let Some(current) = self.next_char() {
        // Ignore /* */ inside strings
        if '\"' == current {
          if let Some(inner_string) = self.delimited('\"') {
            buffer.push('\"');
            buffer.push_str(&inner_string);
            buffer.push('\"');
            continue;
          }
        }
        if let ('/', Some('*')) = (current, self.peek(0)) {
          comment_level += 1;
        } else if let ('*', Some('/')) = (current, self.peek(0)) {
          comment_level -= 1;
        }
        if comment_level == 0 {
          let _ = self.next_char();
          break;
        }
        buffer.push(current);
      }
      position.width = buffer.chars().count();
      return t(BigComment(buffer), position);
    }
    // Parse single line comments
    if let ('/', Some('/')) = (current, self.peek(0)) {
      let _ = self.next_char();
      let mut buffer = String::new();
      while let Some(c) = self.next_char() {
        if c == '\n' {
          break;
        }
        buffer.push(c);
      }
      position.width = buffer.chars().count();
      return t(SmallComment(buffer), position);
    }
    let next = self.peek(0);
    let next_next = self.peek(1);
    // Match single character tokens
    {
      let not_next = move |c| Some(c) != next;
      let kind = match current {
        '(' => LeftParen,
        ')' => RightParen,
        '{' => LeftBrace,
        '}' => RightBrace,
        '[' => LeftSquare,
        ']' => RightSquare,
        ',' => Comma,
        ':' if not_next(':') => Colon,
        ';' if not_next(';') => Semicolon,
        '|' => Pipe,
        '&' => Ampersand,
        '^' => Carrot,
        '#' => Hash,
        '~' => Tilda,
        '.' if not_next('.') => Dot,
        '+' if not_next('.') => Plus,
        '-' if not_next('.') && not_next('>') => Minus,
        '*' if not_next('.') => Star,
        '/' if not_next('.') => Slash,
        '%' => Percent,
        '@' => At,
        '!' if not_next('=') => Bang,
        '?' if not_next('=') => Question,
        '=' if not_next('=') && not_next('>') => Equal,
        '<' if not_next('=') => Less,
        '>' if not_next('=') => Greater,
        _ => Idk,
      };
      if kind != Idk {
        position.width = 1;
        return t(kind, position);
      };
    }
    // Match two character tokens
    if let Some(next) = next {
      let not_next_next = move |c| Some(c) != next_next;
      let kind = match (current, next) {
        ('.', '.') if not_next_next('=') => DotDot,
        ('=', '=') => DoubleEqual,
        ('?', '=') => QuestionEqual,
        ('!', '=') => BangEqual,
        ('<', '=') => LessEqual,
        ('>', '=') => GreaterEqual,
        ('-', '>') => Arrow,
        ('=', '>') => FatArrow,
        (':', ':') => DoubleColon,
        ('|', '>') => Apply,
        (';', ';') => DoubleSemicolon,
        ('+', '.') => PlusDot,
        ('-', '.') => MinusDot,
        ('*', '.') => StarDot,
        ('/', '.') => SlashDot,
        _ => Idk,
      };
      if kind != Idk {
        let _ = self.next();
        position.width = 2;
        return t(kind, position);
      }
    }
    // Match three character tokens
    if let (Some(next), Some(next_next)) = (next, next_next) {
      let kind = match (current, next, next_next) {
        ('.', '.', '=') => DotDotEqual,
        _ => Idk,
      };
      if kind != Idk {
        let _ = self.next();
        let _ = self.next();
        position.width = 3;
        return t(kind, position);
      }
    }
    let mut buffer = String::new();
    // Match character
    if current == '\'' {
      let buffer = self
        .delimited('\'')
        .lint(TokenLint::MissingDelimeter)
        .context("'")
        .span(position)?;
      position.width = buffer.chars().count() + 2;
      let baked = bake_string(&buffer, position)?;
      if baked.len() != 1 {
        return Err(lint(TokenLint::WrongGlyphSize, position, []));
      }
      let kind = GlyphLiteral(
        baked
          .chars()
          .next()
          .lint(TokenLint::WrongGlyphSize)
          .span(position)?,
      );
      return t(kind, position);
    }
    // Match string
    if current == '"' {
      let buffer = self
        .delimited('\"')
        .lint(TokenLint::MissingDelimeter)
        .context("\"")
        .span(position)?;
      position.width = buffer.chars().count() + 2;
      let kind = StringLiteral(bake_string(&buffer, position)?);
      return t(kind, position);
    }
    buffer.push(current);
    // Match number
    if current.is_ascii_digit() {
      // Only one dot per number
      let mut encountered_dot = false;
      while let Some(c) = self.peek(0) {
        if c == '.' {
          if encountered_dot {
            break;
          }
          let Some(next) = self.peek(1) else { break };
          if !next.is_ascii_digit() {
            break;
          }
          encountered_dot = true;
        } else if !(['_', 'x', 'X', 'o', 'O', 'b', 'B'].contains(&c)
          || c.is_ascii_hexdigit())
        {
          break;
        }
        buffer.push(c);
        let _ = self.next_char();
      }
      buffer = buffer.to_lowercase();
      position.width += buffer.chars().count();
      // Determine base
      let (buffer, base) = if let Some(buffer) = buffer.strip_prefix("0b") {
        (buffer.to_string(), Base::Binary)
      } else if let Some(buffer) = buffer.strip_prefix("0o") {
        (buffer.to_string(), Base::Octal)
      } else if let Some(buffer) = buffer.strip_prefix("0x") {
        (buffer.to_string(), Base::Hex)
      } else {
        (buffer, Base::Decimal)
      };
      // Determine integer or float
      if base == Base::Decimal && (encountered_dot || buffer.contains("e")) {
        return t(RealLiteral(buffer), position);
      } else {
        return t(IntegerLiteral(buffer, base), position);
      }
    }
    // Match keyword or identifier
    if current.is_ascii_punctuation()
      || (!current.is_alphanumeric() && current != '_')
    {
      position.width = 1;
      return t(TokenKind::Idk, position);
    }
    while let Some(c) = self.peek(0) {
      if !c.is_ascii_punctuation() && c.is_alphanumeric() || c == '_' {
        let _ = self.next_char();
      } else {
        break;
      }
      buffer.push(c);
    }
    position.width = buffer.chars().count();
    // Match keywords
    {
      let kind = match buffer.as_str() {
        "let" => Let,
        "in" => In,
        "module" => Module,
        "end" => End,
        "match" => Match,
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
        _ => Identifier(buffer),
      };
      return t(kind, position);
    }
  }
}

impl<'a, I: Iterator<Item = char>> Iterator for Tokenizer<I> {
  type Item = Result<Token>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      use TokenKind::*;
      if self.ended {
        return None;
      }
      match self._next() {
        Ok(Token(SmallComment(_) | BigComment(_) | Whitespace(_), _)) => {
          continue;
        },
        Ok(s) => return Some(Ok(s)),
        Err(e) => return Some(Err(e)),
      }
    }
  }
}

fn bake_string(s: &str, mut span: Span) -> Result<String> {
  let mut baked = String::with_capacity(s.len());
  let mut iter = s.chars();
  loop {
    let c = match iter.next() {
      Some(c) => c,
      None => break,
    };
    println!("{c} {span:?}");
    if c == '\\' {
      let (escape, length) = parse_single_escape(&mut iter, span)?;
      println!("+{length}");
      span.start += length + 1;
      baked.push(escape);
    } else {
      span.start += c.len_utf8();
      baked.push(c);
    }
  }
  Ok(baked)
}

fn parse_single_escape(
  iter: &mut impl Iterator<Item = char>,
  mut span: Span,
) -> Result<(char, usize)> {
  span.start += 1;
  span.width = 2;
  Ok(match iter.next() {
    Some('n') => ('\n', 1),   // New line
    Some('r') => ('\r', 1),   // Carriage return
    Some('t') => ('\t', 1),   // Tab
    Some('b') => ('\x08', 1), // Backspace
    Some('\\') => ('\\', 1),  // Backslash
    Some('0') => ('\0', 1),   // Null
    Some('"') => ('\"', 1),   // Double quote
    Some('\'') => ('\'', 1),  // Single quote
    Some('x') => (parse_byte_escape(iter, span)?, 2), // Byte escape
    Some('w') => (parse_wide_escape(iter, span)?, 4), // Wide escape
    _ => {
      return Err(lint(TokenLint::UnrecognizedEscape, span, []));
    },
  })
}

fn hex_digit(c: char) -> Option<u32> {
  if ('0'..='9').contains(&c) {
    Some(c as u32 - '0' as u32)
  } else if ('a'..='f').contains(&c) {
    Some(c as u32 - 'a' as u32 + 10)
  } else {
    None
  }
}

fn parse_byte_escape(
  iter: &mut impl Iterator<Item = char>,
  span: Span,
) -> Result<char> {
  let lint = lint(TokenLint::UnrecognizedEscape, span, []);
  let (b1, b2) = match (iter.next(), iter.next()) {
    (Some(b1), Some(b2)) => (b1.to_ascii_lowercase(), b2.to_ascii_lowercase()),
    _ => {
      return Err(lint);
    },
  };
  let byte = match (hex_digit(b1), hex_digit(b2)) {
    (Some(b1), Some(b2)) => b1 << 8 | b2,
    _ => return Err(lint),
  };
  char::from_u32(byte).ok_or(lint)
}

fn parse_wide_escape(
  iter: &mut impl Iterator<Item = char>,
  span: Span,
) -> Result<char> {
  let lint = lint(TokenLint::UnrecognizedEscape, span, []);
  let (b1, b2, b3, b4) =
    match (iter.next(), iter.next(), iter.next(), iter.next()) {
      (Some(b1), Some(b2), Some(b3), Some(b4)) => (
        b1.to_ascii_lowercase(),
        b2.to_ascii_lowercase(),
        b3.to_ascii_lowercase(),
        b4.to_ascii_lowercase(),
      ),
      _ => {
        return Err(lint);
      },
    };
  let byte = match (hex_digit(b1), hex_digit(b2), hex_digit(b3), hex_digit(b4))
  {
    (Some(b1), Some(b2), Some(b3), Some(b4)) => {
      b1 << 24 | b2 << 16 | b3 << 8 | b4
    },
    _ => return Err(lint),
  };
  char::from_u32(byte).ok_or(lint)
}
