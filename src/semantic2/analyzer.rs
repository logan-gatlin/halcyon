use crate::{Expression, ExpressionKind, Statement, StatementKind};

use super::*;

pub struct Analyzer {
  table: SymbolTable,
}

impl Analyzer {
  /// Analyzing a block:
  /// 1. Name structs
  /// 2. Type structs
  /// 3. Name functions
  /// 4. Name variables (recurse on blocks) (track moves)
  /// 5. Type variables
  /// 6. Type assert variables
  pub fn block(&mut self, mut block: Vec<Statement>) -> Result<Vec<Statement>> {
    // Name structs
    for s in &mut block {
      if let StatementKind::Declaration {
        name,
        value:
          Expression {
            kind: ExpressionKind::StructDef(params, _),
            type_,
            ..
          },
        ..
      } = &mut s.kind
      {
        self.table.declare_struct(name, s.span)?;
      }
    }
    Ok(block)
  }
}
