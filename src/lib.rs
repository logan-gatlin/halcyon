// All warnings and style lints are errors
#![deny(
    clippy::all,
    clippy::exit,
    clippy::empty_structs_with_brackets,
    clippy::if_then_some_else_none,
    clippy::infinite_loop,
    clippy::map_with_unused_argument_over_ranges,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::panic,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::return_and_then,
    clippy::self_named_module_files,
    clippy::string_lit_chars_any,
    clippy::string_lit_as_bytes,
    clippy::string_slice,
    clippy::try_err,
    mismatched_lifetime_syntaxes
)]
#![cfg_attr(test, allow(clippy::panic, clippy::unwrap_used))]
// Debug tools set to warn to help find and remove before deploying
#![warn(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::todo,
    clippy::unwrap_used
)]
#![allow(clippy::large_enum_variant, clippy::result_large_err)]

pub mod hc_core;
pub mod ir;
pub mod logging;
pub mod map;
pub mod operator;
pub mod parse;
pub mod token;
pub mod types;
/*pub use ir::{
    PrettyPrint,
    SymbolTable,
    build_ir,
};*/
//pub use parse::parse;
pub use token::tokenize;

#[cfg(test)]
mod test;

pub use indoc::*;
pub use logging::*;
pub use map::*;

// Grab the version number from Cargo.toml at compile time
pub const COMPILER_VERSION_STRING: &str = env!("CARGO_PKG_VERSION");
pub const WASM_MAGIC_NUMBER: [u8; 4] = [0, b'a', b's', b'm'];
pub const CORE_MODULE_NAME: &str = "core";

pub fn compile_source(
    source: &str,
    logger: &mut FileLogger,
) -> Box<[ir::Module<types::Type>]> {
    parse::parse(source, logger)
        .map(|m| m.modules())
        .unwrap_or_default()
        .into_iter()
        .map(|m| {
            println!("{m:#?}");
            m
        })
        .flat_map(|m| ir::module(m, logger))
        .collect::<Box<[_]>>()
        .into_iter()
        .map(|m| types::resolve_module(m, logger))
        .collect()
}
