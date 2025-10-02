#![allow(clippy::clone_on_copy, clippy::from_over_into)]
#![feature(iterator_try_collect, if_let_guard)]

macro_rules! error {
    ($logger:ident, $span:expr, $($arg:tt)*) => {
        $logger.log(crate::frontend::err(format!($($arg)*)).span($span))
    };
}

mod compile;
mod frontend;
mod ir;
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
pub use map::*;
use optimize::*;
use parse::*;
use semantic::*;
use std::collections::HashMap;
pub use sx::SXRepr;
use token::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[allow(unused_imports)]
use crate::{
    compile::{ModuleEncoder, encoding::Encode},
    std_hc::compile_std,
};

pub fn compile_single(
    input: &str,
    encoder: &mut ModuleEncoder,
    interfaces: &mut HashMap<Path, ModuleInterface>,
) -> Logger {
    let mut logger = Logger::new();
    let tokens = tokenize(input.chars(), &mut logger);
    let parsed_modules = parse(&mut logger, tokens);
    for module in parsed_modules {
        let module_path = module.name.inner.clone().into();
        let ir = build_ir(&mut logger, module, &interfaces);
        let (mut typed_ir, interface) = type_solve(&mut logger, ir);
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
    }
    logger
}

#[wasm_bindgen]
pub fn compile(input: &str) -> std::result::Result<Vec<u8>, String> {
    let mut encoder = ModuleEncoder::new();
    let mut interfaces = HashMap::new();
    let mut logs = vec![];
    logs.extend_from_slice(&compile_std(&mut encoder, &mut interfaces).into_logs());
    logs.extend_from_slice(&compile_single(input, &mut encoder, &mut interfaces).into_logs());
    for log in &logs {
        println!("{:?} :: {}", log.span.unwrap(), log.message);
    }
    if logs.len() != 0 {
        panic!();
    }
    let wasm = encoder.finish();
    let _wat = wasmprinter::print_bytes(&wasm).unwrap();
    wasmparser::validate(&wasm)
        .map_err(|e| format!("COMPILER BUG: WASM failed validation with error:\n{e}"))?;
    Ok(wasm)
}
