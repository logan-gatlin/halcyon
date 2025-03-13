use dyn_clone::*;

use crate::{
  Span,
  token::{Token, TokenKind},
};

#[derive(Clone)]
pub struct Lint {
  pub kind: Box<dyn LintKind>,
  pub span: Option<Span>,
  pub file: Option<String>,
}

pub fn lint<T, L: LintKind + 'static>(lint: L) -> Result<T, Lint> {
  Err(Lint {
    kind: Box::new(lint),
    span: None,
    file: None,
  })
}

pub trait OrLint<T> {
  fn lint<L: LintKind + 'static>(self, lint: L) -> Result<T, Lint>;
}

impl<T, E> OrLint<T> for std::result::Result<T, E> {
  fn lint<L: LintKind + 'static>(self, lint: L) -> Result<T, Lint> {
    match self {
      Ok(v) => Ok(v),
      Err(_) => Err(Lint {
        kind: Box::new(lint),
        span: None,
        file: None,
      }),
    }
  }
}

impl<T> OrLint<T> for std::option::Option<T> {
  fn lint<L: LintKind + 'static>(self, lint: L) -> Result<T, Lint> {
    match self {
      Some(v) => Ok(v),
      None => Err(Lint {
        kind: Box::new(lint),
        span: None,
        file: None,
      }),
    }
  }
}

pub trait WithSpan {
  fn span(self, span: Span) -> Self;
}

impl<T> WithSpan for std::result::Result<T, Lint> {
  fn span(self, span: Span) -> Self {
    match self {
      Ok(v) => Ok(v),
      Err(mut l) => {
        l.span = Some(span);
        Err(l)
      }
    }
  }
}

impl std::fmt::Display for Lint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "[ERR{}] {}{}",
      self.span.map(|s| format!(" {s}")).unwrap_or("".to_string()),
      self.kind.description(),
      format!("\n{}", self.kind.help().unwrap_or("".to_string()))
    )
  }
}

impl std::fmt::Debug for Lint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{{ kind: ~, span: {:?}, file {:?} }}",
      self.span, self.file
    )
  }
}

pub trait LintKind: DynClone {
  fn help(&self) -> Option<String>;
  fn description(&self) -> String;
}

dyn_clone::clone_trait_object!(LintKind);

#[derive(Debug, Clone)]
pub enum TokenLint {
  InvalidGlyph,
  UnknownEscape(char),
  InvalidAsciiEscape,
  InvalidUnicodeEscape,
  UnclosedDelimeter(char),
  GlyphTooLong,
}

impl LintKind for TokenLint {
  fn help(&self) -> Option<String> {
    Some(match self {
      TokenLint::InvalidGlyph => {
        "The input may be corrupted, or have an unexpected encoding".to_string()
      }
      TokenLint::UnknownEscape(_) => "The recognized escape sequences are:
  * \\n - new line
  * \\r - carriage return
  * \\t - tab
  * `\"\\b - back space
  * \\ - backslash
  * \\\" - double quote
  * \\\' - single quote
  * \\aXX - ASCII escape (1 byte)
  * \\uXXXX - Unicode escape (2 bytes)"
        .to_string(),
      TokenLint::InvalidAsciiEscape => {
        "An ASCII escape sequence must contain two hex digits in the range [0x00, 0x7F]".to_string()
      }
      TokenLint::InvalidUnicodeEscape => {
        "A Unicode escape sequence must contain four hex digits in the range [0x0000, 0x7FFF]"
          .to_string()
      }
      TokenLint::UnclosedDelimeter(_) => return None,
      TokenLint::GlyphTooLong => {
        "To write a string literal instead, use \" instead of \'".to_string()
      }
    })
  }

  fn description(&self) -> String {
    match self {
      TokenLint::InvalidGlyph => "Input is not valid UTF-8".to_string(),
      TokenLint::UnknownEscape(c) => format!("`\"\\{c}\"` is not a recognized escape sequence"),
      TokenLint::InvalidAsciiEscape => {
        format!("Invalid ASCII escape sequence")
      }
      TokenLint::InvalidUnicodeEscape => {
        format!("Invalid Unicode escape sequence")
      }
      TokenLint::UnclosedDelimeter(token) => format!("This {token} is never closed"),
      TokenLint::GlyphTooLong => {
        "A glyph literal must contain a single UTF-8 glyph or escape sequence".to_string()
      }
    }
  }
}
