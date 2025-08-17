#![feature(iterator_try_collect, box_patterns, if_let_guard)]
mod compile;
mod ir;
mod lint;
mod operator;
mod parse;
mod semantic;
mod std_hc;
#[cfg(test)]
mod test;
mod token;

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use compile::*;
use ir::*;
use lint::render::Linter;
use parse::*;
use semantic::*;
use std_hc::*;
use token::*;

pub use lint::*;

use wasm_bindgen::prelude::*;

pub fn compiler_print(s: String) {
    let m = OUTPUT.get_or_init(|| Mutex::new("".into()));
    m.lock().unwrap().push_str(&s);
    m.lock().unwrap().push('\n');
}

static OUTPUT: OnceLock<Mutex<String>> = OnceLock::new();

pub fn _compile(input: &str) -> Option<Vec<u8>> {
    OUTPUT
        .get_or_init(|| Mutex::new("".to_string()))
        .lock()
        .unwrap()
        .clear();
    let linter = Linter::new(input.to_string());
    let tokens = tokenize(input.chars()).handle(&linter)?;
    let parsed_modules = parse(tokens).handle(&linter)?;
    let mut encoder = ModuleEncoder::new();
    let mut interfaces = HashMap::new();
    make_std_module(&mut encoder, &mut interfaces);
    for module in parsed_modules {
        let mut ir = build_ir(module, &interfaces).handle(&linter)?;
        let interface = type_solve(&mut ir).handle(&linter)?;
        println!("{ir}");
        interfaces.insert(ir.module_name.clone(), interface);
        encoder.encode_ir(ir);
    }
    let wasm = encoder.finish();
    //let wat = wasmprinter::print_bytes(&wasm).unwrap();
    wasmparser::validate(&wasm)
        .map_err(|e| Lint {
            kind: CompilerBug::FailedValidation.into(),
            context: vec![format!("{e}")],
            span: None,
        })
        .handle(&linter);
    //execute(wasm);
    Some(wasm)
}

#[wasm_bindgen]
pub fn compile(input: &str) -> std::result::Result<Vec<u8>, String> {
    if let Some(b) = _compile(input) {
        Ok(b)
    } else {
        Err(OUTPUT
            .get_or_init(|| Mutex::new("Failed with no reason (BUG)\n".into()))
            .lock()
            .unwrap()
            .clone())
    }
}
