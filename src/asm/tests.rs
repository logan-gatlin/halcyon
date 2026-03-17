#![allow(clippy::unwrap_used)]

use super::*;

use crate::hc_core::compile_core_module;
use crate::types::{
    resolve_module_with_symbols_and_schemes,
    SymbolTable,
};
use crate::{
    parse,
    Logger,
};
use wasmparser::Payload;

/// Handles compile modules.
fn compile_modules(source: &str) -> (Vec<crate::ir::ElaborationResult>, SymbolTable, SourceCatalog) {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("test.hc", source);
    let mut symbols = SymbolTable::new();
    let _ = compile_core_module(&mut symbols, &mut Logger::new());

    let modules = parse::parse(source, &mut file_logger)
        .map(|m| m.modules())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|m| crate::ir::module(m, &mut file_logger))
        .collect::<Vec<_>>();

    let resolved_modules = modules
        .into_iter()
        .map(|m| resolve_module_with_symbols_and_schemes(&mut symbols, m, &mut file_logger))
        .collect::<Vec<_>>();
    let elaborated_modules = resolved_modules
        .into_iter()
        .map(|m| crate::ir::elaborate_module(m, &symbols))
        .collect::<Vec<_>>();

    logger.consume_file(file_logger);
    let source_catalog = logger.source_files();
    logger.print_logs();
    assert!(logger.is_ok());
    (elaborated_modules, symbols, source_catalog)
}

#[test]
/// Handles emits type signature section.
fn emits_type_signature_section() {
    let source = "module demo =\n\tlet f = fn a => a\nend\n";
    let (mut modules, symbols, _) = compile_modules(source);
    let module = modules.pop().unwrap();
    let encoded = encode(lower_module(module, &symbols, &Vec::new()));
    let wasm = encoded.binary;

    let mut found = None;
    for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        let payload = payload.unwrap();
        match payload {
            Payload::CustomSection(reader) if reader.name() == TypeSignatureSection::NAME => {
                found = Some(reader.data().to_vec());
                break;
            }
            _ => {}
        }
    }

    let data = found.expect("type_signature section not found");
    let decoded = TypeSignatureSection::decode_data_slice(&data).expect("decode section");
    assert!(!decoded.defined_terms.is_empty());
}

#[test]
/// Handles type signature preserves definition order.
fn type_signature_preserves_definition_order() {
    let source = "module demo =\n\ttype First = { x: core::Integer }\n\ttype Second = { y: core::Integer }\n\tlet f = fn a => a\nend\n";
    let (mut modules, symbols, _) = compile_modules(source);
    let module = modules.pop().unwrap();
    let encoded = encode(lower_module(module, &symbols, &Vec::new()));
    let wasm = encoded.binary;

    let mut found = None;
    for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        let payload = payload.unwrap();
        match payload {
            Payload::CustomSection(reader) if reader.name() == TypeSignatureSection::NAME => {
                found = Some(reader.data().to_vec());
                break;
            }
            _ => {}
        }
    }

    let data = found.expect("type_signature section not found");
    let decoded = TypeSignatureSection::decode_data_slice(&data).expect("decode section");
    let order = decoded
        .defined_types
        .keys()
        .map(|path| path.minor.clone())
        .collect::<Vec<_>>();
    assert_eq!(order, vec!["First", "Second"]);
}

#[test]
/// Handles exports wasi start symbol without start section.
fn exports_wasi_start_symbol_without_start_section() {
    let source = "module demo =\n\tlet value : core::Integer = core::default\nend\n";
    let (mut modules, symbols, _) = compile_modules(source);
    let module = modules.pop().unwrap();
    let encoded = encode(lower_module(module, &symbols, &Vec::new()));
    let wasm = encoded.binary;

    let mut has_wasi_start_export = false;
    let mut has_start_section = false;

    for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        match payload.unwrap() {
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.unwrap();
                    if export.name == "_start"
                        && matches!(export.kind, wasmparser::ExternalKind::Func)
                    {
                        has_wasi_start_export = true;
                    }
                }
            }
            Payload::StartSection { .. } => {
                has_start_section = true;
            }
            _ => {}
        }
    }

    assert!(
        has_wasi_start_export,
        "module should export a function named _start"
    );
    assert!(
        !has_start_section,
        "module should not emit a wasm start section"
    );
}

#[test]
/// Handles emits standard source map metadata.
fn emits_standard_source_map_metadata() {
    let source = "module demo =\n\tlet value : core::Integer = core::default\nend\n";
    let (mut modules, symbols, source_catalog) = compile_modules(source);
    let module = modules.pop().unwrap();
    let encoded = encode(lower_module(module, &symbols, &source_catalog));

    let mut source_mapping_url = None;
    for payload in wasmparser::Parser::new(0).parse_all(&encoded.binary) {
        let payload = payload.unwrap();
        if let Payload::CustomSection(reader) = payload
            && reader.name() == "sourceMappingURL"
        {
            source_mapping_url = Some(std::str::from_utf8(reader.data()).unwrap().to_string());
        }
    }

    assert_eq!(source_mapping_url.as_deref(), Some("demo.wasm.map"));

    let source_map = encoded
        .source_map
        .expect("encoder should emit a source map json blob");
    let parsed = serde_json::from_str::<serde_json::Value>(&source_map)
        .expect("source map should be valid JSON");
    assert_eq!(parsed["version"], serde_json::Value::from(3));
    assert!(
        parsed["sources"]
            .as_array()
            .is_some_and(|sources| sources.iter().any(|source| source == "test.hc")),
        "source map should include the original halcyon source file"
    );
}

#[test]
/// Handles emits dwarf debug sections.
fn emits_dwarf_debug_sections() {
    let source = "module demo =\n\tlet value : core::Integer = core::default\nend\n";
    let (mut modules, symbols, source_catalog) = compile_modules(source);
    let module = modules.pop().unwrap();
    let encoded = encode(lower_module(module, &symbols, &source_catalog));

    let mut seen = std::collections::BTreeSet::new();
    for payload in wasmparser::Parser::new(0).parse_all(&encoded.binary) {
        let payload = payload.unwrap();
        if let Payload::CustomSection(reader) = payload {
            let name = reader.name();
            if name.starts_with(".debug_") {
                seen.insert(name.to_string());
            }
        }
    }

    assert!(
        seen.contains(".debug_info"),
        "expected .debug_info custom section"
    );
    assert!(
        seen.contains(".debug_line"),
        "expected .debug_line custom section"
    );
}
