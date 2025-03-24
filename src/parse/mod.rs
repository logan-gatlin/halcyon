mod parse_expression;

use multipeek::{IteratorExt, MultiPeek};

use crate::{lint::*, operator::*, token::*};

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
  Block(Vec<Expression>),
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

#[derive(Debug, Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
}

pub fn parse(toks: impl IntoIterator<Item = Token>) {
  todo!()
}
