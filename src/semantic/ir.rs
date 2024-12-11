use crate::{BinaryOp, UnaryOp};

use super::{Type, SID, TID};

#[derive(Clone, Debug)]
pub struct IrBlock {
  nodes: Vec<IrNode>,
}

#[derive(Clone, Debug)]
pub enum IrNode {
  Declaration {
    uid: SID,
    mutable: bool,
    size: usize,
    value: IrExpr,
  },
  Function {
    uid: SID,
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
pub struct IrExpr {
  kind: IrExprKind,
  type_: TID,
}

#[derive(Clone, Debug)]
pub enum IrExprKind {
  Ident(SID),
  UnOp {
    op: UnaryOp,
    child: Box<IrExpr>,
  },
  BinOp {
    op: BinaryOp,
    left: Box<IrExpr>,
    right: Box<IrExpr>,
  },
  Block(IrBlock),
  Call {
    function: SID,
    args: Vec<IrExpr>,
  },
}
