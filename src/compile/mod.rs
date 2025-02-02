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

  pub fn compile(&mut self, module: Module) -> Result<Vec<u8>> {
    let mut flattened = vec![];
    let mut nodes = module
      .nodes
      .into_iter()
      .map(|n| self.flatten_functions(n, &mut flattened, 0))
      .collect::<Vec<_>>();
    nodes.extend(flattened);
    let mut regs = vec![];
    regs.push(Wasm::import {
      ns1: "js".into(),
      ns2: "print_string".into(),
      object: Wasm::function {
        ident: "$print_string".into(),
        params: vec![("".into(), AsmType::i32), ("".into(), AsmType::i32)],
        results: vec![],
        body: vec![],
      }
      .into(),
    });
    regs.push(Wasm::import {
      ns1: "js".into(),
      ns2: "memory".into(),
      object: Wasm::memory { min: 10, max: 100 }.into(),
    });
    regs.push(Wasm::data {
      offset: 0,
      content: module.data,
    });
    let mut instrs = vec![];
    nodes
      .into_iter()
      .map(|n| self.lower(n, &mut regs, &mut instrs))
      .try_collect::<Vec<_>>()
      .unwrap();
    instrs.push(Wasm::start("$main".into()));
    regs.extend(instrs);
    let module = WasmModule(regs);
    let s = module.to_wat();
    match wat::parse_str(s) {
      Ok(wasm) => Ok(wasm),
      Err(err) => {
        error!("Error translating assembly to binary form:\n{err}")
      },
    }
  }
}
