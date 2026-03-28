#![no_main]

mod common;

use std::collections::HashMap;

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

use halcyon_lib::{
    linking,
    Compiler,
    SourceCompileOptions,
};

use common::{
    bounded_source,
    core_symbols,
    source_from_unstructured,
};

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let import_count = u.int_in_range(0..=8).unwrap_or(0);

    let mut import_paths = Vec::with_capacity(import_count);
    let mut import_sources = HashMap::with_capacity(import_count * 2);
    for index in 0..import_count {
        let path = format!("dep{index}.hc");
        let source =
            source_from_unstructured(&mut u, 2_048).unwrap_or_else(|_| bounded_source(data, 2_048));
        let mut wrapped = format!("let dep_value_{index}: core::Unit = ()\n");
        wrapped.push_str(&source);
        wrapped.push('\n');
        import_paths.push(path.clone());
        import_sources.insert(path.clone(), wrapped.clone());
        import_sources.insert(format!("./{path}"), wrapped);
    }

    let root_tail =
        source_from_unstructured(&mut u, 8_192).unwrap_or_else(|_| bounded_source(data, 8_192));
    let mut root_source = String::from("bundle fuzz\n");
    for path in &import_paths {
        root_source.push_str("import \"");
        root_source.push_str(path);
        root_source.push_str("\"\n");
    }
    root_source.push_str(&root_tail);

    let mut compiler = Compiler::with_symbols(core_symbols());
    let compiled = compiler.compile_source(
        "root.hc",
        &root_source,
        SourceCompileOptions::bundle().with_debug_info(halcyon_lib::asm::DebugInfoOptions::none()),
        &mut |path: &str| {
            if let Some(source) = import_sources.get(path) {
                return Some(source.clone());
            }
            let normalized = path.strip_prefix("./").unwrap_or(path);
            import_sources.get(normalized).cloned()
        },
    );
    let artifacts = compiled
        .output
        .unwrap_or_else(|| Vec::new().into_boxed_slice());
    let mut logger = compiled.logger;

    for artifact in artifacts.iter() {
        let _ = wasmparser::validate(&artifact.binary);
    }

    if logger.is_ok() && !artifacts.is_empty() {
        let mut link_logger = logger.new_file("<link>", "");
        let _ = linking::link_artifacts(
            &artifacts,
            linking::LinkOptions {
                module_name: "fuzz-imports".to_string(),
                strict: false,
                emit_source_map: false,
                emit_dwarf: false,
                ..Default::default()
            },
            &mut link_logger,
        );
        logger.consume_file(link_logger);
    }
});
