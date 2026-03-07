use std::collections::HashSet;
use std::path::{
    Path,
    PathBuf,
};

use halcyon_lib::parse::ast;
use halcyon_lib::types::SymbolTable;
use halcyon_lib::{
    Artifact,
    Logger,
    WithContext,
    compile_core_module,
    documentation,
    ir,
    parse,
    types,
    validate_artifact,
};
use wasmtime::{
    Config,
    Engine,
    Linker,
    Module,
    Store,
};
use wasmtime_wasi::p2::WasiCtxBuilder;
use wasmtime_wasi::preview1;

enum Command<'a> {
    Build(&'a [String]),
    Doc(&'a [String]),
    Run(&'a [String]),
    Help,
}

impl<'a> Command<'a> {
    fn parse(args: &'a [String]) -> Self {
        match args.first().map(String::as_str) {
            Some("build") => Self::Build(&args[1..]),
            Some("run") => Self::Run(&args[1..]),
            Some("doc") => Self::Doc(&args[1..]),
            _ => Self::Help,
        }
    }

    fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Build(files) => {
                let artifacts = compile_files(files)?;
                std::fs::create_dir_all("target")?;
                for artifact in &artifacts {
                    artifact.save_wasm_to_file("target")?;
                }
                Ok(())
            }
            Self::Run(files) => {
                let artifacts = compile_files(files)?;
                link_and_run(&artifacts)
            }
            Self::Doc(files) => generate_docs(files),
            Self::Help => {
                print_usage();
                Ok(())
            }
        }
    }
}

fn print_usage() {
    eprintln!("Usage: halcyon <command> [files...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  build <files...>  Compile modules to .wasm files in target/");
    eprintln!("  run <files...>    Compile and run the program in wasmtime");
    eprintln!("  doc <files...>    Generate markdown documentation in docs/");
}

#[derive(Default)]
struct ImportTraversalState {
    visited: HashSet<PathBuf>,
    visiting: Vec<PathBuf>,
}

impl ImportTraversalState {
    fn cycle_chain(
        &self,
        candidate: &Path,
    ) -> Option<Vec<PathBuf>> {
        let start = self.visiting.iter().position(|path| path == candidate)?;
        let mut chain = self.visiting[start..].to_vec();
        chain.push(candidate.to_path_buf());
        Some(chain)
    }
}

fn normalize_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    std::fs::canonicalize(path)
        .map_err(|error| format!("{}: {error}", path.to_str().unwrap_or("<non-utf8-path>")).into())
}

fn decode_import_path_literal(literal: &str) -> Option<String> {
    if literal.len() < 2 || !literal.starts_with('"') || !literal.ends_with('"') {
        return None;
    }

    let mut result = String::new();
    let mut chars = literal[1..literal.len() - 1].chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        let escaped = chars.next()?;
        match escaped {
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            'b' => result.push('\x08'),
            '\\' => result.push('\\'),
            '0' => result.push('\0'),
            '"' => result.push('"'),
            '\'' => result.push('\''),
            'x' => {
                let b1 = decode_hex_nibble(chars.next()?)?;
                let b2 = decode_hex_nibble(chars.next()?)?;
                result.push(char::from_u32((b1 << 4) | b2)?);
            }
            'w' => {
                let b1 = decode_hex_nibble(chars.next()?)?;
                let b2 = decode_hex_nibble(chars.next()?)?;
                let b3 = decode_hex_nibble(chars.next()?)?;
                let b4 = decode_hex_nibble(chars.next()?)?;
                result.push(char::from_u32((b1 << 12) | (b2 << 8) | (b3 << 4) | b4)?);
            }
            _ => return None,
        }
    }

    Some(result)
}

fn decode_hex_nibble(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some((ch as u32) - ('0' as u32)),
        'a'..='f' => Some((ch as u32) - ('a' as u32) + 10),
        'A'..='F' => Some((ch as u32) - ('A' as u32) + 10),
        _ => None,
    }
}

fn report_cycle_diagnostic(
    file_logger: &mut halcyon_lib::FileLogger,
    span: halcyon_lib::Span,
    cycle_chain: &[PathBuf],
) {
    let chain = cycle_chain
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" -> ");
    file_logger
        .error("Cyclic import detected")
        .primary(format!("Import cycle: {chain}"), span)
        .done();
}

fn compile_file_with_imports(
    file_path: &Path,
    state: &mut ImportTraversalState,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_path = normalize_path(file_path)?;

    if state.visited.contains(&canonical_path) {
        return Ok(());
    }

    state.visiting.push(canonical_path.clone());

    let source = std::fs::read_to_string(&canonical_path)
        .map_err(|error| format!("{}: {error}", canonical_path.to_string_lossy()))?;
    let mut file_logger = logger.new_file(canonical_path.to_string_lossy(), source.clone());

    if let Some(source_file) = parse::parse(&source, &mut file_logger) {
        for item in source_file.items() {
            match item {
                ast::TopLevelItem::Import(import_statement) => {
                    for literal in import_statement.path_literals() {
                        let Some(import_path) = decode_import_path_literal(&literal.inner) else {
                            file_logger
                                .error("Invalid import path")
                                .primary("Expected a valid string literal path", literal.span)
                                .done();
                            continue;
                        };

                        let resolved_path = canonical_path
                            .parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join(import_path);
                        let canonical_import = match normalize_path(&resolved_path) {
                            Ok(path) => path,
                            Err(error) => {
                                file_logger
                                    .error("Unable to resolve import")
                                    .primary(error.to_string(), literal.span)
                                    .done();
                                continue;
                            }
                        };

                        if let Some(cycle_chain) = state.cycle_chain(&canonical_import) {
                            report_cycle_diagnostic(&mut file_logger, literal.span, &cycle_chain);
                            continue;
                        }

                        compile_file_with_imports(
                            &canonical_import,
                            state,
                            logger,
                            symbols,
                            artifacts,
                        )?;
                    }
                }
                ast::TopLevelItem::Module(module) => {
                    if let Some(ir_module) = ir::module(module, &mut file_logger) {
                        let resolved = types::resolve_module_with_symbols_and_schemes(
                            symbols,
                            ir_module,
                            &mut file_logger,
                        );
                        let elaborated = ir::elaborate_module(resolved, symbols);
                        let artifact = halcyon_lib::asm::compile_module(elaborated, symbols);
                        artifacts.push(validate_artifact(artifact, logger));
                    }
                }
            }
        }
    }

    logger.consume_file(file_logger);
    let _ = state.visiting.pop();
    state.visited.insert(canonical_path);
    Ok(())
}

