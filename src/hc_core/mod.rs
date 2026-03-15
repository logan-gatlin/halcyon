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
use include_dir::{
    Dir,
    include_dir,
};

use crate::asm;

use crate::logging::WithContext;
use crate::parse::ast::{
    AstNode,
    HasName,
};
use crate::types::SymbolTable;
use crate::types::symbol_table::Symbol;
use crate::{
    Artifact,
    Span,
};

pub const CORE_MODULE_NAME: &str = "core";

const CORE_SOURCE_ROOT: &str = "core";
const CORE_ROOT_FILE_NAME: &str = "bundle.hc";
static CORE_SOURCES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/core");

#[derive(Debug, Default)]
struct CoreSourceExpansionState {
    visited: HashSet<PathBuf>,
    visiting: Vec<PathBuf>,
}

impl CoreSourceExpansionState {
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

fn display_source_path(relative_path: &Path) -> String {
    Path::new(CORE_SOURCE_ROOT)
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/")
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

fn read_core_source_file(source_path: &Path) -> Result<String, std::io::Error> {
    let source_path = source_path.to_string_lossy().replace('\\', "/");
    let file = CORE_SOURCES.get_file(source_path.as_str()).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("embedded file `{source_path}` does not exist"),
        )
    })?;
    std::str::from_utf8(file.contents())
        .map(|content| content.to_string())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}

fn expand_core_source_with_imports(
    source_path: &Path,
    is_root: bool,
    state: &mut CoreSourceExpansionState,
    logger: &mut crate::Logger,
    expanded_source: &mut String,
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

    let source = match read_core_source_file(source_path) {
        Ok(source) => source,
        Err(error) => {
            let mut file_logger = logger.new_file(display_source_path(source_path), "");
            file_logger
                .bug("bundled core source file could not be read")
                .primary(
                    format!(
                        "Failed to read `{}`: {error}",
                        display_source_path(source_path)
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
        for item in source_file.items() {
            match item {
                crate::parse::ast::TopLevelItem::Bundle(bundle_declaration) => {
                    if !is_root {
                        file_logger
                            .bug("imported core source file declared a bundle")
                            .primary(
                                "Imported core source files are part of the root core bundle and must not declare `bundle`.",
                                bundle_declaration.span(),
                            )
                            .done();
                        continue;
                    }
                    expanded_source.push_str(&bundle_declaration.syntax().text().to_string());
                    expanded_source.push('\n');
                }
                crate::parse::ast::TopLevelItem::Import(import_statement) => {
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

                        let normalized_path_text =
                            normalized_path.to_string_lossy().replace('\\', "/");
                        if CORE_SOURCES
                            .get_file(normalized_path_text.as_str())
                            .is_none()
                        {
                            file_logger
                                .bug("bundled core source import path is unknown")
                                .primary(
                                    format!(
                                        "No core source file exists at `{}`.",
                                        display_source_path(&normalized_path)
                                    ),
                                    path_literal.span,
                                )
                                .done();
                            continue;
                        }

                        expand_core_source_with_imports(
                            &normalized_path,
                            false,
                            state,
                            logger,
                            expanded_source,
                        )?;
                    }
                }
                crate::parse::ast::TopLevelItem::Statement(statement) => {
                    expanded_source.push_str(&statement.syntax().text().to_string());
                    expanded_source.push('\n');
                }
            }
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

fn load_expanded_core_source(logger: &mut crate::Logger) -> Option<String> {
    let mut expanded_source = String::new();
    let mut state = CoreSourceExpansionState::default();
    expand_core_source_with_imports(
        Path::new(CORE_ROOT_FILE_NAME),
        true,
        &mut state,
        logger,
        &mut expanded_source,
    )?;
    Some(expanded_source)
}

fn resolve_core_source_bundle(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Option<crate::types::ResolvedModule> {
    let source = load_expanded_core_source(logger)?;
    let mut file_logger = logger.new_file(
        display_source_path(Path::new(CORE_ROOT_FILE_NAME)),
        source.clone(),
    );

    let Some(source_file) = crate::parse::parse(&source, &mut file_logger) else {
        file_logger.escalate_to_bug();
        logger.consume_file(file_logger);
        return None;
    };

    let mut bundle_name = None;
    let mut statements = Vec::new();
    for item in source_file.items() {
        match item {
            crate::parse::ast::TopLevelItem::Bundle(bundle_declaration) => {
                if bundle_name.is_some() {
                    file_logger
                        .bug("bundled core source declared duplicate bundles")
                        .primary(
                            "Core source may only declare one bundle.",
                            bundle_declaration.span(),
                        )
                        .done();
                    logger.consume_file(file_logger);
                    return None;
                }
                bundle_name = bundle_declaration.name_text();
            }
            crate::parse::ast::TopLevelItem::Import(import_statement) => {
                file_logger
                    .bug("expanded bundled core source may not contain import statements")
                    .primary(
                        "Imports should be expanded before lowering core source.",
                        import_statement.span(),
                    )
                    .done();
                logger.consume_file(file_logger);
                return None;
            }
            crate::parse::ast::TopLevelItem::Statement(statement) => statements.push(statement),
        }
    }

    let bundle_name = bundle_name.unwrap_or_else(|| {
        file_logger
            .bug("bundled core source did not contain a bundle declaration")
            .done();
        CORE_MODULE_NAME.to_string()
    });
    if !file_logger.is_ok() {
        logger.consume_file(file_logger);
        return None;
    }

    if bundle_name != CORE_MODULE_NAME {
        file_logger
            .bug("bundled core source declared an unexpected bundle name")
            .primary(
                format!("Expected `bundle {CORE_MODULE_NAME}`, found `bundle {bundle_name}`."),
                Span::Generated,
            )
            .done();
        logger.consume_file(file_logger);
        return None;
    }

    let prelude = all::<CoreType>()
        .map(|symbol| (symbol.path(), crate::ir::NameSpace::Type))
        .collect::<Vec<_>>();

    let Some(ir_bundle) = crate::ir::bundle_statements_with_prelude(
        bundle_name,
        &statements,
        &mut file_logger,
        &prelude,
    ) else {
        file_logger.escalate_to_bug();
        logger.consume_file(file_logger);
        return None;
    };

    let resolved =
        crate::types::resolve_module_with_symbols_and_schemes(symbols, ir_bundle, &mut file_logger);

    if !file_logger.is_ok() {
        file_logger.escalate_to_bug();
        logger.consume_file(file_logger);
        return None;
    }
    logger.consume_file(file_logger);
    Some(resolved)
}
