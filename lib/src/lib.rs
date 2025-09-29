#![allow(clippy::clone_on_copy, clippy::from_over_into)]
#![feature(iterator_try_collect, if_let_guard)]

macro_rules! error {
    ($logger:ident, $span:expr, $($arg:tt)*) => {
        $logger.log(crate::frontend::error(format!($($arg)*)).span($span))
    };
}

mod compile;
mod frontend;
mod ir;
mod lint;
mod map;
mod operator;
mod optimize;
mod parse;
mod semantic;
mod std_hc;
#[cfg(test)]
mod test;
mod token;
pub use frontend::*;
use ir::*;
use lint::render::Linter;
pub use lint::*;
pub use map::*;
use optimize::*;
use parse::*;
use semantic::*;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
pub use sx::SXRepr;
use token::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[allow(unused_imports)]
use crate::{
    compile::{ModuleEncoder, encoding::Encode},
    std_hc::compile_std,
};

pub fn compiler_print(s: String) {
    let m = OUTPUT.get_or_init(|| Mutex::new("".into()));
    m.lock().unwrap().push_str(&s);
    m.lock().unwrap().push('\n');
}

pub static OUTPUT: OnceLock<Mutex<String>> = OnceLock::new();

pub fn compile_single(
    input: &str,
    encoder: &mut ModuleEncoder,
    interfaces: &mut HashMap<Path, ModuleInterface>,
) -> std::result::Result<(), String> {
    let mut logger = Logger::new("source");
    let linter = Linter::new(input.to_string());
    let tokens = tokenize(input.chars(), &mut logger);
    let parsed_modules = parse(&mut logger, tokens);
    for module in parsed_modules {
        /*
        let module_path = module.name.inner.clone().into();
        let ir = build_ir(module, &interfaces).handle(&linter)?;
        let (mut typed_ir, interface) = type_solve(ir).handle(&linter)?;
        match interfaces.get_mut(&module_path) {
            Some(old) => {
                old.merge(interface);
            }
            None => {
                interfaces.insert(module_path, interface);
            }
        };
        optimize_ir(&mut typed_ir);
        //println!("Typed IR:\n{}", typed_ir.clone().sx());
        encoder.encode(typed_ir);
        */
    }
    Ok(())
}

#[wasm_bindgen]
pub fn compile(input: &str) -> std::result::Result<Vec<u8>, String> {
    OUTPUT
        .get_or_init(|| Mutex::new("".to_string()))
        .lock()
        .unwrap()
        .clear();
    let mut encoder = ModuleEncoder::new();
    let mut interfaces = HashMap::new();
    //compile_std(&mut encoder, &mut interfaces)?;
    compile_single(input, &mut encoder, &mut interfaces)?;
    let wasm = encoder.finish();
    let _wat = wasmprinter::print_bytes(&wasm).unwrap();
    wasmparser::validate(&wasm)
        .map_err(|e| format!("COMPILER BUG: WASM failed validation with error:\n{e}"))?;
    Ok(wasm)
}
