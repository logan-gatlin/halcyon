use crate::err::*;
use crate::{Span, Token, TokenKind};

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
    }
  }
}

impl std::fmt::Debug for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.kind)
  }
}

#[derive(Debug, Clone)]
pub enum StatementKind {
  Mutable {
    name: String,
    type_: Option<String>,
    value: Option<Expression>,
  },
  Immutable {
    name: String,
    type_: Option<String>,
    value: Expression,
  },
  Assignment {
    name: String,
    value: Expression,
  },
  If {
    predicate: Expression,
    block: Vec<Statement>,
    else_: Option<Box<Statement>>,
  },
  While {
    predicate: Expression,
    block: Vec<Statement>,
  },
  Print(Expression),
  Expression(Expression),
  Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub struct Statement {
  pub kind: StatementKind,
  pub span: Span,
}

pub type Precedence = usize;

fn binary_prec(tok: &TokenKind) -> Result<(Precedence, bool)> {
  use TokenKind::*;
  Ok(match tok {
    Star | Slash | Percent => (10, false),
    Plus | Minus => (9, false),
    And | Nand => (8, false),
    Xor | Xnor => (7, false),
    Or | Nor => (6, false),
    DoubleEqual | BangEqual | Less | LessEqual | Greater | GreaterEqual => {
      (5, false)
    },
    //Colon => Some((5, false)),
    _ => {
      return error()
        .reason(format!("{:?} is not a valid binary operator", tok));
    },
  })
}

fn unary_prefix_prec(tok: &TokenKind) -> Result<Precedence> {
  use TokenKind::*;
  Ok(match tok {
    Minus | Not => 11,
    Break => 3,
    _ => {
      return error()
        .reason(format!("{tok:?} is not a valid prefix unary operator"));
    },
  })
}

fn unary_postfix_prec(tok: &TokenKind) -> Result<Precedence> {
  use TokenKind::*;
  Ok(match tok {
    Question => 12,
    Bang => 12,
    _ => {
      return error()
        .reason(format!("{tok:?} is not a valid postfix unary operator"));
    },
  })
}

const PARSER_LOOKAHEAD: usize = 3;

type TokenIter<I> = crate::Window<PARSER_LOOKAHEAD, Token, I>;

pub struct Parser<I: Iterator<Item = Token>> {
  iter: TokenIter<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn new(iter: I) -> Self {
    Self {
      iter: TokenIter::new(iter),
    }
  }

  fn skip(&mut self, n: usize) {
    for _ in 0..n {
      let _ = self.next();
    }
  }

  fn next(&mut self) -> Result<Token> {
    self.iter.next().reason("Unexpected end of file")
  }

  fn peek(&self, n: usize) -> Result<Token> {
    self.iter.peek(n).clone().reason("Unexpected end of file")
  }

  fn eat(&mut self, expect: TokenKind) -> Result<Token> {
    match self.look(expect) {
      Ok(t) => {
        self.skip(1);
        Ok(t)
      },
      Err(e) => Err(e),
    }
  }

  fn look(&mut self, expect: TokenKind) -> Result<Token> {
    let next = self.peek(0)?;
    if next.0 == expect {
      Ok(next)
    } else {
      error()
        .reason(format!("Expected {expect:?}, found {:?}", next.0))
        .span(&next.1)
    }
  }

  pub fn file(&mut self) -> Result<Vec<Statement>> {
    use TokenKind as t;
    let mut statements = vec![];
    loop {
      // Trim extra ;
      while self.eat(t::Semicolon).is_ok() {}
      if self.eat(t::EOF).is_ok() {
        return Ok(statements);
      }
      match self.statement() {
        Ok(s) => statements.push(s),
        Err(e) => {
          return Err(e);
        },
      }
    }
  }

