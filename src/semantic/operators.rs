use crate::{BinaryOp, UnaryOp};

use super::Type;

pub struct OpTable {}

pub enum OpDefinition {
  BinaryOp {
    op: BinaryOp,
    left: Type,
    right: Type,
  },
  UnaryOp {
    op: UnaryOp,
    on: Type,
  },
}
