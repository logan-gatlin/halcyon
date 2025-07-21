#![feature(generic_const_exprs, iterator_try_collect, box_patterns)]
#![allow(incomplete_features)]
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

use ir::*;
use lint::render::Linter;
use parse::*;
use semantic::*;
use token::*;

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

pub fn _compile(input: &str) -> Result<Vec<u8>> {
  let start_compile_time = std::time::Instant::now();
  let tokens = tokenize(input.chars())?;
  let parse_tree = parse(tokens)?;
  println!("{parse_tree:#?}");
  /*
  let parse_tree = parse(tokens)?;
  let mut hlir = build_hlir(parse_tree)?;
  type_solve(&mut hlir)?;
  let wasm = compile::compile(hlir);
  let wat = wasmprinter::print_bytes(&wasm).unwrap();
  wasmparser::validate(&wasm).map_err(|e| Lint {
    kind: CompilerBug::FailedValidation.into(),
    context: vec![format!("{e}")],
    span: None,
  })?;
  Ok(wasm)
  */
  todo!()
}

pub fn compile(input: &str) {
  let linter = Linter::new(input.to_string());
  match _compile(input) {
    Ok(b) => {
      if b.len() != 0 {
        std::fs::write("test.wasm", b).unwrap();
      }
    },
    Err(e) => {
      println!(
        "{}",
        "Failed to Compile".apply_style(Color::Red, Attribute::Underline),
      );
      println!("{}", linter.render(e))
    },
  };
}
