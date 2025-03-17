use super::*;

#[derive(Debug, Clone)]
pub enum StatementKind {
  Declaration {
    name: String,
    type_: Option<Expression>,
    value: Expression,
    is_constant: bool,
  },
  Expression(Expression),
  Error(Lint),
}

#[derive(Debug, Clone)]
pub struct Statement {
  pub kind: StatementKind,
  pub span: Span,
}

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn statement(&mut self) -> Result<Statement> {
    use StatementKind as s;
    use TokenKind as t;
    self.eat_newlines();
    let next = self.peek(0);
    let next2 = self.peek(1);
    let mut span = next.1;
    let statement = match (next, next2) {
      // (im)mutable declaration
      (Token(t::Identifier(name), span2), Token(t::Colon, span3)) => {
        self.skip(2);
        span = span + span2 + span3;
        let type_ = if let Some(_) = self.look(0, t::Equal) {
          None
        } else if let Some(_) = self.look(0, t::Colon) {
          None
        } else {
          Some(self.expression(0)?)
        };
        let is_constant = if self.eat(t::Equal).is_some() {
          false
        } else if self.eat(t::Colon).is_some() {
          true
        } else {
          return Err(lint(ParseLint::MissingAssignee, span, &[name.clone()]));
        };
        let value = self.expression(0).span(span)?;
        span = span + value.span;
        let s = Statement {
          kind: s::Declaration {
            name,
            type_,
            value,
            is_constant,
          },
          span,
        };
        s
      }
      // Assignment
      (Token(t::Identifier(name), span2), Token(t::Equal, span3)) => {
        self.skip(2);
        span = span + span2 + span3;
        let value = self.expression(0)?;
        Statement {
          span,
          kind: s::Declaration {
            name,
            type_: None,
            value,
            is_constant: false,
          },
        }
      }
      // Expression
      (Token(_, span2), _) => {
        span = span + span2;
        let expr = self.expression(0)?;
        span = span + expr.span;
        Statement {
          span,
          kind: s::Expression(expr),
        }
      }
    };
    // Check for semicolon
    if self.eat(t::NewLine).is_some()
      || self.eat(t::EOF).is_some()
      || self.eat(t::Semicolon).is_some()
    {
      self.eat_newlines();
      Ok(statement)
    } else {
      return Err(lint(ParseLint::MissingSemicolon, span, &[]));
    }
  }
}
