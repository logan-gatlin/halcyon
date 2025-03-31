#[derive(Clone)]
pub struct SExpression {
  this: String,
  children: Vec<SExpression>,
}

pub fn sexpr(this: impl Into<String>, children: &[SExpression]) -> SExpression {
  SExpression {
    this: this.into(),
    children: children.to_vec(),
  }
}

fn indent(s: impl Into<String>, indent: &str) -> String {
  let s: String = s.into();
  s.lines()
    .map(|s| format!("{indent}{s}\n"))
    .collect::<String>()
    .trim_end()
    .to_string()
}

impl std::fmt::Display for SExpression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.children.len() == 0 {
      write!(f, "{}", self.this)
    } else {
      let children = self
        .children
        .iter()
        .map(|c| format!("{c}"))
        .collect::<Vec<_>>()
        .join("\n");
      write!(f, "({}\n{}\n)", self.this, indent(children, "  "))
    }
  }
}

impl Into<SExpression> for &str {
  fn into(self) -> SExpression {
    sexpr(format!("{self}"), &[])
  }
}
