use std::collections::HashMap;

use crate::{hlir::*, mlir::*};

pub mod assembly;
mod lower;
mod text;
pub mod vm;

pub use assembly::*;
pub use vm::*;

#[derive(Debug, Clone)]
struct BreakTarget {
  block_name: String,
  result_name: String,
}

pub struct Compiler {
  /// Unique salt added to the names of WASM loops, blocks,
  /// and compiler generated temporary registers.
  /// Incremented after every use
  unique_salt: usize,
  /// The name of WASM blocks which can be 'broken' out of
  /// are pushed onto this stack for inner break statements
  /// to refer to
  break_stack: Vec<BreakTarget>,

  module: HlIrModule,
}

impl Compiler {
  pub fn new(module: HlIrModule) -> Self {
    Self {
      unique_salt: 0,
      break_stack: vec![],
      module,
    }
  }

  fn build_data(&mut self) -> Vec<u8> {
    let mut value_map = HashMap::new();
    self
      .module
      .heap
      .iter()
      .fold((0_usize, 0_usize), |(index, item), val| {
        value_map.insert(item, index);
        (index + val.len(), item + 1)
      });
    for n in &mut self.module.nodes {
      if let HlIrKind::Immediate(ConstValue::String {
        ref mut virtual_address,
        ..
      }) = n.kind
      {
        *virtual_address = *value_map.get(virtual_address).unwrap();
      }
    }
    self.module.heap.clone().into_iter().flatten().collect()
  }

  pub fn compile(module: HlIrModule, to_compile: Vec<IrPtr>) -> String {
    let mut this = Self::new(module);
    let heap = this.build_data();
    let mut regs: Vec<Wasm> = Builtin::ALL.into_iter().flat_map(|b| b.import()).collect();
    regs.push(Wasm::Import {
      ns1: "js".to_string(),
      ns2: "memory".to_string(),
      object: Wasm::Memory { min: 10, max: 100 }.into(),
    });
    regs.push(Wasm::Data {
      offset: 0,
      content: heap,
    });
    let mut instrs = vec![];
    for func in to_compile {
      this.lower(func, &mut regs, &mut instrs).unwrap();
    }
    instrs.push(Wasm::Start(this.module.main.clone().unwrap()));
    regs.extend_from_slice(&instrs);
    let mut wasm = String::new();
    for r in regs {
      wasm.push_str(&format!("{}\n", r.to_wat()));
    }
    format!("(module\n{wasm})")
  }
}

impl Type {
  pub fn count_registers(&self) -> usize {
    use Primitive as p;
    match self {
      Type::Primitive(primitive) => match primitive {
        p::nothing | p::unreachable => 0,
        p::glyph | p::integer | p::real | p::boolean => 1,
        p::string => 2,
      },
      Type::Struct { member_types, .. } => member_types.iter().map(|t| t.count_registers()).sum(),
      Type::Function { .. } => 1,
      Type::Type => 0,
      _ => panic!("Counted registers of ambiguous type"),
    }
  }

  pub fn register_types(&self) -> Vec<WasmType> {
    use Primitive as p;
    use WasmType as a;
    match self {
      Type::Primitive(primitive) => match primitive {
        p::nothing | p::unreachable => vec![],
        p::integer => vec![a::I64],
        p::real => vec![a::F64],
        p::boolean => vec![a::I32],
        p::string => vec![a::PTR_T, a::PTR_T],
        p::glyph => vec![a::I32],
      },
      Type::Struct { member_types, .. } => member_types
        .iter()
        .flat_map(|t| t.register_types())
        .collect(),
      Type::Function { .. } => vec![a::FuncRef],
      Type::Type => vec![],
      _ => panic!("Splatted ambiguous type"),
    }
  }
}
