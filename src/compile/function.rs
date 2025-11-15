use crate::semantic::Type;

use super::*;

pub struct FunctionEncoder<'a> {
    pub module_encoder: &'a mut ModuleEncoder,
    type_id: u32,
    pub local_names: HashMap<Path, u32>,
    local_types: Vec<ValType>,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct EncodedFunction {
    pub type_id: u32,
    locals: Vec<ValType>,
    instructions: Vec<Instruction>,
}

impl EncodedFunction {
    pub fn fix_function_ids(mut self, id_map: &[FunctionKind], imports: u32) -> Self {
        self.instructions.iter_mut().for_each(|i| {
            if let RefFunc(id) | Call(id) = i {
                match id_map[*id as usize] {
                    FunctionKind::Import(new_id) => *id = new_id,
                    FunctionKind::Native(new_id) => *id = new_id + imports,
                }
            }
        });
        self
    }
}

impl Into<Function> for EncodedFunction {
    fn into(self) -> Function {
        self.instructions
            .iter()
            .fold(Function::new_with_locals_types(self.locals), |mut f, i| {
                f.instruction(i);
                f
            })
    }
}

impl<'a> FunctionEncoder<'a> {
    pub fn new_main(module_encoder: &'a mut ModuleEncoder) -> Self {
        Self {
            type_id: module_encoder.type_encoder.main_fn_type_id(),
            module_encoder,
            local_names: HashMap::new(),
            local_types: vec![],
            instructions: vec![],
        }
    }

    pub fn new(
        module_encoder: &'a mut ModuleEncoder,
        parameter_name: Path,
        parameter_type: &Type,
    ) -> Self {
        let mut local_names = HashMap::new();
        let type_id = module_encoder.function_type_id();
        local_names.insert(parameter_name, 2);
        let parameter_type = parameter_type.clone().reduce();
        let parameter_valtype = module_encoder.reduced_valtype(&parameter_type);
        let parameter_cast = if parameter_type == ReducedType::AnyRef {
            Nop
        } else {
            RefCastNonNull(HeapType::Concrete(
                module_encoder.reduced_type_id(&parameter_type),
            ))
        };
        FunctionEncoder {
            module_encoder,
            type_id,
            local_names,
            local_types: vec![parameter_valtype],
            instructions: vec![LocalGet(0), parameter_cast, LocalSet(2)],
        }
    }

    pub fn with_capture(&mut self, capture_names: &[Path], capture_types: &[Type]) -> &mut Self {
        let capture_type = self.module_encoder.reduced_type_id(&ReducedType::capture());
        for (id, (name, type_)) in capture_names.iter().zip(capture_types).enumerate() {
            let reduced_type = type_.clone().reduce();
            self.new_local(name, type_).encode([
                LocalGet(1),
                I32Const(id as i32),
                ArrayGet(capture_type),
            ]);
            if reduced_type != ReducedType::AnyRef {
                let type_id = self.module_encoder.reduced_type_id(&reduced_type);
                self.encode(RefCastNonNull(HeapType::Concrete(type_id)));
            }
            self.set_symbol(name);
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
            VariableKind::Global(self.module_encoder.get_global_id(path))
        }
    }

    pub fn new_temporary(&mut self, type_: &Type) -> u32 {
        self._new_local(type_)
    }

    pub fn new_raw_temporary(&mut self, type_: ValType) -> u32 {
        let local_id = 2 + self.local_types.len() as u32;
        self.local_types.push(type_);
        local_id
    }

    pub fn new_local(&mut self, path: &Path, type_: &Type) -> &mut Self {
        let local_id = self._new_local(type_);
        self.local_names.insert(path.clone(), local_id);
        self
    }

    pub fn get_symbol(&mut self, path: &Path) -> &mut Self {
        match self.find_symbol(path) {
            VariableKind::Global(id) => self.encode([GlobalGet(id), RefAsNonNull]),
            VariableKind::Local(id) => self.encode(LocalGet(id)),
        }
    }

    pub fn set_symbol(&mut self, path: &Path) -> &mut Self {
        self.encode(self.find_symbol(path).set())
    }

    #[must_use]
    pub fn finish(&mut self) -> u32 {
        self.encode(End);
        let id = self.module_encoder.element_section.len() as u32;
        self.module_encoder.encode(EncodedFunction {
            locals: self.local_types.clone(),
            instructions: self.instructions.clone(),
            type_id: self.type_id,
        });
        id
    }

    #[must_use]
    pub fn finish_mainfn(&mut self) -> u32 {
        self.encode(End);
        let id = self.module_encoder.element_section.len() as u32;
        let mut locals = vec![
            ValType::Ref(RefType {
                nullable: false,
                heap_type: HeapType::ANY
            });
            2
        ];
        locals.extend_from_slice(&self.local_types);
        self.module_encoder.encode(EncodedFunction {
            locals,
            instructions: self.instructions.clone(),
            type_id: self.type_id,
        });
        id
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
        self.module_encoder.type_encoder.encode(obj);
        self
    }
}

impl Encode<ReducedType> for FunctionEncoder<'_> {
    fn encode(&mut self, obj: ReducedType) -> &mut Self {
        self.module_encoder.type_encoder.encode(obj);
        self
    }
}
