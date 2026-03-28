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
        let artifact = crate::compile_core_module_with_debug_info(
            &mut symbols,
            &mut logger,
            crate::asm::DebugInfoOptions::none(),
        );
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

    let mut compiler = crate::Compiler::with_symbols(cached_core.symbols.clone());
    let mut resolver = crate::NoImports;
    let compiled = compiler.compile_source(
        "input.hc",
        source,
        crate::SourceCompileOptions::demo().with_debug_info(crate::asm::DebugInfoOptions::none()),
        &mut resolver,
    );
    let Some(mut artifacts) = compiled.output.map(|artifacts| artifacts.into_vec()) else {
        return Err(compiled.serialized_diagnostics());
    };

    let mut all_artifacts = Vec::with_capacity(artifacts.len().saturating_add(1));
    all_artifacts.push(cached_core.artifact.clone());
    all_artifacts.append(&mut artifacts);

    let linked = compiler.link_artifacts(
        &all_artifacts,
        crate::linking::LinkOptions {
            module_name: "app".to_string(),
            emit_source_map: false,
            emit_dwarf: false,
            ..Default::default()
        },
    );
    let Some(linked) = linked.output else {
        return Err(linked.serialized_diagnostics());
    };

    Ok(linked.binary.into_boxed_slice())
}
