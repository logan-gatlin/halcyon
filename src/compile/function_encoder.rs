use super::*;

#[derive(Debug, Clone)]
pub struct FunctionEncoder {
    local_names: HashMap<Path, u32>,
    parameter: Option<ValType>,
    closure_capture: bool,
    locals: Vec<ValType>,
    instrs: Vec<Instruction<'static>>,
}

impl FunctionEncoder {
    fn local_offset(&self) -> u32 {
        (self.parameter.is_some() as u32) + (self.closure_capture as u32)
    }

    pub fn new_local(&mut self, name: impl Into<Path>, type_: ValType) -> u32 {
        let id = self.locals.len() as u32 + self.local_offset();
        self.locals.push(type_);
        self.local_names.insert(name.into(), id);
        id
    }

    pub fn get_local_id(&self, mangle: &Path) -> u32 {
        *self.local_names.get(mangle).unwrap()
    }

    pub fn has_local(&self, mangle: &Path) -> bool {
        self.local_names.contains_key(mangle)
    }

    pub fn new_temporary(&mut self, type_: ValType) -> u32 {
        let id = self.locals.len() as u32 + self.local_offset();
        self.locals.push(type_);
        id
    }

    fn push(&mut self, instr: Instruction<'static>) {
        self.instrs.push(instr);
    }

    pub fn extend(&mut self, other: &[Instruction<'static>]) {
        self.instrs.extend_from_slice(other);
    }

    pub fn encode_name_map(&self) -> NameMap {
        self.local_names
            .iter()
            .fold(NameMap::new(), |mut map, (name, id)| {
                map.append(*id, name.as_ref());
                map
            })
    }

    pub fn encode(self) -> Function {
        let mut func = Function::new_with_locals_types(self.locals);
        self.instrs
            .into_iter()
            .fold(&mut func, |func, i| func.instruction(&i))
            .instructions()
            .end();
        func
    }
}

#[allow(dead_code)]
impl ModuleEncoder {
    pub fn new_main_function(&mut self) -> u32 {
        self.function_section.push(self.type_section.len() as u32);
        self.type_section
            .push(RegisteredType::Function(FuncType::new([], [])));
        let code_id = self.code_section.len() as u32;
        self.code_section.push(FunctionEncoder {
            local_names: HashMap::new(),
            parameter: None,
            closure_capture: false,
            locals: vec![],
            instrs: vec![],
        });
        let element_id = self.elements_section.len() as u32;
        self.elements_section.push(FunctionKind::Native(code_id));
        element_id
    }

    pub fn get_local(&self, f: u32, local: impl Into<Path>) -> Instruction<'static> {
        let local = *self.func(f).local_names.get(&local.into()).unwrap();
        Instruction::LocalGet(local)
    }

    pub fn new_local(&mut self, f: u32, name: impl Into<Path>, t: Type) -> u32 {
        let t = self.get_asm_type(t);
        self.func_mut(f).new_local(name, t.val)
    }

    pub fn new_local_val(&mut self, f: u32, name: impl Into<Path>, t: ValType) -> u32 {
        self.func_mut(f).new_local(name, t)
    }

    pub fn new_local_any(&mut self, f: u32, name: impl Into<Path>) -> u32 {
        self.func_mut(f)
            .new_local(name, ValType::Ref(RefType::ANYREF))
    }

    pub fn new_function(
        &mut self,
        type_: impl Into<TypeRef>,
        parameter_name: Path,
        capture_names: Vec<Path>,
        capture_types: Vec<TypeRef>,
    ) -> u32 {
        let type_ = type_.into();
        let Type::Function(parameter_type, _) = type_.clone() else {
            panic!()
        };
        let parameter_valtype = if matches!(*parameter_type, Type::TypeVariable(_)) {
            ValType::Ref(RefType::ANYREF)
        } else {
            self.get_asm_type(*parameter_type.clone()).val
        };
        let closure_type_id = self.get_asm_type(Type::_ClosureCapture).id.unwrap();
        let capture_types = capture_types
            .iter()
            .map(|t| self.get_valtype(t, false))
            .collect::<Vec<_>>();
        let mut code = FunctionEncoder {
            local_names: [(parameter_name.clone(), 0)].into_iter().collect(),
            parameter: Some(parameter_valtype),
            closure_capture: true,
            locals: vec![],
            instrs: vec![],
        };
        println!("{parameter_name} {parameter_valtype:?}");
        let parameter_local = code.new_local(parameter_name, parameter_valtype);
        code.push(LocalGet(0));
        if let Some(id) = self.get_asm_type(*parameter_type).id {
            code.push(RefCastNonNull(HeapType::Concrete(id)));
        }
        code.push(LocalSet(parameter_local));
        capture_names
            .into_iter()
            .zip(capture_types)
            .enumerate()
            .for_each(|(id, (c, t))| {
                use Instruction as i;
                let local = code.new_local(c, t);
                code.push(i::LocalGet(1));
                code.push(i::I32Const(id as i32));
                code.push(i::ArrayGet(closure_type_id));
                let ValType::Ref(RefType { heap_type, .. }) = t else {
                    panic!()
                };
                code.push(i::RefCastNonNull(heap_type));
                code.push(i::LocalSet(local));
            });
        let type_id = self.get_asm_type(type_).raw_id.unwrap();
        self.function_section.push(type_id);
        let code_id = self.code_section.len() as u32;
        self.code_section.push(code);
        let element_id = self.elements_section.len() as u32;
        self.elements_section.push(FunctionKind::Native(code_id));
        element_id
    }

    pub fn push(&mut self, function: u32, instruction: Instruction<'static>) {
        self.func_mut(function).push(instruction);
    }

    pub fn new_curried_function(
        &mut self,
        mut parameter_names: Vec<Path>,
        mut parameter_types: Vec<TypeRef>,
        return_type: TypeRef,
    ) -> (u32, u32) {
        let ftype = Type::curry(&parameter_types, return_type.clone());
        parameter_names.reverse();
        parameter_types.reverse();
        let mut tail = 0;
        let head = self.curry(
            parameter_names,
            parameter_types,
            vec![],
            vec![],
            ftype,
            &mut tail,
        );
        (head, tail)
    }

    fn curry(
        &mut self,
        mut parameter_names: Vec<Path>,
        mut parameter_types: Vec<TypeRef>,
        mut capture_names: Vec<Path>,
        mut capture_types: Vec<TypeRef>,
        ftype: TypeRef,
        tail: &mut u32,
    ) -> u32 {
        let name = parameter_names.pop().unwrap();
        let type_ = parameter_types.pop().unwrap();
        let Type::Function(_, r) = ftype.clone() else {
            unreachable! {}
        };
        let f = self.new_function(
            ftype.clone(),
            name.clone(),
            capture_names.clone(),
            capture_types.clone(),
        );
        *tail = f;
        let new_ftype = r;
        capture_names.push(name);
        capture_types.push(type_.clone());
        if !parameter_names.is_empty() {
            let tail = self.curry(
                parameter_names,
                parameter_types,
                capture_names.clone(),
                capture_types.clone(),
                *new_ftype,
                tail,
            );
            self.push(f, I32Const(tail as i32));
            for (name, _type_) in capture_names.iter().zip(capture_types.iter()) {
                self.push(f, self.get_local(f, name.clone()));
            }
            self.new_capture(f, capture_names.len() as u32);
            self.new_struct(f, ftype);
        }
        f
    }
}
