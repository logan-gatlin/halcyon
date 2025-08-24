use super::*;

#[derive(Debug, Clone)]
pub struct AsmType {
    pub id: Option<u32>,
    pub raw_id: Option<u32>,
    pub val: ValType,
}

impl ModuleEncoder {
    pub fn get_asm_type(&mut self, t: Type) -> AsmType {
        AsmType {
            val: self.get_valtype(&t, false),
            id: self.get_type_id(&t.clone(), false),
            raw_id: self.get_type_id(&t, true),
        }
    }

    fn get_type_id(&mut self, t: &Type, raw: bool) -> Option<u32> {
        match self.get_storage_type(t, raw) {
            StorageType::Val(ValType::Ref(RefType {
                heap_type: HeapType::Concrete(id),
                ..
            })) => Some(id),
            // Anyref does not appear in the type-id list,
            // and the type ID of anyref should never be accessed.
            // Possibly find a cleaner way to do this?
            _ => None,
        }
    }

    pub fn get_valtype(&mut self, t: &Type, raw: bool) -> ValType {
        match self.get_storage_type(t, raw) {
            StorageType::I8 | StorageType::I16 => ValType::I32,
            StorageType::Val(val_type) => val_type,
        }
    }

    fn get_storage_type(&mut self, t: &Type, raw: bool) -> StorageType {
        let register = |this: &mut Self, t: Type, rt: RegisteredType| {
            let id = this.type_section.len() as u32;
            this.type_section.push(rt);
            if raw {
                &mut this.raw_type_map
            } else {
                &mut this.type_map
            }
            .insert(t, id);
            StorageType::Val(ValType::Ref(RefType {
                nullable: false,
                heap_type: HeapType::Concrete(id),
            }))
        };

        if let Some(t) = if raw && let Type::Function(..) = t {
            &self.raw_type_map
        } else {
            &self.type_map
        }
        .get(t)
        {
            return StorageType::Val(ValType::Ref(RefType {
                nullable: false,
                heap_type: HeapType::Concrete(*t),
            }));
        }
        let rt = match t.clone() {
            Type::_ClosureCapture => {
                RegisteredType::Array(StorageType::Val(ValType::Ref(RefType::ANYREF)))
            }
            Type::Any => panic!(),
            Type::TypeVariable(_) => {
                return StorageType::Val(ValType::Ref(RefType::ANYREF));
            }
            Type::Unit => RegisteredType::Struct(vec![]),
            Type::Integer => RegisteredType::Struct(vec![StorageType::Val(ValType::I64)]),
            Type::Real => RegisteredType::Struct(vec![StorageType::Val(ValType::F64)]),
            Type::Boolean => RegisteredType::Struct(vec![StorageType::Val(ValType::I32)]),
            Type::String => RegisteredType::Array(StorageType::I8),
            Type::Glyph => RegisteredType::Struct(vec![StorageType::Val(ValType::I32)]),
            Type::Struct { member_types, .. } => RegisteredType::Struct(
                member_types
                    .into_iter()
                    .map(|t| self.get_storage_type(&t, false))
                    .collect(),
            ),
            Type::Function(_, _) if !raw => {
                let raw_func_type = StorageType::Val(ValType::I32);
                let capture_type = self.get_storage_type(&Type::_ClosureCapture, false);
                RegisteredType::Struct(vec![raw_func_type, capture_type])
            }
            Type::Function(..) => RegisteredType::Function(FuncType::new(
                [
                    ValType::Ref(RefType::ANYREF),
                    self.get_valtype(&Type::_ClosureCapture, false),
                ],
                [ValType::Ref(RefType::ANYREF)],
            )),
            Type::Product(items) => RegisteredType::Struct(
                items
                    .into_iter()
                    .map(|t| self.get_storage_type(&t, false))
                    .collect(),
            ),
            Type::Sum { .. } => RegisteredType::Struct(vec![
                StorageType::Val(ValType::I32),
                StorageType::Val(ValType::Ref(RefType::ANYREF)),
            ]),
            Type::Named(name, types) => {
                return self.get_storage_type(
                    &Type::get_named_type(&name).instantiate_with_substitutions(&types),
                    raw,
                );
            }
        };
        register(self, t.clone(), rt)
    }
}
