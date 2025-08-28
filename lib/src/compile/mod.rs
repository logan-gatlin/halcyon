mod function;
mod lower;
mod types;

use function::*;
use types::*;

use std::collections::HashMap;

use wasm_encoder::{Instruction::*, *};
type Instruction = wasm_encoder::Instruction<'static>;

use crate::{ir::*, semantic::Type};

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

pub struct ModuleEncoder {
    type_section: TypeEncoder,
    code_section: Vec<Function>,
}

impl ModuleEncoder {
    pub fn valtype(&self, type_: &Type) -> ValType {
        let rt = self.type_section.type_map.get(&type_).unwrap();
        self.type_section.value_map.get(rt).unwrap().clone()
    }

    pub fn type_id(&self, type_: &Type) -> u32 {
        let rt = self.type_section.type_map.get(&type_).unwrap();
        self.type_section.id_map.get(rt).unwrap().clone()
    }
}

fn test() {
    let f = Function::new_with_locals_types(vec![]);
}
