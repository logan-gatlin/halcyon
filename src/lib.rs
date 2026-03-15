#![cfg_attr(test, allow(clippy::panic, clippy::unwrap_used))]
pub mod asm;
pub mod documentation;
pub mod hc_core;
pub mod ir;
pub mod linking;
pub mod logging;
pub mod map;
pub mod operator;
pub mod parse;
pub mod types;
#[cfg(target_arch = "wasm32")]
pub mod web;
use parse::ast::{
    AstNode,
    HasName,
    Statement,
};
pub use parse::tokenize;

#[cfg(test)]
mod test;

pub use indoc::*;
pub use logging::*;
pub use map::*;

pub use crate::hc_core::compile_core_module;
use crate::parse::ast::SourceFile;

/// Grabs the version number from Cargo.toml at compile time
pub const COMPILER_VERSION_STRING: &str = env!("CARGO_PKG_VERSION");
pub const WASM_MAGIC_NUMBER: [u8; 4] = [0, b'a', b's', b'm'];
pub const CORE_BUNDLE_NAME: &str = "core";
pub const CORE_MODULE_NAME: &str = CORE_BUNDLE_NAME;

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

#[derive(Debug, Clone)]
pub struct CompileOptions<F>
where
    F: FnMut(String) -> Option<String>,
{
    /// Implicitly declare the bundle name `_`
    pub demo_mode: bool,
    /// Link with the core bundle
    pub use_core: bool,
    /// Attempt to get the source text for a given path
    pub resolve_import: F,
}

fn name_resolution_prelude(symbols: &types::SymbolTable) -> Vec<(ir::Path, ir::NameSpace)> {
    let mut prelude = Vec::new();
    prelude.extend(
        symbols
            .terms()
            .keys()
            .cloned()
            .map(|path| (path, ir::NameSpace::Term)),
    );
    prelude.extend(
        symbols
            .constructors()
            .iter()
            .cloned()
            .map(|path| (path, ir::NameSpace::Constructor)),
    );
    prelude.extend(
        symbols
            .type_definitions()
            .keys()
            .cloned()
            .map(|path| (path, ir::NameSpace::Type)),
    );
    prelude.extend(
        symbols
            .trait_defs()
            .keys()
            .cloned()
            .map(|path| (path, ir::NameSpace::Trait)),
    );
    prelude.extend(
        symbols
            .trait_aliases()
            .keys()
            .cloned()
            .map(|path| (path, ir::NameSpace::Trait)),
    );
    prelude
}

