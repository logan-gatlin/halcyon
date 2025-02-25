use crate::{Base, error};

use super::*;

#[derive(Clone, Debug)]
pub struct Parameters {
  pub arity: usize,
  pub names: Vec<Expression>,
  pub types: Vec<Expression>,
}

impl Default for Parameters {
  fn default() -> Self {
    Self {
      arity: 0,
      names: vec![],
      types: vec![],
    }
  }
}

#[derive(Debug, Clone)]
pub enum Immediate {
  Unit,
  Integer(String, Base),
  Real(String),
  String(String),
  Glyph(char),
  Boolean(bool),
}

impl std::fmt::Display for Immediate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Immediate::Unit => write!(f, "()"),
      Immediate::Integer(i, b) => write!(f, "{i} ({b:?})"),
      Immediate::Glyph(c) => write!(f, "{c}"),
      Immediate::Real(r) => write!(f, "{r}"),
      Immediate::String(s) => write!(f, "{s}"),
      Immediate::Boolean(b) => write!(f, "{b}"),
    }
  }
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
  Immediate(Immediate),
  Identifier {
    name: String,
  },
  Binary {
    op: BinaryOp,
    left: Box<Expression>,
    right: Box<Expression>,
  },
  Unary {
    op: UnaryOp,
    child: Box<Expression>,
  },
  Parenthesis(Box<Expression>),
  FunctionDef {
    params: Parameters,
    returns: Option<Box<Expression>>,
    body: Box<Expression>,
  },
  FunctionCall {
    callee: Box<Expression>,
    args: Vec<Expression>,
  },
  StructDef(Parameters),
  StructLiteral {
    struct_t: Option<Box<Expression>>,
    params: Parameters,
  },
  Field {
    namespace: Box<Expression>,
    field: Box<Expression>,
  },
  Block(Vec<Statement>),
  If {
    predicate: Box<Expression>,
    then: Box<Expression>,
    else_: Option<Box<Expression>>,
  },
  Loop {
    params: Parameters,
    body: Box<Expression>,
  },
  Break {
    expr: Option<Box<Expression>>,
  },
}

#[derive(Clone, Debug)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
}

impl Expression {
  pub fn new(kind: ExpressionKind, span: Span) -> Self {
    Self { kind, span }
  }
}

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
  pub fn expression(&mut self, precedence: Precedence) -> Result<Expression> {
    use ExpressionKind as e;
    use TokenKind as t;
    let next = self.peek(0)?;
    // Unary prefix expression
    let mut current = if let Ok(operator) = UnaryOp::try_from(&next.0) {
      let span = next.1;
      if operator.assoc() == RIGHT_ASSOC {
        return error!("The {operator} operator must come after a value")
          .span(&span);
      }
      self.skip(1);
      let child = self
        .expression(operator.precedence())
        .trace_span(span, format!("while parsing unary {}", operator))
        .span(&span)?;
      let span = span + child.span;
      Expression::new(
        e::Unary {
          op: operator,
          child: child.into(),
        },
        span,
      )
    }
    // Primary
    else {
      self.primary().reason("Expected expression")?
    };
    // Precedence climbing loop
    while let Ok(next) = self.peek(0) {
      // Binary or mixed
      if let Ok(operator) = BinaryOp::try_from(&next.0) {
        {
          let new_precedence = operator.precedence();
          if ((operator.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
            || (new_precedence < precedence)
          {
            return Ok(current);
          }
          self.skip(1);
          let span = next.1;
          let rhs = self
            .expression(new_precedence)
            .trace_span(span, format!("while parsing binary {}", operator))
            .span(&span)?;
          let span = next.1 + rhs.span;
          current = Expression::new(
            e::Binary {
              op: operator,
              left: current.into(),
              right: rhs.into(),
            },
            span,
          );
        }
      }
      // Field or struct literal
      else if let Token(t::Dot, span) = next {
        if FIELD_PREC <= precedence {
          return Ok(current);
        }
        self.skip(1);
        if self.eat(t::LeftBrace).is_ok() {
          let params = self.parameters(span)?;
          self
            .eat(t::RightBrace)
            .trace_span(span, "while parsing struct declaration")?;
          current = Expression::new(
            e::StructLiteral {
              struct_t: Some(current.into()),
              params,
            },
            span,
          )
        } else {
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
      }
      // Function call
      else if let Token(t::LeftParen, mut span) = next {
        if CALL_PREC <= precedence {
          return Ok(current);
        }
        self.skip(1);
        let mut args = vec![];
        loop {
          if self.eat(t::RightParen).is_ok() {
            break;
          }
          let arg = self
            .expression(0)
            .trace_span(span, "while parsing function call")?;
          span = span + arg.span;
          args.push(arg);
          if !self.eat(t::Comma).is_ok() {
            break;
          }
        }
        let Token(_, span2) = self.eat(t::RightParen).span(&span)?;
        current = Expression::new(
          e::FunctionCall {
            callee: current.into(),
            args,
          },
          span,
        );
      }
      // Unary postfix
      else if let Ok(operator) = UnaryOp::try_from(&next.0) {
        self.skip(1);
        let span = next.1;
        if operator.assoc() == LEFT_ASSOC {
          return error!("The {operator} operator must come before a value")
            .span(&span);
        }
        current = Expression::new(
          e::Unary {
            op: operator,
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

  pub fn parameters(&mut self, span: Span) -> Result<Parameters> {
    use TokenKind as t;
    let mut arity = 0;
    let mut names = vec![];
    let mut types = vec![];
    loop {
      // Name
      let name = if let Ok(name) = self.expression(0) {
        name
      } else {
        break;
      };
      names.push(name);
      // Colon
      self.eat(t::Colon)?;
      // Type
      let type_ = if let Ok(type_) = self.expression(0) {
        type_
      } else {
        return error!("Expected expression after ':'").span(&span);
      };
      types.push(type_);
      arity += 1;
      // Comma
      if !self.eat(t::Comma).is_ok() {
        if self.look(0, t::Identifier("".into())).is_ok() {
          return error!("Expected comma (,) here").span(&span);
        }
        break;
      }
    }
    Ok(Parameters {
      arity,
      names,
      types,
    })
  }

  pub fn if_else(&mut self) -> Result<Expression> {
    use TokenKind as t;
    if let Ok(Token(_, span)) = self.eat(t::If) {
      let predicate = self
        .expression(0)
        .trace_span(span, "in predicate of 'if' statement")?;
      let span = span + predicate.span;
      let (block, span2) = self
        .block()
        .trace_span(span, "in block of 'if' statement")?;
      let then = Box::new(Expression {
        kind: ExpressionKind::Block(block),
        span: span2,
      });
      let span = span + span2;
      let else_ = if self.eat(t::Else).is_ok() {
        Some(Box::new(self.if_else()?))
      } else {
        None
      };
      Ok(Expression::new(
        ExpressionKind::If {
          predicate: predicate.into(),
          then,
          else_,
        },
        span,
      ))
    } else {
      let (block, span) = self.block()?;
      Ok(Expression::new(ExpressionKind::Block(block), span))
    }
  }

  pub fn identifier(&mut self) -> Result<(String, Span)> {
    use TokenKind as t;
    match self.peek(0) {
      Ok(Token(t::Identifier(i), span)) => {
        self.skip(1);
        Ok((i, span))
      },
      Ok(t) => error!("Expected identifier, found {}", t.0).span(&t.1),
      Err(e) => Err(e),
    }
  }
}
