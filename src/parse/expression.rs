use crate::{Base, semantic::*};

use super::*;

#[derive(Clone)]
pub struct Parameter {
  pub name: String,
  pub type_str: String,
  pub type_actual: Type,
}

impl std::fmt::Debug for Parameter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {:?}", self.name, self.type_actual)
  }
}

#[derive(Debug, Clone)]
pub enum Immediate {
  Integer(String, Base),
  Real(String),
  String(String),
  Glyph(char),
  Boolean(bool),
}

impl std::fmt::Display for Immediate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Immediate::Integer(i, b) => write!(f, "{i} ({b:?})"),
      Immediate::Glyph(c) => write!(f, "{c}"),
      Immediate::Real(r) => write!(f, "{r}"),
      Immediate::String(s) => write!(f, "{s}"),
      Immediate::Boolean(b) => write!(f, "{b}"),
    }
  }
}

#[derive(Clone)]
pub enum ExpressionKind {
  Immediate(Immediate),
  Identifier(String, UID),
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
    params: Vec<Parameter>,
    returns_str: Option<String>,
    returns_actual: Type,
    body: Vec<Statement>,
    id: usize,
  },
  FunctionCall {
    callee: Box<Expression>,
    args: Vec<Expression>,
    is_reference: bool,
    id: UID,
  },
  StructDef(Vec<Parameter>, usize),
  StructLiteral {
    name: String,
    args: Vec<(String, Expression)>,
  },
  Field {
    namespace: Box<Expression>,
    field: Box<Expression>,
    uid: UID,
  },
}

#[derive(Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
  pub type_: Type,
}

impl Expression {
  pub fn new(kind: ExpressionKind, span: Span, type_: Type) -> Self {
    Self { kind, span, type_ }
  }
}

impl std::fmt::Debug for ExpressionKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ExpressionKind as e;
    match self {
      e::Immediate(i) => write!(f, "{i}"),
      e::Binary {
        op: token,
        left,
        right,
      } => {
        write!(f, "({left:?} {token:?} {right:?})")
      },
      e::Parenthesis(inner) => write!(f, "{inner:?}"),
      e::Unary { op: token, child } => {
        write!(f, "({token:?} {child:?})")
      },
      e::Identifier(i, _) => write!(f, "{i}"),
      e::FunctionCall { callee, args, .. } => {
        write!(f, "({callee:?} call {args:?})")
      },
      e::Field {
        namespace, field, ..
      } => {
        write!(f, "({namespace:?} . {field:?})")
      },
      e::FunctionDef {
        params,
        returns_actual,
        ..
      } => {
        write!(f, "(fn({params:?}) -> {returns_actual:?})")
      },
      e::StructDef(params, _) => write!(f, "struct {{ {params:?} }}"),
      e::StructLiteral { name, args } => write!(f, "{name} {{ {args:?} }}"),
    }
  }
}

impl std::fmt::Debug for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({:?} {})", self.kind, self.type_)
  }
}

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn expression(&mut self, precedence: Precedence) -> Result<Expression> {
    use ExpressionKind as e;
    use TokenKind as t;
    let next = self.peek(0)?;
    // Unary prefix expression
    let mut current = if let Ok(operator) = UnaryOp::try_from(&next.0) {
      self.skip(1);
      let span = next.1;
      if operator.assoc() == RIGHT_ASSOC {
        return error()
          .reason(format!("The {} operator must come after a value", operator))
          .span(&span);
      }
      let child = self
        .expression(operator.precedence())
        .trace(format!("while parsing unary {}", operator))
        .span(&span)?;
      let span = span + child.span;
      Expression::new(
        e::Unary {
          op: operator,
          child: child.into(),
        },
        span,
        Type::Ambiguous,
      )
    }
    // Terminal or paren
    else {
      self.primary().reason("Expected expression")?
    };

    // Precedence climbing loop
    while let Ok(next) = self.peek(0) {
      // Binary infix
      if let Ok(operator) = BinaryOp::try_from(&next.0) {
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
          .trace(format!("while parsing binary {}", operator))
          .span(&span)?;
        let span = next.1 + rhs.span;
        current = Expression::new(
          e::Binary {
            op: operator,
            left: current.into(),
            right: rhs.into(),
          },
          span,
          Type::Ambiguous,
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
            uid: "".into(),
          },
          span,
          Type::Ambiguous,
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
          e::FunctionCall {
            callee: current.into(),
            args,
            is_reference: false,
            id: "".into(),
          },
          span + span2,
          Type::Ambiguous,
        );
      }
      // Unary postfix
      else if let Ok(operator) = UnaryOp::try_from(&next.0) {
        self.skip(1);
        let span = next.1;
        if operator.assoc() == RIGHT_ASSOC {
          return error()
            .reason(format!(
              "The {} operator must come before a value",
              operator
            ))
            .span(&span);
        }
        current = Expression::new(
          e::Unary {
            op: operator,
            child: current.into(),
          },
          span,
          Type::Ambiguous,
        );
      } else {
        break;
      }
    }
    Ok(current)
  }

  fn primary(&mut self) -> Result<Expression> {
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
      // Function definition
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
          params.push(Parameter {
            name,
            type_str: type_,
            type_actual: Type::Ambiguous,
          });
          if !self.eat(t::Comma).is_ok() {
            break;
          }
        }
        let Token(_, span2) = self
          .eat(t::RightParen)
          .span(&span)
          .trace_span(span, "while parsing function definition")?;
        let returns_str = if let Ok(Token(_, span2)) = self.eat(t::Arrow) {
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
        e::FunctionDef {
          params,
          returns_str,
          returns_actual: Type::Ambiguous,
          body,
          id: usize::MAX,
        }
      },
      // Struct definition
      t::Struct => {
        self.skip(1);
        self.eat(t::LeftBrace)?;
        let mut params = vec![];
        loop {
          let (name, span2) = match self.identifier() {
            Ok((name, span)) => (name, span),
            Err(_) => break,
          };
          span = span + span2;
          self
            .eat(t::Colon)
            .trace_span(span, "while parsing struct parameter type")?;
          let (type_, span2) = self
            .identifier()
            .trace_span(span, "while parsing struct parameter type")?;
          span = span + span2;
          params.push(Parameter {
            name,
            type_str: type_,
            type_actual: Type::Ambiguous,
          });
          if !self.eat(t::Comma).is_ok() {
            break;
          }
        }
        self.eat(t::RightBrace)?;
        e::StructDef(params, usize::MAX)
      },
      // Struct literal
      t::Identifier(name) if self.look(1, t::LeftBrace).is_ok() => {
        self.skip(2);
        let mut args = vec![];
        loop {
          let (name, span2) = match self.identifier() {
            Ok((name, span)) => (name, span),
            Err(_) => break,
          };
          span = span + span2;
          self
            .eat(t::Colon)
            .trace_span(span, "while parsing struct parameter type")?;
          let expr = self
            .expression(0)
            .trace_span(span, "while parsing struct parameter type")?;
          span = span + expr.span;
          args.push((name, expr));
          if !self.eat(t::Comma).is_ok() {
            break;
          }
        }
        self.eat(t::RightBrace)?;
        e::StructLiteral { name, args }
      },
      t::Identifier(i) => {
        self.skip(1);
        e::Identifier(i, "".into())
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
        return error()
          .span(&span)
          .reason(format!("Expected expression, found {}", next.0));
      },
    };
    Ok(Expression::new(kind, span, Type::Ambiguous))
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
