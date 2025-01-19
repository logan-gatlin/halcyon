use std::collections::HashMap;

use crate::{BinaryOp, UnaryOp, diagnostic};
use crate::{err::*, error};

use super::Type;

#[derive(Clone, Hash, PartialEq, Eq)]
enum OpDef {
  Binary {
    op: BinaryOp,
    left: Type,
    right: Type,
  },
  Unary {
    op: UnaryOp,
    on: Type,
  },
}

impl OpDef {
  fn reverse(self) -> Self {
    match self {
      OpDef::Binary { op, left, right } => OpDef::Binary {
        op,
        left: right,
        right: left,
      },
      _ => self,
    }
  }
}

pub struct OpTable {
  op_map: HashMap<OpDef, Type>,
}

// For binary ops, its important that we are order agnostic
// (i.e. T1 + T2 == T2 + T1). That means checking for both
// permutations on both definition and usage. There is some
// ugly code here to avoid copies for this check
impl OpTable {
  pub fn new() -> Self {
    Self {
      op_map: HashMap::new(),
    }
  }

  pub fn define_binary(
    &mut self,
    op: BinaryOp,
    left: Type,
    right: Type,
    produces: Type,
  ) -> Result<()> {
    let err: Result<()> = error!(
      "Operator {op} is already defined for types '{}' and '{}'",
      left, &right
    );
    let opdef = OpDef::Binary { op, left, right };
    if self.op_map.contains_key(&opdef) {
      return err;
    }
    let opdef = opdef.reverse();
    if self.op_map.contains_key(&opdef) {
      return err;
    }
    self.op_map.insert(opdef.reverse(), produces);
    Ok(())
  }

  pub fn define_unary(
    &mut self,
    op: UnaryOp,
    on: Type,
    produces: Type,
  ) -> Result<()> {
    let err = error!("Operator {op} is already defined for type '{}'", &on);
    let old = self.op_map.insert(OpDef::Unary { op, on }, produces);
    if old.is_some() { err } else { Ok(()) }
  }

  pub fn try_binary(
    &self,
    op: BinaryOp,
    left: &Type,
    right: &Type,
  ) -> Result<Type> {
    let err =
      error!("Operator {op} is not defined for types '{left}' and '{right}'",);
    let left = left.clone();
    let right = right.clone();
    let opdef = OpDef::Binary { op, left, right };
    let produces_1 = self.op_map.get(&opdef);
    let opdef = opdef.reverse();
    let produces_2 = self.op_map.get(&opdef);
    if let Some(t) = produces_1.or(produces_2) {
      Ok(t.clone())
    } else {
      err
    }
  }

  pub fn try_unary(&self, op: UnaryOp, on: &Type) -> Result<Type> {
    let err = error!("Operator {op} is not defined for type '{on}'");
    match self.op_map.get(&OpDef::Unary { op, on: on.clone() }) {
      Some(t) => Ok(t.clone()),
      None => err,
    }
  }
}
