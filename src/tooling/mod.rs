use std::collections::HashSet;
use std::path::{
    Path,
    PathBuf,
};

use crate::logging::WithContext;
use crate::parse::ast::{
    AstNode,
    HasName,
    Statement,
};
use crate::{
    Logger,
    Span,
    ir,
    types,
};

#[derive(Debug, Clone)]
pub struct AnalysisSourceFile {
    pub id: usize,
    pub path: PathBuf,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct BundleAnalysis {
    pub root_path: PathBuf,
    pub bundle_name: String,
    pub source_files: Box<[AnalysisSourceFile]>,
    pub diagnostics: Box<[crate::SerializedDiagnostic]>,
    pub symbols: types::SymbolTable,
    pub name_index: Option<ir::NameIndex>,
}

#[derive(Debug, Clone)]
pub struct FrontendBundleAnalysis {
    pub root_path: PathBuf,
    pub bundle_name: String,
    pub source_files: Box<[AnalysisSourceFile]>,
    pub diagnostics: Box<[crate::SerializedDiagnostic]>,
    pub name_index: Option<ir::NameIndex>,
    pub module: Option<ir::Module<()>>,
}

impl BundleAnalysis {
    pub fn is_ok(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != "error" && diagnostic.severity != "bug")
    }

    pub fn source_for_file_id(
        &self,
        file_id: usize,
    ) -> Option<&str> {
        self.source_files
            .iter()
            .find(|file| file.id == file_id)
            .map(|file| file.source.as_str())
    }

    pub fn file_id_for_path(
        &self,
        path: &Path,
    ) -> Option<usize> {
        let normalized = normalize_path(path);
        self.source_files
            .iter()
            .find(|file| file.path == normalized)
            .map(|file| file.id)
            .or_else(|| {
                self.source_files
                    .iter()
                    .find(|file| file.path == path)
                    .map(|file| file.id)
            })
    }
}

