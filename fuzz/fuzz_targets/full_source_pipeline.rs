#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::{
    linking,
    Compiler,
    SourceCompileOptions,
};

use common::{
    bounded_source,
    core_symbols,
};

fuzz_target!(|data: &[u8]| {
    let source = bounded_source(data, 16_384);

    let mut compiler = Compiler::with_symbols(core_symbols());
    let mut resolver = halcyon_lib::NoImports;
    let compiled = compiler.compile_source(
        "fuzz.hc",
        &source,
        SourceCompileOptions::demo().with_debug_info(halcyon_lib::asm::DebugInfoOptions::none()),
        &mut resolver,
    );
    let artifacts = compiled
        .output
        .unwrap_or_else(|| Vec::new().into_boxed_slice());
    let mut logger = compiled.logger;

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
