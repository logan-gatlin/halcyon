#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![allow(unused_imports)]
mod assembly;
mod buffer;
mod compile;
mod err;
mod graph;
mod ir;
mod naming;
mod parse;
mod token;
mod typecheck;
use std::ops::Add;

use buffer::*;
use compile::Compiler;
use ir::solver::Solver;
use naming::{Canonizer, control_flow::Analyzer};
use parse::*;
use token::*;
use typecheck::TypeChecker;
use wasm_bindgen::prelude::wasm_bindgen;

pub fn compiler_print(s: impl Into<String>) {
  println!("{}", s.into())
}

#[derive(Clone, Copy, Debug)]
pub struct Span {
  pub row: usize,
  pub column: usize,
}

impl std::fmt::Display for Span {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}:{})", self.row, self.column)
  }
}

impl Add<Span> for Span {
  type Output = Span;

  fn add(self, rhs: Span) -> Self::Output {
    let max = (self.row, self.column).max((rhs.row, rhs.column));
    Span {
      row: max.0,
      column: max.1,
    }
  }
}

fn main() {
  let input = include_str!("../demo.hc").to_string();
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parse_tree = Parser::new(tokenizer).collect::<Vec<_>>();
  let canon_module = Canonizer::canonize_ast(parse_tree).unwrap();
  let cflow = Analyzer::analyze(&canon_module).unwrap();
  let solution = Solver::solve(cflow).unwrap();
  let (module, clean_nodes) =
    TypeChecker::typecheck(canon_module, solution).unwrap();
  Compiler::compile(module, clean_nodes.into_iter().collect());
}
