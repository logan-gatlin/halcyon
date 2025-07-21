use super::*;

impl std::fmt::Display for Literal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Literal::Unit => "()".to_string(),
        Literal::Integer(val, _) => format!("{val}"),
        Literal::Real(val) => format!("{val}"),
        Literal::String(val) => format!("\"{val}\""),
        Literal::Glyph(val) => format!("{val}"),
        Literal::Boolean(val) => format!("{val}"),
      }
    )
  }
}
