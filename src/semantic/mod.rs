mod analyzer;
mod bottom_up;
mod naming;
mod primitives;
mod top_down;
mod types;

use std::collections::HashMap;

use crate::{Parameter, err::*};
pub use analyzer::*;
pub use primitives::*;
pub use types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
  Block,
  Function,
}

// Mangled name
pub type UID = String;

#[derive(Debug, Clone)]
pub struct Symbol {
  name: String,
  type_: Type,
  uid: UID,
  initialized: bool,
  mutable: Option<bool>,
}

#[derive(Debug, Clone)]
enum Definition {
  Ident(Symbol),
  FuncStart,
  BlockStart,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
  pub structs: Vec<StructureDef>,
  pub functions: Vec<FunctionDef>,
  scope: Vec<Definition>,
  pub table: HashMap<UID, Symbol>,
  mangle_num: usize,
  nesting: usize,
}

impl SymbolTable {
  pub fn new() -> Self {
    // Setup builtin symbols
    let prims = primitive_symbols();
    let mut scope = vec![];
    let mut table = HashMap::new();
    for p in prims {
      scope.push(Definition::Ident(p.clone()));
      table.insert(p.uid.clone(), p);
    }
    let nothing_symbol = Symbol {
      name: "Nothing".to_string(),
      type_: Type::Alias(Box::new(Type::Nothing)),
      uid: nothing_mangle(),
      initialized: true,
      mutable: Some(false),
    };
    scope.push(Definition::Ident(nothing_symbol.clone()));
    table.insert(nothing_symbol.uid.clone(), nothing_symbol);
    Self {
      structs: vec![],
      functions: vec![],
      scope,
      table,
      mangle_num: 0,
      nesting: 0,
    }
  }

  pub fn start_block(&mut self) {
    self.scope.push(Definition::BlockStart);
  }

  pub fn end_block(&mut self) {
    let mut escaped = vec![];
    while !self.scope.is_empty() {
      let top = self.scope.pop();
      if let Some(Definition::BlockStart) = top {
        self.scope.append(&mut escaped);
        return;
      } else if let Some(Definition::Ident(s)) = top {
        if !s.initialized {
          escaped.push(Definition::Ident(s));
        }
      }
    }
    unreachable!("Cannot end global scope")
  }

  pub fn resolve_type(&self, uid: &UID) -> Result<Type> {
    let symbol = self.table.get(uid).unwrap();
    if symbol.initialized == false {
      return error().reason("Symbol was never initialized");
    } else {
      Ok(symbol.type_.clone())
    }
  }

  pub fn start_function(&mut self) {
    self.nesting += 1;
    self.scope.push(Definition::FuncStart);
  }

  pub fn end_function(&mut self) {
    self.nesting -= 1;
    let mut escaped = vec![];
    while !self.scope.is_empty() {
      let top = self.scope.pop();
      if let Some(Definition::FuncStart) = top {
        self.scope.append(&mut escaped);
        return;
      } else if let Some(Definition::Ident(s)) = top {
        if !s.initialized {
          escaped.push(Definition::Ident(s));
        }
      }
    }
    unreachable!("Cannot end global scope")
  }

  fn generate_uid(&mut self, name: &str) -> UID {
    let uid = format!("${}${name}", self.mangle_num);
    self.mangle_num += 1;
    uid
  }

  pub fn get_field(&self, struct_id: SID, field_name: &str) -> Result<Type> {
    let struct_def = &self.structs[struct_id].0;
    for (name, sid) in struct_def {
      if name == field_name {
        return self.resolve_type(sid);
      }
    }
    error().reason(format!("Cannot find field '{field_name}'"))
  }

  pub fn create_struct(&mut self, params: Vec<Parameter>) -> SID {
    let sid = self.structs.len();
    let mut new_params = vec![];
    for p in params {
      let s = self.reference_ident(&p.type_str);
      new_params.push((p.name, s.uid));
    }
    self.structs.push(StructureDef(new_params));
    sid
  }

  pub fn create_function(
    &mut self,
    params: Vec<Parameter>,
    returns: Option<String>,
  ) -> Result<FID> {
    let fid = self.functions.len();
    let mut symbols = vec![];
    for p in &params {
      let symbol = self.reference_ident(&p.type_str);
      symbols.push(symbol);
    }
    let returns = if let Some(s) = returns {
      self.reference_ident(&s).uid
    } else {
      nothing_mangle()
    };
    self.start_function();
    let mut new_params = vec![];
    for (p, s) in params.iter().zip(symbols.iter()) {
      let s = self
        .define_ident(&p.name, s.type_.clone(), false)
        .trace("While initializing function parameters")?;
      new_params.push(s.uid);
    }
    self.functions.push(FunctionDef {
      params: new_params,
      returns,
    });
    Ok(fid)
  }

