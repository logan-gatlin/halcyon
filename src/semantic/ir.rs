use crate::{BinaryOp, Immediate, UnaryOp};

use super::*;

#[derive(Debug, Clone)]
pub enum NodeKind {
  Immediate(Immediate),
  Identifier(Mangle),
  StructLiteal {
    names: Vec<String>,
    values: Vec<Node>,
  },
  BinaryOp {
    op: BinaryOp,
    left: Box<Node>,
    right: Box<Node>,
  },
  UnaryOp {
    op: UnaryOp,
    child: Box<Node>,
  },
  Field {
    namespace: Box<Node>,
    index: Box<Node>,
  },
  If {
    predicate: Box<Node>,
    then: Box<Node>,
    else_: Option<Box<Node>>,
  },
  Call {
    callee: Box<Node>,
    params: Vec<Node>,
  },
  Function {
    mangle: Mangle,
    nodes: Box<Node>,
  },
  Declaration {
    mangle: Mangle,
    is_constant: bool,
    type_assert: Option<Type>,
    value: Box<Node>,
  },
  Block {
    nodes: Vec<Node>,
  },
}

#[derive(Debug, Clone)]
pub struct Node {
  pub type_: Type,
  pub kind: NodeKind,
}

impl Analyzer {
}
