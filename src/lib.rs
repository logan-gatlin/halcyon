#![cfg_attr(test, allow(clippy::panic, clippy::unwrap_used))]
pub mod asm;
pub mod bindings;
pub mod documentation;
pub mod hc_core;
pub mod ir;
pub mod linking;
pub mod logging;
pub mod map;
pub mod operator;
pub mod parse;
pub mod profiling;
pub mod tooling;
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

pub use crate::hc_core::{
    compile_core_module,
    compile_core_module_with_debug_info,
};
use crate::parse::ast::SourceFile;

/// Grabs the version number from Cargo.toml at compile time
pub const COMPILER_VERSION_STRING: &str = env!("CARGO_PKG_VERSION");
pub const WASM_MAGIC_NUMBER: [u8; 4] = [0, b'a', b's', b'm'];
pub const CORE_BUNDLE_NAME: &str = "core";
pub const CORE_MODULE_NAME: &str = CORE_BUNDLE_NAME;

#[derive(Debug, Clone)]
pub struct Artifact {
    pub module_name: String,
    pub binary: Vec<u8>,
    pub source_map: Option<String>,
}

impl Artifact {
    pub fn wasm_file_name(&self) -> String {
        format!("{}.wasm", self.module_name)
    }

    pub fn wat_file_name(&self) -> String {
        format!("{}.wat", self.module_name)
    }

    pub fn source_map_file_name(&self) -> String {
        format!("{}.wasm.map", self.module_name)
    }

    pub fn decompile_to_wat(&self) -> Option<String> {
        wasmprinter::print_bytes(&self.binary).ok()
    }

    pub fn save_wasm_to_file(
        &self,
        location: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        let path = location.as_ref().join(self.wasm_file_name());
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
        let path = location.as_ref().join(self.wat_file_name());
        std::fs::write(path, wat)
    }

    pub fn save_source_map_to_file(
        &self,
        location: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        let Some(source_map) = &self.source_map else {
            return Ok(());
        };
        let path = location.as_ref().join(self.source_map_file_name());
        std::fs::write(path, source_map)
    }
}

#[derive(Debug, Clone)]
pub struct Compilation<T> {
    pub output: Option<T>,
    pub logger: Logger,
}

impl<T> Compilation<T> {
    pub fn is_ok(&self) -> bool {
        self.output.is_some() && self.logger.is_ok()
    }

    pub fn into_result(self) -> Result<T, Logger> {
        match self {
            Self {
                output: Some(output),
                logger,
            } if logger.is_ok() => Ok(output),
            Self { logger, .. } => Err(logger),
        }
    }

    pub fn serialized_diagnostics(&self) -> Box<[SerializedDiagnostic]> {
        self.logger.serialize().into_boxed_slice()
    }
}

pub trait ImportResolver {
    fn resolve_import(
        &mut self,
        path: &str,
    ) -> Option<String>;
}

