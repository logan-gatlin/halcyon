#![feature(let_chains, generic_const_exprs, iterator_try_collect, box_patterns)]
#![allow(
  unused_imports,
  dead_code,
  unused_variables,
  unused_imports,
  incomplete_features
)]

mod compile;
mod execute;
mod hlir;
mod lint;
mod memory;
mod operator;
mod parse;
mod semantic;
mod token;

use hlir::*;
use lint::render::Linter;
use parse::*;
use semantic::*;
use std::{
  ops::{Add, AddAssign},
  process::exit,
  time::Instant,
};
use token::*;
use wasm_bindgen::prelude::wasm_bindgen;

pub use lint::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen(module = "/src/abi/element.js")]
extern "C" {
  pub fn _compiler_print(s: String);
  pub fn _compiler_cls();
  pub fn _compiler_wat(s: String);
  pub fn _compiler_exec(bytes: Vec<u8>);
}

#[cfg(not(target_family = "wasm"))]
pub fn _compiler_print(s: String) {
  println!("{s}");
}
#[cfg(not(target_family = "wasm"))]
pub fn _compiler_cls() {
}
#[cfg(not(target_family = "wasm"))]
pub fn _compiler_wat(_s: String) {
}
#[cfg(not(target_family = "wasm"))]
pub fn _compiler_exec(_bytes: Vec<u8>) {
}

pub fn compiler_print(s: impl Into<String>) {
  _compiler_print(s.into());
}

pub fn _compile(input: &str) -> Result<Vec<u8>> {
  _compiler_cls();
  let tokens = tokenize(input.chars())?;
  let parse_tree = parse(tokens)?;
  println!("{parse_tree}");
  let mut hlir = build_hlir(parse_tree)?;
  type_solve(&mut hlir);
  println!("# IR");
  println!("{hlir}");
  let wasm = compile::compile(hlir);
  println!("# WAT");
  println!("{}", wasmprinter::print_bytes(&wasm).unwrap());
  if let Err(e) = wasmparser::validate(&wasm) {
    eprintln!(
      "{}",
      "# !!! VALIDATION ERROR !!!"
        .apply_style(Color::Red, Attribute::Underline)
    );
    eprintln!("{e}");
  }
  Ok(vec![])
}

#[wasm_bindgen]
pub fn compile(input: &str) {
  let linter = Linter::new(input.to_string());
  match _compile(input) {
    Ok(b) => {
      if b.len() != 0 {
        _compiler_exec(b);
      }
    },
    Err(e) => {
      compiler_print(
        "Failed to Compile".apply_style(Color::Red, Attribute::Underline),
      );
      compiler_print(linter.render(e))
    },
  };
}