fn compile_files(file_paths: &[String]) -> Result<Vec<Artifact>, Box<dyn std::error::Error>> {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let mut state = ImportTraversalState::default();

    let core = validate_artifact(compile_core_module(&mut symbols, &mut logger), &mut logger);
    let mut artifacts = vec![core];

    for file_path in file_paths {
        compile_file_with_imports(
            Path::new(file_path),
            &mut state,
            &mut logger,
            &mut symbols,
            &mut artifacts,
        )?;
    }

    logger.print_logs();
    if !logger.is_ok() {
        return Err("Compilation failed".into());
    }

    Ok(artifacts)
}

fn link_and_run(artifacts: &[Artifact]) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.wasm_gc(true);
    config.wasm_reference_types(true);
    config.wasm_function_references(true);

    let engine = Engine::new(&config)?;
    let mut linker: Linker<preview1::WasiP1Ctx> = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |ctx| ctx)?;

    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdio();
    builder.inherit_args();
    let wasi = builder.build_p1();
    let mut store = Store::new(&engine, wasi);

    for artifact in artifacts {
        let module = Module::new(&engine, &artifact.binary)?;
        linker.module(&mut store, &artifact.module_name, &module)?;
    }
    Ok(())
}

fn generate_docs(file_paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let mut state = ImportTraversalState::default();

    compile_core_module(&mut symbols, &mut logger);

    let mut all_docs: Vec<(String, Vec<documentation::Documentation>)> = Vec::new();

    for file_path in file_paths {
        generate_docs_for_file_with_imports(
            Path::new(file_path),
            &mut state,
            &mut logger,
            &mut symbols,
            &mut all_docs,
        )?;
    }

    logger.print_logs();
    if !logger.is_ok() {
        return Err("Compilation failed".into());
    }

    std::fs::create_dir_all("docs")?;
    for (module_name, docs) in &all_docs {
        let markdown = documentation::render_markdown(module_name, docs);
        let path = std::path::Path::new("docs").join(format!("{module_name}.md"));
        std::fs::write(path, markdown)?;
    }

    Ok(())
}

fn generate_docs_for_file_with_imports(
    file_path: &Path,
    state: &mut ImportTraversalState,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
    all_docs: &mut Vec<(String, Vec<documentation::Documentation>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_path = normalize_path(file_path)?;

    if state.visited.contains(&canonical_path) {
        return Ok(());
    }

    state.visiting.push(canonical_path.clone());

    let source = std::fs::read_to_string(&canonical_path)
        .map_err(|error| format!("{}: {error}", canonical_path.to_string_lossy()))?;
    let mut file_logger = logger.new_file(canonical_path.to_string_lossy(), source.clone());

    if let Some(source_file) = parse::parse(&source, &mut file_logger) {
        for item in source_file.items() {
            match item {
                ast::TopLevelItem::Import(import_statement) => {
                    for literal in import_statement.path_literals() {
                        let Some(import_path) = decode_import_path_literal(&literal.inner) else {
                            file_logger
                                .error("Invalid import path")
                                .primary("Expected a valid string literal path", literal.span)
                                .done();
                            continue;
                        };

                        let resolved_path = canonical_path
                            .parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join(import_path);
                        let canonical_import = match normalize_path(&resolved_path) {
                            Ok(path) => path,
                            Err(error) => {
                                file_logger
                                    .error("Unable to resolve import")
                                    .primary(error.to_string(), literal.span)
                                    .done();
                                continue;
                            }
                        };

                        if let Some(cycle_chain) = state.cycle_chain(&canonical_import) {
                            report_cycle_diagnostic(&mut file_logger, literal.span, &cycle_chain);
                            continue;
                        }

                        generate_docs_for_file_with_imports(
                            &canonical_import,
                            state,
                            logger,
                            symbols,
                            all_docs,
                        )?;
                    }
                }
                ast::TopLevelItem::Module(module) => {
                    if let Some(ir_module) = ir::module(module, &mut file_logger) {
                        let resolved = types::resolve_module_with_symbols_and_schemes(
                            symbols,
                            ir_module,
                            &mut file_logger,
                        );
                        let docs = documentation::generate(&resolved, symbols);
                        all_docs.push((resolved.module.name.clone(), docs));
                    }
                }
            }
        }
    }

    logger.consume_file(file_logger);
    let _ = state.visiting.pop();
    state.visited.insert(canonical_path);
    Ok(())
}

#[allow(clippy::print_stdout)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = Command::parse(&args).execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
