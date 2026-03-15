use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WebCompilerLintLabel {
    style: String,
    file_id: u32,
    start: u32,
    end: u32,
    message: String,
}

#[wasm_bindgen]
impl WebCompilerLintLabel {
    #[wasm_bindgen(getter, js_name = style)]
    pub fn style(&self) -> String {
        self.style.clone()
    }

    #[wasm_bindgen(getter, js_name = fileId)]
    pub fn file_id(&self) -> u32 {
        self.file_id
    }

    #[wasm_bindgen(getter, js_name = start)]
    pub fn start(&self) -> u32 {
        self.start
    }

    #[wasm_bindgen(getter, js_name = end)]
    pub fn end(&self) -> u32 {
        self.end
    }

    #[wasm_bindgen(getter, js_name = message)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WebCompilerLint {
    severity: String,
    code: Option<String>,
    message: String,
    notes: Vec<String>,
    labels: Vec<WebCompilerLintLabel>,
}

#[wasm_bindgen]
impl WebCompilerLint {
    #[wasm_bindgen(getter, js_name = severity)]
    pub fn severity(&self) -> String {
        self.severity.clone()
    }

    #[wasm_bindgen(getter, js_name = code)]
    pub fn code(&self) -> Option<String> {
        self.code.clone()
    }

    #[wasm_bindgen(getter, js_name = message)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter, js_name = notes)]
    pub fn notes(&self) -> Box<[JsValue]> {
        self.notes
            .iter()
            .map(|note| JsValue::from_str(note))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    #[wasm_bindgen(getter, js_name = labels)]
    pub fn labels(&self) -> Box<[WebCompilerLintLabel]> {
        self.labels.clone().into_boxed_slice()
    }
}

impl WebCompilerLintLabel {
    fn from_lint_label(label: crate::CompilerLintLabel) -> Self {
        Self {
            style: label.style.as_str().to_string(),
            file_id: label.file_id as u32,
            start: label.start as u32,
            end: label.end as u32,
            message: label.message,
        }
    }
}

impl WebCompilerLint {
    fn from_lint(lint: crate::CompilerLint) -> Self {
        Self {
            severity: lint.severity.as_str().to_string(),
            code: lint.code,
            message: lint.message,
            notes: lint.notes,
            labels: lint
                .labels
                .into_iter()
                .map(WebCompilerLintLabel::from_lint_label)
                .collect(),
        }
    }
}

#[wasm_bindgen]
pub fn compile_source_to_binary(source: &str) -> Result<Box<[u8]>, JsValue> {
    crate::compile_source_linked_with_core(source)
        .map(Vec::into_boxed_slice)
        .map_err(|lints| {
            let web_lints = lints
                .into_iter()
                .map(WebCompilerLint::from_lint)
                .collect::<Vec<_>>();
            let array = Array::new_with_length(web_lints.len() as u32);
            for (index, lint) in web_lints.into_iter().enumerate() {
                array.set(index as u32, JsValue::from(lint));
            }
            array.into()
        })
}