impl FrontendBundleAnalysis {
    pub fn is_ok(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != "error" && diagnostic.severity != "bug")
    }

    pub fn source_for_file_id(
        &self,
        file_id: usize,
    ) -> Option<&str> {
        self.source_files
            .iter()
            .find(|file| file.id == file_id)
            .map(|file| file.source.as_str())
    }

    pub fn file_id_for_path(
        &self,
        path: &Path,
    ) -> Option<usize> {
        let normalized = normalize_path(path);
        self.source_files
            .iter()
            .find(|file| file.path == normalized)
            .map(|file| file.id)
            .or_else(|| {
                self.source_files
                    .iter()
                    .find(|file| file.path == path)
                    .map(|file| file.id)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextPosition {
    pub line: u32,
    pub character: u32,
}

pub fn find_nearest_bundle_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    loop {
        let candidate = current.join("bundle.hc");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn build_core_symbols() -> types::SymbolTable {
    let mut logger = Logger::new();
    let mut symbols = types::SymbolTable::new();
    let _ = crate::compile_core_module_with_debug_info(&mut symbols, &mut logger, false, false);
    symbols
}

pub fn analyze_bundle_with_symbols(
    root_path: &Path,
    base_symbols: &types::SymbolTable,
) -> Result<BundleAnalysis, std::io::Error> {
    analyze_bundle_with_symbols_and_resolver(root_path, base_symbols, &mut |path| {
        std::fs::read_to_string(path).ok()
    })
}

pub fn analyze_bundle_frontend_with_symbols(
    root_path: &Path,
    base_symbols: &types::SymbolTable,
) -> Result<FrontendBundleAnalysis, std::io::Error> {
    analyze_bundle_frontend_with_symbols_and_resolver(root_path, base_symbols, &mut |path| {
        std::fs::read_to_string(path).ok()
    })
}

pub fn analyze_bundle_frontend_with_symbols_and_resolver<F>(
    root_path: &Path,
    base_symbols: &types::SymbolTable,
    resolve_source: &mut F,
) -> Result<FrontendBundleAnalysis, std::io::Error>
where
    F: FnMut(&Path) -> Option<String>,
{
    let root_path = normalize_path(root_path);
    let root_source = resolve_source(&root_path).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{}: unable to read source", root_path.display()),
        )
    })?;

    let root_source_name = root_path.to_string_lossy().replace('\\', "/");
    let mut logger = Logger::new();
    let mut root_file_logger = logger.new_file(root_source_name.clone(), root_source.clone());
    let Some(source_file) = crate::parse::parse(&root_source, &mut root_file_logger) else {
        logger.consume_file(root_file_logger);
        return Ok(FrontendBundleAnalysis {
            root_path,
            bundle_name: "_".to_string(),
            source_files: collect_source_files(&logger),
            diagnostics: logger.serialize().into_boxed_slice(),
            name_index: None,
            module: None,
        });
    };

    let statements = source_file.statements();
    if !matches!(statements.first(), Some(Statement::Bundle(_))) {
        let span = statements.first().map_or_else(
            || {
                root_source
                    .char_indices()
                    .find(|(_, ch)| !ch.is_whitespace())
                    .map(|(start, _)| Span::new(start, 1))
                    .unwrap_or(Span::Generated)
            },
            |statement| {
                match statement.span() {
                    Span::Source { start, .. } => Span::new(start, 1),
                    Span::Generated => Span::Generated,
                }
            },
        );
        root_file_logger
            .error("Missing bundle declaration")
            .primary("Root file must start with `bundle <name>`.", span)
            .done();
    }

    let bundle_name = source_file
        .bundle_declaration()
        .and_then(|bundle| bundle.name_text())
        .unwrap_or_else(|| "_".to_string());
    let prelude = prelude_for_bundle(base_symbols, &bundle_name);
    let lowered = ir::bundle_source_file_with_imports_and_prelude_indexed(
        bundle_name.clone(),
        source_file,
        root_file_logger,
        &mut logger,
        &prelude,
        &mut |lookup_path| {
            let path = Path::new(lookup_path.as_str());
            resolve_source(path)
        },
    );

    let (module, name_index) = if let Some((module, name_index)) = lowered {
        (Some(module), Some(name_index))
    } else {
        (None, None)
    };

    Ok(FrontendBundleAnalysis {
        root_path,
        bundle_name,
        source_files: collect_source_files(&logger),
        diagnostics: logger.serialize().into_boxed_slice(),
        name_index,
        module,
    })
}

pub fn analyze_bundle_with_symbols_and_resolver<F>(
    root_path: &Path,
    base_symbols: &types::SymbolTable,
    resolve_source: &mut F,
) -> Result<BundleAnalysis, std::io::Error>
where
    F: FnMut(&Path) -> Option<String>,
{
    let root_path = normalize_path(root_path);
    let root_source = resolve_source(&root_path).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{}: unable to read source", root_path.display()),
        )
    })?;

    let root_source_name = root_path.to_string_lossy().replace('\\', "/");
    let mut logger = Logger::new();
    let mut symbols = base_symbols.clone();
    let mut root_file_logger = logger.new_file(root_source_name.clone(), root_source.clone());
    let Some(source_file) = crate::parse::parse(&root_source, &mut root_file_logger) else {
        logger.consume_file(root_file_logger);
        return Ok(BundleAnalysis {
            root_path,
            bundle_name: "_".to_string(),
            source_files: collect_source_files(&logger),
            diagnostics: logger.serialize().into_boxed_slice(),
            symbols,
            name_index: None,
        });
    };

    let statements = source_file.statements();
    if !matches!(statements.first(), Some(Statement::Bundle(_))) {
        let span = statements.first().map_or_else(
            || {
                root_source
                    .char_indices()
                    .find(|(_, ch)| !ch.is_whitespace())
                    .map(|(start, _)| Span::new(start, 1))
                    .unwrap_or(Span::Generated)
            },
            |statement| {
                match statement.span() {
                    Span::Source { start, .. } => Span::new(start, 1),
                    Span::Generated => Span::Generated,
                }
            },
        );
        root_file_logger
            .error("Missing bundle declaration")
            .primary("Root file must start with `bundle <name>`.", span)
            .done();
    }
    let bundle_name = source_file
        .bundle_declaration()
        .and_then(|bundle| bundle.name_text())
        .unwrap_or_else(|| "_".to_string());

    let prelude = prelude_for_bundle(&symbols, &bundle_name);
    let lowered = ir::bundle_source_file_with_imports_and_prelude_indexed(
        bundle_name.clone(),
        source_file,
        root_file_logger,
        &mut logger,
        &prelude,
        &mut |lookup_path| {
            let path = Path::new(lookup_path.as_str());
            resolve_source(path)
        },
    );

    let name_index = if let Some((module, name_index)) = lowered {
        let mut typing_logger = logger.new_file(root_source_name, root_source);
        let _ = types::resolve_module_with_symbols_and_schemes(
            &mut symbols,
            module,
            &mut typing_logger,
        );
        logger.consume_file(typing_logger);
        Some(name_index)
    } else {
        None
    };

    Ok(BundleAnalysis {
        root_path,
        bundle_name,
        source_files: collect_source_files(&logger),
        diagnostics: logger.serialize().into_boxed_slice(),
        symbols,
        name_index,
    })
}

