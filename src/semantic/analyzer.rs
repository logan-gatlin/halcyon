use std::collections::HashMap;

use crate::{
  Expression, ExpressionKind, Span, Statement, StatementKind, err::*,
};

use super::*;
use ir::*;

#[derive(Debug, Clone)]
enum Undo {
  FuncGuard,
  BlockGuard,
  Symbol { name: String, prev: Symbol },
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
  type_table: Vec<Type>,
  sym_table: HashMap<String, Symbol>,
  undo_stack: Vec<Undo>,
  path: Vec<String>,
  salt: usize,
  depth: usize,
}

impl SymbolTable {
  fn define_symbol(&mut self, name: &str, type_: TID) {
    let mut path = self.path.clone();
    path.push(name.to_string());
    let undo = match self.sym_table.get(name) {
      Some(prev) => Undo::Symbol {
        name: name.to_string(),
        prev: prev.clone(),
      },
      None => Undo::None,
    };
    self.undo_stack.push(undo);
    let mangle = names::mangle(path, &self.salt.to_string());
    let symbol = Symbol {
      mangle,
      type_,
      life: if self.depth == 0 {
        Lifetime::Static
      } else {
        Lifetime::Dynamic
      },
    };
    self.sym_table.insert(name.to_string(), symbol);
  }

  fn query_symbol(&mut self, name: &str) -> Result<&Symbol> {
    self.sym_table.get(name).ok_or(Diagnostic::new(
      format!("The name {name} is not declared in this scope",),
      None,
    ))
  }

  fn define_type(&mut self, type_: Type) -> TID {
    self.type_table.push(type_);
    self.type_table.len()
  }
}

pub fn analyze_block(stmts: Vec<Statement>) -> Result<Vec<IrNode>> {
  todo!()
}
