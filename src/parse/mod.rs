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

fn skip(iter: it!(), n: usize) {
  for _ in 0..n {
    iter.next();
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

pub fn parse(toks: impl IntoIterator<Item = Token>) {
  let e = expression(&mut toks.into_iter().multipeek(), 0)
    .unwrap()
    .unwrap();
  println!("{e}");
}