pub fn byte_offset_to_utf16_position(
    source: &str,
    byte_offset: usize,
) -> TextPosition {
    let mut clamped = byte_offset.min(source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }

    let mut line = 0u32;
    let mut character = 0u32;
    for ch in source.get(..clamped).unwrap_or("").chars() {
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }

    TextPosition { line, character }
}

pub fn utf16_position_to_byte_offset(
    source: &str,
    line: u32,
    character: u32,
) -> Option<usize> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;
    if line > 0 {
        for (index, ch) in source.char_indices() {
            if ch != '\n' {
                continue;
            }
            current_line += 1;
            if current_line == line {
                line_start = index + ch.len_utf8();
                break;
            }
        }
        if current_line != line {
            return None;
        }
    }

    let mut byte_offset = line_start;
    let mut utf16_col = 0u32;
    while byte_offset < source.len() {
        let ch = source.get(byte_offset..)?.chars().next()?;
        if ch == '\n' {
            break;
        }

        if utf16_col == character {
            return Some(byte_offset);
        }

        let next_utf16_col = utf16_col + ch.len_utf16() as u32;
        if character < next_utf16_col {
            return None;
        }

        utf16_col = next_utf16_col;
        byte_offset += ch.len_utf8();
    }

    (utf16_col == character).then_some(byte_offset)
}

pub fn span_to_utf16_range(
    source: &str,
    span: Span,
) -> Option<(TextPosition, TextPosition)> {
    let Span::Source { start, width, .. } = span else {
        return None;
    };
    Some((
        byte_offset_to_utf16_position(source, start),
        byte_offset_to_utf16_position(source, start + width),
    ))
}

fn prelude_for_bundle(
    symbols: &types::SymbolTable,
    bundle_name: &str,
) -> Vec<(ir::Path, ir::NameSpace)> {
    let mut prelude = crate::name_resolution_prelude(symbols);
    let core_primitive_paths = core_primitive_paths();
    prelude.retain(|(path, namespace)| {
        path.major != bundle_name
            || (bundle_name == crate::hc_core::CORE_MODULE_NAME
                && *namespace == ir::NameSpace::Type
                && core_primitive_paths.contains(path))
    });
    prelude
}

fn core_primitive_paths() -> HashSet<ir::Path> {
    [
        "Unit", "Integer", "Natural", "Real", "Boolean", "String", "Glyph", "Array", "Fn",
    ]
    .into_iter()
    .map(ir::Path::core)
    .collect()
}

fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn collect_source_files(logger: &Logger) -> Box<[AnalysisSourceFile]> {
    logger
        .source_files()
        .into_iter()
        .map(|(id, file_name, source)| {
            AnalysisSourceFile {
                id,
                path: normalize_path(Path::new(&file_name)),
                source,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_position_round_trip_handles_multibyte_text() {
        let source = "a\n\u{03BB}b\n";
        let Some(offset) = utf16_position_to_byte_offset(source, 1, 1) else {
            panic!("position should map to byte offset");
        };
        let position = byte_offset_to_utf16_position(source, offset);
        assert_eq!(position.line, 1);
        assert_eq!(position.character, 1);
    }

    #[test]
    fn utf16_position_to_byte_offset_rejects_mid_surrogate_positions() {
        let source = "a\u{1F600}";
        assert!(utf16_position_to_byte_offset(source, 0, 2).is_none());
    }

    #[test]
    fn prelude_for_core_filters_same_bundle_symbols_except_primitives() {
        let mut symbols = types::SymbolTable::new();
        symbols.insert_type(ir::Path::core("Integer"), types::Type::Integer.def(0));
        symbols.insert_type(ir::Path::core("Option"), types::Type::Unit.def(1));
        symbols.insert_constructor(ir::Path::core("Some"));

        let prelude = prelude_for_bundle(&symbols, crate::hc_core::CORE_MODULE_NAME);

        assert!(prelude.contains(&(ir::Path::core("Integer"), ir::NameSpace::Type)));
        assert!(!prelude.contains(&(ir::Path::core("Option"), ir::NameSpace::Type)));
        assert!(!prelude.contains(&(ir::Path::core("Some"), ir::NameSpace::Constructor)));
    }

    #[test]
    fn frontend_core_analysis_with_core_symbols_avoids_duplicate_diagnostics() {
        let symbols = build_core_symbols();
        let root_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("core/bundle.hc");
        let analysis = analyze_bundle_frontend_with_symbols(&root_path, &symbols)
            .expect("core bundle analysis should read embedded sources from disk");

        let duplicate_diagnostics = analysis
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message.to_lowercase().contains("duplicate"))
            .map(|diagnostic| diagnostic.message.clone())
            .collect::<Vec<_>>();

        assert!(
            duplicate_diagnostics.is_empty(),
            "unexpected duplicate diagnostics while analyzing core: {duplicate_diagnostics:?}"
        );
        assert!(
            analysis.name_index.is_some(),
            "core frontend analysis should produce a name index"
        );
    }

    #[test]
    fn name_index_tracks_module_declarations_and_semantic_module_path_usages() {
        let source = r#"
bundle demo

module foo =
  module bar =
    let value = 1
  end
end

module main =
  use bundle::foo::bar
  use bundle::foo as f
  let from_path = bundle::foo::bar::value
  let from_alias = f::bar::value
end
"#;

        let mut _logger = Logger::new();
        let mut file_logger = _logger.new_file("<module-rename.hc>", source.to_string());
        let source_file = crate::parse::parse(source, &mut file_logger)
            .expect("test source should parse for module usage indexing");
        let (_, name_index) = ir::bundle_statements_with_prelude_indexed(
            "demo".to_string(),
            &source_file.statements(),
            &mut file_logger,
            &[],
        )
        .expect("IR lowering should succeed for module usage indexing test");

        let foo_symbol = ir::ScopedPath {
            path: ir::Path::new("demo", "foo"),
            namespace: ir::NameSpace::Module,
        };
        let bar_symbol = ir::ScopedPath {
            path: ir::Path::new("demo", "foo::bar"),
            namespace: ir::NameSpace::Module,
        };

        let foo_definitions = name_index
            .definitions
            .get(&foo_symbol)
            .expect("module `foo` should have a definition");
        let bar_definitions = name_index
            .definitions
            .get(&bar_symbol)
            .expect("module `foo::bar` should have a definition");
        assert_eq!(foo_definitions.len(), 1, "`foo` should be defined once");
        assert_eq!(
            bar_definitions.len(),
            1,
            "`foo::bar` should be defined once"
        );

        let foo_usages = name_index
            .usages
            .get(&foo_symbol)
            .expect("module `foo` should have semantic usages");
        let bar_usages = name_index
            .usages
            .get(&bar_symbol)
            .expect("module `foo::bar` should have semantic usages");
        assert_eq!(
            foo_usages.len(),
            3,
            "`foo` should be used by direct module paths only"
        );
        assert_eq!(
            bar_usages.len(),
            3,
            "`foo::bar` should be used in all semantic module paths"
        );

        let usage_text = |span: Span| -> String {
            let Span::Source { start, width, .. } = span else {
                panic!("expected source span for indexed usage")
            };
            source
                .get(start..start + width)
                .expect("indexed usage span should be valid UTF-8 boundaries")
                .to_string()
        };
        assert!(
            foo_usages.iter().all(|span| usage_text(*span) == "foo"),
            "`foo` usages should point at `foo` tokens"
        );
        assert!(
            bar_usages.iter().all(|span| usage_text(*span) == "bar"),
            "`foo::bar` usages should point at `bar` tokens"
        );
    }
}
