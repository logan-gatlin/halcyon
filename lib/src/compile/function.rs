use crate::semantic::Type;

use super::*;

pub struct FunctionEncoder<'a> {
    module_encoder: &'a mut ModuleEncoder,
    local_names: HashMap<Path, u32>,
    parameter: Option<ValType>,
    has_closure: bool,
    local_types: Vec<ValType>,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionEncoder<'a> {
    pub fn function(&'a mut self) -> Self {
        FunctionEncoder {
            module_encoder: self.module_encoder,
            local_names: HashMap::new(),
            parameter: None,
            has_closure: false,
            local_types: vec![],
            instructions: vec![],
        }
    }

    pub fn new_local(&mut self, path: Path, type_: &Type) -> &mut Self {
        let vt = self.module_encoder.valtype(type_);
        let local_id = self.local_types.len() as u32
            + (self.has_closure as u32)
            + (self.parameter.is_some() as u32);
        self.local_types.push(vt);
        self.local_names.insert(path, local_id);
        self
    }
}

impl Encode<Function> for FunctionEncoder<'_> {
    fn encode(&mut self, obj: Function) -> &mut Self {
        self.module_encoder.encode(obj);
        self
    }
}

impl Encode<Instruction> for FunctionEncoder<'_> {
    fn encode(&mut self, obj: Instruction) -> &mut Self {
        self.instructions.push(obj);
        self
    }
}

impl Encode<Type> for FunctionEncoder<'_> {
    fn encode(&mut self, obj: Type) -> &mut Self {
        self.module_encoder.type_section.encode(obj);
        self
    }
}
