use consteval::ConstValue;
use operators::OpDef;
//use operators::OpDef;

use crate::{BinaryOp, Span, UnaryOp};

use super::*;

#[derive(Debug, Clone)]
pub struct Module {
  pub data: Vec<u8>,
  pub constants: HashMap<Mangle, Node>,
  pub main: Option<Mangle>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
  Loop {
    names: Vec<Mangle>,
    initials: Vec<Node>,
    body: Box<Node>,
  },
  Break {
    expr: Box<Node>,
  },
  ConstValue(ConstValue),
  Identifier {
    name: String,
    constant: bool,
    mangle: Mangle,
  },
  StructDef {
    mangle: String,
    member_names: Vec<String>,
    member_types: Vec<Node>,
  },
  StructLiteral {
    struct_t: Box<Node>,
    param_names: Vec<String>,
    param_values: Vec<Node>,
  },
  BinaryOp {
    op: BinaryOp,
    opdef: OpDef,
    left: Box<Node>,
    right: Box<Node>,
  },
  UnaryOp {
    op: UnaryOp,
    opdef: OpDef,
    child: Box<Node>,
  },
  Field {
    namespace: Box<Node>,
    index: String,
  },
  If {
    predicate: Box<Node>,
    then: Box<Node>,
    else_: Option<Box<Node>>,
  },
  Call {
    mangle: Mangle,
    callee: Box<Node>,
    params: Vec<Node>,
  },
  Function {
    param_mangles: Vec<Mangle>,
    param_types: Vec<Node>,
    returns: Box<Node>,
    nodes: Box<Node>,
  },
  Declaration {
    name: String,
    global: bool,
    mangle: Mangle,
    type_assert: Option<Box<Node>>,
    value: Box<Node>,
  },
  Block {
    nodes: Vec<Node>,
  },
  Remainder {
    node: Box<Node>,
  },
  /// Constant declaration that got lifted to global scope
  Lifted,
}

#[derive(Debug, Clone)]
pub struct Node {
  pub span: Span,
  pub type_: Type,
  pub kind: NodeKind,
}
