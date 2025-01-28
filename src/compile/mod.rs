pub mod assembly;
pub mod lower;
pub mod normalize;
pub mod text;
pub use assembly::*;
pub use lower::*;
pub use normalize::*;

use crate::semantic::ir::{Module, Node, NodeKind};

pub struct Compiler {
  /// Unique salt added to the names of WASM loops, blocks,
  /// and compiler generated temporary registers.
  /// Incremented after every use
  unique_salt: usize,
  /// The name of WASM blocks which can be 'broken' out of
  /// are pushed onto this stack for inner break statements
  /// to refer to
  break_stack: Vec<String>,
}

impl Compiler {
  pub fn new() -> Self {
    Self {
      unique_salt: 0,
      break_stack: vec![],
    }
  }

  pub fn compile(&mut self, module: Module) {
    let nodes = module.nodes;
    println!("{nodes:#?}");
  }
}