  pub fn modify_ident(
    &mut self,
    uid: UID,
    name: Option<String>,
    type_: Option<Type>,
    mutable: Option<bool>,
    init: bool,
  ) -> Result<()> {
    let symbol = self.table.get_mut(&uid).unwrap();
    if let Some(ref type_) = type_ {
      symbol.type_ = symbol.type_.clone().deduce(type_)?;
    }
    symbol.mutable = match (symbol.mutable, mutable) {
      (Some(true), Some(false)) | (Some(false), Some(true)) => {
        return error().reason("Cannot mutate immutable symbol");
      },
      _ => mutable,
    };
    symbol.initialized |= init;

    let mut nesting = self.nesting;
    for def in &mut self.scope {
      match def {
        Definition::Ident(symbol)
          if (symbol.uid == uid)
            && (nesting == 0 || nesting == self.nesting) =>
        {
          if let Some(ref name) = name {
            symbol.name = name.clone();
          }
          if let Some(ref type_) = type_ {
            symbol.type_ = symbol.type_.clone().deduce(type_)?;
          }
          symbol.mutable = match (symbol.mutable, mutable) {
            (Some(true), Some(false)) | (Some(false), Some(true)) => {
              return error().reason("Cannot mutate immutable symbol");
            },
            _ => mutable,
          };
          symbol.initialized |= init;
          return Ok(());
        },
        Definition::FuncStart => nesting -= 1,
        _ => {},
      }
    }
    Ok(())
    //unreachable!("Symbol {uid} does not exist")
  }

  pub fn define_ident(
    &mut self,
    name: &str,
    type_: Type,
    mutable: bool,
  ) -> Result<Symbol> {
    println!("---{name}---");
    println!("{:#?}", self.lookup_block_scope(name));
    if let Type::Alias(_) | Type::Function(_) = &type_ {
      if mutable {
        return error()
          .reason("Struct and function definitions cannot be mutable");
      }
    }
    if let Ok(s) = self.lookup_block_scope(name) {
      // Re-definition error
      // TODO support name shadowing
      if s.initialized {
        return error()
          .reason(format!("Multiple definitions of '{name}' in this scope"));
      }
      // Referenced before init, modify existing entry
      else {
        self.modify_ident(
          s.uid.clone(),
          None,
          Some(type_),
          Some(mutable),
          true,
        )?;
        return Ok(s);
      }
    }
    // First initialization in this scope
    let uid = self.generate_uid(name);
    let sym = Symbol {
      name: name.into(),
      type_,
      uid: uid.clone(),
      initialized: true,
      mutable: Some(mutable),
    };
    self.scope.push(Definition::Ident(sym.clone()));
    self.table.insert(uid, sym.clone());
    Ok(sym)
  }

  pub fn reference_ident(&mut self, name: &str) -> Symbol {
    match self.lookup_function_scope(name) {
      Ok(s) => s,
      Err(_) => {
        let uid = self.generate_uid(name);
        let sym = Symbol {
          name: name.into(),
          type_: Type::Ambiguous,
          uid: uid.clone(),
          initialized: false,
          mutable: None,
        };
        self.scope.push(Definition::Ident(sym.clone()));
        self.table.insert(uid, sym.clone());
        sym
      },
    }
  }

  pub fn lookup_block_scope(&self, name: &str) -> Result<Symbol> {
    self.lookup_scope(name, Scope::Block)
  }

  pub fn lookup_function_scope(&self, name: &str) -> Result<Symbol> {
    self.lookup_scope(name, Scope::Function)
  }

  fn lookup_scope(&self, name: &str, scope: Scope) -> Result<Symbol> {
    let mut nesting = self.nesting;
    for def in self.scope.iter().rev() {
      match def {
        Definition::Ident(symbol)
          if (symbol.name == name)
            && ((nesting == 0) || nesting == self.nesting) =>
        {
          return Ok(symbol.clone());
        },
        Definition::FuncStart | Definition::BlockStart
          if scope == Scope::Block =>
        {
          break;
        },
        Definition::FuncStart => nesting -= 1,
        _ => {},
      }
    }
    error().reason(format!(
      "Cannot find the definition of '{}' in the current scope",
      name
    ))
  }
}

// I think this works? Box<T> pattern matching weirdness
// going on. If this is ever even used
fn unwrap_aliases(mut t: Type) -> Type {
  loop {
    if let Type::Alias(ref t1) = t {
      if let Type::Alias(t2) = &**t1 {
        t = *t2.clone();
      } else {
        return t;
      }
    } else {
      return t;
    }
  }
}
