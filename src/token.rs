use crate::Span;
use crate::err::*;

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
  Semicolon,

  Dot,
  DotDot,
  Plus,
  Minus,
  Slash,
  Star,
  Percent,
  Arrow,
  FatArrow,
  PlusEqual,
  MinusEqual,
  SlashEqual,
  StarEqual,
  PercentEqual,

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

  Pipe,
  Ampersand,
  Carrot,
  Hash,

  DotDotEqual,

  Identifier(String),
  StringLiteral(String),
  GlyphLiteral(char),
  IntegerLiteral(String, Base),
  FloatLiteral(String),

  If,
  Else,
  And,
  Or,
  Xor,
  Not,
  Nand,
  Nor,
  Xnor,
  Print,
  Break,
  Return,
  Continue,
  For,
  While,
  True,
  False,
  Struct,
  Enum,
  Union,

  Whitespace(String),
  SmallComment(String),
  BigComment(String),

  Error(Diagnostic),
  Idk,
  EOF,
}

impl TokenKind {
  pub fn is_meaningful(&self) -> bool {
    match self {
      Self::Whitespace(_)
      | Self::SmallComment(_)
      | Self::BigComment(_)
      | Self::Idk => false,
      _ => true,
    }
  }
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
    write!(f, "{self:?}")
  }
}

#[derive(Clone, Debug)]
pub struct Token(pub TokenKind, pub Span);

fn t(tk: TokenKind, sp: Span) -> Result<Token> {
  Ok(Token(tk, sp))
}

const TOKENIZER_LOOKAHEAD: usize = 2;

type CharIter<I> = crate::Window<TOKENIZER_LOOKAHEAD, char, I>;

pub struct Tokenizer<I: Iterator<Item = char>> {
  iter: CharIter<I>,
  column: usize,
  row: usize,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
  pub fn new(iter: I) -> Self {
    Self {
      iter: CharIter::new(iter),
      column: 1,
      row: 1,
    }
  }

  fn next_char(&mut self) -> Option<char> {
    match self.iter.next() {
      Some(c) if c == '\n' => {
        self.row += 1;
        self.column = 1;
        Some(c)
      },
      Some(c) => {
        self.column += 1;
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
    self.iter.peek(n).clone()
  }

  fn _next(&mut self) -> Result<Token> {
    use TokenKind::*;
    let position = Span {
      row: self.row,
      column: self.column,
    };
    let current = match self.next_char() {
      Some(std::char::REPLACEMENT_CHARACTER) => {
        return t(
          Error(Diagnostic {
            reason: "Non-UTF8 encoded glyph".into(),
            span: Some(position),
            backtrace: vec![],
          }),
          position,
        );
      },
      Some(c) => c,
      None => return t(EOF, position),
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
        ':' => Colon,
        ';' => Semicolon,
        '|' => Pipe,
        '&' => Ampersand,
        '^' => Carrot,
        '#' => Hash,
        '.' if not_next('.') => Dot,
        '+' if not_next('=') => Plus,
        '-' if not_next('=') && not_next('>') => Minus,
        '*' if not_next('=') => Star,
        '/' if not_next('=') => Slash,
        '%' if not_next('=') => Percent,
        '!' if not_next('=') => Bang,
        '?' if not_next('=') => Question,
        '=' if not_next('=') && not_next('>') => Equal,
        '<' if not_next('=') => Less,
        '>' if not_next('=') => Greater,
        _ => Idk,
      };
      if kind != Idk {
        return t(kind, position);
      };
    }
    // Match two character tokens
    if let Some(next) = next {
      let not_next_next = move |c| Some(c) != next_next;
      let kind = match (current, next) {
        ('.', '.') if not_next_next('=') => DotDot,
        ('+', '=') => PlusEqual,
        ('-', '=') => MinusEqual,
        ('*', '=') => StarEqual,
        ('/', '=') => SlashEqual,
        ('%', '=') => PercentEqual,
        ('=', '=') => DoubleEqual,
        ('?', '=') => QuestionEqual,
        ('!', '=') => BangEqual,
        ('<', '=') => LessEqual,
        ('>', '=') => GreaterEqual,
        ('-', '>') => Arrow,
        ('=', '>') => FatArrow,
        _ => Idk,
      };
      if kind != Idk {
        let _ = self.next();
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
        return t(kind, position);
      }
    }
    let mut buffer = String::new();
    // Match character
    if current == '\'' {
      let buffer = self
        .delimited('\'')
        .reason("Single quote (') was opened, but never closed")
        .span(&position)?;
      let baked = bake_string(&buffer)?;
      if baked.len() != 1 {
        return error()
          .reason("Single quote (') contains more than one character")
          .span(&position);
      }
      let kind = GlyphLiteral(
        baked
          .chars()
          .next()
          .reason("Single quote (') contains no characters")
          .span(&position)?,
      );
      return t(kind, position);
    }
    // Match string
    if current == '"' {
      let buffer = self
        .delimited('\"')
        .reason("Double quote (\") was opened, but never closed")
        .span(&position)?;
      let kind = StringLiteral(bake_string(&buffer)?);
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
        } else if !(c == '_' || c == 'x' || c.is_ascii_hexdigit()) {
          break;
        }
        buffer.push(c);
        let _ = self.next_char();
      }
      buffer = buffer.to_lowercase();
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
        return t(FloatLiteral(buffer), position);
      } else {
        return t(IntegerLiteral(buffer, base), position);
      }
    }
    // Match keyword or identifier
    while let Some(c) = self.peek(0) {
      if c.is_alphanumeric() || c == '_' {
        let _ = self.next_char();
      } else {
        break;
      }
      buffer.push(c);
    }
    // Match keywords
    {
      let kind = match buffer.as_str() {
        "if" => If,
        "else" => Else,
        "and" => And,
        "or" => Or,
        "xor" => Xor,
        "nand" => Nand,
        "nor" => Nor,
        "xnor" => Xnor,
        "for" => For,
        "while" => While,
        "print" => Print,
        "break" => Break,
        "return" => Return,
        "continue" => Continue,
        "not" => Not,
        "true" => True,
        "false" => False,
        "struct" => Struct,
        "enum" => Enum,
        "union" => Union,
        _ => Identifier(buffer),
      };
      return t(kind, position);
    }
  }
}

