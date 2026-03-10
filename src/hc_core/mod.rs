/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod symbols;
mod types;

pub use symbols::CoreSymbol;
pub use types::CoreType;

use enum_iterator::all;

use crate::asm;

use crate::Artifact;
use crate::logging::WithContext;
use crate::types::SymbolTable;
use crate::types::symbol_table::Symbol;

pub const CORE_MODULE_NAME: &str = "core";

pub fn compile_core_module(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Artifact {
    register_core_primitive_types(symbols);

    let resolved_modules = resolve_core_source_modules(symbols, logger);
    if !logger.is_ok() {
        return Artifact {
            module_name: CORE_MODULE_NAME.to_string(),
            ir_module: None,
            binary: Vec::new(),
        };
    }

    let mut wasm_module = None;
    for resolved in resolved_modules {
        let elaborated = crate::ir::elaborate_module(resolved, symbols);
        wasm_module = Some(asm::lower_module(elaborated, symbols));
    }

    Artifact {
        module_name: CORE_MODULE_NAME.to_string(),
        ir_module: None,
        binary: wasm_module.map_or_else(Vec::new, asm::encode),
    }
}

fn register_core_primitive_types(symbols: &mut SymbolTable) {
    all::<CoreType>().for_each(|symbol| {
        symbols.insert(symbol);
    });
}

fn resolve_core_source_modules(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Vec<crate::types::ResolvedModule> {
    let source = include_str!("core.hc");
    let mut file_logger = logger.new_file("core.hc", source);

    let Some(source_file) = crate::parse::parse(source, &mut file_logger) else {
        file_logger.escalate_to_bug();
        logger.consume_file(file_logger);
        return vec![];
    };

    let module_nodes = source_file.modules();
    if module_nodes.is_empty() {
        file_logger
            .bug("bundled core module source did not contain any modules")
            .done();
        logger.consume_file(file_logger);
        return vec![];
    }

    let prelude = all::<CoreType>()
        .map(|symbol| (symbol.path(), crate::ir::NameSpace::Type))
        .collect::<Vec<_>>();

    let mut resolved = Vec::new();
    for module_node in module_nodes {
        let Some(ir_module) =
            crate::ir::module_with_prelude(module_node, &mut file_logger, &prelude)
        else {
            file_logger.escalate_to_bug();
            logger.consume_file(file_logger);
            return resolved;
        };

        resolved.push(crate::types::resolve_module_with_symbols_and_schemes(
            symbols,
            ir_module,
            &mut file_logger,
        ));
    }

    if !file_logger.is_ok() {
        file_logger.escalate_to_bug();
        logger.consume_file(file_logger);
        return vec![];
    }
    logger.consume_file(file_logger);
    resolved
}
