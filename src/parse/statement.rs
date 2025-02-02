use super::*;

#[derive(Debug, Clone)]
pub enum StatementKind {
  Declaration {
    name: String,
    type_str: Option<String>,
    value: Expression,
    is_constant: bool,
  },
  Expression(Expression),
  Remainder(Expression),
  Error(Diagnostic),
}

#[derive(Debug, Clone)]
pub struct Statement {
  pub kind: StatementKind,
  pub span: Span,
}

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
  pub fn statement(&mut self) -> Result<Statement> {
    use StatementKind as s;
    use TokenKind as t;
    while let Ok(_) = self.eat(t::Semicolon) {}
    let next = self.peek(0)?;
    let next2 = self.peek(1);
    let mut span = next.1;
    let statement = match (next, next2) {
      // (im)mutable declaration
      (Token(t::Identifier(name), span2), Ok(Token(t::Colon, span3))) => {
        self.skip(2);
        span = span + span2 + span3;
        let type_str = match self.eat(t::Identifier("".into())) {
          Ok(Token(t::Identifier(s), span2)) => {
            span = span + span2;
            Some(s)
          },
          _ => None,
        };
        let is_constant = if self.eat(t::Equal).is_ok() {
          false
        } else if self.eat(t::Colon).is_ok() {
          true
        } else {
          return error!("Declaration of '{name}' must be initialized")
            .span(&span);
        };
        let value = self
          .expression(0)
          .trace_span(span, "while parsing declaration")?;
        span = span + value.span;
        let no_semicolon =
          if let ExpressionKind::FunctionDef { .. } = value.kind {
            true
          } else if let ExpressionKind::StructDef(_) = value.kind {
            true
          } else {
            false
          };
        let s = Statement {
          kind: s::Declaration {
            name,
            type_str,
            value,
            is_constant,
          },
          span,
        };
        if no_semicolon {
          return Ok(s);
        }
        s
      },
      // Assignment
      (Token(t::Identifier(name), span2), Ok(Token(t::Equal, span3))) => {
        self.skip(2);
        span = span + span2 + span3;
        let value = self
          .expression(0)
          .trace_span(span, "while parsing assignment")?;
        Statement {
          span,
          kind: s::Declaration {
            name,
            type_str: None,
            value,
            is_constant: false,
          },
        }
      },
      // Expression
      (Token(_, span2), _) => {
        span = span + span2;
        let expr = self
          .expression(0)
          .trace_span(span, "while parsing expression statement")?;
        span = span + expr.span;
        if self.look(0, t::RightBrace).is_ok() {
          return Ok(Statement {
            span,
            kind: s::Remainder(expr),
          });
        } else {
          use ExpressionKind as e;
          // Optional semicolon for some expressions
          match expr.kind {
            e::Block(..) | e::If { .. } => {
              return Ok(Statement {
                span,
                kind: s::Expression(expr),
              });
            },
            _ => Statement {
              span,
              kind: s::Expression(expr),
            },
          }
        }
      },
    };
    // Check for semicolon
    if self.eat(t::Semicolon).is_ok() {
      Ok(statement)
    } else {
      error!("Expected ;").span(&span)
    }
  }
}