impl<I: Iterator<Item = char>> Iterator for Tokenizer<I> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    match self._next() {
      Ok(s) => Some(s),
      Err(e) => Some(Token(
        TokenKind::Error(e.clone()),
        e.span.expect("error without span in tokenizer"),
      )),
    }
  }
}

fn bake_string(s: &str) -> Result<String> {
  let mut baked = String::with_capacity(s.len());
  let mut it = s.chars();
  loop {
    match it.next() {
      Some('\\') => baked.push(match it.next() {
        Some('n') => '\n',   // New line
        Some('r') => '\r',   // Carriage return
        Some('t') => '\t',   // Tab
        Some('b') => '\x08', // Backspace
        Some('\\') => '\\',  // Backslash
        Some('\0') => '\0',  // Null
        Some('"') => '\"',   // Double quote
        Some('\'') => '\'',  // Single quote
        Some('x') => {
          // Ascii escapes
          let mut a = || {
            let a = u32::from_str_radix(&it.next()?.to_string(), 16).ok()?;
            let b = u32::from_str_radix(&it.next()?.to_string(), 16).ok()?;
            let num = (a << 4) | b;
            char::from_u32(num)
          };
          a().reason(format!("Found invalid ASCII (\\aXX) escape sequence"))?
        },
        Some('u') => {
          // Unicode escapes
          let mut a = || {
            let a = u32::from_str_radix(&it.next()?.to_string(), 16).ok()?;
            let b = u32::from_str_radix(&it.next()?.to_string(), 16).ok()?;
            let c = u32::from_str_radix(&it.next()?.to_string(), 16).ok()?;
            let d = u32::from_str_radix(&it.next()?.to_string(), 16).ok()?;
            let num = (a << 12) | (b << 8) | (c << 4) | d;
            char::from_u32(num)
          };
          a().reason("Found invalid Unicode (\\uXXXX) escape sequence")?
        },
        _ => {
          return Err(Diagnostic::new("Found invalid escape sequence", None));
        },
      }),
      // Unremarkable character
      Some(c) => baked.push(c),
      None => break,
    }
  }
  Ok(baked)
}
