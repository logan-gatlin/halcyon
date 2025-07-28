mod builtins;

use std::collections::HashMap;

use crate::{
  Handle, compile::ModuleEncoder, ir::*, operator::*, semantic::ModuleInterface,
};

pub const BUILTIN_MODULE_NAME: &str = "builtin";
pub const STDLIB_MODULE_NAME: &str = "std";

const TRUE: wasm_encoder::Instruction<'static> =
  wasm_encoder::Instruction::I32Const(1);
const FALSE: wasm_encoder::Instruction<'static> =
  wasm_encoder::Instruction::I32Const(0);

#[allow(redundant_semicolons)]
#[allow(unused_assignments)]
pub fn make_std_module(
  encoder: &mut ModuleEncoder,
  interfaces: &mut HashMap<Path, ModuleInterface>,
) {
  let e = encoder;
  let builtin_interface = builtins::make_builtin_module(e);
  interfaces.insert(Path::from(BUILTIN_MODULE_NAME), builtin_interface);

  // Create standard library
  let input = include_str!("stdlib.hc");
  let linter = crate::Linter::new(input.to_string());
  let tokens = crate::tokenize(input.chars()).handle(&linter);
  let parsed_module = crate::parse(tokens)
    .handle(&linter)
    .first()
    .expect("stdlib.hc is empty")
    .clone();
  let mut ir_module =
    crate::build_ir(parsed_module, interfaces).handle(&linter);
  let std_interface = crate::type_solve(&mut ir_module).handle(&linter);
  e.encode_ir(ir_module);
  // Finalize stdlib
  interfaces.insert(Path::from(STDLIB_MODULE_NAME), std_interface);
}