  pub fn statement(&mut self) -> Result<Statement> {
    use StatementKind as s;
    use TokenKind as t;
    let next = self.peek(0);
    let next2 = self.peek(1);
    let statement = match (next, next2) {
      // (im)mutable declaration
      (Ok(Token(t::Identifier(name), span)), Ok(Token(t::Colon, _))) => {
        self.skip(2);
        let type_ = match self.eat(t::Identifier("".into())) {
          Ok(Token(t::Identifier(s), _)) => Some(s),
          _ => None,
        };
        match self.eat(t::Equal).or_else(|_| self.eat(t::Colon)) {
          Ok(Token(t::Colon, _)) => Statement {
            kind: s::Immutable {
              name,
              type_,
              value: self
                .expression(0)
                .trace("while parsing immutable declaration")?,
            },
            span,
          },
          Ok(Token(t::Equal, _)) => Statement {
            kind: s::Mutable {
              name,
              type_,
              value: Some(
                self
                  .expression(0)
                  .trace("while parsing mutable declaration")?,
              ),
            },
            span,
          },
          _ => return error().reason("Expected expression here"),
        }
      },
      (Ok(Token(t::Identifier(name), span)), Ok(Token(t::Equal, _))) => {
        self.skip(2);
        let value = self
          .expression(0)
          .trace("while parsing assignment expression")?;
        Statement {
          kind: s::Assignment { name, value },
          span,
        }
      },
      // If
      (Ok(Token(t::If, span)), _) => {
        self.skip(1);
        let predicate = self
          .expression(0)
          .reason("Expected predicate after 'if' keyword")
          .span(&span)?;
        let block = self.block().trace("while parsing if statement")?;
        return Ok(Statement {
          span,
          kind: s::If {
            predicate,
            block: block.0,
            else_: None,
          },
        });
      },
      // While
      (Ok(Token(t::While, span)), _) => {
        self.skip(1);
        let predicate = self
          .expression(0)
          .reason("Expected predicate after 'while' keyword")
          .span(&span)?;
        let block = self.block().trace("while parsing while statement")?;
        return Ok(Statement {
          span,
          kind: s::While {
            predicate,
            block: block.0,
          },
        });
      },
      // (DEBUG) print
      (Ok(Token(t::Print, span)), _) => {
        self.skip(1);
        let expr = self.expression(0).trace("while parsing print statement")?;
        Statement {
          span: span + expr.span,
          kind: s::Print(expr),
        }
      },
      // Block
      (Ok(Token(t::LeftBrace, _)), _) => {
        // Skip check for semicolon
        let (block, span) =
          self.block().trace("while parsing block statement")?;
        return Ok(Statement {
          kind: s::Block(block),
          span,
        });
      },
      // Expression
      _ => {
        let expr = self
          .expression(0)
          .trace("while parsing expression statement")?;
        Statement {
          span: expr.span,
          kind: s::Expression(expr),
        }
      },
    };
    // Check for semicolon
    if self.eat(t::Semicolon).is_ok() {
      Ok(statement)
    } else {
      error().reason("Expected ;")
    }
  }

  pub fn expression(
    &mut self,
    mut precedence: Precedence,
  ) -> Result<Expression> {
    use ExpressionKind as e;
    use TokenKind as t;
    let next = self.peek(0)?;
    // Unary prefix expression
    let mut current = if let Ok(p) = unary_prefix_prec(&next.0) {
      let operator = self.next().expect("unreachable");
      let child = self
        .expression(p)
        .trace(format!("while parsing unary {:?}", operator.0))
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
      self.primary()?
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
        let operator = self.next().expect("unreachable");
        let rhs = self
          .expression(new_precedence)
          .trace(format!("while parsing binary {:?}", operator.0))
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
      // Unary postfix
      else if let Ok(new_precedence) = unary_postfix_prec(&next.0) {
        let operator = self.next().expect("unreachable");
        let span = next.1 + operator.1;
        precedence = new_precedence;
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
    let span = next.1;
    let kind = match next.0 {
      t::IntegerLiteral(i) => e::Integer(i),
      t::FloatLiteral(f) => e::Real(f),
      t::StringLiteral(s) => e::String(s),
      t::True => e::Boolean(true),
      t::Identifier(i) => e::Identifier(i),
      t::LeftParen => {
        self.eat(t::LeftParen).expect("unreachable");
        let expr = self
          .expression(0)
          .trace("while parsing parenthesized expression")?;
        self
          .look(t::RightParen)
          .reason("Unclosed '('")
          .span(&expr.span)?;
        e::Parenthesis(expr.into())
      },
      _ => {
        return error()
          .span(&span)
          .reason(format!("Expected primary, found {:?}", next.0));
      },
    };
    self.skip(1);
    Ok(Expression { kind, span })
  }

  fn block(&mut self) -> Result<(Vec<Statement>, Span)> {
    use TokenKind as t;
    let mut span = self.eat(t::LeftBrace).reason("Expected block")?.1;
    let mut statements = vec![];
    loop {
      let next = self.peek(0)?;
      span = span + next.1;
      match self.eat(t::RightBrace) {
        Ok(t) => {
          span = span + t.1;
          break;
        },
        _ => {
          let statement = self.statement()?;
          span = span + statement.span;
          statements.push(statement);
        },
      };
    }
    Ok((statements, span))
  }
}
