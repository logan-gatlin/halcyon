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
    // Function definition
    t::Fn => {
      skip(iter, 1);
      let mut export_name = None;
      let mut arguments = vec![];
      let mut argument_spans = vec![];
      let mut types = vec![];
      // Directive
      if let Some(t) = eat(iter, t::Hash) {
        span += t.1;
        let Some(Token(t::Identifier(name), span2)) = iter.next() else {
          return Err(lint(ParseLint::InvalidDirective, span, &[]));
        };
        span += span2;
        if name != "export" {
          return Err(lint(ParseLint::InvalidDirective, span, &[]));
        }
        span += eat(iter, t::LeftParen)
          .ok_or(lint(ParseLint::InvalidDirective, span, &[]))?
          .1;
        let Some(Token(t::StringLiteral(name), span2)) = iter.next() else {
          return Err(lint(ParseLint::InvalidDirective, span, &[]));
        };
        export_name = Some(name);
        span += span2;
        span += eat(iter, t::RightParen)
          .ok_or(lint(ParseLint::InvalidDirective, span, &[]))?
          .1;
      }
      loop {
        // End of arguments
        if let Some(t) = eat(iter, t::FatArrow) {
          span += t.1;
          break;
        }
        // Argument
        else if let Some(Token(t::Identifier(s), new_span)) =
          eat(iter, t::Identifier("".to_string()))
        {
          arguments.push(s);
          argument_spans.push(new_span);
          types.push(None);
          span += new_span;
        }
        // Type qualified argument
        else if let Some(Token(_, mut new_span)) = eat(iter, t::LeftParen) {
          let Some(TokenKind::Identifier(argument)) =
            eat(iter, t::Identifier("".into())).map(|t| {
              new_span += t.1;
              t.0
            })
          else {
            return Err(lint(ParseLint::InvalidFunctionArgument, new_span, &[]));
          };
          arguments.push(argument);
          new_span += eat(iter, t::Colon)
            .ok_or(lint(ParseLint::InvalidFunctionArgument, new_span, &[]))?
            .1;
          let type_ =
            expression(iter, 1)?.ok_or(lint(ParseLint::InvalidFunctionArgument, new_span, &[]))?;
          types.push(Some(type_));
          new_span += eat(iter, t::RightParen)
            .ok_or(lint(ParseLint::InvalidFunctionArgument, new_span, &[]))?
            .1;
          argument_spans.push(new_span);
          span += new_span;
        }
        // Invalid argument
        else if let Some(Token(kind, span)) = iter.peek_nth(0) {
          return Err(lint(
            ParseLint::InvalidFunctionArgument,
            *span,
            &[format!("{kind}")],
          ));
        }
        // End of input
        else {
          return Err(lint(ParseLint::ExpectedExpression, span, &[]));
        }
      }
      let body = expression(iter, 1)?.ok_or(lint(ParseLint::ExpectedExpression, span, &[]))?;
      e::FunctionDef {
        export_name,
        arguments,
        argument_spans,
        types,
        body: body.into(),
      }
    }
    // Declaration
    t::Let | t::Type => {
      let is_type = iter.next().unwrap().0 == t::Type;
      let l = lint(ParseLint::InvalidLet, span, &[]);
      let Some(Token(t::Identifier(assignee), span2)) = iter.peek_nth(0).cloned() else {
        return Err(l.clone());
      };
      let assignee_span = span2;
      span += span2;
      skip(iter, 1);
      let is_recursive = if peek(iter, 0, t::Equal).is_some() {
        false
      } else if peek(iter, 0, t::DoubleColon).is_some() {
        true
      } else {
        return Err(l.clone());
      };
      skip(iter, 1);
      let value = expression(iter, 0)?.ok_or(l.clone())?;
      span += value.span;
      let in_ = if peek(iter, 0, t::In).is_some() {
        skip(iter, 1);
        let expr = expression(iter, 0)?.ok_or(l)?;
        span += expr.span;
        Some(Box::new(expr))
      } else {
        None
      };
      e::Let {
        is_type,
        is_recursive,
        assignee_span,
        assignee,
        value: Box::new(value),
        in_,
      }
    }
    // Struct def and lit
    t::LeftBrace => {
      skip(iter, 1);
      let mut idents = vec![];
      let mut exprs = vec![];
      let mut is_definition = None;
      loop {
        if eat(iter, t::RightBrace).is_some() {
          break;
        }
        let Some(Token(t::Identifier(name), idspan)) = eat(iter, t::Identifier("".into())) else {
          return Err(lint(ParseLint::InvalidStructure, span, &[]));
        };
        span += idspan;
        idents.push(name.clone());
        match is_definition {
          Some(true) => {
            eat(iter, t::Colon).ok_or(lint(ParseLint::InvalidStructure, span, &[]))?;
          }
          Some(false) => {
            eat(iter, t::Equal).ok_or(lint(ParseLint::InvalidStructure, span, &[]))?;
          }
          None if peek(iter, 0, t::Equal).is_some() => {
            is_definition = Some(false);
            skip(iter, 1);
          }
          None if peek(iter, 0, t::Colon).is_some() => {
            is_definition = Some(true);
            skip(iter, 1);
          }
          _ => return Err(lint(ParseLint::InvalidStructure, span, &[])),
        };
        exprs.push(expression(iter, 6)?.ok_or(lint(ParseLint::InvalidStructure, span, &[]))?);
        if eat(iter, t::Comma).is_none() {
          eat(iter, t::RightBrace).ok_or(lint(ParseLint::MissingComma, span, &[]))?;
          break;
        }
      }
      if idents.len() == 0 {
        e::Literal(Literal::Unit)
      } else {
        e::Structure {
          is_definition: is_definition.unwrap(),
          lhs: idents,
          rhs: exprs,
        }
      }
    }
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
    }
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
      let inner =
        expression(iter, 0)?.ok_or(lint(TokenLint::MissingDelimeter, span, &[")".into()]))?;
      span += inner.span;
      match eat(iter, t::RightParen) {
        Some(t) => span += t.1,
        None => {
          return Err(lint(TokenLint::MissingDelimeter, span, &[")".to_string()]));
        }
      };
      inner.kind
    }
    _ => return Ok(None),
  };
  Ok(Some(Expression { kind, span }))
}
