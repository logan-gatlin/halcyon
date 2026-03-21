#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::{
    CompileOptions,
    Logger,
    linking,
};

use common::{
    bounded_source,
    core_symbols,
};

fuzz_target!(|data: &[u8]| {
    let source = bounded_source(data, 16_384);

    let mut symbols = core_symbols();
    let mut logger = Logger::new();

    let artifacts = halcyon_lib::compile_source_with_options(
        "fuzz.hc",
        &source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: true,
            use_core: false,
            emit_source_map: false,
            emit_dwarf: false,
            resolve_import: |_| None,
        },
    );

    for artifact in artifacts.iter() {
        let _ = wasmparser::validate(&artifact.binary);
        let _ = artifact.decompile_to_wat();
    }

    if logger.is_ok() && !artifacts.is_empty() {
        let mut link_logger = logger.new_file("<link>", "");
        let linked = linking::link_artifacts(
            &artifacts,
            linking::LinkOptions {
                module_name: "fuzz-linked".to_string(),
                strict: false,
                emit_source_map: false,
                emit_dwarf: false,
                ..Default::default()
            },
            &mut link_logger,
        );
        logger.consume_file(link_logger);
        if let Some(artifact) = linked {
            let _ = wasmparser::validate(&artifact.binary);
        }
    }
});
