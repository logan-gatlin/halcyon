use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
struct CachedCore {
    artifact: crate::Artifact,
    symbols: crate::types::SymbolTable,
}

static CACHED_CORE: OnceLock<Result<CachedCore, Box<[crate::SerializedDiagnostic]>>> =
    OnceLock::new();

fn cached_core() -> Result<&'static CachedCore, Box<[crate::SerializedDiagnostic]>> {
    match CACHED_CORE.get_or_init(|| {
        let mut symbols = crate::types::SymbolTable::new();
        let mut logger = crate::Logger::new();
        let artifact =
            crate::compile_core_module_with_debug_info(&mut symbols, &mut logger, false, false);
        if !logger.is_ok() {
            return Err(logger.serialize().into_boxed_slice());
        }
        Ok(CachedCore { artifact, symbols })
    }) {
        Ok(cached) => Ok(cached),
        Err(diagnostics) => Err(diagnostics.clone()),
    }
}

#[wasm_bindgen]
pub fn compile_source_to_binary(
    source: &str
) -> Result<Box<[u8]>, Box<[crate::SerializedDiagnostic]>> {
    let cached_core = cached_core()?;

    let mut symbols = cached_core.symbols.clone();
    let mut logger = crate::Logger::new();
    let mut artifacts = crate::compile_source_with_options(
        "input.hc",
        source,
        &mut logger,
        &mut symbols,
        crate::CompileOptions {
            demo_mode: true,
            use_core: false,
            emit_source_map: false,
            emit_dwarf: false,
            resolve_import: |_| None,
        },
    )
    .into_vec();

    if !logger.is_ok() {
        return Err(logger.serialize().into_boxed_slice());
    }

    let mut all_artifacts = Vec::with_capacity(artifacts.len().saturating_add(1));
    all_artifacts.push(cached_core.artifact.clone());
    all_artifacts.append(&mut artifacts);

    let mut linking_logger = logger.linking_logger();
    let Some(linked) = crate::linking::link_artifacts(
        &all_artifacts,
        crate::linking::LinkOptions {
            module_name: "app".to_string(),
            emit_source_map: false,
            emit_dwarf: false,
            ..Default::default()
        },
        &mut linking_logger,
    ) else {
        logger.consume_file(linking_logger);
        return Err(logger.serialize().into_boxed_slice());
    };
    logger.consume_file(linking_logger);

    Ok(linked.binary.into_boxed_slice())
}
