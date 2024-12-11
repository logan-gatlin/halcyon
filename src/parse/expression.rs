use crate::Base;

use super::*;

#[derive(Clone, Debug)]
pub struct Parameters {
  pub arity: usize,
  pub names: Vec<String>,
  pub type_names: Vec<String>,
}

impl Default for Parameters {
  fn default() -> Self {
    Self {
      arity: 0,
      names: vec![],
      type_names: vec![],
    }
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
    returns_str: Option<String>,
    body: Vec<Statement>,
  },
  FunctionCall {
    callee: Box<Expression>,
    args: Vec<Expression>,
  },
  StructDef(Parameters),
  StructLiteral {
    name: String,
    args: Vec<(String, Expression)>,
  },
  Field {
    namespace: Box<Expression>,
    field: Box<Expression>,
  },
  Block(Vec<Statement>),
  If {
    predicate: Box<Expression>,
    block: Vec<Statement>,
    else_: Option<Box<Expression>>,
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

impl std::fmt::Display for ExpressionKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ExpressionKind as e;
    match self {
      e::Immediate(i) => write!(f, "{i}"),
      e::Binary {
        op: token,
        left,
        right,
      } => {
        write!(f, "({left} {token} {right})")
      }
      e::Parenthesis(inner) => write!(f, "{inner}"),
      e::Unary { op: token, child } => {
        write!(f, "({token} {child})")
      }
      e::Identifier { name, .. } => write!(f, "{name}"),
      e::FunctionCall { callee, args, .. } => {
        write!(f, "({callee} call {args:?})")
      }
      e::Field {
        namespace, field, ..
      } => {
        write!(f, "({namespace} . {field})")
      }
      e::FunctionDef {
        params,
        returns_str,
        ..
      } => {
        write!(f, "(fn({params:?}) -> {returns_str:?})")
      }
      e::StructDef(params) => write!(f, "struct {{ {params:?} }}"),
      e::StructLiteral { name, args, .. } => write!(f, "{name} {{ {args:?} }}"),
      e::Block(block) => {
        write!(f, "{{\n")?;
        for s in block {
          write!(f, "{:#?}", s)?;
        }
        write!(f, "}}")
      }
      e::If { block, else_, .. } => {
        write!(f, "{{\n")?;
        for s in block {
          write!(f, "{:#?}", s)?;
        }
        write!(f, "}}")?;
        if let Some(else_) = else_ {
          write!(f, "{else_}")?;
        }
        Ok(())
      }
    }
  }
}

impl std::fmt::Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({})", self.kind)
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
      )
    }
    // Primary
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
            }
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
          },
          span + span2,
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
        );
      } else {
        break;
      }
    }
    Ok(current)
  }

  pub fn parameters(&mut self, mut span: Span) -> Result<Parameters> {
    use TokenKind as t;
    let mut arity = 0;
    let mut names = vec![];
    let mut type_names = vec![];
    let mut strongly_typed = false;
    loop {
      // Param name
      let (name, span2) = match self.identifier() {
        Ok((ident, span)) => (ident, span),
        Err(_) => break,
      };
      println!("{name}");
      span = span + span2;
      names.push(name.clone());
      // Param type (optional)
      if self.eat(t::Colon).is_ok() {
        let (type_name, span2) = self
          .identifier()
          .trace_span(span + span2, format!("While parsing type of '{}'", name))?;
        strongly_typed = true;
        span = span + span2;
        type_names.push(type_name);
      } else {
        type_names.push("".into());
      }
      arity += 1;
      // Comma
      if !self.eat(t::Comma).is_ok() {
        if self.look(0, t::Identifier("".into())).is_ok() {
          return error().reason("Expected comma (,) here").span(&span);
        }
        break;
      }
    }
    // Back-propogate type names
    if strongly_typed {
      let mut last_name = "".to_string();
      for name in type_names.iter_mut().rev() {
        if name.is_empty() {
          if last_name.is_empty() {
            return error()
              .reason("Cannot mix typed and untyped parameters")
              .span(&span);
          } else {
            *name = last_name.clone();
          }
        } else {
          last_name = name.clone();
        }
      }
    } else {
      return error()
        .reason("Untyped parameters are not allowed (temporarily)")
        .span(&span);
    }
    Ok(Parameters {
      arity,
      names,
      type_names,
    })
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
      t::If => return self.if_else(),
      t::LeftBrace => {
        let (block, span1) = self.block()?;
        span = span + span1;
        e::Block(block)
      }
      // Function definition
      t::LeftParen
        if (self.look(1, t::Identifier("".into())).is_ok()
          && (self.look(2, t::Colon).is_ok() || self.look(2, t::Comma).is_ok()))
          || self.look(1, t::RightParen).is_ok() =>
      {
        self.skip(1);
        let params = self.parameters(span)?;
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
          body,
        }
      }
      // Struct definition
      t::Struct => {
        self.skip(1);
        self.eat(t::LeftBrace)?;
        let params = self.parameters(span)?;
        self.eat(t::RightBrace)?;
        e::StructDef(params)
      }
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
      }
      t::Identifier(i) => {
        self.skip(1);
        e::Identifier { name: i }
      }
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
      }
      _ => {
        return error()
          .span(&span)
          .reason(format!("Expected expression, found {}", next.0));
      }
    };
    Ok(Expression::new(kind, span))
  }

  fn if_else(&mut self) -> Result<Expression> {
    use TokenKind as t;
    if let Ok(Token(_, span)) = self.eat(t::If) {
      let predicate = self
        .expression(0)
        .trace_span(span, "in predicate of 'if' statement")?;
      let span = span + predicate.span;
      let block = self
        .block()
        .trace_span(span, "in block of 'if' statement")?;
      let (block, span) = (block.0, span + block.1);
      let else_ = if self.eat(t::Else).is_ok() {
        Some(Box::new(self.if_else()?))
      } else {
        None
      };
      Ok(Expression::new(
        ExpressionKind::If {
          predicate: predicate.into(),
          block,
          else_,
        },
        span,
      ))
    } else {
      let (block, span) = self.block()?;
      Ok(Expression::new(ExpressionKind::Block(block), span))
    }
  }

  fn identifier(&mut self) -> Result<(String, Span)> {
    use TokenKind as t;
    match self.peek(0) {
      Ok(Token(t::Identifier(i), span)) => {
        self.skip(1);
        Ok((i, span))
      }
      Ok(t) => error()
        .reason(format!("Expected identifier, found {}", t.0))
        .span(&t.1),
      Err(e) => Err(e),
    }
  }
}
