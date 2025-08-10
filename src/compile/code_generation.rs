use super::*;

#[allow(dead_code)]
impl ModuleEncoder {
    /// Returns (table_id, type_id)
    pub fn new_import<P, R>(
        &mut self,
        major: impl Into<String>,
        minor: impl Into<String>,
        params: P,
        returns: R,
    ) -> (u32, u32)
    where
        P: IntoIterator<Item = ValType>,
        R: IntoIterator<Item = ValType>,
    {
        let ft = FuncType::new(params, returns);
        let type_id = self.type_section.len() as u32;
        self.type_section.push(RegisteredType::Function(ft));
        let import_id = self.import_section.len() as u32;
        self.import_section.push(Import {
            major: major.into(),
            minor: minor.into(),
            entity: EntityType::Function(type_id),
        });
        let element_id = self.elements_section.len() as u32;
        self.elements_section.push(FunctionKind::Import(import_id));
        (element_id, type_id)
    }

    pub fn get_symbol(&mut self, current_function: u32, mangle: &Path) {
        if self.func_mut(current_function).has_local(mangle) {
            let i = self.get_local(current_function, mangle.clone());
            self.push(current_function, i);
        } else if let Some(global_id) = self.global_map.get(mangle) {
            self.push(current_function, GlobalGet(*global_id));
            self.push(current_function, RefAsNonNull);
        } else {
            unreachable!("No symbol exists: {mangle}")
        }
    }

    pub fn get_global_id(&self, path: &Path) -> u32 {
        *self.global_map.get(path).unwrap()
    }

    pub fn make_new_array(&mut self, type_: Type, size: u32) -> Instruction<'static> {
        Instruction::ArrayNewFixed {
            array_type_index: self.get_asm_type(type_).id,
            array_size: size,
        }
    }

    pub fn make_capture(&mut self, size: u32) -> Instruction<'static> {
        Instruction::ArrayNewFixed {
            array_type_index: self.get_asm_type(Type::_ClosureCapture).id,
            array_size: size,
        }
    }

    pub fn make_struct(&mut self, type_: Type) -> Instruction<'static> {
        Instruction::StructNew(self.get_asm_type(type_).id)
    }

    pub fn make_unwrap_primitive(&mut self, type_: Type) -> Instruction<'static> {
        let type_id = self.get_asm_type(type_.clone()).id;
        match type_ {
            Type::Integer | Type::Real | Type::Boolean | Type::String | Type::Glyph => StructGet {
                struct_type_index: type_id,
                field_index: 0,
            },
            _ => todo!(),
        }
    }

    pub fn new_array(&mut self, function: u32, type_: Type, size: u32) {
        let array_t = self.get_asm_type(type_).id;
        self.push(
            function,
            Instruction::ArrayNewFixed {
                array_type_index: array_t,
                array_size: size,
            },
        );
    }

    pub fn new_capture(&mut self, function: u32, size: u32) {
        self.new_array(function, Type::_ClosureCapture, size);
    }

    pub fn new_struct(&mut self, function: u32, type_: Type) {
        let type_id = self.get_asm_type(type_).id;
        self.push(function, Instruction::StructNew(type_id));
    }

    pub fn func_mut(&mut self, index: u32) -> &mut FunctionEncoder {
        let FunctionKind::Native(index) = self.elements_section[index as usize] else {
            panic!()
        };
        &mut self.code_section[index as usize]
    }

    pub fn func(&self, index: u32) -> &FunctionEncoder {
        let FunctionKind::Native(index) = self.elements_section[index as usize] else {
            panic!()
        };
        &self.code_section[index as usize]
    }

    pub fn call_raw_function(&mut self, function: u32, callee_id: u32, callee_type: Type) {
        self.push(function, I32Const(callee_id as i32));
        let callee_type = self.get_asm_type(callee_type).raw_id;
        self.push(
            function,
            CallIndirect {
                type_index: callee_type,
                table_index: 0,
            },
        );
    }

    pub fn push_constant(&mut self, function: u32, c: ConstValue) {
        match c {
            ConstValue::Unit => {
                let tid = self.get_asm_type(Type::Unit).id;
                self.push(function, StructNew(tid));
            }
            ConstValue::Integer(i) => {
                self.push(function, I64Const(i));
                let tid = self.get_asm_type(Type::Integer).id;
                self.push(function, StructNew(tid));
            }
            ConstValue::Real(r) => {
                self.push(function, F64Const((r).into()));
                let tid = self.get_asm_type(Type::Real).id;
                self.push(function, StructNew(tid));
            }
            ConstValue::Boolean(b) => {
                self.push(function, I32Const(if b { 1 } else { 0 }));
                let tid = self.get_asm_type(Type::Boolean).id;
                self.push(function, StructNew(tid));
            }
            ConstValue::String(s) => {
                for b in s.bytes() {
                    self.push(function, I32Const(b as i32));
                }
                let array_type_index = self.get_asm_type(Type::String).id;
                self.push(
                    function,
                    ArrayNewFixed {
                        array_type_index,
                        array_size: s.len() as u32,
                    },
                );
            }
            ConstValue::Glyph(g) => {
                self.push(function, I32Const(g as i32));
                let tid = self.get_asm_type(Type::Glyph).id;
                self.push(function, StructNew(tid));
            }
        }
    }

    pub fn unwrap_primitive(&mut self, function: u32, type_: impl Into<TypeRef>) {
        let type_ = type_.into();
        let type_id = self.get_asm_type(type_.clone()).id;
        match type_ {
            Type::Integer | Type::Real | Type::Boolean | Type::String | Type::Glyph => {
                self.push(
                    function,
                    StructGet {
                        struct_type_index: type_id,
                        field_index: 0,
                    },
                );
            }
            _ => todo!(),
        }
    }
}
