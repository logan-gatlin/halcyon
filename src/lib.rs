#![feature(generic_const_exprs, iterator_try_collect, box_patterns)]
#![allow(incomplete_features)]

mod builtin;
mod compile;
mod hlir;
mod lint;
mod operator;
mod parse;
mod semantic;
#[cfg(test)]
mod test;
mod token;

use hlir::*;
use lint::render::Linter;
use parse::*;
use semantic::*;
use token::*;

pub use lint::*;

pub fn _compile(input: &str) -> Result<Vec<u8>> {
  let start_compile_time = std::time::Instant::now();
  let tokens = tokenize(input.chars())?;
  let parse_tree = parse(tokens)?;
  println!("{parse_tree}");
  let mut hlir = build_hlir(parse_tree)?;
  type_solve(&mut hlir)?;
  //println!("# IR");
  //println!("{hlir}");
  let wasm = compile::compile(hlir);
  //println!("# WAT");
  let wat = wasmprinter::print_bytes(&wasm).unwrap();
  //println!("{}", wat);
  std::fs::write("test.wat", wat).unwrap();
  if let Err(e) = wasmparser::validate(&wasm) {
    eprintln!(
      "{}",
      "# !!! VALIDATION ERROR !!!".apply_style(Color::Red, Attribute::Underline)
    );
    eprintln!("{e}");
    return Err(lint_nospan(CompilerBug::FailedValidation));
  }
  println!(
    "{}",
    format!("Binary size: {:.2} kb", (wasm.len() as f64) / 1024.0)
      .apply_style(Color::Yellow, Attribute::Normal)
  );
  println!(
    "{}",
    format!(
      "Compiled Successfully in {}ms",
      std::time::Instant::now()
        .duration_since(start_compile_time)
        .as_millis()
    )
    .apply_style(Color::Green, Attribute::Bold)
  );
  let mut config = wasmtime::Config::default();
  config.wasm_gc(true);
  config.wasm_function_references(true);
  let engine = wasmtime::Engine::new(&config).unwrap();
  let module = wasmtime::Module::new(&engine, &wasm).unwrap();
  let mut linker = wasmtime::Linker::new(&engine);
  linker
    .func_wrap(
      "sys",
      "print_integer",
      |caller: wasmtime::Caller<'_, ()>, num: i32| {
        println!("WASM: {num}");
      },
    )
    .unwrap();
  let mut store = wasmtime::Store::new(&engine, ());
  let instance = linker.instantiate(&mut store, &module).unwrap();
  println!(
    "{}",
    "Executed without errors".apply_style(Color::Green, Attribute::Bold)
  );
  Ok(wasm)
}

pub fn compile(input: &str) {
  let linter = Linter::new(input.to_string());
  match _compile(input) {
    Ok(b) => {
      if b.len() != 0 {
        std::fs::write("test.wasm", b).unwrap();
      }
    }
    Err(e) => {
      println!(
        "{}",
        "Failed to Compile".apply_style(Color::Red, Attribute::Underline),
      );
      println!("{}", linter.render(e))
    }
  };
}
