pub mod build_ir;

use std::collections::HashMap;

use crate::diagnostic;
use crate::err::*;
use crate::error;
use crate::ir::Block;
use crate::ir::Ir;
use crate::ir::IrPtr;

pub type Mangle = String;

/// Name mangle syntax:
/// mangle ::= "$" path salt
/// path ::= {path-element}*
/// path-element ::= length ident
/// ident ::= _a-zA-Z {_a-zA-Z0-9}*
/// length ::= {0-9}+
/// salt ::= {a-zA-Z}*
pub fn mangle_name(path: Vec<String>, salt: &str) -> Mangle {
  let mut buf: Vec<u8> = vec![];
  for p in path {
    let puny = punycode::encode(&p).unwrap();
    let bytes = format!("{}{puny}", puny.len());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}

/// Builtin mangle syntax:
/// "$" {ident}
pub fn mangle_builtin(name: impl std::fmt::Display) -> Mangle {
  format!("{name}")
}

#[derive(Debug, Clone)]
pub struct Symbol {
  pub mangle: Mangle,
  pub scope_depth: usize,
  pub is_constant: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
  ScopeStart,
  Modify {
    name: String,
    old_value: Option<Symbol>,
  },
}

/// Convert parse tree to AST
pub struct Analyzer {
  pub scope_depth: usize,
  pub salt: usize,
  pub path: Vec<String>,
  pub event_stack: Vec<Event>,
  pub heap: Vec<Vec<u8>>,
  pub constants: HashMap<Mangle, IrPtr>,
  pub blocks: Vec<Block>,
  pub main: Option<Mangle>,
  pub break_targets: Vec<IrPtr>,
  _name_to_symbol: HashMap<String, Symbol>,
}

impl Analyzer {
  pub(crate) const TERMINUS: IrPtr = 0;
  pub(crate) const UNREACHABLE: IrPtr = 1;

  fn new() -> Self {
    let this = Self {
      path: vec![String::new()],
      event_stack: vec![],
      heap: vec![],
      constants: HashMap::new(),
      scope_depth: 0,
      salt: 0,
      blocks: vec![Block::Terminal, Block::Unreachable],
      main: None,
      break_targets: vec![],
      _name_to_symbol: HashMap::new(),
    };
    this
  }

  pub(crate) fn new_block(&mut self) -> IrPtr {
    let ptr = self.blocks.len();
    self.blocks.push(Block::basic());
    ptr
  }

  pub(crate) fn push(&mut self, ir_ptr: IrPtr, ir: Ir) {
    self.blocks[ir_ptr].push(ir);
  }

  pub(crate) fn name_to_symbol(&self, name: &str) -> Result<&Symbol> {
    self
      ._name_to_symbol
      .get(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  pub(crate) fn next_salt(&mut self) -> String {
    let returned_salt = self.salt.to_string();
    self.salt += 1;
    returned_salt
  }

  pub(crate) fn allocate(&mut self, bytes: &[u8]) -> usize {
    self.heap.push(bytes.into());
    self.heap.len() - 1
  }

  pub(crate) fn define_unique(&mut self, hint: &str) -> Mangle {
    let name = String::from(hint);
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
    mangle
  }

  pub(crate) fn define_name(
    &mut self,
    name: impl Into<String>,
    is_constant: bool,
  ) -> Result<Mangle> {
    let name = name.into();
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
    let old_value = self.name_to_symbol(&name).ok().cloned();
    if let Some(old) = &old_value {
      if old.scope_depth == self.scope_depth && is_constant && old.is_constant {
        return error!("Conflicting definitions of '{name}' in the same scope");
      }
    }
    let event = Event::Modify {
      old_value,
      name: name.clone(),
    };
    self.event_stack.push(event);
    self._name_to_symbol.insert(
      name.clone(),
      Symbol {
        mangle: mangle.clone(),
        scope_depth: self.scope_depth,
        is_constant,
      },
    );
    Ok(mangle)
  }

  pub(crate) fn enscope(&mut self) {
    self.event_stack.push(Event::ScopeStart);
    self.scope_depth += 1;
  }

  pub(crate) fn descope(&mut self) {
    while let Some(e) = self.event_stack.pop() {
      match e {
        Event::ScopeStart => {
          self.scope_depth -= 1;
          break;
        },
        Event::Modify { name, old_value } => {
          if let Some(old) = old_value {
            self._name_to_symbol.insert(name, old);
          } else {
            self._name_to_symbol.remove(&name);
          }
        },
      }
    }
  }

  pub(crate) fn start_function(&mut self) {
    let mut to_reset = vec![];
    for (name, symbol) in self._name_to_symbol.iter() {
      if !symbol.is_constant {
        self.event_stack.push(Event::Modify {
          name: name.clone(),
          old_value: Some(symbol.clone()),
        });
        to_reset.push(name.clone())
      }
    }
    for name in to_reset {
      self._name_to_symbol.remove(&name);
    }
  }
}
