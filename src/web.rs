use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_source_to_binary(
    source: &str
) -> Result<Box<[u8]>, Box<[crate::SerializedDiagnostic]>> {
    crate::compile_source_linked_with_core_logger(source)
        .map(Vec::into_boxed_slice)
        .map_err(|logger| logger.serialize().into_boxed_slice())
}
