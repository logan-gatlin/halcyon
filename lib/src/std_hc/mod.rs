mod builtin;
mod numbers;
mod strings;

use numbers::*;
use strings::*;

pub use builtin::*;
use std::collections::HashMap;

pub const STD_MODULE_NAME: &str = "std";

use crate::{
    WithSpan,
    compile::*,
    compile_single,
    ir::{IrKind, Path},
    operator::{BinaryOp, UnaryOp},
    semantic::*,
    std_hc::numbers::compile_numbers,
};

pub fn compile_std(
    enc: &mut ModuleEncoder,
    interfaces: &mut HashMap<Path, ModuleInterface>,
) -> std::result::Result<(), String> {
    let mut std_interface = ModuleInterface::default();
    let mut string_interface = ModuleInterface::default();
    let mut integer_interface = ModuleInterface::default();
    let mut real_interface = ModuleInterface::default();
    let mut init_fn = enc.main_function();
    compile_builtin(&mut init_fn, &mut std_interface);
    compile_string(&mut init_fn, &mut string_interface);
    compile_numbers(&mut init_fn, &mut integer_interface, &mut real_interface);
    interfaces.extend([
        (Path::from(STD_MODULE_NAME), std_interface),
        (Path::from(STRING_MODULE_NAME), string_interface),
        (Path::from(INTEGER_MODULE_NAME), integer_interface),
        (Path::from(REAL_MODULE_NAME), real_interface),
    ]);
    let id = init_fn.finish_mainfn();
    enc.init_functions.push(id);
    compile_single(include_str!("./stdlib.hc"), enc, interfaces)?;
    Ok(())
}
