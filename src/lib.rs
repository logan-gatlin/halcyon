#![cfg_attr(test, allow(clippy::panic, clippy::unwrap_used))]
pub mod asm;
pub mod hc_core;
pub mod ir;
pub mod logging;
pub mod map;
pub mod operator;
pub mod parse;
pub mod token;
pub mod types;
pub use token::tokenize;

#[cfg(test)]
mod test;

pub use indoc::*;
pub use logging::*;
pub use map::*;

pub use crate::hc_core::compile_core_module;

/// Grabs the version number from Cargo.toml at compile time
pub const COMPILER_VERSION_STRING: &str = env!("CARGO_PKG_VERSION");
pub const WASM_MAGIC_NUMBER: [u8; 4] = [0, b'a', b's', b'm'];
pub const CORE_MODULE_NAME: &str = "core";

#[derive(Debug, Clone)]
pub struct Artifact {
    pub module_name: String,
    pub ir_module: Option<ir::Module<types::Type>>,
    pub binary: Vec<u8>,
}

impl Artifact {
    pub fn decompile_to_wat(&self) -> Option<String> {
        wasmprinter::print_bytes(&self.binary).ok()
    }
    pub fn save_wasm_to_file(
        &self,
        location: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        let path = location.as_ref().join(format!("{}.wasm", self.module_name));
        std::fs::write(path, &self.binary)
    }
    pub fn save_wat_to_file(
        &self,
        location: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        let Some(wat) = self.decompile_to_wat() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not printable as WAT",
            ));
        };
        let path = location.as_ref().join(format!("{}.wat", self.module_name));
        std::fs::write(path, wat)
    }
}

pub fn compile_source(
    source: &str,
    logger: &mut FileLogger,
    symbols: &mut types::SymbolTable,
) -> Box<[Artifact]> {
    parse::parse(source, logger)
        .into_iter()
        .flat_map(|m| m.modules())
        .flat_map(|m| ir::module(m, logger))
        .collect::<Box<_>>()
        .into_iter()
        .map(|m| types::resolve_module_with_symbols_and_schemes(symbols, m, logger))
        .collect::<Box<_>>()
        .into_iter()
        .map(|m| ir::elaborate_module(m, symbols))
        .map(|m| asm::compile_module(m, symbols))
        .collect()
}

pub fn validate_artifact(
    art: Artifact,
    logger: &mut Logger,
) -> Artifact {
    fn find_wat_line_for_offset(
        offset_map: &[(usize, Option<usize>)],
        target_offset: usize,
    ) -> usize {
        let mut best_line = 1;
        for &(line, offset) in offset_map {
            if let Some(offset) = offset
                && offset <= target_offset
            {
                best_line = line;
            }
        }
        best_line
    }

    fn wat_line_byte_offset(
        wat: &str,
        line: usize,
    ) -> usize {
        wat.lines()
            .take(line.saturating_sub(1))
            .map(|line| line.len() + 1)
            .sum()
    }

    let file_name = format!("{}.wat", art.module_name);
    let mut wat_with_offsets = String::new();
    let offset_map = wasmprinter::Config::new()
        .offsets_and_lines(&art.binary, &mut wat_with_offsets)
        .map(|iter| {
            iter.enumerate()
                .map(|(idx, (offset, _text))| (idx + 1, offset))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let wat = if wat_with_offsets.is_empty() {
        art.decompile_to_wat()
    } else {
        Some(wat_with_offsets)
    };
    let Some(wat) = wat else {
        let mut dummy_file = logger.new_file(file_name, "");
        dummy_file
            .bug("Failed to produces a valid WASM file")
            .done();
        logger.consume_file(dummy_file);
        return art;
    };
    let mut file_logger = logger.new_file(file_name, wat.clone());
    if let Err(error) = wasmparser::validate(&art.binary) {
        let line = find_wat_line_for_offset(&offset_map, error.offset());
        let byte_start = wat_line_byte_offset(&wat, line);
        let byte_end = wat_line_byte_offset(&wat, line + 1).min(wat.len());
        let span = Span::new(byte_start, byte_end.saturating_sub(byte_start));
        file_logger
            .bug(error.message())
            .primary("Failed validation here", span)
            .done();
    }
    logger.consume_file(file_logger);
    art
}
