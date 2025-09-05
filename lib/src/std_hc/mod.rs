mod builtin;
mod strings;

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
};

pub fn compile_std(
    enc: &mut ModuleEncoder,
    interfaces: &mut HashMap<Path, ModuleInterface>,
) -> std::result::Result<(), String> {
    let mut std_interface = ModuleInterface::default();
    let mut string_interface = ModuleInterface::default();
    let mut init_fn = enc.main_function();
    compile_builtin(&mut init_fn, &mut std_interface);
    compile_string(&mut init_fn, &mut string_interface);
    interfaces.insert(Path::from(STD_MODULE_NAME), std_interface);
    interfaces.insert(Path::from(STRING_MODULE_NAME), string_interface);
    let id = init_fn.finish_mainfn();
    enc.init_functions.push(id);
    compile_single(include_str!("./stdlib.hc"), enc, interfaces)?;
    Ok(())
}
