use super::*;
impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
  pub fn primary(&mut self) -> Result<Expression> {
    use ExpressionKind as e;
    use Immediate as im;
    use TokenKind as t;
    let next = self.peek(0)?;
    let mut span = next.1;
    let kind = match next.0 {
      t::IntegerLiteral(i, b) => {
        self.skip(1);
        e::Immediate(im::Integer(i, b))
      },
      t::FloatLiteral(f) => {
        self.skip(1);
        e::Immediate(im::Real(f))
      },
      t::StringLiteral(s) => {
        self.skip(1);
        e::Immediate(im::String(s))
      },
      t::GlyphLiteral(c) => {
        self.skip(1);
        e::Immediate(im::Glyph(c))
      },
      t::True => {
        self.skip(1);
        e::Immediate(im::Boolean(true))
      },
      t::False => {
        self.skip(1);
        e::Immediate(im::Boolean(false))
      },
      t::Break => {
        self.skip(1);
        let expr = if let Ok(Token(t::Semicolon, _)) = self.peek(0) {
          None
        } else {
          Some(Box::new(
            self
              .expression(0)
              .trace_span(span, "While parsing break expression")?,
          ))
        };
        e::Break { expr: expr.into() }
      },
      t::If => return self.if_else(),
      t::LeftBrace => {
        let (block, span1) = self.block()?;
        span = span + span1;
        e::Block(block)
      },
      // Function definition
      t::LeftParen
        if (self.look(1, t::Identifier("".into())).is_ok()
          && (self.look(2, t::Colon).is_ok()
            || self.look(2, t::Comma).is_ok()))
          || self.look(1, t::RightParen).is_ok() =>
      {
        self.skip(1);
        let params = self.parameters(span)?;
        let Token(_, span2) = self
          .eat(t::RightParen)
          .span(&span)
          .trace_span(span, "while parsing function definition")?;
        let returns = if let Ok(Token(_, span2)) = self.eat(t::Arrow) {
          span = span + span2;
          let expr = self
            .expression(0)
            .trace_span(span, "while parsing function return type")?;
          span = span + span2;
          Some(expr.into())
        } else {
          None
        };
        span = span + span2;
        if returns.is_none()
          && params.arity == 0
          && self.peek(0).is_ok_and(|t| t.0 != t::LeftBrace)
        {
          e::Immediate(Immediate::Unit)
        } else {
          let (body, span2) = self
            .block()
            .trace_span(span, "while parsing function body")?;
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
      },
      // Struct definition
      t::Struct => {
        self.skip(1);
        self.eat(t::LeftBrace)?;
        let params = self.parameters(span)?;
        self.eat(t::RightBrace)?;
        e::StructDef(params)
      },
      // Anonymous struct initialization
      t::Dot if matches!(self.peek(1), Ok(Token(t::LeftBrace, _))) => {
        self.skip(2);
        let params = self.parameters(span)?;
        self.eat(t::RightBrace)?;
        e::StructLiteral {
          struct_t: None,
          parameters: params,
        }
      },
      t::Identifier(i) => {
        self.skip(1);
        e::Identifier { name: i }
      },
      // Loop
      t::Loop => {
        self.skip(1);
        let params = if let Ok(Token(t::LeftBrace, _)) = self.peek(0) {
          Parameters::default()
        } else {
          self.parameters(span)?
        };
        let (body, span2) =
          self.block().trace_span(span, "while parsing loop body")?;
        let body = Box::new(Expression {
          kind: ExpressionKind::Block(body),
          span: span2,
        });
        span = span + span2;
        e::Loop {
          parameters: params,
          body,
        }
      },
      // Parenthetical
      t::LeftParen => {
        self.skip(1);
        let expr = self
          .expression(0)
          .trace("while parsing parenthesized expression")?;
        self
          .eat(t::RightParen)
          .reason("Unclosed '('")
          .span(&expr.span)?;
        e::Parenthesis(expr.into())
      },
      _ => {
        return error!("Expected expression, found {}", next.0).span(&span);
      },
    };
    Ok(Expression::new(kind, span))
  }
}
