use std::collections::HashSet;
use std::path::{
    Path,
    PathBuf,
};

use halcyon_lib::parse::ast::{
    self,
    AstNode,
    HasName,
};
use halcyon_lib::types::SymbolTable;
use halcyon_lib::{
    Artifact,
    Logger,
    Span,
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
            Self::Build(paths) => {
                let root_path = single_root_path(paths, "build")?;
                let artifacts = compile_bundle(root_path)?;
                std::fs::create_dir_all("target")?;
                for artifact in &artifacts {
                    artifact.save_wasm_to_file("target")?;
                }
                Ok(())
            }
            Self::Run(paths) => {
                let root_path = single_root_path(paths, "run")?;
                let artifacts = compile_bundle(root_path)?;
                link_and_run(&artifacts)
            }
            Self::Doc(paths) => {
                let root_path = single_root_path(paths, "doc")?;
                generate_docs(root_path)
            }
            Self::Help => {
                print_usage();
                Ok(())
            }
        }
    }
}

fn print_usage() {
    eprintln!("Usage: halcyon <command> <bundle-root>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  build <bundle-root>  Compile bundle to .wasm files in target/");
    eprintln!("  run <bundle-root>    Compile and run the program in wasmtime");
    eprintln!("  doc <bundle-root>    Generate markdown documentation in docs/");
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

#[derive(Debug, Clone)]
struct BundleSourceFragment {
    file_path: PathBuf,
    source: String,
    statements: Vec<ast::Statement>,
}

fn single_root_path<'a>(
    paths: &'a [String],
    command: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    if paths.len() != 1 {
        return Err(format!("`{command}` expects exactly one bundle root path").into());
    }
    Ok(paths[0].as_str())
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

fn name_resolution_prelude(symbols: &SymbolTable) -> Vec<(ir::Path, ir::NameSpace)> {
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

fn report_cycle_diagnostic(
    file_logger: &mut halcyon_lib::FileLogger,
    span: Span,
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

fn collect_bundle_fragments_with_imports(
    file_path: &Path,
    is_root: bool,
    state: &mut ImportTraversalState,
    logger: &mut Logger,
    bundle_name: &mut Option<String>,
    fragments: &mut Vec<BundleSourceFragment>,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_path = normalize_path(file_path)?;

    if state.visited.contains(&canonical_path) {
        return Ok(());
    }

    state.visiting.push(canonical_path.clone());

    let source = std::fs::read_to_string(&canonical_path)
        .map_err(|error| format!("{}: {error}", canonical_path.to_string_lossy()))?;
    let mut file_logger = logger.new_file(canonical_path.to_string_lossy(), source.clone());

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(source_file) = parse::parse(&source, &mut file_logger) {
            let items = source_file.items();
            if is_root && !matches!(items.first(), Some(ast::TopLevelItem::Bundle(_))) {
                let span = items.first().map_or_else(
                    || {
                        source
                            .char_indices()
                            .find(|(_, ch)| !ch.is_whitespace())
                            .map(|(start, _)| Span::new(start, 1))
                            .unwrap_or(Span::Generated)
                    },
                    |item| {
                        match item.span() {
                            Span::Source { start, .. } => Span::new(start, 1),
                            Span::Generated => Span::Generated,
                        }
                    },
                );
                file_logger
                    .error("Missing bundle declaration")
                    .primary("Root file must start with `bundle <name>`.", span)
                    .done();
            }

            let mut local_bundle_name: Option<String> = None;
            let mut local_bundle_span = Span::Generated;
            let mut statements = Vec::new();
            let mut import_literals = Vec::new();

            for item in items {
                match item {
                    ast::TopLevelItem::Bundle(bundle_declaration) => {
                        if local_bundle_name.is_some() {
                            file_logger
                                .error("Duplicate bundle declaration")
                                .primary(
                                    "A source file may only declare one bundle.",
                                    bundle_declaration.span(),
                                )
                                .done();
                            continue;
                        }
                        local_bundle_name = bundle_declaration.name_text();
                        local_bundle_span = bundle_declaration.span();
                    }
                    ast::TopLevelItem::Import(import_statement) => {
                        import_literals.extend(import_statement.path_literals());
                    }
                    ast::TopLevelItem::Statement(statement) => statements.push(statement),
                }
            }

            if is_root {
                if let Some(local_bundle_name) = local_bundle_name {
                    *bundle_name = Some(local_bundle_name);
                }
            } else if let Some(local_bundle_name) = local_bundle_name {
                let active_bundle_name = bundle_name
                    .as_deref()
                    .unwrap_or("<unknown-bundle>")
                    .to_string();
                file_logger
                    .error("Unexpected bundle declaration")
                    .primary(
                        format!(
                            "Imported files are part of bundle `{active_bundle_name}` and may not declare `bundle {local_bundle_name}`."
                        ),
                        local_bundle_span,
                    )
                    .done();
            }

            for literal in import_literals {
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

                collect_bundle_fragments_with_imports(
                    &canonical_import,
                    false,
                    state,
                    logger,
                    bundle_name,
                    fragments,
                )?;
            }

            fragments.push(BundleSourceFragment {
                file_path: canonical_path.clone(),
                source: source.clone(),
                statements,
            });
        }

        Ok(())
    })();

    logger.consume_file(file_logger);
    let _ = state.visiting.pop();
    state.visited.insert(canonical_path);
    result
}

fn lower_and_resolve_fragments(
    fragments: &[BundleSourceFragment],
    bundle_name: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<types::ResolvedModule> {
    let mut resolved_fragments = Vec::new();
    for fragment in fragments {
        let mut file_logger = logger.new_file(
            fragment.file_path.to_string_lossy(),
            fragment.source.clone(),
        );
        let prelude = name_resolution_prelude(symbols);
        if let Some(ir_module) = ir::bundle_statements_with_prelude(
            bundle_name.to_string(),
            &fragment.statements,
            &mut file_logger,
            &prelude,
        ) {
            let resolved = types::resolve_module_with_symbols_and_schemes(
                symbols,
                ir_module,
                &mut file_logger,
            );
            resolved_fragments.push(resolved);
        } else if file_logger.is_ok() {
            file_logger
                .error("IR lowering failed")
                .primary(
                    "Bundle lowering failed without a specific diagnostic.",
                    Span::Generated,
                )
                .done();
        }
        logger.consume_file(file_logger);
    }
    resolved_fragments
}

fn combine_resolved_fragments(
    bundle_name: &str,
    resolved_fragments: Vec<types::ResolvedModule>,
) -> types::ResolvedModule {
    let mut statements = Vec::new();
    let mut resolved_fragments = resolved_fragments.into_iter();
    let mut combined_schemes = if let Some(first) = resolved_fragments.next() {
        statements.extend(first.module.statements.into_vec());
        first.schemes
    } else {
        return types::ResolvedModule {
            module: ir::Module {
                name: bundle_name.to_string(),
                statements: statements.into_boxed_slice(),
            },
            schemes: Default::default(),
        };
    };

    for resolved in resolved_fragments {
        statements.extend(resolved.module.statements.into_vec());
        combined_schemes.extend(resolved.schemes);
    }

    types::ResolvedModule {
        module: ir::Module {
            name: bundle_name.to_string(),
            statements: statements.into_boxed_slice(),
        },
        schemes: combined_schemes,
    }
}

fn compile_bundle(root_path: &str) -> Result<Vec<Artifact>, Box<dyn std::error::Error>> {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let mut state = ImportTraversalState::default();
    let mut fragments = Vec::new();
    let mut bundle_name = None;

    let core = validate_artifact(compile_core_module(&mut symbols, &mut logger), &mut logger);

    collect_bundle_fragments_with_imports(
        Path::new(root_path),
        true,
        &mut state,
        &mut logger,
        &mut bundle_name,
        &mut fragments,
    )?;

    if !logger.is_ok() {
        logger.print_logs();
        return Err("Compilation failed".into());
    }

    let bundle_name = bundle_name.unwrap_or_else(|| "_".to_string());
    let resolved_fragments =
        lower_and_resolve_fragments(&fragments, &bundle_name, &mut logger, &mut symbols);

    if !logger.is_ok() {
        logger.print_logs();
        return Err("Compilation failed".into());
    }

    let merged_resolved = combine_resolved_fragments(&bundle_name, resolved_fragments);
    let elaborated = ir::elaborate_module(merged_resolved, &symbols);
    let bundle_artifact = validate_artifact(
        halcyon_lib::asm::compile_module(elaborated, &symbols),
        &mut logger,
    );

    logger.print_logs();
    if !logger.is_ok() {
        return Err("Compilation failed".into());
    }

    Ok(vec![core, bundle_artifact])
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

fn generate_docs(root_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let mut state = ImportTraversalState::default();
    let mut fragments = Vec::new();
    let mut bundle_name = None;

    compile_core_module(&mut symbols, &mut logger);

    collect_bundle_fragments_with_imports(
        Path::new(root_path),
        true,
        &mut state,
        &mut logger,
        &mut bundle_name,
        &mut fragments,
    )?;

    if !logger.is_ok() {
        logger.print_logs();
        return Err("Compilation failed".into());
    }

    let bundle_name = bundle_name.unwrap_or_else(|| "_".to_string());
    let resolved_fragments =
        lower_and_resolve_fragments(&fragments, &bundle_name, &mut logger, &mut symbols);

    logger.print_logs();
    if !logger.is_ok() {
        return Err("Compilation failed".into());
    }

    let merged_resolved = combine_resolved_fragments(&bundle_name, resolved_fragments);
    let docs = documentation::generate(&merged_resolved, &symbols);

    std::fs::create_dir_all("docs")?;
    let markdown = documentation::render_markdown(&bundle_name, &docs);
    let path = std::path::Path::new("docs").join(format!("{bundle_name}.md"));
    std::fs::write(path, markdown)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_bundle_tests_execute_without_failures() {
        let root_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests.hc");
        let root_path = root_path.to_string_lossy().to_string();
        let artifacts =
            compile_bundle(&root_path).expect("core test bundle should compile successfully");
        link_and_run(&artifacts).expect("core test bundle should execute without failures");
    }
}

#[allow(clippy::print_stdout)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = Command::parse(&args).execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
