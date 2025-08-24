mod builtins;
mod strings;

use std::collections::HashMap;

use crate::{Handle, compile::ModuleEncoder, ir::*, operator::*, semantic::*};

pub const BUILTIN_MODULE_NAME: &str = "builtin";

const TRUE: wasm_encoder::Instruction<'static> = wasm_encoder::Instruction::I32Const(1);
const FALSE: wasm_encoder::Instruction<'static> = wasm_encoder::Instruction::I32Const(0);

#[allow(redundant_semicolons)]
#[allow(unused_assignments)]
pub fn make_std_module(
    encoder: &mut ModuleEncoder,
    interfaces: &mut HashMap<Path, ModuleInterface>,
) -> Option<()> {
    let e = encoder;
    let builtin_interface = builtins::make_builtin_module(e);
    interfaces.insert(Path::from(BUILTIN_MODULE_NAME), builtin_interface);

    // Create standard library
    let input = include_str!("stdlib.hc");
    let input = "";
    let linter = crate::Linter::new(input.to_string());
    let tokens = crate::tokenize(input.chars()).handle(&linter)?;
    for parsed_module in crate::parse(tokens).handle(&linter)? {
        let mut ir_module = crate::build_ir(parsed_module.clone(), interfaces).handle(&linter)?;
        let interface = crate::type_solve(&mut ir_module).handle(&linter)?;
        interfaces.insert(Path::from(parsed_module.name), interface);
        e.encode_ir(ir_module);
    }
    Some(())
}

fn make_function(
    encoder: &mut ModuleEncoder,
    name: &str,
    parameter_types: Vec<Type>,
    return_type: Type,
) -> u32 {
    let this_type = Type::curry(&parameter_types, return_type.clone());
    let (head, tail) = encoder.new_curried_function(
        (0..parameter_types.len())
            .map(|i| format!("{i}").into())
            .collect(),
        parameter_types,
        return_type,
    );
    use wasm_encoder::Instruction::*;
    let path = Path::from(BUILTIN_MODULE_NAME).child(name);
    encoder.push(encoder.main_fn, I32Const(head as i32));
    encoder.new_capture(encoder.main_fn, 0u32);
    encoder.new_struct(encoder.main_fn, this_type.clone());
    let global_id = encoder.new_global(path, this_type);
    encoder.push(encoder.main_fn, GlobalSet(global_id));
    tail
}
