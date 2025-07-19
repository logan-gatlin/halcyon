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

pub fn execute(wasm: Vec<u8>) {
  use wasmtime::*;
  let mut config = Config::default();
  config.wasm_gc(true);
  config.wasm_function_references(true);
  let engine = Engine::new(&config).unwrap();
  let module = Module::new(&engine, &wasm).unwrap();
  let mut linker = Linker::new(&engine);
  let mut store = Store::new(&engine, ());
  let memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
  linker
    .func_wrap(
      "sys",
      "print_integer",
      move |_callee: Caller<'_, ()>, num: i64| {
        println!("WASM: {num}");
      },
    )
    .unwrap()
    .func_wrap(
      "sys",
      "print_real",
      move |_callee: Caller<'_, ()>, num: f64| {
        println!("WASM: {num}");
      },
    )
    .unwrap()
    .define(&mut store, "sys", "memory", Extern::Memory(memory))
    .unwrap();
  let _instance = linker.instantiate(&mut store, &module).unwrap();
  println!(
    "{}",
    "Executed without errors".apply_style(Color::Green, Attribute::Bold)
  );
}

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
      "# !!! VALIDATION ERROR !!!"
        .apply_style(Color::Red, Attribute::Underline)
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
  execute(wasm.clone());
  Ok(wasm)
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
