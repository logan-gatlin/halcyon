use super::{Analyzer, Type};
use crate::{
  Expression, ExpressionKind, Immediate, Statement, StatementKind,
  err::*,
  semantic::{Primitive, SymbolTable},
};

impl Analyzer {
  pub fn stmt_top_down(
    &mut self,
    mut stmt: Box<Statement>,
    expect: &Type,
  ) -> Result<Box<Statement>> {
    todo!()
  }

  pub fn expr_top_down(
    &mut self,
    mut expr: Box<Expression>,
    expect: &Type,
  ) -> Result<Box<Expression>> {
    todo!()
  }
}
