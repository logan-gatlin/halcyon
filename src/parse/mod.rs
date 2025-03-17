pub(crate) mod expression;
pub mod operators;
pub mod primary;
pub(crate) mod statement;
pub use expression::*;
use multipeek::{MultiPeek, multipeek};
pub use operators::*;
pub use statement::*;

pub use crate::lint::*;
use crate::{Span, Token, TokenKind, token::TokenLint};

pub enum ParseLint {
  UnexpectedToken = 2000,
  MissingBody = 2001,
  MissingBinaryOperand = 2002,
  MissingPrefixUnaryOperand = 2003,
  MissingPostfixUnaryOperand = 2004,
  MissingComma = 2005,
  MissingFunctionParameterType = 2006,
  ExpectedIdentifier = 2007,
  MissingAssignee = 2008,
  MissingSemicolon = 2009,
}

pub fn parse(iter: impl IntoIterator<Item = Token>) -> Vec<Statement> {
  Parser::new(iter.into_iter()).collect()
}

pub struct Parser<I>
where
  I: Iterator<Item = Token>,
{
  iter: MultiPeek<I>,
  last_span: Span,
  finished: bool,
}

impl<I: Iterator<Item = Token>> Iterator for Parser<I> {
  type Item = Statement;

  fn next(&mut self) -> Option<Self::Item> {
    if self.finished || self.look(0, TokenKind::EOF).is_some() {
      return None;
    }
    match self.statement() {
      Ok(s) => Some(s),
      Err(e) => {
        self.error_correct();
        Some(Statement {
          span: e.span.expect("No span for tokenizer error"),
          kind: StatementKind::Error(e),
        })
      },
    }
  }
}

impl<I: Iterator<Item = Token>> Parser<I> {
  pub fn new(iter: I) -> Self {
    Self {
      iter: multipeek(iter),
      last_span: Span { start: 0, width: 0 },
      finished: false,
    }
  }

  fn error_correct(&mut self) {
    loop {
      let next = self.next_tok();
      self.last_span = next.1;
      match next.0 {
        TokenKind::EOF
        | TokenKind::Semicolon
        | TokenKind::RightBrace
        | TokenKind::NewLine => break,
        _ => {},
      }
    }
  }

  fn skip(&mut self, n: usize) {
    for _ in 0..n {
      let _ = self.next_tok();
    }
  }

  fn next_tok(&mut self) -> Token {
    if self.finished {
      return Token(TokenKind::EOF, self.last_span);
    }
    let token = self
      .iter
      .next()
      .unwrap_or(Token(TokenKind::EOF, self.last_span));
    self.last_span = token.1;
    if token.0 == TokenKind::EOF {
      self.finished = true;
    }
    token
  }

  fn peek(&mut self, n: usize) -> Token {
    self
      .iter
      .peek_nth(n)
      .cloned()
      .unwrap_or(Token(TokenKind::EOF, self.last_span))
  }

  fn peek_not_newline(&mut self) -> Token {
    while self.look(0, TokenKind::NewLine).is_some()
      && self.look(1, TokenKind::NewLine).is_some()
    {
      self.skip(1);
    }
    if self.look(0, TokenKind::NewLine).is_none() {
      self.peek(0)
    } else {
      self.peek(1)
    }
  }

  fn eat_newlines(&mut self) {
    while let Some(_) = self.eat(TokenKind::NewLine) {}
  }

  fn eat(&mut self, expect: TokenKind) -> Option<Token> {
    self.look(0, expect)?;
    let next = self.next_tok();
    Some(next)
  }

  fn look(&mut self, n: usize, expect: TokenKind) -> Option<Token> {
    let next = self.peek(n);
    if next.0 == expect { Some(next) } else { None }
  }

  fn body(
    &mut self,
    lint_context: impl Into<String>,
  ) -> Result<(Vec<Statement>, Span)> {
    use TokenKind as t;
    let mut span = self
      .eat(t::LeftBrace)
      .lint(ParseLint::MissingBody as LintKind)
      .context(lint_context)?
      .1;
    let mut statements = vec![];
    loop {
      span = span + self.peek(0).1;
      match self.peek(0) {
        Token(t::RightBrace, s) => {
          span += s;
          self.skip(1);
          break;
        },
        Token(t::EOF, _) => {
          return Err(lint(
            TokenLint::MissingDelimeter as LintKind,
            span,
            &["}".to_string()],
          ));
        },
        _ => {
          let statement = self.statement()?;
          span = span + statement.span;
          statements.push(statement);
        },
      }
    }
    Ok((statements, span))
  }
}
