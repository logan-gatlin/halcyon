pub mod build_mlir;
pub mod dependencies;
pub mod evaluate;

pub use build_mlir::*;
pub use dependencies::*;
pub use evaluate::*;

use std::collections::{HashMap, HashSet};

use crate::{hlir::*, lint::*, memory::*, operator::*, parse::*};

#[derive(Clone, Debug)]
pub enum BlockKind {
  Constant(Option<ConstValue>),
  Function {
    parameters: Vec<Mangle>,
    parameter_spans: Vec<Span>,
    return_type: Option<Mangle>,
    return_span: Option<Span>,
    value: Option<ConstValue>,
  },
  TypeAssert(Option<Type>),
  Parameter(Option<Type>),
  GlobalScope(Option<ConstValue>),
}

#[derive(Clone, Debug)]
pub struct Block {
  pub kind: BlockKind,
  body: Vec<MlIrNode>,
}

impl Block {
  pub fn new(kind: BlockKind) -> Self {
    Self { kind, body: vec![] }
  }

  pub fn push(&mut self, node: MlIrNode) {
    self.body.push(node);
  }
}

#[derive(Debug, Clone)]
pub struct MlIrModule {
  pub blocks: HashMap<Mangle, Block>,
  pub dependencies: HashMap<Mangle, HashSet<Mangle>>,
}

#[derive(Clone)]
pub struct MlIrNode {
  pub span: Span,
  pub kind: MlIrKind,
}

impl std::fmt::Debug for MlIrNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self.kind,)
  }
}

#[derive(Clone)]
pub enum MlIrKind {
  /// Push a constant value
  Const(ConstValue),
  /// Pop 1 value, assign the value to a name
  Set(Mangle),
  /// Push a named value
  Get(Mangle),
  /// Pop 2 values, apply a binary operator, push the result
  BinaryOp {
    kind: BinaryOp,
  },
  /// Pop 1 value, apply a unary operator, push the result
  UnaryOp {
    kind: UnaryOp,
  },
  /// Pop 1 value, push the named field value
  Field(String),
  /// Pop N values, construct a new value and push it
  StructLiteral {
    param_names: Vec<String>,
  },
  /// Pop N type values, push a new struct definition
  StructDef {
    fields: Vec<String>,
  },
  /// Pop 1 type, assert that the next value on the stack is
  /// of that type. Keep this second value on the stack
  TypeAssert,
  /// Pop 1 function, pop N argument values, call the
  /// function and push its return value
  Call {
    arity: usize,
    spans: Vec<Span>,
  },
  /// Clear the stack of any values up to the last enscope
  Drop,
  /// Pop 1 boolean, if true continue, otherwise go to else
  If,
  Else,
  End,
  Repeat,
  Loop,
  Break,
}

impl std::fmt::Debug for MlIrKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MlIrKind::Const(const_value) => write!(f, "push {const_value}"),
      MlIrKind::Set(mangle) => write!(f, "set {mangle}"),
      MlIrKind::Get(mangle) => write!(f, "get {mangle}"),
      MlIrKind::BinaryOp { kind } => write!(f, "operator {kind}"),
      MlIrKind::UnaryOp { kind } => write!(f, "operator {kind}"),
      MlIrKind::Field(name) => write!(f, "field {name}"),
      MlIrKind::StructLiteral { param_names } => {
        write!(f, "struct literal {}", param_names.len())
      }
      MlIrKind::StructDef {
        fields: param_names,
      } => {
        write!(f, "struct definition {}", param_names.len())
      }
      MlIrKind::TypeAssert => write!(f, "type assert",),
      MlIrKind::Call { arity, .. } => write!(f, "call {arity}"),
      MlIrKind::Drop => write!(f, "drop"),
      MlIrKind::If => write!(f, "if"),
      MlIrKind::Else => write!(f, "else"),
      MlIrKind::Loop => write!(f, "loop"),
      MlIrKind::End => write!(f, "end"),
      MlIrKind::Repeat => write!(f, "repeat"),
      MlIrKind::Break => write!(f, "break"),
    }
  }
}
