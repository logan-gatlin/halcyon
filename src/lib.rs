// All warnings and style lints are errors
#![deny(
    clippy::all,
    clippy::exit,
    clippy::expect_used,
    clippy::empty_structs_with_brackets,
    clippy::if_then_some_else_none,
    clippy::infinite_loop,
    clippy::map_with_unused_argument_over_ranges,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::panic,
    clippy::pattern_type_mismatch,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::return_and_then,
    clippy::self_named_module_files,
    clippy::shadow_unrelated,
    clippy::string_lit_chars_any,
    clippy::string_lit_as_bytes,
    clippy::string_slice,
    clippy::try_err,
    clippy::unwrap_used
)]
// Debug tools set to warn to help find and remove before deploying
#![warn(clippy::print_stdout, clippy::print_stderr, clippy::todo)]
#![feature(iterator_try_collect, if_let_guard)]
pub mod ir;
pub mod logging;
pub mod map;
pub mod operator;
pub mod parse;
pub mod semantic;
pub mod std_hc;
#[cfg(test)]
mod test;
pub mod token;

pub use indoc::*;
pub use logging::*;
pub use map::*;
use parse::*;
use token::*;

use crate::ir::PrettyPrint;

pub fn compile(input: &str) -> (Vec<u8>, LoggerT) {
    let mut logger = LoggerT::new(0);
    let tokens = tokenize(input.chars(), &mut logger);
    let parse_trees = parse(&mut logger, tokens);

    for p in parse_trees {
        let ir_module = ir::build_ir(&mut logger, p);
        println!("{}", ir_module.pretty());
    }
    if !logger.is_ok() {
        panic!();
    }
    (vec![], logger)
}
