/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod types;

use crate::asm;
use crate::ir::*;
use crate::semantic::{
    AbstractType,
    Type,
};

pub const CORE_MODULE_NAME: &str = "core";

fn core(s: impl Into<String>) -> Path {
    Path::new(CORE_MODULE_NAME, s)
}

pub fn compile_core_module() -> Vec<u8> {
    let mut syms = SymbolTable::new();
    types::type_definitions(&mut syms);
    let mut module = asm::Module::new(CORE_MODULE_NAME.into());
    let mut init_func = module.new_function();
    terms::operator_definitions(&mut init_func, &mut syms);
    asm::encode(module)
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use crate::asm::validate_wasm;
    use crate::{
        Logger,
        SymbolTable,
        link_binary,
    };

    #[test]
    fn core_validates() {
        let mut logger = Logger::new();
        let mut symbols = SymbolTable::new();
        let bin = link_binary(
            "<core-module>",
            &super::compile_core_module(),
            &mut logger,
            &mut symbols,
        )
        .unwrap()
        .binary;
        let wat_logs = validate_wasm("<core-module>", &bin, &mut logger);
        logger.consume_file(wat_logs);
        logger.print_logs();
        assert!(logger.is_ok());
    }
}
