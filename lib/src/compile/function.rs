use crate::semantic::Type;

use super::*;

pub struct FunctionEncoder<'a> {
    pub module_encoder: &'a mut ModuleEncoder,
    id: u32,
    parameter: ValType,
    local_names: HashMap<Path, u32>,
    local_types: Vec<ValType>,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct EncodedFunction {
    parameters: Vec<ValType>,
    locals: Vec<ValType>,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionEncoder<'a> {
    pub fn new(
        module_encoder: &'a mut ModuleEncoder,
        id: u32,
        parameter_name: Path,
        parameter_type: &Type,
    ) -> Self {
        let mut local_names = HashMap::new();
        local_names.insert(parameter_name, 0);
        let parameter = module_encoder.valtype(parameter_type);
        let local_types = vec![parameter];
        FunctionEncoder {
            module_encoder,
            id,
            parameter,
            local_names,
            local_types,
            instructions: vec![],
        }
    }

    pub fn function(&'a mut self, parameter_name: Path, parameter_type: &Type) -> Self {
        self.module_encoder.function(parameter_name, parameter_type)
    }

    pub fn with_capture(&mut self, capture_names: &[Path], capture_types: &[Type]) -> &mut Self {
        for (name, type_) in capture_names.iter().zip(capture_types) {
            self.new_local(name, type_);
        }
        self
    }

    fn _new_local(&mut self, type_: &Type) -> u32 {
        let vt = self.module_encoder.valtype(type_);
        // Add 2 because of parameter and closure capture
        let local_id = 2 + self.local_types.len() as u32;
        self.local_types.push(vt);
        local_id
    }

    fn find_symbol(&self, path: &Path) -> VariableKind {
        if let Some(id) = self.local_names.get(path).cloned() {
            VariableKind::Local(id)
        } else {
            VariableKind::Global(self.module_encoder.find_symbol(path))
        }
    }

    pub fn new_temporary(&mut self, type_: &Type) -> u32 {
        self._new_local(type_)
    }

    pub fn new_local(&mut self, path: &Path, type_: &Type) -> &mut Self {
        let local_id = self._new_local(type_);
        self.local_names.insert(path.clone(), local_id);
        self
    }

    pub fn get_symbol(&mut self, path: &Path) -> &mut Self {
        self.encode(self.find_symbol(path).get())
    }

    pub fn set_symbol(&mut self, path: &Path) -> &mut Self {
        self.encode(self.find_symbol(path).get())
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
