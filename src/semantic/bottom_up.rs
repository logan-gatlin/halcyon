use super::Analyzer;
use crate::{
  Expression, ExpressionKind, Immediate, Statement, StatementKind,
  err::*,
  semantic::{Primitive, SymbolTable},
};

use super::Type;

impl Analyzer {
  pub fn stmt_bottom_up(
    &mut self,
    mut stmt: Box<Statement>,
  ) -> Result<Box<Statement>> {
    todo!()
  }

  // Bottom up type inference
  pub fn expr_bottom_up(
    &mut self,
    mut expr: Box<Expression>,
  ) -> Result<Box<Expression>> {
    todo!()
  }
}
