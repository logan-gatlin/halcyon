#![allow(clippy::unwrap_used)]

use std::sync::OnceLock;

use super::*;
use crate::types::SymbolTable;
use crate::{
    Logger,
    compile_core_module_with_debug_info,
};

fn core_binary() -> &'static [u8] {
    static CORE_BINARY: OnceLock<Vec<u8>> = OnceLock::new();
    CORE_BINARY
        .get_or_init(|| {
            let mut logger = Logger::new();
            let mut symbols = SymbolTable::new();
            let artifact =
                compile_core_module_with_debug_info(&mut symbols, &mut logger, false, false);
            assert!(
                logger.is_ok(),
                "core module should compile in bindings tests"
            );
            artifact.binary
        })
        .as_slice()
}

#[test]
fn generates_bindings_with_wasi_imports() {
    let generated = generate_js_bindings(core_binary(), GenerateBindingsOptions::default())
        .expect("core bindings should generate successfully");

    let wasi_module = generated
        .spec
        .imports
        .iter()
        .find(|module| module.module == "wasi_snapshot_preview1")
        .expect("core module should require wasi imports");

    assert!(
        wasi_module
            .functions
            .iter()
            .any(|function| function.import_name == "fd_write"),
        "wasi import list should include fd_write"
    );
    assert!(
        generated.javascript.contains("validateImports"),
        "generated JS should include runtime validation"
    );
    assert!(
        generated.typescript.contains("interface HalcyonImports"),
        "generated TS should include imports interface"
    );
}

#[test]
fn generation_is_deterministic() {
    let first = generate_js_bindings(core_binary(), GenerateBindingsOptions::default())
        .expect("first generation should succeed");
    let second = generate_js_bindings(core_binary(), GenerateBindingsOptions::default())
        .expect("second generation should succeed");

    assert_eq!(first.json, second.json);
    assert_eq!(first.javascript, second.javascript);
    assert_eq!(first.typescript, second.typescript);
}

#[test]
fn generated_json_is_valid() {
    let generated = generate_js_bindings(core_binary(), GenerateBindingsOptions::default())
        .expect("bindings generation should succeed");
    let parsed = serde_json::from_str::<serde_json::Value>(&generated.json)
        .expect("bindings JSON should parse");

    assert_eq!(
        parsed["module_name"],
        serde_json::Value::String(generated.spec.module_name.clone())
    );
    assert!(parsed["imports"].is_array());
    assert!(parsed["exports"].is_array());
}
