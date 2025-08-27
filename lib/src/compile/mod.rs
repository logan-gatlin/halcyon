mod lower;
mod types;

use std::collections::HashMap;

use wasm_encoder::{Instruction::*, *};
type Instruction = wasm_encoder::Instruction<'static>;

use crate::ir::*;

// 14

pub trait Encode<T> {
    fn encode(&mut self, obj: T) -> &mut Self;
}

impl<T, U, const N: usize> Encode<[T; N]> for U
where
    U: Encode<T>,
{
    fn encode(&mut self, objs: [T; N]) -> &mut Self {
        for obj in objs {
            self.encode(obj);
        }
        self
    }
}

impl Encode<Function> for ModuleEncoder {
    fn encode(&mut self, obj: Function) -> &mut Self {
        self.code_section.push(obj);
        self
    }
}

impl<'a> Encode<Function> for FunctionEncoder<'a> {
    fn encode(&mut self, obj: Function) -> &mut Self {
        self.module_encoder.encode(obj);
        self
    }
}

impl<'a> Encode<Instruction> for FunctionEncoder<'a> {
    fn encode(&mut self, obj: Instruction) -> &mut Self {
        self.instructions.push(obj);
        self
    }
}

pub struct ModuleEncoder {
    code_section: Vec<Function>,
}

impl ModuleEncoder {
    pub fn function(&mut self) -> FunctionEncoder<'_> {
        FunctionEncoder {
            module_encoder: self,
            local_names: HashMap::new(),
            parameter: None,
            has_closure: false,
            local_types: vec![],
            instructions: vec![],
        }
    }
}

pub struct FunctionEncoder<'a> {
    module_encoder: &'a mut ModuleEncoder,
    local_names: HashMap<Path, u32>,
    parameter: Option<ValType>,
    has_closure: bool,
    local_types: Vec<ValType>,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionEncoder<'a> {
    pub fn function(&'a mut self) -> FunctionEncoder<'a> {
        FunctionEncoder {
            module_encoder: self.module_encoder,
            local_names: HashMap::new(),
            parameter: None,
            has_closure: false,
            local_types: vec![],
            instructions: vec![],
        }
    }
}

fn test() {
    let f = Function::new_with_locals_types(vec![]);
}
