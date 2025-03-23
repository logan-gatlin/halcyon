#![feature(let_chains)]
#![feature(generic_const_exprs)]
#![feature(iterator_try_collect)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(incomplete_features)]
mod compile;
mod graph;
mod hlir;
mod lint;
mod memory;
mod mlir;
mod operator;
mod parse;
mod token;
mod typecheck;

use compile::Compiler;
use hlir::*;
use lint::render::Linter;
use memory::*;
use mlir::*;
use parse::*;
use std::{
  ops::{Add, AddAssign},
  process::exit,
  time::Instant,
};
use token::*;
use typecheck::*;
use wasm_bindgen::prelude::wasm_bindgen;

pub use lint::*;

#[wasm_bindgen(module = "/src/abi/console.js")]
extern "C" {
  pub fn _compiler_print(s: String);
  pub fn _compiler_cls();
  pub fn _compiler_wat(s: String);
  pub fn _compiler_exec(bytes: Vec<u8>);
}

pub fn compiler_print(s: impl Into<String>) {
  _compiler_print(s.into());
}

pub fn fail_compile() -> ! {
  exit(1);
}

pub fn _compile(input: &str) -> Result<Vec<u8>> {
  _compiler_cls();
  let tokens = tokenize(input.chars())?;
  let parse_tree = parse(tokens);
  let mut hlir = build_hlir(parse_tree)?;
  let mlir = build_mlir(&mut hlir)?;
  type_check(&mut hlir, &mlir)?;
  let to_compile = sanitize(&mut hlir, &mlir)?;
  if let Some((ir, main)) = to_compile {
    let asm = Compiler::compile(hlir, ir.into_iter().collect::<Vec<_>>(), main);
    _compiler_wat(asm.clone());
    let bytes =
      wat::parse_str(asm).map_err(|_| lint_nospan(EvalLint::Unreachable))?;
    Ok(bytes)
  } else {
    Ok(vec![])
  }
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
    Err(e) => compiler_print(linter.render(e)),
  };
}
