use std::collections::HashSet;
use std::path::{
    Path,
    PathBuf,
};

use halcyon_lib::asm::custom_section::TypeSignatureSection;
use halcyon_lib::asm::module_section::LoweredModuleSection;
use halcyon_lib::parse::ast::{
    self,
    AstNode,
    HasName,
};
use halcyon_lib::types::SymbolTable;
use halcyon_lib::{
    compile_core_module,
    compile_source_with_options,
    documentation,
    ir,
    linking,
    parse,
    types,
    validate_artifact,
    Artifact,
    CompileOptions,
    Logger,
    Span,
    WithContext,
    CORE_BUNDLE_NAME,
    WASM_MAGIC_NUMBER,
};
use tracing_subscriber::EnvFilter;
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
                let linked = compile_and_link_inputs(paths, "app")?;
                std::fs::create_dir_all("target")?;
                linked.save_wasm_to_file("target")?;
                linked.save_source_map_to_file("target")?;
                Ok(())
            }
            Self::Run(paths) => {
                let linked = compile_and_link_inputs(paths, "app")?;
                link_and_run(std::slice::from_ref(&linked))
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
    eprintln!("Usage: halcyon <command> <input>...");
    eprintln!();
    eprintln!("Commands:");
    eprintln!(
        "  build <input>...    Compile source/binary inputs and emit one linked .wasm in target/"
    );
    eprintln!("  run <input>...      Compile source/binary inputs and run the linked program");
    eprintln!(
        "                      Inputs are linked and initialized in the exact order provided."
    );
    eprintln!("  doc <bundle-root>    Generate JSON documentation in docs/");
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

fn ensure_inputs<'a>(
    paths: &'a [String],
    command: &str,
) -> Result<&'a [String], Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err(format!("`{command}` expects at least one input path").into());
    }
    Ok(paths)
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

