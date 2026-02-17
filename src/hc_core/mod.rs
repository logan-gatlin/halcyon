/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod types;

pub use terms::CoreSymbols;
pub use types::CoreTypes;

use crate::new_ir::Path;

pub const CORE_MODULE_NAME: &str = "core";

/*
pub fn compile_core_module(
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Option<Artifact> {
    let mut syms = SymbolTable::new();
    types::type_definitions(&mut syms);
    let mut module = asm::Module::new(CORE_MODULE_NAME.into());
    let init_name = core("[init]");
    let mut init_func = module.new_function(init_name.clone());
    terms::operator_definitions(&mut init_func, &mut syms);
    terms::put_str_definition(&mut init_func, &mut syms);
    module.start = init_name;
    module.sig = TypeSignatureSection::new(CORE_MODULE_NAME, &syms);
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
*/
