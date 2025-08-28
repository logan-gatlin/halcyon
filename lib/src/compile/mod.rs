mod encoding;
mod function;
mod lower;
mod types;

use encoding::*;
use function::*;
use types::*;

use std::collections::HashMap;
// No glob import, conflict with Encode trait
use wasm_encoder::{
    BlockType, FuncType, Function, HeapType, Instruction::*, RefType, StorageType, ValType,
};

type Instruction = wasm_encoder::Instruction<'static>;

use crate::{ir::*, semantic::Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKind {
    Global(u32),
    Local(u32),
}

impl VariableKind {
    pub fn set(self) -> Instruction {
        match self {
            VariableKind::Global(id) => GlobalSet(id),
            VariableKind::Local(id) => LocalSet(id),
        }
    }

    pub fn get(self) -> Instruction {
        match self {
            VariableKind::Global(id) => GlobalGet(id),
            VariableKind::Local(id) => LocalGet(id),
        }
    }
}

pub struct ModuleEncoder {
    type_section: TypeEncoder,
    code_section: Vec<EncodedFunction>,
    function_count: u32,
}

impl ModuleEncoder {
    pub fn valtype(&self, type_: &Type) -> ValType {
        let rt = self.type_section.type_map.get(type_).unwrap();
        self.type_section.value_map.get(rt).unwrap().clone()
    }

    pub fn type_id(&self, type_: &Type) -> u32 {
        let rt = self.type_section.type_map.get(type_).unwrap();
        self.type_section.id_map.get(rt).unwrap().clone()
    }

    pub fn find_symbol(&self, path: &Path) -> u32 {
        todo!()
    }

    pub fn function(&mut self, parameter_name: Path, parameter_type: &Type) -> FunctionEncoder {
        let id = self.function_count;
        self.function_count += 1;
        FunctionEncoder::new(self, id, parameter_name, parameter_type)
    }
}

impl Encode<EncodedFunction> for ModuleEncoder {
    fn encode(&mut self, obj: EncodedFunction) -> &mut Self {
        self.code_section.push(obj);
        self
    }
}
