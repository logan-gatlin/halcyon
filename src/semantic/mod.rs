mod analyzer;
mod primitives;
mod types;

use crate::{Parameter, err::*};
pub use analyzer::*;
pub use primitives::*;
pub use types::*;
#[allow(non_camel_case_types)]
pub type uid = u32;

// Variable and function numbering
#[derive(Clone, Copy, Debug)]
pub enum VarKind {
  Global(uid),
  Local(uid),
  Param(uid),
  Function(uid),
  Undefined,
}

impl VarKind {
  pub fn unwrap(self) -> uid {
    match self {
      VarKind::Global(i)
      | VarKind::Local(i)
      | VarKind::Param(i)
      | VarKind::Function(i) => i,
      VarKind::Undefined => unreachable!("Failed unwrapping uid"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct Symbol {
  pub name: String,
  pub type_: Type,
  pub mutable: bool,
  pub kind: VarKind,
}

#[derive(Debug, Clone)]
pub enum Definition {
  Symbol(Symbol),
  BlockStart,
  FuncStart,
}

fn next(array: &mut [uid]) -> uid {
  let current = array.last_mut().unwrap();
  let ret = *current;
  *current += 1;
  ret
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
  syms: Vec<Definition>,
  nesting: usize,
  local_varno: Vec<uid>,
  global_varno: Vec<uid>,
  funcno: Vec<uid>,
}

impl SymbolTable {
  pub fn new() -> Self {
    Self {
      syms: vec![],
      nesting: 0,
      global_varno: vec![0],
      local_varno: vec![0],
      funcno: vec![0],
    }
  }

  fn define_symbol(
    &mut self,
    name: String,
    type_: Type,
    mutable: bool,
  ) -> Result<VarKind> {
    let kind = match type_ {
      Type::Prim(_) | Type::Struct(_) => {
        if self.nesting == 0 {
          VarKind::Global(next(&mut self.global_varno))
        } else {
          VarKind::Local(next(&mut self.local_varno))
        }
      },
      Type::StructDef(_) => {
        if mutable {
          return error().reason("Struct definition must be immutable");
        }
        VarKind::Undefined
      },
      Type::Function { .. } => {
        if !mutable {
          VarKind::Function(next(&mut self.funcno))
        } else {
          return error().reason("Function declaration must be immutable");
        }
      },
      _ => VarKind::Undefined,
    };
    self.syms.push(Definition::Symbol(Symbol {
      name,
      type_,
      mutable,
      kind,
    }));
    Ok(kind)
  }

  fn define_param(&mut self, name: String, type_: Type) -> Result<VarKind> {
    let kind = VarKind::Param(next(&mut self.local_varno));
    self.syms.push(Definition::Symbol(Symbol {
      name,
      type_,
      mutable: false,
      kind,
    }));
    Ok(kind)
  }

  fn start_func(&mut self) {
    self.nesting += 1;
    self.local_varno.push(0);
    self.syms.push(Definition::FuncStart);
  }

  fn end_func(&mut self) {
    self.nesting -= 1;
    self.local_varno.pop();
    while !self.syms.is_empty() {
      if let Some(Definition::FuncStart) = self.syms.pop() {
        return;
      }
    }
    unreachable!("Tried to exit global scope in symbol table")
  }

  fn start_block(&mut self) {
    self.syms.push(Definition::BlockStart);
  }

  fn end_block(&mut self) {
    while !self.syms.is_empty() {
      if let Some(Definition::BlockStart) = self.syms.pop() {
        return;
      }
    }
    unreachable!("Tried to exit global scope in symbol table")
  }

  fn find_symbol(&self, find_name: &str) -> Result<Symbol> {
    let mut nesting = self.nesting;
    println!("Looking for {find_name}, scope = {nesting}");
    for s in self.syms.iter().rev() {
      match s {
        Definition::Symbol(sym)
          // Only search function local and global scope
          if nesting == self.nesting || nesting == 0 => {
          println!("{}, {:?}, {nesting}", sym.name, sym.type_);
          if find_name == sym.name {
            return Ok(sym.clone());
          }
        },
        Definition::FuncStart => {
          nesting -= 1;
        },
        _ => {},
      };
    }
    error().reason(format!("Symbol '{find_name}' is not defined"))
  }

  fn get_type(&self, name: &str) -> Result<Type> {
    for s in self.syms.iter().rev() {
      if let Definition::Symbol(Symbol {
        name: name2,
        type_: Type::StructDef(params),
        ..
      }) = s
      {
        if name == name2 {
          return Ok(Type::Struct(params.clone()));
        }
      }
    }
    if let Some(p) = Primitive::from_string(name) {
      return Ok(Type::Prim(p));
    }
    error().reason(format!("Type {name} is not defined"))
  }
}
