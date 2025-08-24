mod color;
pub mod kinds;
pub mod render;
pub mod span;

pub(super) use color::*;
pub use kinds::*;
pub use span::*;

use crate::{compiler_print, render::Linter};

pub type Result<T> = std::result::Result<T, Lint>;

#[derive(Clone, Debug)]
pub struct Lint {
    pub kind: usize,
    pub context: Vec<String>,
    pub span: Option<Span>,
}

pub fn lint(kind: impl Into<usize>, span: Span, context: impl IntoIterator<Item = String>) -> Lint {
    Lint {
        kind: kind.into(),
        context: context.into_iter().collect(),
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

pub trait ResidualLint<T> {
    fn lint(self, lint: impl Into<usize>) -> Result<T>;
}

impl<T, E> ResidualLint<T> for std::result::Result<T, E> {
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

impl<T> ResidualLint<T> for std::option::Option<T> {
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

pub trait ResidualSpan {
    fn span(self, span: Span) -> Self;
}
impl<T> ResidualSpan for std::result::Result<T, Lint> {
    fn span(self, span: Span) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(mut l) => {
                l.span = l.span.or(Some(span));
                Err(l)
            }
        }
    }
}

pub trait ResidualContext {
    fn context(self, ctx: impl Into<String>) -> Self;
}

impl<T> ResidualContext for std::result::Result<T, Lint> {
    fn context(self, parameter: impl Into<String>) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(mut l) => {
                l.context.push(parameter.into());
                Err(l)
            }
        }
    }
}

pub trait Handle<T> {
    fn handle(self, linter: &Linter) -> Option<T>;
}

impl<T> Handle<T> for Result<T> {
    fn handle(self, linter: &Linter) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                compiler_print(
                    "Failed to Compile"
                        .apply_style(Color::Red, Attribute::Underline)
                        .to_string(),
                );
                compiler_print(linter.render(e).to_string());
                None
            }
        }
    }
}
