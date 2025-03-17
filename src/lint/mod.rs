pub mod render;
pub mod span;

pub use span::*;

use crate::token::{Token, TokenKind};

pub type LintKind = usize;
pub type Result<T> = std::result::Result<T, Lint>;

#[derive(Clone, Debug)]
pub struct Lint {
  pub kind: LintKind,
  pub context: Vec<String>,
  pub span: Option<Span>,
}

pub fn lint(kind: LintKind, span: Span, context: &[String]) -> Lint {
  Lint {
    kind: kind.into(),
    context: context.to_vec(),
    span: Some(span),
  }
}

pub fn lint_nospan(kind: LintKind) -> Lint {
  Lint {
    kind: kind.into(),
    span: None,
    context: vec![],
  }
}

pub trait OrLint<T> {
  fn lint(self, lint: LintKind) -> Result<T>;
}

impl<T, E> OrLint<T> for std::result::Result<T, E> {
  fn lint(self, lint: LintKind) -> Result<T> {
    match self {
      Ok(v) => Ok(v),
      Err(_) => Err(Lint {
        kind: lint,
        context: vec![],
        span: None,
      }),
    }
  }
}

impl<T> OrLint<T> for std::option::Option<T> {
  fn lint(self, lint: LintKind) -> Result<T> {
    match self {
      Some(v) => Ok(v),
      None => Err(Lint {
        kind: lint,
        context: vec![],
        span: None,
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
        l.span = l.span.or(Some(span));
        Err(l)
      },
    }
  }
}

pub trait WithContext {
  fn context(self, ctx: impl Into<String>) -> Self;
}

impl<T> WithContext for std::result::Result<T, Lint> {
  fn context(self, parameter: impl Into<String>) -> Self {
    match self {
      Ok(v) => Ok(v),
      Err(mut l) => {
        l.context.push(parameter.into());
        Err(l)
      },
    }
  }
}
