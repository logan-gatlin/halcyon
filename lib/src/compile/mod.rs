use std::collections::HashMap;

use wasm_encoder::{Instruction::*, *};
type Instruction = wasm_encoder::Instruction<'static>;

use crate::ir::*;

// 14

pub trait Encode<T> {
    fn encode(&mut self, obj: T);
}

pub struct ModuleEncoder {
    code_section: Vec<Function>,
}

impl Encode<Function> for ModuleEncoder {
    fn encode(&mut self, obj: Function) {
        self.code_section.push(obj);
    }
}

impl ModuleEncoder {
    pub fn function(&mut self) -> FunctionEncoder<'_> {
        FunctionEncoder {
            parent: self,
            local_names: HashMap::new(),
            parameter: None,
            has_closure: false,
            local_types: vec![],
            instructions: vec![],
        }
    }
}

pub struct FunctionEncoder<'a, T: Encode<Function>> {
    parent: &'a mut T,
    local_names: HashMap<Path, u32>,
    parameter: Option<ValType>,
    has_closure: bool,
    local_types: Vec<ValType>,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionEncoder<'a> {
    pub fn function(&'a mut self) -> FunctionEncoder<'a> {
        FunctionEncoder {
            parent: &mut self.parent,
            local_names: HashMap::new(),
            parameter: None,
            has_closure: false,
            local_types: vec![],
            instructions: vec![],
        }
    }
}
