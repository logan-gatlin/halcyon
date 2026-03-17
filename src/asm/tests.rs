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

#[test]
/// Handles demo init keeps demo source origins.
fn demo_init_keeps_demo_source_origins() {
    let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/test/demo.hc");
    let source_name = source_path.to_string_lossy().to_string();
    let source = std::fs::read_to_string(&source_path).expect("demo source file should be readable");
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();

    let artifacts = crate::compile_source_with_options(
        &source_name,
        &source,
        &mut logger,
        &mut symbols,
        crate::CompileOptions {
            demo_mode: false,
            use_core: true,
            resolve_import: |_| None,
        },
    );
    assert!(logger.is_ok(), "source compilation should succeed");

    let mut link_logger = logger.linking_logger();
    let linked_artifact = crate::linking::link_artifacts(
        &artifacts,
        crate::linking::LinkOptions {
            module_name: "app".to_string(),
            ..Default::default()
        },
        &mut link_logger,
    )
    .expect("linking should succeed");
    logger.consume_file(link_logger);
    assert!(logger.is_ok(), "linking should succeed");

    let binary = linked_artifact.binary;

    let mut linked = None;
    for payload in wasmparser::Parser::new(0).parse_all(&binary) {
        let payload = payload.expect("valid wasm payload");
        if let Payload::CustomSection(reader) = payload
            && reader.name() == super::module_section::LoweredModuleSection::NAME
        {
            linked = super::module_section::LoweredModuleSection::decode_data_slice(reader.data());
            break;
        }
    }
    let linked = linked.expect("linked lowered module section should decode");

    let demo_init = linked
        .functions
        .iter()
        .find(|(path, _)| path.major == "demo" && path.minor == "[init]")
        .map(|(_, function)| function)
        .expect("linked module should include demo init function");
    let origin_files = demo_init
        .op_origins
        .iter()
        .flatten()
        .map(|origin| origin.file_name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        demo_init
            .op_origins
            .iter()
            .flatten()
            .any(|origin| origin.file_name.ends_with("src/test/demo.hc")),
        "demo init should retain source origins pointing to demo.hc, got {origin_files:?}"
    );

    let resolved = super::resolve_module(linked.clone()).expect("linked module should resolve");
    let demo_index = *resolved
        .function_indices
        .get(&crate::ir::Path::new("demo", "[init]"))
        .expect("resolved module should index demo init");
    assert_eq!(demo_index, 481, "expected demo init function index to match runtime frame index");
}
