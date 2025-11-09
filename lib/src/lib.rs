#![allow(clippy::clone_on_copy, clippy::from_over_into)]
#![feature(iterator_try_collect, if_let_guard)]

pub mod ir2;
pub mod logging;
pub mod map;
pub mod module_data;
pub mod operator;
pub mod parse;
pub mod semantic;
pub mod std_hc;
#[cfg(test)]
mod test;
pub mod token;

pub use logging::*;
pub use map::*;
use parse::*;
use token::*;

use crate::ir2::PrettyPrint;

pub fn compile(input: &str) -> (Vec<u8>, LoggerT) {
    let mut logger = LoggerT::new(0);
    let tokens = tokenize(input.chars(), &mut logger);
    let parse_trees = parse(&mut logger, tokens);

    for p in parse_trees {
        let ir_module = ir2::build_ir(&mut logger, p);
        println!("{}", ir_module.pretty());
    }
    if !logger.is_ok() {
        panic!();
    }
    (vec![], logger)
}
