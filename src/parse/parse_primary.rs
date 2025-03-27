use super::*;

pub fn primary(iter: &mut MultiPeek<impl Iterator<Item = Token>>) -> Result<Option<Expression>> {
  use ExpressionKind as e;
  use Literal as l;
  use TokenKind as t;
  let next = match iter.peek_nth(0) {
    Some(n) => n,
    None => return Ok(None),
  };
  let mut span = next.1;
  let kind = match next.0.clone() {
    // Literals
    t::IntegerLiteral(value, base) => {
      skip(iter, 1);
      e::Literal(l::Integer(value, base))
    }
    t::RealLiteral(value) => {
      skip(iter, 1);
      e::Literal(l::Real(value))
    }
    t::StringLiteral(value) => {
      skip(iter, 1);
      e::Literal(l::String(value))
    }
    t::GlyphLiteral(value) => {
      skip(iter, 1);
      e::Literal(l::Glyph(value))
    }
    t::True => {
      skip(iter, 1);
      e::Literal(l::Boolean(true))
    }
    t::False => {
      skip(iter, 1);
      e::Literal(l::Boolean(false))
    }
    t::Identifier(name) => {
      skip(iter, 1);
      e::Identifier(name)
    }
    // Control flow
    t::Pipe => {
      let mut predicates = vec![];
      let mut branches = vec![];
      let mut else_branch = None;
      loop {
        skip(iter, 1);
        if peek(iter, 0, t::Else).is_some() {
          skip(iter, 1);
          eat_ws(iter);
          else_branch = match expression(iter, 0)? {
            Some(p) => Some(Box::new(p)),
            None => return Err(lint(ParseLint::InvalidGuard, span, &[])),
          };
          break;
        }
        let predicate = match expression(iter, 0)? {
          Some(p) => p,
          None => return Err(lint(ParseLint::InvalidGuard, span, &[])),
        };
        eat_ws(iter);
        span += eat(iter, t::Then)
          .ok_or(lint(ParseLint::InvalidGuard, span, &[]))?
          .1;
        eat_ws(iter);
        let branch = match expression(iter, 0)? {
          Some(t) => t,
          None => return Err(lint(ParseLint::InvalidIf, span, &[])),
        };
        next_not_ws(iter);
        predicates.push(predicate);
        branches.push(branch);
        if !(peek(iter, 0, t::Pipe).is_some()
          || (peek(iter, 0, t::NewLine).is_some() && peek(iter, 1, t::Pipe).is_some()))
        {
          break;
        }
        eat_ws(iter);
      }
      e::Guard {
        predicates,
        branches,
        else_branch,
      }
    }
    t::If => {
      skip(iter, 1);
      let predicate = match expression(iter, 0)? {
        Some(p) => p,
        None => return Err(lint(ParseLint::InvalidIf, span, &[])),
      };
      eat_ws(iter);
      span += eat(iter, t::Then)
        .ok_or(lint(ParseLint::InvalidIf, span, &[]))?
        .1;
      eat_ws(iter);
      let then_branch = match expression(iter, 0)? {
        Some(t) => t,
        None => return Err(lint(ParseLint::InvalidIf, span, &[])),
      };
      span += then_branch.span;
      next_not_ws(iter);
      // Allow one newline before `else`
      if let Some(t1) = iter.peek_nth(0).cloned()
        && let Some(t2) = iter.peek_nth(1).cloned()
        && t1.0 == t::NewLine
        && t2.0 == t::Else
      {
        eat_ws(iter);
      }
      let else_branch = if let Some(tok) = eat(iter, t::Else) {
        span += tok.1;
        eat_ws(iter);
        let else_branch = match expression(iter, 0)? {
          Some(e) => e,
          None => return Ok(None),
        };
        Some(Box::new(else_branch))
      } else {
        None
      };
      e::If {
        predicate: predicate.into(),
        then: then_branch.into(),
        else_: else_branch,
      }
    }
    t::Loop => {
      skip(iter, 1);
      let parameters = expression(iter, 0)?.ok_or(lint(ParseLint::InvalidLoop, span, &[]))?;
      eat_ws(iter);
      let body = expression(iter, 0)?.ok_or(lint(ParseLint::InvalidLoop, span, &[]))?;
      e::Loop {
        parameters: parameters.into(),
        body: body.into(),
      }
    }
    // Parenthesis
    t::LeftParen => {
      skip(iter, 1);
      eat_ws(iter);
      // Unit literal
      if let Some(tok) = eat(iter, t::RightParen) {
        span += tok.1;
        return Ok(Some(Expression {
          kind: e::Literal(Literal::Unit),
          span,
        }));
      }
      // Wrapped expression
      let inner =
        expression(iter, 0)?.ok_or(lint(TokenLint::MissingDelimeter, span, &[")".into()]))?;
      span += inner.span;
      eat_ws(iter);
      match eat(iter, t::RightParen) {
        Some(t) => span += t.1,
        None => {
          return Err(lint(TokenLint::MissingDelimeter, span, &[")".to_string()]));
        }
      };
      inner.kind
    }
    t::LeftBrace => {
      skip(iter, 1);
      let mut block_span = span;
      let mut exprs: Vec<Expression> = vec![];
      loop {
        eat_ws(iter);
        if let Some(Token(t::RightBrace, brace_span)) = iter.peek_nth(0).cloned() {
          skip(iter, 1);
          block_span += brace_span;
          break;
        }
        let expr = expression(iter, 0)?
          .ok_or(lint(TokenLint::MissingDelimeter, block_span, &["}".into()]))?;
        exprs.push(expr);
      }
      if exprs.len() == 0 {
        return Err(lint(ParseLint::EmptyBlock, block_span, &[]));
      }
      e::Block(exprs)
    }
    _ => return Ok(None),
  };
  Ok(Some(Expression { kind, span }))
}
