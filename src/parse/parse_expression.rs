use super::*;

pub fn expression(
  iter: &mut MultiPeek<impl Iterator<Item = Token>>,
  precedence: usize,
) -> Result<Option<Expression>> {
  use ExpressionKind as e;
  use TokenKind as t;
  let next = iter.peek_nth(0).unwrap();
  let span = next.1;
  // Unary prefix
  let mut current = if let Ok(op) = UnaryOp::try_from(&next.0) {
    if op.assoc() == RIGHT_ASSOC {
      return Err(lint(ParseLint::BadPostfix, span, &[op.to_string()]));
    }
    skip(iter, 1);
    let operand = expression(iter, op.precedence())?.ok_or(lint(
      ParseLint::BadPrefix,
      span,
      &[op.to_string()],
    ))?;
    let op =
      if let (UnaryOp::Minus, ExpressionKind::Literal(Literal::Real(_))) =
        (op, &operand.kind)
      {
        UnaryOp::MinusDot
      } else {
        op
      };
    Expression {
      span: span + operand.span,
      kind: e::Unary {
        op,
        child: operand.into(),
      },
    }
  }
  // Primary
  else {
    match primary(iter)? {
      Some(p) => p,
      None => return Ok(None),
    }
  };
  // Precedence climbing loop
  while let Some(next) = iter.peek_nth(0)
    && next.0 != t::EOF
  {
    // Binary operator
    if let Ok(op) = BinaryOp::try_from(&next.0) {
      let new_precedence = op.precedence();
      if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
        || (new_precedence < precedence)
      {
        return Ok(Some(current));
      }
      let op_span = next.1;
      skip(iter, 1);
      let rhs = expression(iter, new_precedence)?.ok_or(lint(
        ParseLint::BadInfix,
        op_span,
        &[format!("{op}")],
      ))?;
      current = Expression {
        span: span + rhs.span,
        kind: e::Binary {
          op,
          left: current.into(),
          right: rhs.into(),
        },
      };
    }
    // Function call
    else if t::EOF != next.0
      && t::RightParen != next.0
      && t::RightSquare != next.0
      && t::RightBrace != next.0
      && precedence < CALL_PREC
    {
      let arguments = match expression(iter, CALL_PREC)? {
        Some(a) => a,
        None => break,
      };
      current = Expression {
        span: span + arguments.span,
        kind: e::FunctionCall {
          callee: current.into(),
          arguments: arguments.into(),
        },
      }
    } else {
      break;
    }
  }
  Ok(Some(current))
}
