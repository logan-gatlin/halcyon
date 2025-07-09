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
  Let {
    is_type: bool,
    is_recursive: bool,
    assignee_span: Span,
    assignee: String,
    value: Box<Expression>,
    in_: Option<Box<Expression>>,
  },
  Literal(Literal),
  Identifier(String),
  Binary {
    op: BinaryOp,
    left: Box<Expression>,
    right: Box<Expression>,
  },
  Unary {
    op: UnaryOp,
    child: Box<Expression>,
  },
  FunctionDef {
    export_name: Option<String>,
    arguments: Vec<String>,
    argument_spans: Vec<Span>,
    types: Vec<Option<Expression>>,
    body: Box<Expression>,
  },
  FunctionCall {
    callee: Box<Expression>,
    arguments: Box<Expression>,
  },
  If {
    predicate: Box<Expression>,
    then: Box<Expression>,
    else_: Option<Box<Expression>>,
  },
  Structure {
    is_definition: bool,
    lhs: Vec<String>,
    rhs: Vec<Expression>,
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

pub fn parse(toks: impl IntoIterator<Item = Token>) -> Result<Expression> {
  let mut iter = toks.into_iter().multipeek();
  let parsed = match expression(&mut iter, 0)? {
    Some(e) => Ok(e),
    None => Err(lint_nospan(ParseLint::EmptyInput)),
  }?;
  if let Some(t) = iter.peek_nth(0)
    && t.0 != TokenKind::EOF
  {
    println!("{:?}", t.1);
    Err(lint(
      ParseLint::ExpectedExpression,
      t.1,
      &[format!("{}", t.0)],
    ))
  } else {
    Ok(parsed)
  }
}
