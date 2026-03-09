/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod types;

pub use terms::CoreTerm;
pub use types::CoreType;

use enum_iterator::all;

use crate::asm;

use crate::Artifact;
use crate::ir::Path;
use crate::logging::WithContext;
use crate::types::symbol_table::{
    Symbol,
    SymbolKind,
};
use crate::types::{
    SymbolTable,
    Type,
};

pub const CORE_MODULE_NAME: &str = "core";

pub fn compile_core_module(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Artifact {
    register_core_primitive_symbols(symbols);

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

fn register_core_primitive_symbols(symbols: &mut SymbolTable) {
    all::<CoreType>().for_each(|symbol| {
        symbols.insert(symbol);
    });
    all::<CoreTerm>().for_each(|symbol| {
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

pub(crate) fn core_impl_arguments(arguments: &[Type]) -> Vec<Type> {
    arguments.iter().map(normalize_impl_argument).collect()
}

pub(crate) fn core_impl_path(
    method_path: &Path,
    arguments: &[Type],
) -> Path {
    let args = core_impl_arguments(arguments);
    let arg_key = args.iter().map(type_key).collect::<Vec<_>>().join("_");
    let minor = if arg_key.is_empty() {
        format!("[impl] {} {}", method_path.major, method_path.minor)
    } else {
        format!(
            "[impl] {} {} {}",
            method_path.major, method_path.minor, arg_key
        )
    };
    Path::new(CORE_MODULE_NAME, minor)
}

pub(crate) fn normalize_impl_argument(type_: &Type) -> Type {
    let mut current = type_.clone();
    while let Type::ForAll(body) = current {
        current = *body;
    }
    current
}

fn type_key(type_: &Type) -> String {
    type_
        .pretty()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
