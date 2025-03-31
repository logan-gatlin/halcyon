use super::*;

fn match_rhs(
  iter: &mut MultiPeek<impl Iterator<Item = Token>>,
  mut span: Span,
) -> Result<(Vec<Expression>, Vec<Expression>)> {
  use TokenKind as t;
  let mut predicates = vec![];
  let mut branches = vec![];
  loop {
    span = eat(iter, t::Pipe)
      .ok_or(lint(ParseLint::InvalidMatch, span, &[]))?
      .1;
    predicates.push(expression(iter, 0)?.ok_or(lint(
      ParseLint::InvalidMatch,
      span,
      &[],
    ))?);
    span = eat(iter, t::Then)
      .ok_or(lint(ParseLint::InvalidMatch, span, &[]))?
      .1;
    branches.push(expression(iter, 0)?.ok_or(lint(
      ParseLint::InvalidMatch,
      span,
      &[],
    ))?);
    if peek(iter, 0, t::Pipe).is_none() {
      break Ok((predicates, branches));
    }
  }
}

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
    // Match expression
    else if t::Match == next.0 && precedence < MATCH_PREC {
      skip(iter, 1);
      let (predicates, branches) = match_rhs(iter, span)?;
      current = Expression {
        span,
        kind: e::Match {
          on: current.into(),
          predicates,
          branches,
        },
      }
    }
    // Loop expression
    else if t::Loop == next.0 && precedence < LOOP_PREC {
      skip(iter, 1);
      current = Expression {
        kind: e::Loop {
          parameters: current.into(),
          body: expression(iter, LOOP_PREC)?
            .ok_or(lint(ParseLint::InvalidLoop, span, &[]))?
            .into(),
        },
        span,
      }
    }
    // Unary postfix
    else if let Ok(op) = UnaryOp::try_from(&next.0) {
      let new_precedence = op.precedence();
      if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
        || (new_precedence < precedence)
      {
        return Ok(Some(current));
      }
      if op.assoc() == LEFT_ASSOC {
        return Err(lint(ParseLint::BadPrefix, next.1, &[op.to_string()]));
      }
      skip(iter, 1);
      current = Expression {
        span: span + current.span,
        kind: e::Unary {
          op,
          child: current.into(),
        },
      };
    }
    // Function call
    else if t::EOF != next.0
      && t::RightParen != next.0
      && t::RightSquare != next.0
      && t::RightBrace != next.0
      && precedence <= CALL_PREC
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
