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

impl Into<SExpression> for &Expression {
  fn into(self) -> SExpression {
    use ExpressionKind::*;
    match &self.kind {
      Literal(literal) => sexpr(format!("{literal}"), &[]),
      Identifier { name } => sexpr(format!("{name}"), &[]),
      Binary { op, left, right } => sexpr(
        format!("{op}"),
        &[left.as_ref().into(), right.as_ref().into()],
      ),
      Unary { op, child } => sexpr(format!("{op}"), &[child.as_ref().into()]),
      FunctionCall { callee, arguments } => {
        sexpr(format!("{callee}"), &[arguments.as_ref().into()])
      },
      Block(expressions) => sexpr(
        "block",
        &expressions
          .into_iter()
          .map(|e| e.into())
          .collect::<Vec<_>>(),
      ),
      If {
        predicate,
        then,
        else_,
      } => {
        if let Some(else_) = else_ {
          sexpr("if", &[then.as_ref().into(), else_.as_ref().into()])
        } else {
          sexpr("if", &[then.as_ref().into()])
        }
      },
      Loop { parameters, body } => {
        sexpr("loop", &[parameters.as_ref().into(), body.as_ref().into()])
      },
    }
  }
}

impl std::fmt::Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let sexpr: SExpression = self.into();
    write!(f, "{sexpr}")
  }
}
