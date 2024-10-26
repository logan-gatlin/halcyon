mod analyzer;
mod primitives;
mod types;

use crate::err::*;
pub use analyzer::*;
pub use primitives::*;
pub use types::*;

// Mangled name
pub type UID = String;

#[derive(Debug, Clone)]
pub struct Symbol {
  name: String,
  type_: Type,
  uid: UID,
}

#[derive(Debug, Clone)]
enum Definition {
  Ident(Symbol),
  FuncStart,
  BlockStart,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
  defs: Vec<Definition>,
  mangle_num: usize,
  nesting: usize,
}

impl SymbolTable {
  pub fn new() -> Self {
    Self {
      defs: vec![],
      mangle_num: 0,
      nesting: 0,
    }
  }

  fn generate_uid(&mut self, name: &str) -> UID {
    let uid = format!("${name}${}", self.mangle_num);
    self.mangle_num += 1;
    uid
  }

  pub fn start_func(&mut self, params: Vec<Type>, returns: Type) -> UID {
    self.nesting += 1;
    let uid = self.generate_uid("func");
    let type_ = Type::FunctionDef {
      params,
      returns: returns.into(),
      id: uid.clone(),
    };
    self.define("<anonymous function>", type_.clone());
    self.defs.push(Definition::FuncStart);
    uid
  }

  pub fn end_func(&mut self) {
    while !self.defs.is_empty() {
      if let Some(Definition::FuncStart) = self.defs.pop() {
        return;
      }
    }
    unreachable!("Cannot end global scope")
  }

  pub fn start_block(&mut self) {
    while !self.defs.is_empty() {
      if let Some(Definition::BlockStart) = self.defs.pop() {
        return;
      }
    }
    unreachable!("Cannot end global scope")
  }

  pub fn define(&mut self, name: &str, type_: Type) -> UID {
    let uid = self.generate_uid(name);
    self.defs.push(Definition::Ident(Symbol {
      name: name.to_string(),
      type_,
      uid: uid.clone(),
    }));
    uid
  }

  pub fn define_func(&mut self, name: &str, uid: UID) {
    for def in &mut self.defs {
      match def {
        Definition::Ident(symbol) if symbol.uid == uid => {
          symbol.name = name.to_string();
          return;
        },
        _ => {},
      }
      panic!("Tried to name nonexistant function");
    }
  }

  pub fn lookup_typedef(&self, name: &str) -> Result<Type> {
    let mut nesting = self.nesting;
    for def in &self.defs {
      match def {
        Definition::Ident(symbol)
          if (symbol.name == name)
            && (nesting == 0 || nesting == self.nesting) =>
        {
          if let Type::StructDef(vec) = &symbol.type_ {
            return Ok(Type::Struct(
              vec.iter().map(|p| p.type_actual.clone()).collect(),
            ));
          }
        },
        Definition::FuncStart => nesting -= 1,
        _ => {},
      }
    }
    if let Some(p) = Primitive::from_string(name) {
      return Ok(Type::Prim(p));
    }
    error().reason(format!("Cannot find type definition '{}'", name))
  }

  pub fn lookup(&self, name: &str) -> Result<Symbol> {
    let mut nesting = self.nesting;
    for def in &self.defs {
      match def {
        Definition::Ident(symbol)
          if (symbol.name == name)
            && (nesting == 0 || nesting == self.nesting) =>
        {
          if let Type::StructDef(_) = symbol.type_ {
            continue;
          }
          return Ok(symbol.clone());
        },
        Definition::FuncStart => nesting -= 1,
        _ => {},
      }
    }
    error().reason(format!("Cannot find the definition of '{}'", name))
  }
}
