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
  let tokens = tokenize(input.chars())?;
  let parse_tree = parse(tokens)?;
  let mut hlir = build_hlir(parse_tree)?;
  type_solve(&mut hlir)?;
  //println!("# IR");
  //println!("{hlir}");
  let wasm = compile::compile(hlir);
  //println!("# WAT");
  let wat = wasmprinter::print_bytes(&wasm).unwrap();
  println!("{}", wat);
  std::fs::write("test.wat", wat).unwrap();
  if let Err(e) = wasmparser::validate(&wasm) {
    eprintln!(
      "{}",
      "# !!! VALIDATION ERROR !!!"
        .apply_style(Color::Red, Attribute::Underline)
    );
    eprintln!("{e}");
  }
  println!("Binary size: {:.2} kb", (wasm.len() as f64) / 1024.0);
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
