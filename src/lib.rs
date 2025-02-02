#![feature(let_chains)]
#![feature(iterator_try_collect)]
mod compile;
mod err;
mod lookahead;
mod parse;
mod semantic;
mod token;
mod treewalk;
use std::{collections::HashMap, ops::Add};

use compile::Compiler;
use err::*;
use lookahead::*;
use parse::*;
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

fn test_expression(expr: &str) {
  let source = expr.to_string();
  let tokens = Tokenizer::new(source.chars()).filter(|t| t.0.is_meaningful());
  let mut parser = Parser::new(tokens);
  println!("{:#?}", parser.statement().unwrap());
}

fn compile(input: String) -> Result<Vec<u8>> {
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parser = Parser::new(tokenizer);
  let mut an = semantic::Analyzer::new();
  let n = an.typecheck_program(parser.into_iter())?;
  let mut c = Compiler::new();
  c.compile(n.clone())
}

#[wasm_bindgen]
pub fn parse_tree(input: String) -> String {
  let tokenizer = Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful());
  let parser = Parser::new(tokenizer);
  let mut an = semantic::Analyzer::new();
  match an.typecheck_program(parser.into_iter()) {
    Ok(m) => format!("{m:#?}"),
    Err(e) => format!("{e}"),
  }
}

#[wasm_bindgen]
pub fn check_errors(input: String) -> String {
  match compile(input) {
    Ok(_) => "Compiled successfully!".to_string(),
    Err(e) => format!("{e}"),
  }
}

#[wasm_bindgen]
pub fn try_compile(input: String) -> Vec<u8> {
  compile(input).unwrap_or_default()
}
