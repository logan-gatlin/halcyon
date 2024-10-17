mod expression;
mod statement;
pub use expression::*;
pub use statement::*;

use crate::err::*;
use crate::{Span, Token, TokenKind};

pub type Precedence = usize;

const FIELD_PREC: Precedence = 13;
const CALL_PREC: Precedence = 12;

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
      return error().reason(format!("{} is not a valid binary operator", tok));
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
        .reason(format!("{tok} is not a valid prefix unary operator"));
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
        .reason(format!("{tok} is not a valid postfix unary operator"));
    },
  })
}

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
      error()
        .reason(format!("Expected {expect}, found {}", next.0))
        .span(&next.1)
    }
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
