mod color;
pub mod kinds;
pub mod render;
pub mod sexpr;
pub mod span;

pub(super) use color::*;
pub use kinds::*;
pub use sexpr::*;
pub use span::*;

pub type Result<T> = std::result::Result<T, Lint>;

#[derive(Clone, Debug)]
pub struct Lint {
  pub kind: usize,
  pub context: Vec<String>,
  pub span: Option<Span>,
}

pub fn lint(kind: impl Into<usize>, span: Span, context: &[String]) -> Lint {
  Lint {
    kind: kind.into(),
    context: context.to_vec(),
    span: Some(span),
  }
}

pub fn lint_nospan(kind: impl Into<usize>) -> Lint {
  Lint {
    kind: kind.into(),
    span: None,
    context: vec![],
  }
}

pub trait OrLint<T> {
  fn lint(self, lint: impl Into<usize>) -> Result<T>;
}

impl<T, E> OrLint<T> for std::result::Result<T, E> {
  fn lint(self, lint: impl Into<usize>) -> Result<T> {
    match self {
      Ok(v) => Ok(v),
      Err(_) => Err(Lint {
        kind: lint.into(),
        context: vec![],
        span: None,
      }),
    }
  }
}

impl<T> OrLint<T> for std::option::Option<T> {
  fn lint(self, lint: impl Into<usize>) -> Result<T> {
    match self {
      Some(v) => Ok(v),
      None => Err(Lint {
        kind: lint.into(),
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
