mod primitives;
mod types;

use crate::err::*;
pub use primitives::*;
pub use types::*;

#[derive(Debug, Clone)]
pub enum Symbol {
  Var(String, Type, bool),
  Type(String, Type),
  BlockStart,
  FuncStart,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
  syms: Vec<Symbol>,
}

impl SymbolTable {
  fn define_var(&mut self, name: String, type_: Type, mutable: bool) {
    self.syms.push(Symbol::Var(name, type_, mutable));
  }

  fn define_type(&mut self, name: String, type_: Type) {
    self.syms.push(Symbol::Type(name, type_));
  }

  fn start_func(&mut self) {
    self.syms.push(Symbol::FuncStart);
  }

  fn end_func(&mut self) {
    while !self.syms.is_empty() {
      if let Some(Symbol::FuncStart) = self.syms.pop() {
        return;
      }
    }
    unreachable!("Tried to exit global scope in symbol table")
  }

  fn start_block(&mut self) {
    self.syms.push(Symbol::BlockStart);
  }

  fn end_block(&mut self) {
    while !self.syms.is_empty() {
      if let Some(Symbol::BlockStart) = self.syms.pop() {
        return;
      }
    }
    unreachable!("Tried to exit global scope in symbol table")
  }

  fn get_var(&self, name: &str) -> Result<(Type, bool)> {
    for s in self.syms.iter().rev() {
      if let Symbol::Var(name2, type_, mutable) = s {
        if name == name2 {
          return Ok((type_.clone(), *mutable));
        }
      }
    }
    error().reason(format!("Identifier {name} is not defined"))
  }

  fn get_type(&self, name: &str) -> Result<Type> {
    for s in self.syms.iter().rev() {
      if let Symbol::Type(name2, t) = s {
        if name == name2 {
          return Ok(t.clone());
        }
      }
    }
    if let Some(p) = Primitive::from_string(name) {
      return Ok(Type::Prim(p));
    }
    error().reason(format!("Type {name} is not defined"))
  }
}
