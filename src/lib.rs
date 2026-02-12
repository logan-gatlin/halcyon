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
use wasmparser::{
    KnownCustom,
    Parser,
};

// Grab the version number from Cargo.toml at compile time
pub const COMPILER_VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

use crate::asm::custom_section::TypeSignatureSection;
use crate::asm::validate_wasm;

#[derive(Debug, Clone)]
pub enum Artifact {
    /// A pre-compiled WASM binary linked into the project
    Binary {
        module_name: String,
        binary: Vec<u8>,
    },
    /// A compiled Halcyon source module
    Source {
        module_name: String,
        parse_tree: parse::ParsedModule,
        ir_module: ir::Module,
        /// `None` if compilation failed
        asm_module: Option<asm::Module>,
        /// Empty if compilation failed
        binary: Vec<u8>,
    },
}

impl Artifact {
    pub fn module_name(&self) -> &str {
        match self {
            Self::Binary { module_name, .. } | Self::Source { module_name, .. } => module_name,
        }
    }

    pub fn binary(&self) -> &[u8] {
        match self {
            Self::Binary { binary, .. } | Self::Source { binary, .. } => binary,
        }
    }

    pub fn into_binary(self) -> Vec<u8> {
        match self {
            Self::Binary { binary, .. } | Self::Source { binary, .. } => binary,
        }
    }
}

pub fn compile_file(
    file_name: &str,
    input: &[u8],
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<Artifact> {
    if let Some([0x0, 0x61, 0x73, 0x6D] /* WASM magic number */) = input.get(0..4) {
        link_binary(file_name, input, logger, symbols)
            .map(|a| vec![a])
            .unwrap_or_default()
    } else {
        let input = String::from_utf8_lossy(input);
        compile_source(file_name, &input, logger, symbols)
    }
}

pub fn link_binary(
    file_name: &str,
    input: &[u8],
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Option<Artifact> {
    let mut file_logger = validate_wasm(file_name, input, logger);
    let parser = Parser::new(0);
    let mut signature_section = None;
    let mut module_name = None;
    let mut corrupted_module = |mut l: FileLogger| {
        l.error("Module is missing necessary metadata")
            .note("Was it produced by the same version of Halcyon compiler?")
            .done();
        logger.consume_file(l);
    };
    for payload in parser.parse_all(input) {
        if let Ok(wasmparser::Payload::CustomSection(r)) = payload {
            match r.name() {
                TypeSignatureSection::NAME => {
                    let Some(s) = TypeSignatureSection::decode_data_slice(r.data()) else {
                        corrupted_module(file_logger);
                        return None;
                    };
                    signature_section = Some(s);
                }
                "name" => {
                    let KnownCustom::Name(mut n) = r.as_known() else {
                        corrupted_module(file_logger);
                        return None;
                    };
                    let Some(parsed_module_name) = n.find_map(|n| {
                        if let Ok(wasmparser::Name::Module { name, .. }) = n {
                            Some(name)
                        } else {
                            None
                        }
                    }) else {
                        corrupted_module(file_logger);
                        return None;
                    };
                    module_name = Some(parsed_module_name);
                }
                _ => {}
            }
        }
    }
    let (Some(module_name), Some(sig)) = (module_name, signature_section) else {
        corrupted_module(file_logger);
        return None;
    };
    for path in &sig.imported_types {
        if !symbols.types.contains_key(path) {
            file_logger
                .error(format!("Missing definition of type `{path}`"))
                .done();
        }
    }
    for (path, at) in sig.defined_types {
        let named_type = semantic::Type::Instantiation(
            path.clone(),
            at.variables
                .iter()
                .map(|v| semantic::Type::Variable(*v))
                .collect(),
        );
        match &at.base {
            semantic::Type::Sum {
                variant_names,
                variant_types,
                ..
            } => {
                for (tag, (variant_name, variant_type)) in
                    variant_names.iter().zip(variant_types).enumerate()
                {
                    let cons_path = ir::Path::new(path.major.clone(), variant_name);
                    symbols.constructors.insert(
                        cons_path.clone(),
                        if *variant_type == semantic::Type::Unit {
                            ir::Constructor::SumConstant {
                                tag,
                                sum_type: named_type.clone(),
                            }
                        } else {
                            ir::Constructor::SumFunction {
                                tag,
                                sum_type: named_type.clone(),
                                parameter_type: variant_type.clone(),
                            }
                        },
                    );
                    symbols.terms.insert(
                        cons_path,
                        if *variant_type == semantic::Type::Unit {
                            named_type.clone()
                        } else {
                            semantic::Type::func(variant_type.clone(), named_type.clone())
                        },
                    );
                }
            }
            semantic::Type::Struct { .. } => {
                let cons_path = ir::Path::new(path.major.clone(), &path.minor);
                symbols.constructors.insert(
                    cons_path.clone(),
                    ir::Constructor::Structure(named_type.clone()),
                );
                symbols.terms.insert(
                    cons_path,
                    semantic::Type::func(named_type.clone(), named_type),
                );
            }
            _ => {}
        }
        symbols.types.insert(path, at);
    }
    for (path, mut t) in sig.defined_terms {
        t.visit(|t: &mut semantic::Type| {
            if let semantic::Type::Instantiation(type_path, _) = t
                && !symbols.types.contains_key(type_path)
            {
                file_logger
                    .error(format!("Missing definition of type `{type_path}`"))
                    .done();
            }
        });
        symbols.terms.insert(path, t);
    }
    logger.consume_file(file_logger);
    Some(Artifact::Binary {
        module_name: module_name.to_string(),
        binary: input.to_vec(),
    })
}

pub fn compile_source(
    file_name: &str,
    input: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<Artifact> {
    let mut file_logger = logger.new_file(file_name, input);
    let tokens = tokenize(input.chars(), &mut file_logger);
    let parse_trees = parse(&mut file_logger, tokens);
    let mut artifacts = vec![];

    for p in parse_trees {
        let mut ir_module = build_ir(&mut file_logger, symbols, p.clone());
        semantic::analyze(&mut ir_module, symbols, &mut file_logger);
        let (asm_module, binary) = if logger.is_ok() {
            let asm_module = asm::lower_module(ir_module.clone(), symbols);
            let binary = asm::encode(asm_module.clone());
            (Some(asm_module), binary)
        } else {
            (None, vec![])
        };
        artifacts.push(Artifact::Source {
            module_name: p.name.inner.clone(),
            parse_tree: p,
            ir_module,
            asm_module,
            binary,
        });
    }
    artifacts
}
