#![allow(clippy::unwrap_used)]

use super::*;

use crate::asm::module_section::LoweredModuleSection;
use crate::hc_core::compile_core_module;
use crate::types::SymbolTable;
use crate::{
    Logger,
    compile_source,
    validate_artifact,
};

fn compile_bundle_artifact(
    source: &str,
    file_name: &str,
    symbols: &mut SymbolTable,
    logger: &mut Logger,
) -> Artifact {
    let mut file_logger = logger.new_file(file_name, source);
    let artifacts = compile_source(source, &mut file_logger, symbols);
    logger.consume_file(file_logger);
    assert!(logger.is_ok(), "bundle compilation failed: {file_name}");
    artifacts
        .into_vec()
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("expected one artifact from {file_name}"))
}

fn decode_linked_module(binary: &[u8]) -> Module {
    for payload in wasmparser::Parser::new(0).parse_all(binary) {
        let payload = payload.unwrap();
        if let wasmparser::Payload::CustomSection(reader) = payload
            && reader.name() == LoweredModuleSection::NAME
        {
            return LoweredModuleSection::decode_data_slice(reader.data())
                .expect("valid lowered module section");
        }
    }
    panic!("missing lowered module section")
}

#[test]
fn preserves_explicit_input_start_order() {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();

    let core = compile_core_module(&mut symbols, &mut logger);
    let alpha = compile_bundle_artifact(
        "bundle alpha\nlet base : core::Integer = core::default\n",
        "alpha.hc",
        &mut symbols,
        &mut logger,
    );
    let beta = compile_bundle_artifact(
        "bundle beta\nlet result : core::Integer = alpha::base\n",
        "beta.hc",
        &mut symbols,
        &mut logger,
    );

    let linked = link_artifacts(
        &[core, beta, alpha],
        LinkOptions {
            module_name: "app".to_string(),
            ..Default::default()
        },
    )
    .expect("link should succeed");

    let _ = validate_artifact(linked.clone(), &mut logger);
    assert!(logger.is_ok(), "linked artifact should validate");

    let linked_module = decode_linked_module(&linked.binary);
    assert_eq!(linked_module.export_policy, ExportPolicy::Qualified);

    let start_function = linked_module
        .functions
        .get(&linked_module.start)
        .expect("linked start function exists");
    let call_order = start_function
        .ops
        .iter()
        .filter_map(|instruction| {
            if let Instruction::Call(path) = instruction {
                Some(path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(
        call_order,
        vec![
            Path::new("core", "[init]"),
            Path::new("beta", "[init]"),
            Path::new("alpha", "[init]"),
        ]
    );
}

#[test]
fn strict_mode_rejects_unresolved_global_imports() {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();

    let core = compile_core_module(&mut symbols, &mut logger);
    let _alpha = compile_bundle_artifact(
        "bundle alpha\nlet base : core::Integer = core::default\n",
        "alpha.hc",
        &mut symbols,
        &mut logger,
    );
    let beta = compile_bundle_artifact(
        "bundle beta\nlet result : core::Integer = alpha::base\n",
        "beta.hc",
        &mut symbols,
        &mut logger,
    );

    let error = link_artifacts(&[core, beta], LinkOptions::default()).expect_err("must fail");
    assert!(matches!(
        error,
        LinkError::UnresolvedGlobalImport { path } if path.major == "alpha"
    ));
}

#[test]
fn rejects_duplicate_module_names() {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();

    let core = compile_core_module(&mut symbols, &mut logger);

    let error =
        link_artifacts(&[core.clone(), core], LinkOptions::default()).expect_err("must fail");
    assert!(matches!(
        error,
        LinkError::DuplicateModuleName { name } if name == "core"
    ));
}

#[test]
fn linked_output_can_be_relinked() {
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();

    let core = compile_core_module(&mut symbols, &mut logger);
    let alpha = compile_bundle_artifact(
        "bundle alpha\nlet base : core::Integer = core::default\n",
        "alpha.hc",
        &mut symbols,
        &mut logger,
    );
    let beta = compile_bundle_artifact(
        "bundle beta\nlet result : core::Integer = alpha::base\n",
        "beta.hc",
        &mut symbols,
        &mut logger,
    );

    let linked = link_artifacts(
        &[core, alpha, beta],
        LinkOptions {
            module_name: "bundle_one".to_string(),
            ..Default::default()
        },
    )
    .expect("first link should succeed");

    let gamma = compile_bundle_artifact(
        "bundle gamma\nlet value : core::Integer = beta::result\n",
        "gamma.hc",
        &mut symbols,
        &mut logger,
    );

    let relinked = link_binaries(
        &[linked.binary.as_slice(), gamma.binary.as_slice()],
        LinkOptions {
            module_name: "bundle_two".to_string(),
            ..Default::default()
        },
    )
    .expect("relink should succeed");

    let _ = validate_artifact(relinked.clone(), &mut logger);
    assert!(logger.is_ok(), "relinked artifact should validate");

    let relinked_module = decode_linked_module(&relinked.binary);
    assert!(
        relinked_module
            .globals
            .contains_key(&Path::new("beta", "result"))
    );
    assert!(
        relinked_module
            .globals
            .contains_key(&Path::new("gamma", "value"))
    );
}

#[test]
fn reports_missing_linker_metadata() {
    let binary = wasm_encoder::Module::new().finish();
    let error = link_binaries(&[binary.as_slice()], LinkOptions::default()).expect_err("must fail");
    assert!(matches!(
        error,
        LinkError::MissingLoweredModuleSection { .. }
    ));
}