#[tracing::instrument(skip_all, fields(file = %file_path.display()))]
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
            if is_root && !matches!(items.first(), Some(ast::Statement::Bundle(_))) {
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
                    ast::Statement::Bundle(bundle_declaration) => {
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
                    ast::Statement::Import(import_statement) => {
                        import_literals.extend(import_statement.path_literals());
                    }
                    statement => statements.push(statement),
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

#[tracing::instrument(skip_all, fields(bundle = %bundle_name, fragment_count = fragments.len()))]
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

#[tracing::instrument(skip_all, fields(root = %root_path))]
fn compile_source_bundle(
    root_path: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Result<Artifact, Box<dyn std::error::Error>> {
    let root_path = normalize_path(Path::new(root_path))?;
    let source = std::fs::read_to_string(&root_path)
        .map_err(|error| format!("{}: {error}", root_path.display()))?;
    let source_name = root_path.to_string_lossy().to_string();

    let mut artifacts = compile_source_with_options(
        &source_name,
        &source,
        logger,
        symbols,
        CompileOptions {
            demo_mode: false,
            use_core: false,
            resolve_import: |path| std::fs::read_to_string(Path::new(&path)).ok(),
        },
    )
    .into_vec();

    if !logger.is_ok() {
        logger.print_logs();
        return Err("Compilation failed".into());
    }

    let Some(bundle_artifact) = artifacts.pop() else {
        return Err("Compilation produced no artifacts".into());
    };

    Ok(bundle_artifact)
}

fn is_wasm_binary(binary: &[u8]) -> bool {
    binary.starts_with(&WASM_MAGIC_NUMBER)
}

fn type_definition_compatible(
    left: &halcyon_lib::types::TypeDefinition,
    right: &halcyon_lib::types::TypeDefinition,
) -> bool {
    left.parameters == right.parameters
        && left.parameter_kinds == right.parameter_kinds
        && left.body == right.body
        && left.kind == right.kind
}

fn register_signature_symbols(
    symbols: &mut SymbolTable,
    signature: &TypeSignatureSection,
) -> Result<(), Box<dyn std::error::Error>> {
    for (path, definition) in signature.defined_types.iter() {
        if let Some(existing_definition) = symbols.type_definitions().get(path) {
            if !type_definition_compatible(existing_definition, definition) {
                return Err(
                    format!("Type `{path}` in binary input conflicts with existing type").into(),
                );
            }
        } else {
            symbols.insert_type(path.clone(), definition.clone());
            symbols.insert_constructor(path.clone());
        }
    }

    for (path, scheme) in signature.defined_terms.iter() {
        if let Some(existing_scheme) = symbols.terms().get(path) {
            if existing_scheme != scheme {
                return Err(
                    format!("Term `{path}` in binary input conflicts with existing term").into(),
                );
            }
        } else {
            symbols.insert_term(path.clone(), scheme.clone());
        }
    }

    Ok(())
}

fn load_binary_artifact(
    input_path: &Path,
    binary: Vec<u8>,
    symbols: &mut SymbolTable,
) -> Result<Artifact, Box<dyn std::error::Error>> {
    let mut saw_module_section = false;
    let mut saw_signature_section = false;
    let mut lowered_module = None;
    let mut signature = None;

    for payload in wasmparser::Parser::new(0).parse_all(&binary) {
        let payload = payload.map_err(|error| {
            format!(
                "{}: failed to parse wasm payload: {}",
                input_path.display(),
                error.message()
            )
        })?;
        if let wasmparser::Payload::CustomSection(reader) = payload {
            if reader.name() == LoweredModuleSection::NAME {
                saw_module_section = true;
                lowered_module = LoweredModuleSection::decode_data_slice(reader.data());
            } else if reader.name() == TypeSignatureSection::NAME {
                saw_signature_section = true;
                signature = TypeSignatureSection::decode_data_slice(reader.data());
            }
        }
    }

    if !saw_module_section {
        return Err(format!(
            "{}: missing linker metadata `{}`",
            input_path.display(),
            LoweredModuleSection::NAME
        )
        .into());
    }
    if !saw_signature_section {
        return Err(format!(
            "{}: missing linker metadata `{}`",
            input_path.display(),
            TypeSignatureSection::NAME
        )
        .into());
    }

    let lowered_module = lowered_module.ok_or_else(|| {
        format!(
            "{}: invalid linker metadata `{}`",
            input_path.display(),
            LoweredModuleSection::NAME
        )
    })?;
    let signature = signature.ok_or_else(|| {
        format!(
            "{}: invalid linker metadata `{}`",
            input_path.display(),
            TypeSignatureSection::NAME
        )
    })?;

    register_signature_symbols(symbols, &signature)?;

    Ok(Artifact {
        module_name: lowered_module.name,
        ir_module: None,
        binary,
        source_map: None,
    })
}

#[tracing::instrument(skip_all, fields(input_count = input_paths.len()))]
fn collect_input_artifacts(
    input_paths: &[String]
) -> Result<Vec<Artifact>, Box<dyn std::error::Error>> {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let core = validate_artifact(compile_core_module(&mut symbols, &mut logger), &mut logger);

    if !logger.is_ok() {
        logger.print_logs();
        return Err("Compilation failed".into());
    }

    let mut artifacts = Vec::new();
    let mut includes_core = false;

    for input_path in input_paths {
        let input_path = Path::new(input_path);
        let normalized_path = normalize_path(input_path)?;
        let data = std::fs::read(&normalized_path)
            .map_err(|error| format!("{}: {error}", normalized_path.display()))?;

        let artifact = if is_wasm_binary(&data) {
            validate_artifact(
                load_binary_artifact(&normalized_path, data, &mut symbols)?,
                &mut logger,
            )
        } else {
            let source_path = normalized_path.to_string_lossy().to_string();
            compile_source_bundle(&source_path, &mut logger, &mut symbols)?
        };

        if artifact.module_name == CORE_BUNDLE_NAME {
            includes_core = true;
        }
        artifacts.push(artifact);
    }

    logger.print_logs();
    if !logger.is_ok() {
        return Err("Compilation failed".into());
    }

    if !includes_core {
        artifacts.insert(0, core);
    }

    Ok(artifacts)
}

fn compile_and_link_inputs(
    input_paths: &[String],
    linked_module_name: &str,
) -> Result<Artifact, Box<dyn std::error::Error>> {
    let input_paths = ensure_inputs(input_paths, "build/run")?;
    let artifacts = collect_input_artifacts(input_paths)?;
    let mut logger = Logger::new();
    let mut link_logger = logger.linking_logger();
    let Some(linked) = linking::link_artifacts(
        &artifacts,
        linking::LinkOptions {
            module_name: linked_module_name.to_string(),
            ..Default::default()
        },
        &mut link_logger,
    ) else {
        logger.consume_file(link_logger);
        logger.print_logs();
        return Err("Compilation failed".into());
    };
    logger.consume_file(link_logger);

    Ok(linked)
}

#[tracing::instrument(skip_all, fields(artifact_count = artifacts.len()))]
fn link_and_run(artifacts: &[Artifact]) -> Result<(), Box<dyn std::error::Error>> {
    if artifacts.is_empty() {
        return Err("No artifacts to run".into());
    }

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

    for artifact in artifacts.iter().take(artifacts.len().saturating_sub(1)) {
        let module = Module::new(&engine, &artifact.binary)?;
        linker.module(&mut store, &artifact.module_name, &module)?;
    }

    let entry_artifact = &artifacts[artifacts.len() - 1];
    let entry_module = Module::new(&engine, &entry_artifact.binary)?;
    let entry_instance = linker.instantiate(&mut store, &entry_module)?;
    let start = entry_instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    start.call(&mut store, ())?;

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
    let json = documentation::render_json(&bundle_name, &docs)?;
    let path = std::path::Path::new("docs").join(format!("{bundle_name}.json"));
    std::fs::write(path, json)?;

    Ok(())
}

#[allow(clippy::print_stdout)]
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("HALCYON_LOG"))
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = Command::parse(&args).execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_file(
        file_name: &str,
        contents: &[u8],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("halcyon-cli-tests-{unique}"));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(file_name);
        std::fs::write(&path, contents)?;
        Ok(path)
    }

    #[test]
    fn core_bundle_tests_execute_without_failures() {
        let root_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests.hc");
        let root_path = root_path.to_string_lossy().to_string();
        let linked = compile_and_link_inputs(&[root_path], "app")
            .expect("core test bundle should compile successfully");
        link_and_run(&[linked]).expect("core test bundle should execute without failures");
    }

    #[test]
    fn accepts_binary_and_source_inputs_in_sequence() {
        let mut logger = Logger::new();
        let mut symbols = SymbolTable::new();
        let _ = validate_artifact(compile_core_module(&mut symbols, &mut logger), &mut logger);

        let alpha_source = b"bundle alpha\nlet value : core::Integer = core::default\n";
        let alpha_source_path = write_temp_file("alpha.hc", alpha_source)
            .expect("alpha source should be written to temp dir");
        let alpha_artifact = compile_source_bundle(
            &alpha_source_path.to_string_lossy(),
            &mut logger,
            &mut symbols,
        )
        .expect("alpha source should compile");
        let alpha_binary_path = write_temp_file("alpha.wasm", &alpha_artifact.binary)
            .expect("alpha wasm should be written to temp dir");

        let beta_source = b"bundle beta\nlet result : core::Integer = alpha::value\n";
        let beta_source_path = write_temp_file("beta.hc", beta_source)
            .expect("beta source should be written to temp dir");

        let linked = compile_and_link_inputs(
            &[
                alpha_binary_path.to_string_lossy().to_string(),
                beta_source_path.to_string_lossy().to_string(),
            ],
            "app",
        )
        .expect("mixed binary+source inputs should compile and link");

        let _ = validate_artifact(linked, &mut logger);
        assert!(logger.is_ok(), "linked artifact should validate");
    }
}
