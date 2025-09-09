mod array;
mod builtin;
mod numbers;
mod strings;
mod wasm;

use array::*;
use numbers::*;
use strings::*;
use wasm::*;

pub use builtin::*;
use std::collections::HashMap;
use wasm_encoder::Instruction;

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
    let (
        mut std_interface,
        mut string_interface,
        mut integer_interface,
        mut real_interface,
        mut wasm_interface,
        mut array_interface,
    ) = Default::default();
    let mut init_fn = enc.main_function();
    compile_builtin(&mut init_fn, &mut std_interface);
    compile_string(&mut init_fn, &mut string_interface);
    compile_numbers(&mut init_fn, &mut integer_interface, &mut real_interface);
    compile_wasm(&mut init_fn, &mut wasm_interface);
    compile_array(&mut init_fn, &mut array_interface);
    interfaces.extend([
        (Path::from(STD_MODULE_NAME), std_interface),
        (Path::from(STRING_MODULE_NAME), string_interface),
        (Path::from(INTEGER_MODULE_NAME), integer_interface),
        (Path::from(REAL_MODULE_NAME), real_interface),
        (Path::from(ARRAY_MODULE_NAME), array_interface),
        (Path::from(WASM_MODULE_NAME), wasm_interface),
    ]);
    let id = init_fn.finish_mainfn();
    enc.init_functions.push(id);
    compile_single(include_str!("./stdlib.hc"), enc, interfaces)?;
    Ok(())
}

pub fn one_param(
    enc: &mut FunctionEncoder,
    interface: &mut ModuleInterface,
    path: Path,
    parameter: Type,
    returns: Type,
    f: impl Fn(&mut FunctionEncoder) + 'static,
) {
    let p1 = Path::from("a");
    let type_ = Type::func(parameter.clone(), returns.clone());
    enc.encode(type_.clone());
    enc.module_encoder.new_global(&path, &type_);
    interface.values.insert(path.clone(), type_.clone());
    enc.encode(curry_function([(p1.clone(), parameter)], returns, f))
        .set_symbol(&path);
}

fn n_params(
    enc: &mut FunctionEncoder,
    interface: &mut ModuleInterface,
    path: Path,
    parameters: impl IntoIterator<Item = Type>,
    returns: Type,
    f: impl Fn(&mut FunctionEncoder) + 'static,
) {
    let parameters = parameters.into_iter().collect::<Vec<_>>();
    let type_ = Type::curry(&parameters, returns.clone());
    enc.encode(type_.clone());
    enc.module_encoder.new_global(&path, &type_);
    interface.values.insert(path.clone(), type_.clone());
    let parameters = parameters
        .into_iter()
        .enumerate()
        .map(|(id, t)| {
            (
                Path::from(
                    char::from_u32(('a' as usize + id) as u32)
                        .unwrap()
                        .to_string(),
                ),
                t.clone(),
            )
        })
        .collect::<Vec<_>>();
    enc.encode(curry_function(parameters, returns, f))
        .set_symbol(&path);
}

fn p(n: usize) -> Path {
    Path::from(
        char::from_u32(('a' as usize + n) as u32)
            .unwrap()
            .to_string(),
    )
}

fn constant(enc: &mut FunctionEncoder, path: Path, type_: Type, value: Instruction<'static>) {
    let type_id = enc.module_encoder.type_id(&type_);
    enc.module_encoder.new_global(&path, &type_);
    enc.encode([value, StructNew(type_id)]).set_symbol(&path);
}
