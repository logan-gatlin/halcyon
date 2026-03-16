/*!
    The `core` bundle contains symbols that are required by the compiler.
    These include standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod symbols;
mod types;

pub use symbols::CoreSymbol;
pub use types::CoreType;

use std::collections::HashSet;
use std::path::{
    Component,
    Path,
    PathBuf,
};

use enum_iterator::all;

use crate::logging::WithContext;
use crate::parse::ast::{
    self,
    AstNode,
    HasName,
};
use crate::types::SymbolTable;
use crate::{
    Artifact,
    Span,
    asm,
};

pub const CORE_MODULE_NAME: &str = "core";

const CORE_SOURCE_ROOT: &str = "core";
const CORE_ROOT_FILE_NAME: &str = "bundle.hc";

#[derive(Debug, Default)]
struct CoreSourceTraversalState {
    visited: HashSet<PathBuf>,
    visiting: Vec<PathBuf>,
}

impl CoreSourceTraversalState {
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
struct CoreSourceFragment {
    source_path: PathBuf,
    source: String,
    statements: Vec<ast::Statement>,
}

#[tracing::instrument(skip_all)]
pub fn compile_core_module(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Artifact {
    register_core_primitive_types(symbols);

    let Some(resolved_bundle) = resolve_core_source_bundle(symbols, logger) else {
        return Artifact {
            module_name: CORE_MODULE_NAME.to_string(),
            ir_module: None,
            binary: Vec::new(),
        };
    };

    if !logger.is_ok() {
        return Artifact {
            module_name: CORE_MODULE_NAME.to_string(),
            ir_module: None,
            binary: Vec::new(),
        };
    }

    let elaborated = crate::ir::elaborate_module(resolved_bundle, symbols);
    asm::compile_module(elaborated, symbols)
}

fn register_core_primitive_types(symbols: &mut SymbolTable) {
    all::<CoreType>().for_each(|symbol| {
        symbols.insert(symbol);
    });
}

fn core_source_root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(CORE_SOURCE_ROOT)
}

fn display_source_path(relative_path: &Path) -> String {
    Path::new(CORE_SOURCE_ROOT)
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn name_resolution_prelude(symbols: &SymbolTable) -> Vec<(crate::ir::Path, crate::ir::NameSpace)> {
    let mut prelude = Vec::new();
    prelude.extend(
        symbols
            .terms()
            .keys()
            .cloned()
            .map(|path| (path, crate::ir::NameSpace::Term)),
    );
    prelude.extend(
        symbols
            .constructors()
            .iter()
            .cloned()
            .map(|path| (path, crate::ir::NameSpace::Constructor)),
    );
    prelude.extend(
        symbols
            .type_definitions()
            .keys()
            .cloned()
            .map(|path| (path, crate::ir::NameSpace::Type)),
    );
    prelude.extend(
        symbols
            .trait_defs()
            .keys()
            .cloned()
            .map(|path| (path, crate::ir::NameSpace::Trait)),
    );
    prelude.extend(
        symbols
            .trait_aliases()
            .keys()
            .cloned()
            .map(|path| (path, crate::ir::NameSpace::Trait)),
    );
    prelude
}

fn decode_hex_nibble(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some((ch as u32) - ('0' as u32)),
        'a'..='f' => Some((ch as u32) - ('a' as u32) + 10),
        'A'..='F' => Some((ch as u32) - ('A' as u32) + 10),
        _ => None,
    }
}

fn decode_import_path_literal(literal: &str) -> Option<String> {
    if literal.len() < 2 || !literal.starts_with('"') || !literal.ends_with('"') {
        return None;
    }

    let mut result = String::new();
    let mut chars = literal.strip_prefix('"')?.strip_suffix('"')?.chars();
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

fn normalize_import_path(
    source_path: &Path,
    import_path: &str,
) -> Option<PathBuf> {
    let import_path = Path::new(import_path);
    if import_path.is_absolute() {
        return None;
    }

    let parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    let joined = parent.join(import_path);

    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(segment) => normalized.push(segment),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    if normalized.as_os_str().is_empty() {
        return None;
    }

    Some(normalized)
}

fn read_core_source_file(
    source_root: &Path,
    source_path: &Path,
) -> Result<String, std::io::Error> {
    std::fs::read_to_string(source_root.join(source_path))
}

fn collect_core_source_fragments_with_imports(
    source_root: &Path,
    source_path: &Path,
    is_root: bool,
    state: &mut CoreSourceTraversalState,
    logger: &mut crate::Logger,
    bundle_name: &mut Option<String>,
    fragments: &mut Vec<CoreSourceFragment>,
) -> Option<()> {
    if state.visited.contains(source_path) {
        return Some(());
    }

    if let Some(cycle_chain) = state.cycle_chain(source_path) {
        let mut file_logger = logger.new_file(display_source_path(source_path), "");
        let chain = cycle_chain
            .iter()
            .map(|path| display_source_path(path))
            .collect::<Vec<_>>()
            .join(" -> ");
        file_logger
            .bug("bundled core source contains a cyclic import")
            .primary(format!("Import cycle: {chain}"), Span::Generated)
            .done();
        logger.consume_file(file_logger);
        return None;
    }

    let source = match read_core_source_file(source_root, source_path) {
        Ok(source) => source,
        Err(error) => {
            let mut file_logger = logger.new_file(display_source_path(source_path), "");
            file_logger
                .bug("bundled core source file could not be read")
                .primary(
                    format!(
                        "Failed to read `{}`: {error}",
                        source_root.join(source_path).display()
                    ),
                    Span::Generated,
                )
                .done();
            logger.consume_file(file_logger);
            return None;
        }
    };

    state.visiting.push(source_path.to_path_buf());

    let mut file_logger = logger.new_file(display_source_path(source_path), source.clone());
    let result = (|| -> Option<()> {
        let source_file = crate::parse::parse(&source, &mut file_logger)?;

        let mut local_bundle_name: Option<String> = None;
        let mut local_bundle_span = Span::Generated;
        let mut statements = Vec::new();

        for item in source_file.items() {
            match item {
                ast::TopLevelItem::Bundle(bundle_declaration) => {
                    if local_bundle_name.is_some() {
                        file_logger
                            .bug("bundled core source declared duplicate bundles")
                            .primary(
                                "Core source may only declare one bundle.",
                                bundle_declaration.span(),
                            )
                            .done();
                        continue;
                    }
                    local_bundle_name = bundle_declaration.name_text();
                    local_bundle_span = bundle_declaration.span();
                }
                ast::TopLevelItem::Import(import_statement) => {
                    if !statements.is_empty() {
                        fragments.push(CoreSourceFragment {
                            source_path: source_path.to_path_buf(),
                            source: source.clone(),
                            statements: std::mem::take(&mut statements),
                        });
                    }

                    for path_literal in import_statement.path_literals() {
                        let Some(decoded_path) = decode_import_path_literal(&path_literal.inner)
                        else {
                            file_logger
                                .bug("bundled core source import path literal is invalid")
                                .primary("Expected a valid string literal path.", path_literal.span)
                                .done();
                            continue;
                        };

                        let Some(normalized_path) =
                            normalize_import_path(source_path, &decoded_path)
                        else {
                            file_logger
                                .bug("bundled core source import path escaped the source root")
                                .primary(
                                    format!("Invalid import path `{decoded_path}`."),
                                    path_literal.span,
                                )
                                .done();
                            continue;
                        };

                        if let Some(cycle_chain) = state.cycle_chain(&normalized_path) {
                            let chain = cycle_chain
                                .iter()
                                .map(|path| display_source_path(path))
                                .collect::<Vec<_>>()
                                .join(" -> ");
                            file_logger
                                .bug("bundled core source contains a cyclic import")
                                .primary(format!("Import cycle: {chain}"), path_literal.span)
                                .done();
                            continue;
                        }

                        let imported_source_path = source_root.join(&normalized_path);
                        if !imported_source_path.is_file() {
                            file_logger
                                .bug("bundled core source import path is unknown")
                                .primary(
                                    format!(
                                        "No core source file exists at `{}`.",
                                        imported_source_path.display()
                                    ),
                                    path_literal.span,
                                )
                                .done();
                            continue;
                        }

                        collect_core_source_fragments_with_imports(
                            source_root,
                            &normalized_path,
                            false,
                            state,
                            logger,
                            bundle_name,
                            fragments,
                        )?;
                    }
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
                .unwrap_or(CORE_MODULE_NAME)
                .to_string();
            file_logger
                .bug("imported core source file declared a bundle")
                .primary(
                    format!(
                        "Imported core source files are part of bundle `{active_bundle_name}` and must not declare `bundle {local_bundle_name}`."
                    ),
                    local_bundle_span,
                )
                .done();
        }

        if !statements.is_empty() {
            fragments.push(CoreSourceFragment {
                source_path: source_path.to_path_buf(),
                source: source.clone(),
                statements,
            });
        }

        Some(())
    })();

    let file_ok = file_logger.is_ok();
    if !file_ok {
        file_logger.escalate_to_bug();
    }
    logger.consume_file(file_logger);

    let _ = state.visiting.pop();
    if result.is_none() || !file_ok {
        return None;
    }

    state.visited.insert(source_path.to_path_buf());
    Some(())
}

fn combine_resolved_fragments(
    bundle_name: &str,
    resolved_fragments: Vec<crate::types::ResolvedModule>,
) -> crate::types::ResolvedModule {
    let mut statements = Vec::new();
    let mut resolved_fragments = resolved_fragments.into_iter();
    let mut combined_schemes = if let Some(first) = resolved_fragments.next() {
        statements.extend(first.module.statements.into_vec());
        first.schemes
    } else {
        return crate::types::ResolvedModule {
            module: crate::ir::Module {
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

    crate::types::ResolvedModule {
        module: crate::ir::Module {
            name: bundle_name.to_string(),
            statements: statements.into_boxed_slice(),
        },
        schemes: combined_schemes,
    }
}

fn resolve_core_source_bundle(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Option<crate::types::ResolvedModule> {
    let source_root = core_source_root_dir();
    if !source_root.is_dir() {
        let mut file_logger =
            logger.new_file(display_source_path(Path::new(CORE_ROOT_FILE_NAME)), "");
        file_logger
            .bug("bundled core source directory was not found")
            .primary(
                format!(
                    "Expected core source directory at `{}`.",
                    source_root.display()
                ),
                Span::Generated,
            )
            .done();
        logger.consume_file(file_logger);
        return None;
    }

    let mut state = CoreSourceTraversalState::default();
    let mut fragments = Vec::new();
    let mut bundle_name = None;
    collect_core_source_fragments_with_imports(
        &source_root,
        Path::new(CORE_ROOT_FILE_NAME),
        true,
        &mut state,
        logger,
        &mut bundle_name,
        &mut fragments,
    )?;

    let root_source = fragments
        .iter()
        .find(|fragment| fragment.source_path == Path::new(CORE_ROOT_FILE_NAME))
        .map(|fragment| fragment.source.clone())
        .unwrap_or_default();

    let mut root_file_logger = logger.new_file(
        display_source_path(Path::new(CORE_ROOT_FILE_NAME)),
        root_source,
    );
    let bundle_name = bundle_name.unwrap_or_else(|| {
        root_file_logger
            .bug("bundled core source did not contain a bundle declaration")
            .done();
        CORE_MODULE_NAME.to_string()
    });

    if bundle_name != CORE_MODULE_NAME {
        root_file_logger
            .bug("bundled core source declared an unexpected bundle name")
            .primary(
                format!("Expected `bundle {CORE_MODULE_NAME}`, found `bundle {bundle_name}`."),
                Span::Generated,
            )
            .done();
    }

    if !root_file_logger.is_ok() {
        root_file_logger.escalate_to_bug();
    }
    logger.consume_file(root_file_logger);
    if !logger.is_ok() {
        return None;
    }

    let mut resolved_fragments = Vec::new();
    let mut wasm_type_defs = indexmap::IndexMap::new();
    let mut lowering_salt = 0;
    for fragment in &fragments {
        let mut file_logger = logger.new_file(
            display_source_path(&fragment.source_path),
            fragment.source.clone(),
        );
        let prelude = name_resolution_prelude(symbols);
        if let Some(ir_module) = crate::ir::bundle_statements_with_prelude_and_wasm_types_and_salt(
            bundle_name.clone(),
            &fragment.statements,
            &mut file_logger,
            &prelude,
            &mut wasm_type_defs,
            &mut lowering_salt,
        ) {
            let resolved = crate::types::resolve_module_with_symbols_and_schemes(
                symbols,
                ir_module,
                &mut file_logger,
            );
            resolved_fragments.push(resolved);
        } else if file_logger.is_ok() {
            file_logger
                .bug("bundled core source lowering failed")
                .primary(
                    "Core source lowering failed without a specific diagnostic.",
                    Span::Generated,
                )
                .done();
        }

        if !file_logger.is_ok() {
            file_logger.escalate_to_bug();
        }
        logger.consume_file(file_logger);
    }

    if !logger.is_ok() {
        return None;
    }

    Some(combine_resolved_fragments(&bundle_name, resolved_fragments))
}
