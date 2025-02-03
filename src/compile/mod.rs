pub mod assembly;
mod interpreter;
pub mod lower;
pub mod normalize;
pub mod text;

pub use assembly::*;

pub const PAGE_SIZE: usize = 64_000;
use crate::semantic::ir::Module;
use crate::{err::*, error};

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
}
