use crate::{Expression, ExpressionKind, Statement, StatementKind};

use super::*;

pub struct Analyzer {
  table: SymbolTable,
}

impl Analyzer {
  pub fn new() -> Self {
    Self {
      table: SymbolTable::new(),
    }
  }

  pub fn typecheck() {
    todo!()
  }

  /// Analyzing a block:
  /// 1. Name structs
  /// 2. Type structs
  /// 3. Name and type functions
  /// 4. Name variables (recurse on blocks) (track moves)
  /// 5. Type variables
  /// 6. Type assert variables
  pub fn block(&mut self, mut block: Vec<Statement>) -> Result<Vec<Statement>> {
    // 1. Name structs
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
        *type_ = Type::Nothing;
        self.table.declare_struct(name, s.span)?;
      }
    }
    // 2. Type structs
    for s in &mut block {
      if let StatementKind::Declaration {
        name,
        value:
          Expression {
            kind: ExpressionKind::StructDef(params, _),
            ..
          },
        ..
      } = &mut s.kind
      {
        self.table.declare_struct(name, s.span)?;
      }
    }
    // 3. Name and type functions
    for s in &mut block {
      if let StatementKind::Declaration {
        name,
        value:
          Expression {
            kind:
              ExpressionKind::FunctionDef {
                params,
                returns_str,
                returns_actual,
                body,
                uid,
              },
            type_,
            span,
          },
        ..
      } = &mut s.kind
      {
        let uid = self.table.define_function(
          name,
          params.clone(),
          returns_str.as_ref().map(|s| s.as_str()),
          *span,
        )?;
        *type_ = Type::Function(uid);
      }
    }
    Ok(block)
  }
}
