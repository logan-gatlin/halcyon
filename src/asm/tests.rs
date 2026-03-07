#![allow(clippy::unwrap_used)]

use super::*;

use crate::hc_core::compile_core_module;
use crate::types::{
    SymbolTable,
    resolve_module_with_symbols_and_schemes,
};
use crate::{
    Logger,
    parse,
};
use wasmparser::Payload;

fn compile_modules(source: &str) -> (Vec<crate::ir::ElaborationResult>, SymbolTable) {
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
    logger.print_logs();
    assert!(logger.is_ok());
    (elaborated_modules, symbols)
}

#[test]
fn emits_type_signature_section() {
    let source = "module demo =\n\tlet f = fn a => a\nend\n";
    let (mut modules, symbols) = compile_modules(source);
    let module = modules.pop().unwrap();
    let wasm = encode(lower_module(module, &symbols));

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
fn type_signature_preserves_definition_order() {
    let source = "module demo =\n\ttype First = { x: core::integer }\n\ttype Second = { y: core::integer }\n\tlet f = fn a => a\nend\n";
    let (mut modules, symbols) = compile_modules(source);
    let module = modules.pop().unwrap();
    let wasm = encode(lower_module(module, &symbols));

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
