mod err;
mod lookahead;
mod parse;
mod token;
mod treewalk;
use std::ops::Add;

use lookahead::*;
use parse::*;
use token::*;
use treewalk::Interpreter;

#[derive(Clone, Copy, Debug)]
pub struct Span {
  pub row: usize,
  pub column: usize,
}

impl Add<Span> for Span {
  type Output = Span;

  fn add(self, rhs: Span) -> Self::Output {
    Span {
      row: usize::min(self.row, rhs.row),
      column: usize::max(self.column, rhs.column),
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

fn main() {
  let src = include_str!("../demo.lang");
  println!("{src}");
  let tokens: Vec<_> = Tokenizer::new(src.chars())
    .filter(|t| t.0.is_meaningful())
    .collect();
  let parsed = Parser::new(tokens.into_iter()).file().unwrap();
  let mut interp = Interpreter::new(parsed.into_iter());
  interp.run().unwrap();
}
