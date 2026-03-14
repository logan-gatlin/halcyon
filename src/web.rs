use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_source_to_binary(source: &str) -> Result<Box<[u8]>, JsValue> {
    crate::compile_source_linked_with_core(source)
        .map(Vec::into_boxed_slice)
        .map_err(|messages| {
            let errors = Array::new();
            for message in messages {
                errors.push(&JsValue::from_str(&message));
            }
            errors.into()
        })
}