impl<F> ImportResolver for F
where
    F: FnMut(&str) -> Option<String>,
{
    fn resolve_import(
        &mut self,
        path: &str,
    ) -> Option<String> {
        self(path)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoImports;

impl ImportResolver for NoImports {
    fn resolve_import(
        &mut self,
        _path: &str,
    ) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceCompileOptions {
    pub allow_implicit_bundle: bool,
    pub include_core: bool,
    pub debug_info: asm::DebugInfoOptions,
}

impl SourceCompileOptions {
    pub const fn bundle() -> Self {
        Self {
            allow_implicit_bundle: false,
            include_core: false,
            debug_info: asm::DebugInfoOptions::none(),
        }
    }

    pub const fn demo() -> Self {
        Self {
            allow_implicit_bundle: true,
            include_core: false,
            debug_info: asm::DebugInfoOptions::none(),
        }
    }

    pub const fn with_core(
        self,
        include_core: bool,
    ) -> Self {
        Self {
            include_core,
            ..self
        }
    }

    pub const fn with_implicit_bundle(
        self,
        allow_implicit_bundle: bool,
    ) -> Self {
        Self {
            allow_implicit_bundle,
            ..self
        }
    }

    pub const fn with_debug_info(
        self,
        debug_info: asm::DebugInfoOptions,
    ) -> Self {
        Self { debug_info, ..self }
    }
}

impl Default for SourceCompileOptions {
    fn default() -> Self {
        Self::bundle()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    symbols: types::SymbolTable,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_symbols(symbols: types::SymbolTable) -> Self {
        Self { symbols }
    }

    pub fn symbols(&self) -> &types::SymbolTable {
        &self.symbols
    }

    pub fn symbols_mut(&mut self) -> &mut types::SymbolTable {
        &mut self.symbols
    }

    pub fn into_symbols(self) -> types::SymbolTable {
        self.symbols
    }

    pub fn compile_core(
        &mut self,
        debug_info: asm::DebugInfoOptions,
    ) -> Compilation<Artifact> {
        let mut logger = Logger::new();
        let artifact = hc_core::compile_core_module_with_debug_info(
            &mut self.symbols,
            &mut logger,
            debug_info,
        );
        let output = match logger.is_ok() {
            true => Some(artifact),
            false => None,
        };
        Compilation { output, logger }
    }

    pub fn compile_source<R>(
        &mut self,
        source_name: &str,
        source: &str,
        options: SourceCompileOptions,
        resolver: &mut R,
    ) -> Compilation<Box<[Artifact]>>
    where
        R: ImportResolver,
    {
        let mut logger = Logger::new();
        let artifacts = compile_source_with_options(
            source_name,
            source,
            &mut logger,
            &mut self.symbols,
            CompilePipelineOptions {
                allow_implicit_bundle: options.allow_implicit_bundle,
                include_core: options.include_core,
                debug_info: options.debug_info,
                resolve_import: |path| resolver.resolve_import(path.as_str()),
            },
        );
        let output = match logger.is_ok() {
            true => Some(artifacts),
            false => None,
        };
        Compilation { output, logger }
    }

    pub fn link_artifacts(
        &self,
        artifacts: &[Artifact],
        options: linking::LinkOptions,
    ) -> Compilation<Artifact> {
        let mut logger = Logger::new();
        let mut linking_logger = logger.linking_logger();
        let linked = linking::link_artifacts(artifacts, options, &mut linking_logger);
        logger.consume_file(linking_logger);
        let output = match linked {
            Some(artifact) if logger.is_ok() => Some(artifact),
            _ => None,
        };
        Compilation { output, logger }
    }

    pub fn compile_and_link_source<R>(
        &mut self,
        source_name: &str,
        source: &str,
        compile_options: SourceCompileOptions,
        resolver: &mut R,
        link_options: linking::LinkOptions,
    ) -> Compilation<Artifact>
    where
        R: ImportResolver,
    {
        let compiled = self.compile_source(source_name, source, compile_options, resolver);
        let Some(artifacts) = compiled.output else {
            return Compilation {
                output: None,
                logger: compiled.logger,
            };
        };

        let mut logger = compiled.logger;
        let mut linking_logger = logger.linking_logger();
        let linked = linking::link_artifacts(&artifacts, link_options, &mut linking_logger);
        logger.consume_file(linking_logger);
        let output = match linked {
            Some(artifact) if logger.is_ok() => Some(artifact),
            _ => None,
        };
        Compilation { output, logger }
    }

    pub fn compile_demo(
        &mut self,
        source: &str,
    ) -> Compilation<Vec<u8>> {
        let mut resolver = NoImports;
        let linked = self.compile_and_link_source(
            "input.hc",
            source,
            SourceCompileOptions::demo().with_core(true),
            &mut resolver,
            linking::LinkOptions {
                module_name: "app".to_string(),
                emit_source_map: false,
                emit_dwarf: false,
                ..Default::default()
            },
        );
        let mut logger = linked.logger;
        let output = match linked.output {
            Some(artifact) => {
                let binary = artifact.binary;
                validate_generated_wasm(&binary, &mut logger).then_some(binary)
            }
            None => None,
        };
        Compilation { output, logger }
    }
}

#[derive(Debug, Clone)]
struct CompilePipelineOptions<F>
where
    F: FnMut(String) -> Option<String>,
{
    allow_implicit_bundle: bool,
    include_core: bool,
    debug_info: asm::DebugInfoOptions,
    resolve_import: F,
}

pub(crate) fn name_resolution_prelude(
    symbols: &types::SymbolTable
) -> Vec<(ir::Path, ir::NameSpace)> {
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
    allow_implicit_bundle: bool,
) -> String {
    let statements = source_file.statements();
    if !allow_implicit_bundle && !matches!(statements.first(), Some(Statement::Bundle(_))) {
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
    source_catalog: &asm::SourceCatalog,
    debug_info: asm::DebugInfoOptions,
) -> Option<Artifact> {
    let _profile_total = profiling::scope("pipeline.compile_ir_bundle.total");

    let resolved = {
        let _profile = profiling::scope("pipeline.resolve_module");
        types::resolve_with_symbols(symbols, ir_bundle, logger)
    };
    if !logger.is_ok() {
        return None;
    }

    let elaborated = {
        let _profile = profiling::scope("pipeline.elaborate_module");
        ir::elaborate_module(resolved, symbols)
    };
    Some({
        let _profile = profiling::scope("pipeline.asm_compile_module");
        asm::compile_module(elaborated, symbols, source_catalog, debug_info)
    })
}

fn compile_ir_bundle_artifacts(
    ir_bundle: ir::Module<()>,
    symbols: &mut types::SymbolTable,
    logger: &mut FileLogger,
    source_catalog: &asm::SourceCatalog,
    debug_info: asm::DebugInfoOptions,
) -> Box<[Artifact]> {
    compile_ir_bundle(ir_bundle, symbols, logger, source_catalog, debug_info)
        .map(|compiled| vec![compiled].into_boxed_slice())
        .unwrap_or_else(no_artifacts)
}

fn no_artifacts() -> Box<[Artifact]> {
    Vec::new().into_boxed_slice()
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

#[cfg(test)]
fn compile_source_linked_with_core_logger(source: &str) -> Result<Vec<u8>, Logger> {
    let mut compiler = Compiler::new();
    compiler.compile_demo(source).into_result()
}

#[tracing::instrument(skip_all)]
fn compile_source_with_options<F>(
    source_name: &str,
    source: &str,
    logger: &mut Logger,
    symbols: &mut types::SymbolTable,
    mut options: CompilePipelineOptions<F>,
) -> Box<[Artifact]>
where
    F: FnMut(String) -> Option<String>,
{
    let _profile_total = profiling::scope("pipeline.compile_source.total");
    let mut artifacts = Vec::new();
    let debug_info = options.debug_info;

    if options.include_core {
        artifacts.push({
            let _profile = profiling::scope("pipeline.compile_source.core");
            hc_core::compile_core_module_with_debug_info(symbols, logger, options.debug_info)
        });
        if !logger.is_ok() {
            return artifacts.into_boxed_slice();
        }
    }

    let mut source_file_logger = logger.new_file(source_name, source);
    let Some(source_file) = ({
        let _profile = profiling::scope("pipeline.compile_source.parse");
        parse::parse(source, &mut source_file_logger)
    }) else {
        logger.consume_file(source_file_logger);
        return artifacts.into_boxed_slice();
    };

    let bundle_name = bundle_name_for_source_file(
        &source_file,
        &mut source_file_logger,
        options.allow_implicit_bundle,
    );
    if !source_file_logger.is_ok() {
        logger.consume_file(source_file_logger);
        return artifacts.into_boxed_slice();
    }

    let prelude = name_resolution_prelude(symbols);
    let Some(lowered_bundle) = ({
        let _profile = profiling::scope("pipeline.compile_source.lower_with_imports");
        ir::lower_source_file_with_imports(
            bundle_name,
            source_file,
            source_file_logger,
            logger,
            ir::LoweringOptions::with_prelude(&prelude),
            &mut options.resolve_import,
        )
    }) else {
        return artifacts.into_boxed_slice();
    };

    if !logger.is_ok() {
        return artifacts.into_boxed_slice();
    }

    let mut typing_file_logger = logger.new_file(source_name, source);
    let source_catalog = logger.source_files();
    let compiled_artifacts = {
        let _profile = profiling::scope("pipeline.compile_source.type_elab_codegen");
        compile_ir_bundle_artifacts(
            lowered_bundle.module,
            symbols,
            &mut typing_file_logger,
            &source_catalog,
            debug_info,
        )
    };
    logger.consume_file(typing_file_logger);
    if compiled_artifacts.is_empty() {
        return artifacts.into_boxed_slice();
    }

    artifacts.extend(compiled_artifacts.into_vec());
    artifacts.into_boxed_slice()
}

#[cfg(test)]
#[tracing::instrument(skip_all)]
fn compile_source(
    source: &str,
    logger: &mut FileLogger,
    symbols: &mut types::SymbolTable,
) -> Box<[Artifact]> {
    let Some(source_file) = parse::parse(source, logger) else {
        return no_artifacts();
    };

    let bundle_name = bundle_name_for_source_file(&source_file, logger, true);
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
            }
            parse::ast::Statement::Import(_) => {}
            other => statements.push(other),
        }
    }

    if !logger.is_ok() {
        return no_artifacts();
    }

    let prelude = name_resolution_prelude(symbols);
    let Some(lowered_bundle) = ir::lower_statements(
        bundle_name,
        &statements,
        logger,
        ir::LoweringOptions::with_prelude(&prelude),
    ) else {
        return no_artifacts();
    };

    compile_ir_bundle_artifacts(
        lowered_bundle.module,
        symbols,
        logger,
        &Vec::new(),
        asm::DebugInfoOptions::default(),
    )
}

#[tracing::instrument(skip_all, fields(module = %art.module_name))]
pub fn validate_artifact(
    art: Artifact,
    logger: &mut Logger,
) -> Artifact {
    let _profile_total = profiling::scope("pipeline.validate_artifact.total");

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
    let offset_map = {
        let _profile = profiling::scope("pipeline.validate_artifact.offsets_and_lines");
        wasmprinter::Config::new()
            .offsets_and_lines(&art.binary, &mut wat_with_offsets)
            .map(|iter| {
                iter.enumerate()
                    .map(|(idx, (offset, _text))| (idx + 1, offset))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
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
    if let Err(error) = {
        let _profile = profiling::scope("pipeline.validate_artifact.wasmparser_validate");
        wasmparser::validate(&art.binary)
    } {
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

#[cfg(test)]
fn compile_source_linked_with_core(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_linked_with_core_logger(source).map_err(|logger| logger.into_diagnostics())
}
