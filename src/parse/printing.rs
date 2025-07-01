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
      Let {
        is_type,
        is_recursive: false,
        assignee_span,
        assignee,
        value,
        in_,
      } => {
        if let Some(in_) = in_ {
          sexpr(
            "let",
            &[
              sexpr("=", &[value.as_ref().into()]),
              sexpr("in", &[in_.as_ref().into()]),
            ],
          )
        } else {
          sexpr("let", &[sexpr("=", &[value.as_ref().into()])])
        }
      },
      Let {
        is_type,
        is_recursive: true,
        assignee_span,
        assignee,
        value,
        in_,
      } => {
        if let Some(in_) = in_ {
          sexpr(
            "let",
            &[
              sexpr("::", &[value.as_ref().into()]),
              sexpr("in", &[in_.as_ref().into()]),
            ],
          )
        } else {
          sexpr("let", &[sexpr("::", &[value.as_ref().into()])])
        }
      },
      Literal(literal) => {
        let sexpr = sexpr(format!("{literal}"), &[]);
        sexpr
      },
      Identifier(name) => sexpr(format!("{name}"), &[]),
      Binary { op, left, right } => sexpr(
        format!("{op}"),
        &[left.as_ref().into(), right.as_ref().into()],
      ),
      Unary { op, child } => sexpr(format!("{op}"), &[child.as_ref().into()]),
      FunctionCall { callee, arguments } => {
        sexpr("call", &[callee.as_ref().into(), arguments.as_ref().into()])
      },
      If {
        predicate,
        then,
        else_,
      } => {
        if let Some(else_) = else_ {
          sexpr(
            "if",
            &[
              predicate.as_ref().into(),
              then.as_ref().into(),
              else_.as_ref().into(),
            ],
          )
        } else {
          sexpr("if", &[predicate.as_ref().into(), then.as_ref().into()])
        }
      },
      Structure { lhs, rhs, .. } => sexpr(
        "structure",
        &lhs
          .into_iter()
          .zip(rhs.into_iter())
          .map(|(r, l)| sexpr("field", &[r.as_str().into(), l.into()]))
          .collect::<Vec<_>>(),
      ),
    }
  }
}

impl std::fmt::Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let sexpr: SExpression = self.into();
    write!(f, "{sexpr}")
  }
}
