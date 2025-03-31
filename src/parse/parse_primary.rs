use super::*;

pub fn primary(
  iter: &mut MultiPeek<impl Iterator<Item = Token>>,
) -> Result<Option<Expression>> {
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
    },
    t::RealLiteral(value) => {
      skip(iter, 1);
      e::Literal(l::Real(value))
    },
    t::StringLiteral(value) => {
      skip(iter, 1);
      e::Literal(l::String(value))
    },
    t::GlyphLiteral(value) => {
      skip(iter, 1);
      e::Literal(l::Glyph(value))
    },
    t::True => {
      skip(iter, 1);
      e::Literal(l::Boolean(true))
    },
    t::False => {
      skip(iter, 1);
      e::Literal(l::Boolean(false))
    },
    t::Identifier(name) => {
      skip(iter, 1);
      e::Identifier(name)
    },
    t::LeftBrace => {
      skip(iter, 1);
      let mut idents = vec![];
      let mut exprs = vec![];
      let mut is_definition = None;
      loop {
        if eat(iter, t::RightBrace).is_some() {
          break;
        }
        let Some(Token(t::Identifier(name), idspan)) =
          eat(iter, t::Identifier("".into()))
        else {
          return Err(lint(ParseLint::InvalidStructure, span, &[]));
        };
        span += idspan;
        idents.push(name.clone());
        match is_definition {
          Some(true) => {
            eat(iter, t::Colon).ok_or(lint(
              ParseLint::InvalidStructure,
              span,
              &[],
            ))?;
          },
          Some(false) => {
            eat(iter, t::Equal).ok_or(lint(
              ParseLint::InvalidStructure,
              span,
              &[],
            ))?;
          },
          None if peek(iter, 0, t::Equal).is_some() => {
            is_definition = Some(false);
            skip(iter, 1);
          },
          None if peek(iter, 0, t::Colon).is_some() => {
            is_definition = Some(true);
            skip(iter, 1);
          },
          _ => return Err(lint(ParseLint::InvalidStructure, span, &[])),
        };
        exprs.push(expression(iter, 6)?.ok_or(lint(
          ParseLint::InvalidStructure,
          span,
          &[],
        ))?);
        if eat(iter, t::Comma).is_none() {
          eat(iter, t::RightBrace).ok_or(lint(
            ParseLint::MissingComma,
            span,
            &[],
          ))?;
          break;
        }
      }
      if idents.len() == 0 {
        return Err(lint(ParseLint::EmptyBlock, span, &[]));
      }
      e::Structure {
        is_definition: is_definition.unwrap(),
        lhs: idents,
        rhs: exprs,
      }
    },
    // Control flow
    t::If => {
      skip(iter, 1);
      let predicate = match expression(iter, 0)? {
        Some(p) => p,
        None => return Err(lint(ParseLint::InvalidIf, span, &[])),
      };
      span += eat(iter, t::Then)
        .ok_or(lint(ParseLint::InvalidIf, span, &[]))?
        .1;
      let then_branch = match expression(iter, 0)? {
        Some(t) => t,
        None => return Err(lint(ParseLint::InvalidIf, span, &[])),
      };
      span += then_branch.span;
      let else_branch = if let Some(tok) = eat(iter, t::Else) {
        span += tok.1;
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
    },
    // Parenthesis
    t::LeftParen => {
      skip(iter, 1);
      // Unit literal
      if let Some(tok) = eat(iter, t::RightParen) {
        span += tok.1;
        return Ok(Some(Expression {
          kind: e::Literal(Literal::Unit),
          span,
        }));
      }
      // Wrapped expression
      let inner = expression(iter, 0)?.ok_or(lint(
        TokenLint::MissingDelimeter,
        span,
        &[")".into()],
      ))?;
      span += inner.span;
      match eat(iter, t::RightParen) {
        Some(t) => span += t.1,
        None => {
          return Err(lint(
            TokenLint::MissingDelimeter,
            span,
            &[")".to_string()],
          ));
        },
      };
      inner.kind
    },
    _ => return Ok(None),
  };
  Ok(Some(Expression { kind, span }))
}
