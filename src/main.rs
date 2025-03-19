#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![allow(unused_imports)]
mod compile;
mod graph;
mod hlir;
mod lint;
mod mlir;
mod operator;
mod parse;
mod token;
//mod typecheck;

use hlir::*;
use lint::render::{Linter, UnwrapLint};
pub use lint::*;
use mlir::*;
use parse::*;
use std::{
  ops::{Add, AddAssign},
  process::exit,
  time::Instant,
};
use token::*;
use wasm_bindgen::prelude::wasm_bindgen;

pub fn compiler_print(s: impl Into<String>) {
  println!("{}", s.into())
}

pub fn fail_compile() -> ! {
  exit(1);
}

fn main() {
  let start_time = Instant::now();
  let input = include_str!("../demo.hc").to_string();
  let linter = Linter::new(input.clone());
  let tokens = tokenize(input.chars()).unwrap_lint(&linter);
  let parse_tree = parse(tokens);
  let hlir = build_hlir(parse_tree).unwrap_lint(&linter);
  let mlir = build_mlir(&hlir);
  for block in mlir.blocks.clone() {
    if let BlockKind::Constant { .. } | BlockKind::GlobalScope = &block.1.kind {
      let value = mlir.evaluate_block(&block.0).unwrap();
      println!("value = {value}");
    }
  }
  /*
  let cflow = Analyzer::analyze(&canon_module).unwrap_lint(&linter);
  let solution = Solver::solve(cflow).unwrap_lint(&linter);
  let (module, clean_nodes) = TypeChecker::typecheck(canon_module, solution).unwrap_lint(&linter);
  let assembly = Compiler::compile(module, clean_nodes.into_iter().collect());
  println!("{assembly}");
  compiler_print(format!(
    "Compiled successfully in {}ms",
    Instant::now().duration_since(start_time).as_millis(),
  ));
  std::fs::write("test.wat", assembly).unwrap();
  */
}
