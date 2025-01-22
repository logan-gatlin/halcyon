mod expression;
mod statement;
pub use expression::*;
pub use statement::*;

use crate::{Span, Token, TokenKind};
use crate::{err::*, error};

pub type Precedence = usize;

macro_rules! op {
  ($name:ident; $($op:ident, $prec:expr, $assoc:expr);*;) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($op,)*
    }

    impl $name {
      pub fn precedence(&self) -> Precedence {
        match self {
          $(Self::$op => $prec),*
        }
      }

      pub fn assoc(&self) -> bool {
        match self {
          $(Self::$op => $assoc),*
        }
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
      }
    }

    impl TryFrom<&TokenKind> for $name {
      type Error = Diagnostic;
      fn try_from(value: &TokenKind) -> Result<Self> {
        match value {
          $(TokenKind::$op => Ok(Self::$op),)*
          _ => Err(Diagnostic {
            reason: format!("Invalid operator {value}"),
            span: None,
            backtrace: Vec::new(),
          }),
        }
      }
    }
  }
}

const RIGHT_ASSOC: bool = true;
const LEFT_ASSOC: bool = false;

// Name, precedence, associativity;
op! {
  BinaryOp;
  Star, 10, LEFT_ASSOC;
  Slash, 10, LEFT_ASSOC;
  Percent, 10, LEFT_ASSOC;
  Plus, 9, LEFT_ASSOC;
  Minus, 9, LEFT_ASSOC;
  Nand, 8, LEFT_ASSOC;
  Xor, 7, LEFT_ASSOC;
  Xnor, 7, LEFT_ASSOC;
  Or, 6, LEFT_ASSOC;
  Nor, 6, LEFT_ASSOC;
  DoubleEqual, 5, LEFT_ASSOC;
  BangEqual, 5, LEFT_ASSOC;
  Less, 5, LEFT_ASSOC;
  LessEqual, 5, LEFT_ASSOC;
  Greater, 5, LEFT_ASSOC;
  GreaterEqual, 5, LEFT_ASSOC;
  And, 4, LEFT_ASSOC;
}

op! {
  UnaryOp;
  Plus, 11, RIGHT_ASSOC;
  Tilda, 11, RIGHT_ASSOC;
  Minus, 11, LEFT_ASSOC;
  Not, 11, LEFT_ASSOC;
}

pub fn is_mixed_op(t: &TokenKind) -> bool {
  BinaryOp::try_from(t).is_ok() && UnaryOp::try_from(t).is_ok()
}

const FIELD_PREC: Precedence = 13;
const CALL_PREC: Precedence = 12;

const PARSER_LOOKAHEAD: usize = 3;

type TokenIter<I> = crate::Window<PARSER_LOOKAHEAD, Token, I>;

pub struct Parser<I: Iterator<Item = Token>> {
  iter: TokenIter<I>,
}

impl<I: Iterator<Item = Token>> Iterator for Parser<I> {
  type Item = Statement;

  fn next(&mut self) -> Option<Self::Item> {
    use StatementKind as s;
    use TokenKind as t;
    // Trim extra ;
    while self.eat(t::Semicolon).is_ok() {}
    loop {
      if self.eat(t::EOF).is_ok() || self.iter.finished {
        return None;
      }
      match self.statement() {
        Ok(s) => return Some(s),
        Err(e) => {
          loop {
            let next = self.next_tok();
            match next {
              Ok(Token(t::Semicolon | t::RightBrace | t::EOF, _)) => break,
              _ => {},
            }
          }
          return Some(Statement {
            span: e.span.unwrap_or(Span { row: 0, column: 0 }),
            kind: s::Error(e),
          });
        },
      }
    }
  }
}

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn new(iter: I) -> Self {
    Self {
      iter: TokenIter::new(iter),
    }
  }

  fn skip(&mut self, n: usize) {
    for _ in 0..n {
      let _ = self.next_tok();
    }
  }

  fn next_tok(&mut self) -> Result<Token> {
    match self.iter.next() {
      Some(Token(TokenKind::Error(e), span)) => Err(e).span(&span),
      r => r.reason("Unexpected end of file"),
    }
  }

  fn peek(&self, n: usize) -> Result<Token> {
    match self.iter.peek(n).clone() {
      Some(Token(TokenKind::Error(e), span)) => Err(e).span(&span),
      r => r.reason("Unexpected end of file"),
    }
  }

  fn eat(&mut self, expect: TokenKind) -> Result<Token> {
    match self.look(0, expect) {
      Ok(t) => {
        self.skip(1);
        Ok(t)
      },
      Err(e) => Err(e),
    }
  }

  fn look(&mut self, n: usize, expect: TokenKind) -> Result<Token> {
    let next = self.peek(n)?;
    if next.0 == expect {
      Ok(next)
    } else {
      error!("Expected {expect}, found {}", next.0).span(&next.1)
    }
  }

  fn block(&mut self) -> Result<(Vec<Statement>, Span)> {
    use TokenKind as t;
    let mut span = self.eat(t::LeftBrace).reason("Expected block")?.1;
    let mut statements = vec![];
    loop {
      span = span + self.peek(0)?.1;
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