fn bundle_name_for_source_file(
    source_file: &SourceFile,
    logger: &mut FileLogger,
    demo_mode: bool,
) -> String {
    let statements = source_file.statements();
    if !demo_mode && !matches!(statements.first(), Some(Statement::Bundle(_))) {
        let span = statements.first().map_or(Span::Generated, |statement| {
            match statement.span() {
                Span::Source { start, .. } => Span::new(start, 1),
                Span::Generated => Span::Generated,
            }
        });
        logger
            .error("Missing bundle declaration")
            .primary("Root file must start with `bundle <name>`.", span)
            .done();
    }

    statements
        .into_iter()
        .find_map(|statement| {
            if let Statement::Bundle(bundle_declaration) = statement {
                bundle_declaration.name_text()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "_".to_string())
}

fn compile_ir_bundle(
    ir_bundle: ir::Module<()>,
    symbols: &mut types::SymbolTable,
    logger: &mut FileLogger,
) -> Option<Artifact> {
    let resolved = types::resolve_module_with_symbols_and_schemes(symbols, ir_bundle, logger);
    if !logger.is_ok() {
        return None;
    }

    let elaborated = ir::elaborate_module(resolved, symbols);
    Some(asm::compile_module(elaborated, symbols))
}

fn link_artifacts_with_module_name(
    artifacts: &[Artifact],
    module_name: &str,
    logger: &mut Logger,
) -> Option<Vec<u8>> {
    let mut linking_logger = logger.linking_logger();
    let linked = linking::link_artifacts(
        artifacts,
        linking::LinkOptions {
            module_name: module_name.to_string(),
            ..Default::default()
        },
        &mut linking_logger,
    );
    logger.consume_file(linking_logger);
    linked.map(|artifact| artifact.binary)
}

fn validate_generated_wasm(
    binary: &[u8],
    logger: &mut Logger,
) -> bool {
    if let Err(error) = wasmparser::validate(binary) {
        let mut validation_logger = logger.new_file("<wasm-validation>", "");
        validation_logger
            .error("Invalid generated WebAssembly")
            .primary(error.message(), Span::Generated)
            .done();
        logger.consume_file(validation_logger);
        return false;
    }

    true
}

fn compile_source_linked_with_core_logger(source: &str) -> Result<Vec<u8>, Logger> {
    let mut symbols = types::SymbolTable::new();
    let mut logger = Logger::new();

    let artifacts = compile_source_with_options(
        "input.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: true,
            use_core: true,
            resolve_import: |_| None,
        },
    );
    if !logger.is_ok() {
        return Err(logger);
    }

    let Some(binary) = link_artifacts_with_module_name(&artifacts, "app", &mut logger) else {
        return Err(logger);
    };

    if !validate_generated_wasm(&binary, &mut logger) {
        return Err(logger);
    }

    Ok(binary)
}

#[tracing::instrument(skip_all)]
pub fn compile_source_with_options<F>(
    source_name: &str,
    source: &str,
    logger: &mut Logger,
    symbols: &mut types::SymbolTable,
    mut options: CompileOptions<F>,
) -> Box<[Artifact]>
where
    F: FnMut(String) -> Option<String>,
{
    let mut artifacts = Vec::new();

    if options.use_core {
        artifacts.push(compile_core_module(symbols, logger));
        if !logger.is_ok() {
            return artifacts.into_boxed_slice();
        }
    }

    let mut source_file_logger = logger.new_file(source_name, source);
    let Some(source_file) = parse::parse(source, &mut source_file_logger) else {
        logger.consume_file(source_file_logger);
        return artifacts.into_boxed_slice();
    };

    let bundle_name =
        bundle_name_for_source_file(&source_file, &mut source_file_logger, options.demo_mode);
    if !source_file_logger.is_ok() {
        logger.consume_file(source_file_logger);
        return artifacts.into_boxed_slice();
    }

    let prelude = name_resolution_prelude(symbols);
    let Some(ir_bundle) = ir::bundle_source_file_with_imports_and_prelude(
        bundle_name,
        source_file,
        source_file_logger,
        logger,
        &prelude,
        &mut options.resolve_import,
    ) else {
        return artifacts.into_boxed_slice();
    };

    if !logger.is_ok() {
        return artifacts.into_boxed_slice();
    }

    let mut typing_file_logger = logger.new_file(source_name, source);
    let compiled = compile_ir_bundle(ir_bundle, symbols, &mut typing_file_logger);
    logger.consume_file(typing_file_logger);
    let Some(compiled) = compiled else {
        return artifacts.into_boxed_slice();
    };

    artifacts.push(compiled);
    artifacts.into_boxed_slice()
}

#[tracing::instrument(skip_all)]
pub fn compile_source(
    source: &str,
    logger: &mut FileLogger,
    symbols: &mut types::SymbolTable,
) -> Box<[Artifact]> {
    let Some(source_file) = parse::parse(source, logger) else {
        return Vec::new().into_boxed_slice();
    };

    let mut bundle_name = "_".to_string();
    let mut saw_bundle_declaration = false;
    let mut statements = Vec::new();
    for statement in source_file.statements() {
        match statement {
            parse::ast::Statement::Bundle(bundle_declaration) => {
                if saw_bundle_declaration {
                    logger
                        .error("Duplicate bundle declaration")
                        .primary(
                            "A source file may only declare one bundle.",
                            bundle_declaration.span(),
                        )
                        .done();
                    continue;
                }
                saw_bundle_declaration = true;
                bundle_name = bundle_declaration
                    .name_text()
                    .unwrap_or_else(|| "_".to_string());
            }
            parse::ast::Statement::Import(_) => {}
            other => statements.push(other),
        }
    }

    if !logger.is_ok() {
        return Vec::new().into_boxed_slice();
    }

    let prelude = name_resolution_prelude(symbols);
    let Some(ir_bundle) =
        ir::bundle_statements_with_prelude(bundle_name, &statements, logger, &prelude)
    else {
        return Vec::new().into_boxed_slice();
    };

    let Some(compiled) = compile_ir_bundle(ir_bundle, symbols, logger) else {
        return Vec::new().into_boxed_slice();
    };

    vec![compiled].into_boxed_slice()
}

#[tracing::instrument(skip_all, fields(module = %art.module_name))]
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

pub fn compile_source_linked_with_core(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_linked_with_core_logger(source).map_err(|logger| logger.into_diagnostics())
}
