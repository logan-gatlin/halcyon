#![feature(iterator_try_collect, box_patterns)]
mod builtin;
mod compile;
mod ir;
mod linking;
mod lint;
mod operator;
mod parse;
mod semantic;
#[cfg(test)]
mod test;
mod token;

use std::collections::HashMap;

use builtin::*;
use compile::*;
use ir::*;
use lint::render::Linter;
use parse::*;
use semantic::*;
use token::*;

// TODO:
// Sum types
// Parametric types

pub use lint::*;

pub fn execute(wasm: Vec<u8>) {
  use wasmtime::*;
  let mut config = Config::default();
  config.wasm_gc(true);
  config.wasm_function_references(true);
  let engine = Engine::new(&config).unwrap();
  let module = Module::new(&engine, &wasm).unwrap();
  let linker = Linker::new(&engine);
  let mut store = Store::new(&engine, ());
  let _instance = linker.instantiate(&mut store, &module).unwrap();
}

pub fn compile(input: &str) {
  let linter = Linter::new(input.to_string());
  let tokens = tokenize(input.chars()).handle(&linter);
  let parsed_modules = parse(tokens).handle(&linter);
  let mut encoder = ModuleEncoder::new();
  let mut interfaces = HashMap::new();
  make_std_module(&mut encoder, &mut interfaces);
  for module in parsed_modules {
    let mut ir = build_ir(module, &interfaces).handle(&linter);
    let interface = type_solve(&mut ir).handle(&linter);
    println!("{ir}");
    interfaces.insert(ir.module_name.clone(), interface);
    encoder.encode_ir(ir);
  }
  let wasm = encoder.finish();
  let wat = wasmprinter::print_bytes(&wasm).unwrap();
  std::fs::write("demo.wat", wat).unwrap();
  std::fs::write("demo.wasm", &wasm).unwrap();
  wasmparser::validate(&wasm)
    .map_err(|e| Lint {
      kind: CompilerBug::FailedValidation.into(),
      context: vec![format!("{e}")],
      span: None,
    })
    .handle(&linter);
  execute(wasm);
}
