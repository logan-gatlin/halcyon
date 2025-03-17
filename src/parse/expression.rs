use crate::{Base, token::TokenLint};

use super::*;

#[derive(Clone, Debug)]
pub struct Parameters {
  pub arity: usize,
  pub names: Vec<String>,
  pub types: Vec<Expression>,
  pub spans: Vec<Span>,
}

impl Default for Parameters {
  fn default() -> Self {
    Self {
      arity: 0,
      names: vec![],
      types: vec![],
      spans: vec![],
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
    parameters: Parameters,
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
    parameters: Parameters,
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
    parameters: Parameters,
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

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn expression(&mut self, precedence: Precedence) -> Result<Expression> {
    use ExpressionKind as e;
    use TokenKind as t;
    let next = self.peek(0);
    // Unary prefix expression
    let mut current = if let Ok(operator) = UnaryOp::try_from(&next.0) {
      let span = next.1;
      if operator.assoc() == RIGHT_ASSOC {
        return Err(lint(
          ParseLint::MissingPostfixUnaryOperand as LintKind,
          span,
          &[format!("{operator}")],
        ));
      }
      self.skip(1);
      let child = self.expression(operator.precedence()).span(span)?;
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
      self.primary().span(next.1)?
    };
    // Precedence climbing loop
    while let next = self.peek(0)
      && next.0 != t::EOF
    {
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
          let rhs = self.expression(new_precedence).span(span)?;
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
        if self.eat(t::LeftBrace).is_some() {
          let params = self.parameters(span)?;
          self
            .eat(t::RightBrace)
            .lint(TokenLint::MissingDelimeter as LintKind)
            .context("}")
            .span(span)?;
          current = Expression::new(
            e::StructLiteral {
              struct_t: Some(current.into()),
              parameters: params,
            },
            span,
          )
        } else {
          let field = self.expression(FIELD_PREC).span(span)?;
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
          if self.eat(t::RightParen).is_some() {
            break;
          }
          let arg = self.expression(0).span(span)?;
          span = span + arg.span;
          args.push(arg);
          if !self.eat(t::Comma).is_some() {
            if self.eat(t::RightParen).is_some() {
              break;
            } else {
              return Err(lint(ParseLint::MissingComma as LintKind, span, &[]));
            }
          }
        }
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
          return Err(lint(
            ParseLint::MissingPrefixUnaryOperand as LintKind,
            span,
            &[format!("{operator}")],
          ));
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
    let mut types = vec![];
    let mut spans = vec![];
    loop {
      // Name
      let name_token = self.peek(0);
      let name = match name_token.0 {
        t::Identifier(s) => s,
        t::RightBrace | t::RightParen | t::RightSquare => break,
        _ => {
          return Err(lint(
            TokenLint::MissingDelimeter as LintKind,
            span,
            &["}".to_string()],
          ));
        },
      };
      self.skip(1);
      let name_span = name_token.1;
      span += name_span;
      names.push(name.clone());
      // Colon
      let colon_span = self
        .eat(t::Colon)
        .lint(ParseLint::MissingFunctionParameterType as LintKind)
        .span(name_span)?
        .1;
      span += colon_span;
      // Type
      let type_ = match self.expression(0).span(span) {
        Ok(t) => t,
        Err(_) => {
          return Err(lint(
            ParseLint::MissingFunctionParameterType as LintKind,
            name_span + colon_span,
            &[name],
          ));
        },
      };
      span += type_.span;
      spans.push(name_span + type_.span);
      types.push(type_);
      arity += 1;
      // Comma
      if !self.eat(t::Comma).is_some() {
        if self.look(0, t::Identifier("".into())).is_some() {
          return Err(lint(ParseLint::MissingComma as LintKind, span, &[]));
        }
        break;
      }
    }
    Ok(Parameters {
      arity,
      names,
      types,
      spans,
    })
  }

  pub fn if_else(&mut self) -> Result<Expression> {
    use TokenKind as t;
    if let Some(Token(_, span)) = self.eat(t::If) {
      let predicate = self.expression(0).span(span)?;
      let span = span + predicate.span;
      let (block, span2) = self.body("if").span(span)?;
      let then = Box::new(Expression {
        kind: ExpressionKind::Block(block),
        span: span2,
      });
      let span = span + span2;
      let else_ = if self.eat(t::Else).is_some() {
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
      let (block, span) = self.body("else")?;
      Ok(Expression::new(ExpressionKind::Block(block), span))
    }
  }

  pub fn identifier(&mut self) -> Result<(String, Span)> {
    use TokenKind as t;
    match self.peek(0) {
      Token(t::Identifier(i), span) => {
        self.skip(1);
        Ok((i, span))
      },
      _ => Err(lint(
        ParseLint::ExpectedIdentifier as LintKind,
        self.last_span,
        &[],
      )),
    }
  }
}
