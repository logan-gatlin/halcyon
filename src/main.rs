#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![allow(dead_code)]
#![allow(unused_variables)]
mod err;
mod evaluate;
mod lookahead;
mod parse;
mod semantic;
mod token;
use std::ops::Add;

//use compile::Compiler;
use lookahead::*;
use parse::*;
use semantic::{Analyzer, typecheck::type_module};
use token::*;
use wasm_bindgen::prelude::wasm_bindgen;

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

/*
fn compile(input: String) -> Result<String> {
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parser = Parser::new(tokenizer);
  let mut an = semantic::Analyzer::new();
  let n = an.typecheck_program(parser.into_iter())?;
  let mut c = Compiler::new();
  c.compile(n.clone())
}

fn assemble(assembly: String) -> Result<Vec<u8>> {
  Compiler::assemble(assembly)
}
*/

#[wasm_bindgen]
pub fn parse_tree(input: String) -> String {
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parser = Parser::new(tokenizer);
  parser
    .into_iter()
    .map(|s| format!("{s:#?}"))
    .collect::<Vec<_>>()
    .join("\n")
}

#[wasm_bindgen]
pub fn ast(input: String) -> String {
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parser = Parser::new(tokenizer);
  let ast = Analyzer::analyze(parser);
  match ast {
    Ok(m) => format!("{m:#?}"),
    Err(e) => format!("{e}"),
  }
}
/*
#[wasm_bindgen]
pub fn check_errors(input: String) -> String {
  match compile(input) {
    Ok(_) => "Compiled successfully!".to_string(),
    Err(e) => format!("{e}"),
  }
}

#[wasm_bindgen]
pub fn try_compile(input: String) -> Vec<u8> {
  let asm = compile(input).unwrap_or_default();
  assemble(asm).unwrap_or_default()
}
*/
fn main() {
  let input = include_str!("../demo.hal").to_string();
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parser = Parser::new(tokenizer);
  let ast = Analyzer::analyze(parser).unwrap();
  type_module(ast).unwrap();
}

/*
#[test]
fn compile_file() {
  let assembly = compile(include_str!("../demo.hal").to_string()).unwrap();
  std::fs::write("./test.wat", assembly.clone()).unwrap();
  let bytes = assemble(assembly).unwrap();
  std::fs::write("./test.wasm", bytes).unwrap();
}
*/
