/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod types;

use crate::asm::custom_section::SignatureSection;
use crate::ir::*;
use crate::semantic::{
    AbstractType,
    Type,
};
use crate::{
    asm,
    link_binary,
    Artifact,
    Logger,
};

pub const CORE_MODULE_NAME: &str = "core";

fn core(s: impl Into<String>) -> Path {
    Path::new(CORE_MODULE_NAME, s)
}

pub fn compile_core_module(
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Option<Artifact> {
    let mut syms = SymbolTable::new();
    types::type_definitions(&mut syms);
    let mut module = asm::Module::new(CORE_MODULE_NAME.into());
    let mut init_func = module.new_function();
    terms::operator_definitions(&mut init_func, &mut syms);
    module.sig = SignatureSection::new(CORE_MODULE_NAME, &syms);
    link_binary("<core-module>", &asm::encode(module), logger, symbols)
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use crate::{
        Logger,
        SymbolTable,
    };

    #[test]
    fn core_validates() {
        let mut logger = Logger::new();
        let mut symbols = SymbolTable::new();
        super::compile_core_module(&mut logger, &mut symbols);
        logger.print_logs();
        assert!(logger.is_ok());
    }
}
