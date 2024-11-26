pub mod analyzer;
mod builtin;
pub mod primitives;

pub use primitives::*;

use crate::{BinaryOp, UnaryOp, semantic::Primitive};

pub type UID = String;

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Prim(Primitive),
  Nothing,
  Never,
  Struct(UID),
  Function(UID),
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "ambiguous"),
      Type::Prim(primitive) => write!(f, "{primitive}"),
      Type::Nothing => write!(f, "nothing"),
      Type::Struct(s) => write!(f, "struct {s}"),
      Type::Function(func) => write!(f, "func {func}"),
      Type::Never => write!(f, "never"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct IrBlock {
  nodes: Vec<IrNode>,
}

#[derive(Clone, Debug)]
pub enum IrNode {
  Declaration {
    uid: UID,
    mutable: bool,
    size: usize,
    value: IrExpr,
  },
  Function {
    uid: UID,
    parameters: Vec<Type>,
    block: IrBlock,
  },
  Conditional {
    branches: Vec<(IrExpr, IrBlock)>,
    default: IrBlock,
  },
  Expr(IrExpr),
}

#[derive(Clone, Debug)]
pub enum IrExpr {
  Ident(UID),
  UnOp {
    op: UnaryOp,
    child: Box<IrExpr>,
  },
  BinOp {
    op: BinaryOp,
    left: Box<IrExpr>,
    right: Box<IrExpr>,
  },
  Call {
    function: UID,
    args: Vec<IrExpr>,
  },
}
