mod parse_expression;
mod parse_primary;
mod printing;

use multipeek::{IteratorExt, MultiPeek};
use parse_expression::*;
use parse_primary::*;

use crate::{lint::*, operator::*, token::*};

#[derive(Debug, Clone)]
pub enum Literal {
  Unit,
  Integer(String, Base),
  Real(String),
  String(String),
  Glyph(char),
  Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
  Literal(Literal),
  Identifier(String),
  // for type inferred constants
  Binary {
    op: BinaryOp,
    left: Box<Expression>,
    right: Box<Expression>,
  },
  Unary {
    op: UnaryOp,
    child: Box<Expression>,
  },
  FunctionCall {
    callee: Box<Expression>,
    arguments: Box<Expression>,
  },
  Block(Vec<Expression>),
  If {
    predicate: Box<Expression>,
    then: Box<Expression>,
    else_: Option<Box<Expression>>,
  },
  Guard {
    predicates: Vec<Expression>,
    branches: Vec<Expression>,
    else_branch: Option<Box<Expression>>,
  },
  Loop {
    parameters: Box<Expression>,
    body: Box<Expression>,
  },
}

#[derive(Debug, Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
}

macro_rules! it {
  () => {
    &mut MultiPeek<impl Iterator<Item = Token>>
  };
}

fn peek(iter: it!(), n: usize, expect: TokenKind) -> Option<Token> {
  match iter.peek_nth(n) {
    Some(t) if t.0 == expect => Some(t.clone()),
    _ => None,
  }
}

fn skip(iter: it!(), n: usize) {
  for _ in 0..n {
    iter.next();
  }
}

fn next_not_ws(iter: it!()) {
  while iter.peek_nth(0).map(|t| &t.0) == Some(&TokenKind::NewLine)
    && iter.peek_nth(1).map(|t| &t.0) == Some(&TokenKind::NewLine)
  {
    skip(iter, 1)
  }
}

fn eat(iter: it!(), kind: TokenKind) -> Option<Token> {
  let next = iter.peek();
  if let Some(next) = next
    && next.0 == kind
  {
    let next = next.clone();
    skip(iter, 1);
    Some(next.clone())
  } else {
    None
  }
}

fn eat_ws(iter: it!()) {
  while let Some(_) = eat(iter, TokenKind::NewLine) {}
}

pub fn parse(toks: impl IntoIterator<Item = Token>) -> Result<Expression> {
  let mut iter = toks.into_iter().multipeek();
  let mut program = vec![];
  let mut span = Span { start: 0, width: 0 };
  eat_ws(&mut iter);
  loop {
    if peek(&mut iter, 0, TokenKind::EOF).is_some() {
      break;
    }
    let e = expression(&mut iter, 0)?.unwrap();
    eat(&mut iter, TokenKind::NewLine).ok_or(lint(ParseLint::MissingNewLine, e.span, &[]))?;
    eat_ws(&mut iter);
    span += e.span;
    program.push(e);
  }
  if program.len() == 0 {
    Err(lint(ParseLint::EmptyInput, span, &[]))
  } else if program.len() == 1 {
    Ok(program[0].clone())
  } else {
    Ok(Expression {
      kind: ExpressionKind::Block(program),
      span,
    })
  }
}
