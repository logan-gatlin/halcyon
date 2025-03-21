use super::*;

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn primary(&mut self) -> Result<Expression> {
    use ExpressionKind as e;
    use Immediate as im;
    use TokenKind as t;
    let next = self.peek(0);
    let mut span = next.1;
    let kind = match next.0 {
      t::IntegerLiteral(i, b) => {
        self.skip(1);
        e::Immediate(im::Integer(i, b))
      }
      t::FloatLiteral(f) => {
        self.skip(1);
        e::Immediate(im::Real(f))
      }
      t::StringLiteral(s) => {
        self.skip(1);
        e::Immediate(im::String(s))
      }
      t::GlyphLiteral(c) => {
        self.skip(1);
        e::Immediate(im::Glyph(c))
      }
      t::True => {
        self.skip(1);
        e::Immediate(im::Boolean(true))
      }
      t::False => {
        self.skip(1);
        e::Immediate(im::Boolean(false))
      }
      t::Break => {
        self.skip(1);
        let expr = if let Token(t::NewLine, _) = self.peek(0) {
          None
        } else {
          Some(Box::new(self.expression(0)?))
        };
        e::Break { expr: expr.into() }
      }
      t::If => return self.if_else(),
      t::LeftBrace => {
        let (block, span1) = self.body("block").span(span)?;
        span = span + span1;
        e::Block(block)
      }
      // Function definition
      t::LeftParen
        if (self.look(1, t::Identifier("".into())).is_some()
          && (self.look(2, t::Colon).is_some() || self.look(2, t::Comma).is_some()))
          || self.look(1, t::RightParen).is_some() =>
      {
        self.skip(1);
        let params = self.parameters(span)?;
        let Token(_, span2) = self
          .eat(t::RightParen)
          .lint(TokenLint::MissingDelimeter)
          .context(")")
          .span(span)?;
        span += span2;
        let returns = if let Some(Token(_, span2)) = self.eat(t::Arrow) {
          span = span + span2;
          let expr = self.expression(0).span(span)?;
          span += span2;
          Some(expr.into())
        } else {
          None
        };
        span = span + span2;
        if returns.is_none() && params.arity == 0 && self.peek_not_newline().0 != t::LeftBrace {
          e::Immediate(Immediate::Unit)
        } else {
          self.eat_newlines();
          let (body, span2) = self.body("function").span(span)?;
          let body = Box::new(Expression {
            kind: e::Block(body),
            span: span2,
          });
          span = span + span2;
          e::FunctionDef {
            parameters: params,
            returns,
            body,
          }
        }
      }
      // Struct definition
      t::Struct => {
        self.skip(1);
        self
          .eat(t::LeftBrace)
          .lint(TokenLint::MissingDelimeter)
          .context("{")
          .span(span)?;
        self.eat_newlines();
        let params = self.parameters(span)?;
        for s in &params.spans {
          span += *s;
        }
        self.eat_newlines();
        self
          .eat(t::RightBrace)
          .lint(TokenLint::MissingDelimeter)
          .context("}")
          .span(span)?;
        e::StructDef(params)
      }
      // Anonymous struct initialization
      t::Dot if matches!(self.peek(1), Token(t::LeftBrace, _)) => {
        self.skip(2);
        let params = self.parameters(span)?;
        self
          .eat(t::RightBrace)
          .lint(TokenLint::MissingDelimeter)
          .context("}")?;
        e::StructLiteral {
          struct_t: None,
          parameters: params,
        }
      }
      t::Identifier(i) => {
        self.skip(1);
        e::Identifier { name: i }
      }
      // Loop
      t::Loop => {
        self.skip(1);
        let params = self
          .parameters(span)
          .lint(ParseLint::MissingLoopParameter)
          .span(span)?;
        let (body, span2) = self.body("loop").span(span)?;
        let body = Box::new(Expression {
          kind: ExpressionKind::Block(body),
          span: span2,
        });
        span = span + span2;
        e::Loop {
          parameters: params,
          body,
        }
      }
      // Parenthetical
      t::LeftParen => {
        self.skip(1);
        let expr = self.expression(0).span(span)?;
        self
          .eat(t::RightParen)
          .lint(TokenLint::MissingDelimeter)
          .context(")")
          .span(expr.span)?;
        e::Parenthesis(expr.into())
      }
      _ => {
        return Err(lint(ParseLint::UnexpectedToken, self.last_span, &[]));
      }
    };
    Ok(Expression::new(kind, span))
  }
}
