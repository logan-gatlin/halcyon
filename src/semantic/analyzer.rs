use std::collections::HashMap;

use crate::{err::*, Expression, ExpressionKind, Statement, StatementKind};

use super::*;
use ir::*;

#[derive(Debug, Clone)]
enum Undo {
  FuncGuard,
  BlockGuard,
  Symbol { name: String, prev: Vec<Symbol> },
  Push { name: String },
  None,
}

#[derive(Debug, Clone)]
struct Symbol {
  mangle: SID,
  type_: TID,
  life: Lifetime,
}

#[derive(Debug, Clone)]
struct SymbolTable {
  types: HashMap<TID, Type>,
  table: HashMap<String, Vec<Symbol>>,
  undo_stack: Vec<Undo>,
  path: Vec<String>,
  salt: usize,
}

impl SymbolTable {
  fn define(&mut self, name: &str, type_: TID, life: Lifetime) {
    if !self.table.contains_key(name) {
      self.table.insert(name.to_string(), vec![]);
    }
    let symbols = self.table.get_mut(name).unwrap();
    let mut path = self.path.clone();
    path.push(name.to_string());
    let mangle = names::mangle(path, &format!("{:#x}", self.salt));
    let symbol = Symbol {
      mangle,
      type_,
      life,
    };
    let undo = if 
  }
}

pub fn analyze_block(stmts: Vec<Statement>) -> Result<Vec<IrNode>> {
  todo!()
}
