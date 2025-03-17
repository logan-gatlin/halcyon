#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![allow(unused_imports)]
mod assembly;
mod compile;
mod graph;
mod ir;
mod lint;
mod naming;
mod parse;
mod token;
mod typecheck;

use compile::Compiler;
use ir::solver::Solver;
use lint::render::{Linter, UnwrapLint};
pub use lint::*;
use naming::{Canonizer, control_flow::Analyzer};
use parse::*;
use std::{
  ops::{Add, AddAssign},
  process::exit,
  time::Instant,
};
use token::*;
use typecheck::TypeChecker;
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
  let canon_module = Canonizer::canonize_ast(parse_tree).unwrap_lint(&linter);
  let cflow = Analyzer::analyze(&canon_module).unwrap_lint(&linter);
  let solution = Solver::solve(cflow).unwrap_lint(&linter);
  let (module, clean_nodes) =
    TypeChecker::typecheck(canon_module, solution).unwrap_lint(&linter);
  let assembly = Compiler::compile(module, clean_nodes.into_iter().collect());
  println!("{assembly}");
  compiler_print(format!(
    "Compiled successfully in {}ms",
    Instant::now().duration_since(start_time).as_millis(),
  ));
  std::fs::write("test.wat", assembly).unwrap();
}
