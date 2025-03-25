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
    let operand = if op.assoc() == RIGHT_ASSOC {
      None
    } else {
      skip(iter, 1);
      expression(iter, op.precedence())?
    }
    .ok_or(lint(ParseLint::BadPostfix, span, &[op.to_string()]))?;
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
    // Unary postfix
    if let Ok(op) = UnaryOp::try_from(&next.0) {
      let new_precedence = op.precedence();
      if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
        || (new_precedence < precedence)
      {
        return Ok(Some(current));
      }
      skip(iter, 1);
      if op.assoc() == LEFT_ASSOC {
        return Err(lint(ParseLint::BadPrefix, span, &[op.to_string()]));
      }
      current = Expression {
        span: span + current.span,
        kind: e::Unary {
          op,
          child: current.into(),
        },
      };
    }
    // Function call
    else if t::LeftParen == next.0 {
      let arguments = expression(iter, 0)?.unwrap();
      current = Expression {
        span: span + arguments.span,
        kind: e::FunctionCall {
          callee: current.into(),
          arguments: arguments.into(),
        },
      }
    }
    // Binary operator
    else if let Ok(op) = BinaryOp::try_from(&next.0) {
      let new_precedence = op.precedence();
      if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
        || (new_precedence < precedence)
      {
        return Ok(Some(current));
      }
      skip(iter, 1);
      // Some operators are allowed to eat whitespace
      if op == BinaryOp::Comma || op == BinaryOp::Equal || op == BinaryOp::Colon
      {
        eat_ws(iter);
      }
      // Allow trailing comma
      if op == BinaryOp::Comma {
        let next = iter.peek_nth(0).map(|t| t.0.clone());
        if matches!(next, Some(t::RightParen))
          || matches!(next, Some(t::RightBrace))
          || matches!(next, Some(t::RightSquare))
          || matches!(next, Some(t::Semicolon))
          || matches!(next, Some(t::EOF))
          || matches!(next, None)
        {
          return Ok(Some(current));
        }
      }
      let rhs = expression(iter, new_precedence)?.ok_or(lint(
        ParseLint::BadInfix,
        span,
        &[format!("{op}")],
      ))?;
      current = Expression {
        span: span + rhs.span,
        kind: e::Binary {
          op,
          left: current.into(),
          right: rhs.into(),
        },
      }
    } else {
      break;
    }
  }
  Ok(Some(current))
}
