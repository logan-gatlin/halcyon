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

#[tracing::instrument(skip_all)]
pub fn compile_core_module(
    symbols: &mut SymbolTable,
    logger: &mut crate::Logger,
) -> Artifact {
    fn empty_core_artifact() -> Artifact {
        Artifact {
            module_name: CORE_MODULE_NAME.to_string(),
            ir_module: None,
            binary: Vec::new(),
        }
    }

    register_core_primitive_types(symbols);

    let root_file = Path::new(CORE_ROOT_FILE_NAME);
    let root_source = match read_core_source_file(root_file) {
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
    let artifacts = crate::compile_source_with_options(
        &root_source_name,
        &root_source,
        logger,
        symbols,
        crate::CompileOptions {
            demo_mode: false,
            use_core: false,
            resolve_import: |path| {
                let normalized_path = path.replace('\\', "/");
                let relative_path = normalized_path
                    .strip_prefix(core_prefix.as_str())
                    .unwrap_or(normalized_path.as_str());
                read_core_source_file(Path::new(relative_path)).ok()
            },
        },
    );

    if !logger.is_ok() {
        return empty_core_artifact();
    }

    artifacts
        .into_vec()
        .into_iter()
        .find(|artifact| artifact.module_name == CORE_MODULE_NAME)
        .unwrap_or_else(empty_core_artifact)
}

fn register_core_primitive_types(symbols: &mut SymbolTable) {
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
