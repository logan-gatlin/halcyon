#![feature(let_chains)]
#![feature(iterator_try_collect)]
mod err;
mod lookahead;
mod parse;
mod semantic;
mod token;
mod treewalk;
use std::ops::Add;

use err::*;
use lookahead::*;
use parse::*;
use token::*;

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

fn test_tokenization() {
  let test_str = r#"
  ( ) [ ] { } . .. , :  
  ; + - * / -> => += -=
  *= /= ! != = == <= >=
  ? ?= < > literal 10 
  0x10 0b10 10.0 1.0..2.0
  2.0..=3.0 if else and
  or xor not nand nor xnor 
  print break for while true
  false "\u263b" '\x30'
  "#;
  let mut parser = Tokenizer::new(test_str.chars());
  while let Some(tok) = parser.next() {
    println!("{tok:?}");
  }
}

fn test_expression(expr: &str) {
  let source = expr.to_string();
  let tokens = Tokenizer::new(source.chars()).filter(|t| t.0.is_meaningful());
  let mut parser = Parser::new(tokens);
  println!("{:#?}", parser.statement().unwrap());
}

fn tokenize(input: &'static str) -> impl Iterator<Item = Token> {
  Tokenizer::new(input.chars()).filter(|t| t.0.is_meaningful())
}

fn parse(input: &'static str) -> impl Iterator<Item = Statement> {
  Parser::new(tokenize(input))
}

fn main() -> Result<()> {
  let program: Vec<_> = parse(include_str!("../demo.hal")).collect();
  let mut an = semantic::Analyzer::new();
  let n = an.analyze_scope(program.into_iter()).unwrap();
  an.print_table();
  //println!("{n:#?}");
  Ok(())
}
