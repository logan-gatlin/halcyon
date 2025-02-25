use crate::Span;

pub type Result<T> = std::result::Result<T, Diagnostic>;

#[derive(Clone)]
pub struct Diagnostic {
  pub reason: String,
  pub span: Option<Span>,
  pub backtrace: Vec<String>,
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
    Err(crate::err::Diagnostic {
      reason: format!($($arg)*),
      span: None,
      backtrace: Vec::new(),
    })
  };
}
#[macro_export]
macro_rules! diagnostic {
  ($($arg:tt)*) => {
    crate::err::Diagnostic {
      reason: format!($($arg)*),
      span: None,
      backtrace: Vec::new(),
    }
  };
}

impl Diagnostic {
  pub fn new(reason: impl Into<String>, span: Option<Span>) -> Self {
    Self {
      reason: reason.into(),
      span,
      backtrace: vec![],
    }
  }
}

impl std::fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(span) = self.span.as_ref() {
      write!(f, "({}:{}) {}\n", span.row, span.column, self.reason)?;
    } else {
      write!(f, "(E) {}\n", self.reason)?;
    }
    for (_i, b) in self.backtrace.iter().enumerate() {
      write!(f, "> {}\n", b)?;
    }
    Ok(())
  }
}

impl std::fmt::Debug for Diagnostic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}")
  }
}

impl From<std::io::Error> for Diagnostic {
  fn from(value: std::io::Error) -> Self {
    Self {
      reason: format!("{}", value),
      span: None,
      backtrace: vec![],
    }
  }
}

impl From<std::num::ParseIntError> for Diagnostic {
  fn from(value: std::num::ParseIntError) -> Self {
    use std::num::IntErrorKind::*;
    match value.kind() {
      PosOverflow | NegOverflow => {
        Diagnostic::new("Integer value is too large to represent", None)
      },
      InvalidDigit => {
        Diagnostic::new("Integer value containts invalid digits", None)
      },
      _ => Diagnostic::new("Integer value could not be parsed", None),
    }
  }
}

impl From<std::num::ParseFloatError> for Diagnostic {
  fn from(_value: std::num::ParseFloatError) -> Self {
    Diagnostic::new("Float value could not be parsed", None)
  }
}

pub trait IntoDiagnostic<T, S: Into<String>> {
  fn reason(self, s: S) -> Result<T>;
  fn trace(self, s: S) -> Result<T>;
  fn trace_span(self, span: Span, s: S) -> Result<T>;
}

pub trait WithSpan<T> {
  fn span(self, span: &Span) -> Result<T>;
}

impl<T> WithSpan<T> for Result<T> {
  fn span(self, span: &Span) -> Result<T> {
    self.map_err(|mut e| {
      e.span = e.span.or(Some(span.clone()));
      e
    })
  }
}

impl<T, S: Into<String>> IntoDiagnostic<T, S> for Option<T> {
  fn reason(self, s: S) -> Result<T> {
    match self {
      Some(t) => Ok(t),
      None => Err(Diagnostic {
        reason: s.into(),
        span: None,
        backtrace: vec![],
      }),
    }
  }

  fn trace(self, s: S) -> Result<T> {
    match self {
      Some(t) => Ok(t),
      None => Err(Diagnostic {
        reason: "".into(),
        span: None,
        backtrace: vec![s.into()],
      }),
    }
  }

  fn trace_span(self, span: Span, s: S) -> Result<T> {
    self.trace(format!("{} {}", span, s.into()))
  }
}

impl<T, E: Into<Diagnostic>, S: Into<String>> IntoDiagnostic<T, S>
  for std::result::Result<T, E>
{
  fn reason(self, s: S) -> Result<T> {
    self.map_err(|e| e.into()).map_err(|mut e| {
      if e.reason == "" {
        e.reason = s.into();
      }
      e
    })
  }

  fn trace(self, s: S) -> Result<T> {
    self.map_err(|e| e.into()).map_err(|mut e| {
      e.backtrace.push(s.into());
      e
    })
  }

  fn trace_span(self, span: Span, s: S) -> Result<T> {
    self.trace(format!("{} {}", span, s.into())).span(&span)
  }
}
