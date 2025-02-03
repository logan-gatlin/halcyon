pub(crate) mod expression;
pub mod operators;
pub mod primary;
pub(crate) mod statement;
pub use expression::*;
pub use operators::*;
pub use statement::*;

use crate::{Span, Token, TokenKind};
use crate::{err::*, error};

const PARSER_LOOKAHEAD: usize = 3;

type TokenIter<'a, I> = crate::Window<'a, PARSER_LOOKAHEAD, Token, I>;

pub struct Parser<'a, I: Iterator<Item = Token>>
where
  I: Iterator<Item = Token>,
  I: 'a,
{
  iter: TokenIter<'a, I>,
}

impl<'a, I: Iterator<Item = Token>> Iterator for Parser<'a, I> {
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

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
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
