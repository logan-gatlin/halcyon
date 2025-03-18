mod build_mlir;

use std::collections::HashMap;

use crate::{Span, graph::Graph, hlir::*, operator::*, parse::*};

#[derive(Clone, Debug)]
pub enum BlockKind {
  Constant { evaluation: Option<ConstValue> },
  Function { parameters: Vec<Mangle> },
  TypeAssert,
  Parameter,
  GlobalScope,
}

#[derive(Clone, Debug)]
pub struct Block {
  kind: BlockKind,
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
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
  pub mangle: Mangle,
  pub arity: usize,
  pub parameter_mangles: Vec<Mangle>,
  pub returns_mangle: Option<Mangle>,
  pub block: IrPtr,
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
  TypeAssert(Option<Mangle>),
  /// Pop 1 function, pop N argument values, call the
  /// function and push its return value
  Call {
    arity: usize,
  },
  /// Clear the stack of any values up to the last enscope
  Drop,
  // Pop 1 boolean, continue if true, jump otherwise
  Branch(usize),
  // Jump to label
  Jump(usize),
  Label(usize),
  /// Inserts a scope guard, prevents popping values pushed
  /// before this point
  StartScope,
  /// Remove a previously placed scope guard, leaving any
  /// remaining values on the stack
  EndScope,
  Noop,
}

impl std::fmt::Debug for MlIrKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MlIrKind::Const(const_value) => write!(f, "push {const_value}"),
      MlIrKind::Set(mangle) => write!(f, "set {mangle}"),
      MlIrKind::Get(mangle) => write!(f, "get {mangle}"),
      MlIrKind::BinaryOp { kind } => write!(f, "binary {kind}"),
      MlIrKind::UnaryOp { kind } => write!(f, "unary {kind}"),
      MlIrKind::Field(name) => write!(f, "field {name}"),
      MlIrKind::StructLiteral { param_names } => {
        write!(f, "struct literal {}", param_names.len())
      },
      MlIrKind::StructDef {
        fields: param_names,
      } => {
        write!(f, "struct definition {}", param_names.len())
      },
      MlIrKind::TypeAssert(mangle) => write!(
        f,
        "type assert{}",
        if let Some(mangle) = mangle {
          format!(" ({mangle})")
        } else {
          format!("")
        }
      ),
      MlIrKind::Call { arity } => write!(f, "call {arity}"),
      MlIrKind::Drop => write!(f, "drop"),
      MlIrKind::Branch(label) => {
        write!(f, "branch {label}")
      },
      MlIrKind::Jump(label) => write!(f, "jump {label}"),
      MlIrKind::Label(s) => write!(f, "label {s}"),
      MlIrKind::StartScope => write!(f, "start scope"),
      MlIrKind::EndScope => write!(f, "end scope"),
      MlIrKind::Noop => write!(f, "noop"),
    }
  }
}
