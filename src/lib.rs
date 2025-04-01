#![feature(let_chains, generic_const_exprs, iterator_try_collect, box_patterns)]
#![allow(unused_imports, dead_code, unused_imports, incomplete_features)]
mod compile;
mod hlir;
mod lint;
mod memory;
mod mlir;
mod operator;
mod parse;
mod token;

use hlir::*;
use lint::render::Linter;
use parse::*;
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
pub fn _compiler_cls() {}
#[cfg(not(target_family = "wasm"))]
pub fn _compiler_wat(_s: String) {}
#[cfg(not(target_family = "wasm"))]
pub fn _compiler_exec(_bytes: Vec<u8>) {}
#[cfg(not(target_family = "wasm"))]
pub fn compiler_print(s: impl Into<String>) {
  _compiler_print(s.into());
}

pub fn _compile(input: &str) -> Result<Vec<u8>> {
  _compiler_cls();
  let tokens = tokenize(input.chars())?;
  let parse_tree = parse(tokens)?;
  println!("{parse_tree}");
  let hlir = build_hlir(parse_tree)?;
  let hlir_sexpr: SExpression = (&hlir).into();
  println!("{hlir_sexpr}");
  println!("{hlir:#?}");
  Ok(vec![])
  /*
  let mut hlir = build_hlir(parse_tree)?;
  let mlir = build_mlir(&mut hlir)?;
  type_check(&mut hlir, &mlir)?;
  let to_compile = sanitize(&mut hlir, &mlir)?;
  if let Some((ir, main)) = to_compile {
    let asm = Compiler::compile(hlir, ir.into_iter().collect::<Vec<_>>(), main);
    compiler_print(
      "Compiled Successfully".apply_style(Color::Green, Attribute::Underline),
    );
    _compiler_wat(asm.clone());
    let bytes =
      wat::parse_str(asm).map_err(|_| lint_nospan(EvalLint::Unreachable))?;
    Ok(bytes)
  } else {
    compiler_print(
      "Compiled Successfully".apply_style(Color::Green, Attribute::Underline),
    );
    Ok(vec![])
  }
  */
}

#[wasm_bindgen]
pub fn compile(input: &str) {
  let linter = Linter::new(input.to_string());
  match _compile(input) {
    Ok(b) => {
      if b.len() != 0 {
        _compiler_exec(b);
      }
    }
    Err(e) => {
      compiler_print("Failed to Compile".apply_style(Color::Red, Attribute::Underline));
      compiler_print(linter.render(e))
    }
  };
}
