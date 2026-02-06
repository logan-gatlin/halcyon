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

pub mod asm;
pub mod hc_core;
pub mod ir;
pub mod logging;
pub mod map;
pub mod operator;
pub mod parse;
pub mod semantic;
pub mod token;

pub use ir::{
    PrettyPrint,
    SymbolTable,
    build_ir,
};
pub use parse::parse;
pub use token::tokenize;

#[cfg(test)]
mod test;

pub use indoc::*;
pub use logging::*;
pub use map::*;

#[derive(Debug, Clone)]
pub struct Artifact {
    pub module_name: String,
    pub parse_tree: parse::ParsedModule,
    pub ir_module: ir::Module,
    /// `None` if compilation failed
    pub asm_module: Option<asm::Module>,
    /// Empty if compilation failed
    pub binary: Vec<u8>,
}

impl Artifact {
    pub fn is_ok(&self) -> bool {
        self.asm_module.is_some() && !self.binary.is_empty()
    }
}

pub fn compile(
    input: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<Artifact> {
    let tokens = tokenize(input.chars(), logger);
    let parse_trees = parse(logger, tokens);
    let mut artifacts = vec![];

    for p in parse_trees {
        let mut ir_module = build_ir(logger, symbols, p.clone());
        semantic::analyze(&mut ir_module, symbols, logger);
        let (asm_module, binary) = if logger.is_ok() {
            let asm_module = asm::lower_module(ir_module.clone(), symbols);
            let binary = asm::encode(asm_module.clone());
            (Some(asm_module), binary)
        } else {
            (None, vec![])
        };
        artifacts.push(Artifact {
            module_name: p.name.inner.clone(),
            parse_tree: p,
            ir_module,
            asm_module,
            binary,
        });
    }
    artifacts
}
