#![allow(clippy::clone_on_copy, clippy::from_over_into)]
#![feature(iterator_try_collect, box_patterns, if_let_guard)]
//mod compile;
mod compile;
mod ir;
mod lint;
mod map;
mod operator;
mod parse;
mod semantic;
mod std_hc;
#[cfg(test)]
mod test;
mod token;
//use compile::*;
use ir::*;
use lint::render::Linter;
use parse::*;
use semantic::*;
use sx::SXRepr;
//use std_hc::*;
pub use lint::*;
pub use map::*;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
use token::*;
use wasm_bindgen::prelude::*;

use crate::{
    compile::{ModuleEncoder, encoding::Encode},
    operator::BinaryOp,
    std_hc::compile_builtin,
};
pub const BUILTIN_MODULE: &str = "builtin";

pub fn compiler_print(s: String) {
    let m = OUTPUT.get_or_init(|| Mutex::new("".into()));
    m.lock().unwrap().push_str(&s);
    m.lock().unwrap().push('\n');
}

static OUTPUT: OnceLock<Mutex<String>> = OnceLock::new();

#[allow(unused_mut)]
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
    compile_builtin(&mut encoder, &mut interfaces);
    for module in parsed_modules {
        let module_path = module.name.inner.clone().into();
        let ir = build_ir(module, &interfaces).handle(&linter)?;
        let (typed_ir, interface) = type_solve(ir);
        interfaces.insert(module_path, interface);
        println!("Typed IR:\n{}", typed_ir.clone().sx());
        encoder.encode(typed_ir);
    }
    let wasm = encoder.finish();
    let wat = wasmprinter::print_bytes(&wasm).unwrap();
    println!("{wat}");
    wasmparser::validate(&wasm).unwrap();
    Some(vec![])
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
