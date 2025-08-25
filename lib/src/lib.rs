#![allow(clippy::clone_on_copy)]
#![feature(iterator_try_collect, box_patterns, if_let_guard)]
//mod compile;
mod ir;
mod lint;
mod operator;
mod parse;
mod semantic;
//mod std_hc;
mod map;
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

use crate::operator::BinaryOp;
pub const BUILTIN_MODULE: &str = "builtin";

pub fn compiler_print(s: String) {
    let m = OUTPUT.get_or_init(|| Mutex::new("".into()));
    m.lock().unwrap().push_str(&s);
    m.lock().unwrap().push('\n');
}

static OUTPUT: OnceLock<Mutex<String>> = OnceLock::new();

macro_rules! bt {
    ($($name:expr, $value:expr;)*) => {
        {
            let mut map = HashMap::new();
            $(map.insert(Path::from(format!("{BUILTIN_MODULE}:{}", $name)), $value);)*
            map
        }
    };
}

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
    let mut interfaces = HashMap::new();
    interfaces.insert(
        Path::from(BUILTIN_MODULE),
        ModuleInterface {
            types: bt! {
                "integer", Type::Integer;
                "string", Type::String;
            },
            values: bt! {
                BinaryOp::Plus, Type::curry(&[Type::Integer, Type::Integer], Type::Integer);
                BinaryOp::DoubleEqual, Type::curry(&[Type::Variable(0), Type::Variable(0)], Type::Boolean);
            },
            constructors: HashMap::new(),
        },
    );
    for module in parsed_modules {
        let ir = build_ir(module, &interfaces).handle(&linter)?;
        let typed_ir = type_solve(ir);
        println!("Typed IR:\n{}", typed_ir.clone().sx());
    }
    Universe::print();
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
