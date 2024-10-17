use super::*;

#[derive(Clone)]
pub struct Parameter {
  name: String,
  type_: String,
}

impl std::fmt::Debug for Parameter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.name, self.type_)
  }
}

#[derive(Clone)]
pub enum ExpressionKind {
  Integer(i64),
  Real(f64),
  String(String),
  Boolean(bool),
  Identifier(String),
  Binary {
    token: TokenKind,
    left: Box<Expression>,
    right: Box<Expression>,
  },
  Unary {
    token: TokenKind,
    child: Box<Expression>,
  },
  Parenthesis(Box<Expression>),
  Function {
    params: Vec<Parameter>,
    returns: Option<String>,
    body: Vec<Statement>,
  },
  Call {
    callee: Box<Expression>,
    args: Vec<Expression>,
  },
  Field {
    namespace: Box<Expression>,
    field: Box<Expression>,
  },
}

#[derive(Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
}

impl Expression {
  pub fn new(kind: ExpressionKind, span: Span) -> Self {
    Self { kind, span }
  }
}

impl std::fmt::Debug for ExpressionKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ExpressionKind as e;
    match self {
      e::Integer(i) => write!(f, "{i}"),
      e::Binary { token, left, right } => {
        write!(f, "({left:?} {token:?} {right:?})")
      },
      e::Parenthesis(inner) => write!(f, "{inner:?}"),
      e::Unary { token, child } => {
        write!(f, "({token:?} {child:?})")
      },
      e::Real(fp) => write!(f, "{fp}"),
      e::String(s) => write!(f, r#""{s}""#),
      e::Identifier(i) => write!(f, "{i}"),
      e::Boolean(b) => write!(f, "{b}"),
      e::Call { callee, args } => write!(f, "({callee:?} call {args:?})"),
      e::Field { namespace, field } => {
        write!(f, "({namespace:?} . {field:?})")
      },
      e::Function {
        params, returns, ..
      } => {
        write!(f, "(fn({params:?}) -> {returns:?})")
      },
    }
  }
}

impl std::fmt::Debug for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.kind)
  }
}

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn expression(&mut self, precedence: Precedence) -> Result<Expression> {
    use ExpressionKind as e;
    use TokenKind as t;
    let next = self.peek(0)?;
    // Unary prefix expression
    let mut current = if let Ok(p) = unary_prefix_prec(&next.0) {
      let operator = self.next_tok().expect("unreachable");
      let child = self
        .expression(p)
        .trace(format!("while parsing unary {}", operator.0))
        .span(&operator.1)?;
      let span = child.span + operator.1;
      Expression::new(
        e::Unary {
          token: operator.0,
          child: child.into(),
        },
        span,
      )
    }
    // Terminal or paren
    else {
      self.primary().reason("Expected expression")?
    };

    // Precedence climbing loop
    while let Ok(next) = self.peek(0) {
      // Binary infix
      if let Ok((new_precedence, left_assoc)) = binary_prec(&next.0) {
        if (!left_assoc && new_precedence <= precedence)
          || (new_precedence < precedence)
        {
          return Ok(current);
        }
        let operator = self.next_tok().expect("unreachable");
        let rhs = self
          .expression(new_precedence)
          .trace(format!("while parsing binary {}", operator.0))
          .span(&operator.1)?;
        let span = next.1 + rhs.span;
        current = Expression::new(
          e::Binary {
            token: operator.0,
            left: current.into(),
            right: rhs.into(),
          },
          span,
        );
      }
      // Field
      else if let Token(t::Dot, span) = next {
        if FIELD_PREC <= precedence {
          return Ok(current);
        }
        self.skip(1);
        let field = self
          .expression(FIELD_PREC)
          .trace_span(span, "in field expression")?;
        current = Expression::new(
          e::Field {
            namespace: current.into(),
            field: field.into(),
          },
          span,
        )
      }
      // Function call
      else if let Token(t::LeftParen, mut span) = next {
        if CALL_PREC <= precedence {
          return Ok(current);
        }
        self.skip(1);
        let mut args = vec![];
        loop {
          match self.expression(0) {
            Ok(a) => {
              span = span + a.span;
              args.push(a)
            },
            Err(_) => break,
          };
          if !self.eat(t::Comma).is_ok() {
            break;
          }
        }
        let Token(_, span2) = self.eat(t::RightParen).span(&span)?;
        current = Expression::new(
          e::Call {
            callee: current.into(),
            args,
          },
          span + span2,
        );
      }
      // Unary postfix
      else if let Ok(_) = unary_postfix_prec(&next.0) {
        let operator = self.next_tok().expect("unreachable");
        let span = next.1 + operator.1;
        current = Expression::new(
          e::Unary {
            token: operator.0,
            child: current.into(),
          },
          span,
        );
      } else {
        break;
      }
    }
    Ok(current)
  }

  fn primary(&mut self) -> Result<Expression> {
    use ExpressionKind as e;
    use TokenKind as t;
    let next = self.peek(0)?;
    let mut span = next.1;
    let kind = match next.0 {
      t::IntegerLiteral(i) => e::Integer(i),
      t::FloatLiteral(f) => e::Real(f),
      t::StringLiteral(s) => e::String(s),
      t::True => e::Boolean(true),
      t::Identifier(i) => e::Identifier(i),
      // function
      t::LeftParen
        if (self.look(1, t::Identifier("".into())).is_ok()
          && self.look(2, t::Colon).is_ok())
          || self.look(1, t::RightParen).is_ok() =>
      {
        self.skip(1);
        let mut params = vec![];
        loop {
          let (name, span2) = match self.identifier() {
            Ok((name, span)) => (name, span),
            Err(_) => break,
          };
          span = span + span2;
          self
            .eat(t::Colon)
            .trace_span(span, "while parsing function parameter type")?;
          let (type_, span2) = self
            .identifier()
            .trace_span(span, "while parsing function parameter type")?;
          span = span + span2;
          params.push(Parameter { name, type_ });
          if !self.eat(t::Comma).is_ok() {
            break;
          }
        }
        let Token(_, span2) = self
          .eat(t::RightParen)
          .span(&span)
          .trace_span(span, "while parsing function definition")?;
        let returns = if let Ok(Token(_, span2)) = self.eat(t::Arrow) {
          span = span + span2;
          let (identifier, span2) = self
            .identifier()
            .trace_span(span, "while parsing function return type")?;
          span = span + span2;
          Some(identifier)
        } else {
          None
        };
        span = span + span2;
        let (body, span2) = self
          .block()
          .trace_span(span, "while parsing function body")?;
        span = span + span2;
        e::Function {
          params,
          returns,
          body,
        }
      },
      // parenthetical
      t::LeftParen => {
        self.skip(1);
        let expr = self
          .expression(0)
          .trace("while parsing parenthesized expression")?;
        self
          .look(0, t::RightParen)
          .reason("Unclosed '('")
          .span(&expr.span)?;
        e::Parenthesis(expr.into())
      },
      _ => {
        return error()
          .span(&span)
          .reason(format!("Expected expression, found {}", next.0));
      },
    };
    self.skip(1);
    Ok(Expression { kind, span })
  }

  fn identifier(&mut self) -> Result<(String, Span)> {
    use TokenKind as t;
    match self.peek(0) {
      Ok(Token(t::Identifier(i), span)) => {
        self.skip(1);
        Ok((i, span))
      },
      Ok(t) => error()
        .reason(format!("Expected identifier, found {}", t.0))
        .span(&t.1),
      Err(e) => Err(e),
    }
  }
}
