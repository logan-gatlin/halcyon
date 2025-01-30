pub mod assembly;
mod interpreter;
pub mod lower;
pub mod normalize;
pub mod text;
use std::collections::HashMap;

pub use assembly::*;
pub use lower::*;
pub use normalize::*;

use crate::semantic::{
  Type,
  ir::{Module, Node, NodeKind},
};

pub struct Compiler {
  /// Unique salt added to the names of WASM loops, blocks,
  /// and compiler generated temporary registers.
  /// Incremented after every use
  unique_salt: usize,
  /// The name of WASM blocks which can be 'broken' out of
  /// are pushed onto this stack for inner break statements
  /// to refer to
  break_stack: Vec<String>,
  symtable: HashMap<String, Type>,
}

impl Compiler {
  pub fn new(symtable: HashMap<String, Type>) -> Self {
    Self {
      unique_salt: 0,
      break_stack: vec![],
      symtable,
    }
  }

  pub fn compile(&mut self, module: Module) {
    let mut flattened = vec![];
    let mut nodes = module
      .nodes
      .into_iter()
      .map(|n| self.flatten_functions(n, &mut flattened, 0))
      .collect::<Vec<_>>();
    nodes.extend(flattened);
    let mut regs = vec![];
    let mut instrs = vec![];
    nodes
      .into_iter()
      .map(|n| self.lower(n, &mut regs, &mut instrs))
      .try_collect::<Vec<_>>()
      .unwrap();
    regs.extend(instrs);
    let module = WasmModule(regs);
    let s = module.to_wat();
    println!("{s}");
    let binary = wat::parse_str(s).unwrap();
  }
}
