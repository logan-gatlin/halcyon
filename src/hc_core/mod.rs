/*!
    The `core` bundle contains symbols that are required by the compiler.
    These include standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod symbols;
mod types;

pub use symbols::CoreSymbol;
pub use types::CoreType;

use std::path::Path;
use std::sync::OnceLock;

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

use enum_iterator::all;
use include_dir::{
    Dir,
    include_dir,
};

use crate::logging::WithContext;
use crate::types::SymbolTable;
use crate::{
    Artifact,
    Span,
};

pub const CORE_MODULE_NAME: &str = "core";

const CORE_SOURCE_ROOT: &str = "core";
const CORE_ROOT_FILE_NAME: &str = "bundle.hc";
static CORE_SOURCES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/core");
static CORE_SOURCES_HASH: OnceLock<String> = OnceLock::new();

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CoreTestCacheKey {
    emit_source_map: bool,
    emit_dwarf: bool,
}

#[cfg(test)]
#[derive(Debug, Clone)]
struct CoreTestCacheEntry {
    artifact: Artifact,
    symbols: SymbolTable,
}

#[cfg(test)]
static CORE_TEST_CACHE: OnceLock<Mutex<HashMap<CoreTestCacheKey, CoreTestCacheEntry>>> =
    OnceLock::new();

#[cfg(test)]
fn core_test_cache() -> &'static Mutex<HashMap<CoreTestCacheKey, CoreTestCacheEntry>> {
    CORE_TEST_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn fnv1a_update(
    mut state: u64,
    bytes: &[u8],
) -> u64 {
    for byte in bytes {
        state ^= *byte as u64;
        state = state.wrapping_mul(0x100000001B3);
    }
    state
}

fn collect_core_file_paths(
    dir: &Dir<'_>,
    out: &mut Vec<String>,
) {
    out.extend(
        dir.files()
            .map(|file| file.path().to_string_lossy().replace('\\', "/")),
    );
    for child in dir.dirs() {
        collect_core_file_paths(child, out);
    }
}

pub fn core_sources_fingerprint() -> &'static str {
    CORE_SOURCES_HASH
        .get_or_init(|| {
            let mut paths = Vec::new();
            collect_core_file_paths(&CORE_SOURCES, &mut paths);
            paths.sort();

            let mut state = 0xCBF29CE484222325u64;
            for path in paths {
                state = fnv1a_update(state, path.as_bytes());
                state = fnv1a_update(state, &[0]);
                if let Some(file) = CORE_SOURCES.get_file(path.as_str()) {
                    state = fnv1a_update(state, file.contents());
                }
                state = fnv1a_update(state, &[0xFF]);
            }
            format!("{state:016x}")
        })
        .as_str()
}

#[tracing::instrument(skip_all)]
pub fn compile_core_module(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Artifact {
    compile_core_module_with_debug_info(symbols, logger, true, true)
}

#[tracing::instrument(skip_all)]
pub fn compile_core_module_with_debug_info(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
    emit_source_map: bool,
    emit_dwarf: bool,
) -> Artifact {
    let _profile_total = crate::profiling::scope("core.compile.total");

    #[cfg(test)]
    let test_cache_key = CoreTestCacheKey {
        emit_source_map,
        emit_dwarf,
    };

    fn empty_core_artifact() -> Artifact {
        Artifact {
            module_name: CORE_MODULE_NAME.to_string(),
            ir_module: None,
            binary: Vec::new(),
            source_map: None,
        }
    }

    {
        let _profile = crate::profiling::scope("core.register_primitives");
        register_core_primitive_types(symbols);
    }

    #[cfg(test)]
    if let Some(cache_entry) = core_test_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&test_cache_key)
        .cloned()
    {
        let _profile = crate::profiling::scope("core.cache.test.hit");
        symbols.absorb(&cache_entry.symbols);
        return cache_entry.artifact;
    }

    let root_file = Path::new(CORE_ROOT_FILE_NAME);
    let root_source = {
        let _profile = crate::profiling::scope("core.read_root_source");
        read_core_source_file(root_file)
    };
    let root_source = match root_source {
        Ok(source) => source,
        Err(error) => {
            let mut file_logger = logger.new_file(display_source_path(root_file), "");
            file_logger
                .bug("bundled core source file could not be read")
                .primary(
                    format!(
                        "Failed to read `{}`: {error}",
                        display_source_path(root_file)
                    ),
                    Span::Generated,
                )
                .done();
            logger.consume_file(file_logger);
            return empty_core_artifact();
        }
    };

    let root_source_name = display_source_path(root_file);
    let core_prefix = format!("{CORE_SOURCE_ROOT}/");
    let artifacts = {
        let _profile = crate::profiling::scope("core.compile_source");
        crate::compile_source_with_options(
            &root_source_name,
            &root_source,
            logger,
            symbols,
            crate::CompileOptions {
                demo_mode: false,
                use_core: false,
                emit_source_map,
                emit_dwarf,
                resolve_import: |path| {
                    let normalized_path = path.replace('\\', "/");
                    let relative_path = normalized_path
                        .strip_prefix(core_prefix.as_str())
                        .unwrap_or(normalized_path.as_str());
                    read_core_source_file(Path::new(relative_path)).ok()
                },
            },
        )
    };

    if !logger.is_ok() {
        return empty_core_artifact();
    }

    let artifact = artifacts
        .into_vec()
        .into_iter()
        .find(|artifact| artifact.module_name == CORE_MODULE_NAME)
        .unwrap_or_else(empty_core_artifact);

    #[cfg(test)]
    if logger.is_ok() {
        let _profile = crate::profiling::scope("core.cache.test.store");
        core_test_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                test_cache_key,
                CoreTestCacheEntry {
                    artifact: artifact.clone(),
                    symbols: symbols.clone(),
                },
            );
    }

    artifact
}

pub fn register_core_primitive_types(symbols: &mut SymbolTable) {
    all::<CoreType>().for_each(|symbol| {
        symbols.insert(symbol);
    });
}

fn display_source_path(relative_path: &Path) -> String {
    Path::new(CORE_SOURCE_ROOT)
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_core_source_file(source_path: &Path) -> Result<String, std::io::Error> {
    let source_path = source_path.to_string_lossy().replace('\\', "/");
    let file = CORE_SOURCES.get_file(source_path.as_str()).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("embedded file `{source_path}` does not exist"),
        )
    })?;
    std::str::from_utf8(file.contents())
        .map(|content| content.to_string())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}
