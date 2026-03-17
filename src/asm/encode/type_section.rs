use std::collections::HashMap;

use super::super::*;
use wasm_encoder::{
    ArrayType,
    ConstExpr,
    FieldType,
    FuncType,
    HeapType,
    RefType,
    StorageType,
    ValType,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConcreteType {
    Function(FuncType),
    Array(ArrayType),
    StructType(Box<[FieldType]>),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TypeSection {
    type_section: Vec<ConcreteType>,
    cache: HashMap<ConcreteType, u32>,
}

impl TypeSection {
    /// Creates a new instance.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Handles get or insert.
    fn get_or_insert(
        &mut self,
        ct: ConcreteType,
    ) -> u32 {
        if let Some(index) = self.cache.get(&ct) {
            *index
        } else {
            self.type_section.push(ct.clone());
            let index = (self.type_section.len() - 1) as u32;
            self.cache.insert(ct, index);
            index
        }
    }

    /// Handles new struct.
    pub(crate) fn new_struct(
        &mut self,
        fields: &[Type],
    ) -> u32 {
        let fields = fields
            .iter()
            .map(|f| {
                FieldType {
                    element_type: StorageType::Val(self.valtype_of(f)),
                    mutable: true,
                }
            })
            .collect();
        self.get_or_insert(ConcreteType::StructType(fields))
    }

    /// Handles new array.
    pub(crate) fn new_array(
        &mut self,
        inner: &Type,
    ) -> u32 {
        let ct = ConcreteType::Array(ArrayType(FieldType {
            element_type: self.storagetype_of(inner),
            mutable: true,
        }));
        self.get_or_insert(ct)
    }

    /// Handles new function.
    pub(crate) fn new_function(
        &mut self,
        parameters: &[Type],
        returns: &[Type],
    ) -> u32 {
        let ct = ConcreteType::Function(FuncType::new(
            parameters
                .iter()
                .map(|p| self.valtype_of(p))
                .collect::<Box<_>>(),
            returns
                .iter()
                .map(|p| self.valtype_of(p))
                .collect::<Box<_>>(),
        ));
        self.get_or_insert(ct)
    }

    /// Handles storagetype of.
    fn storagetype_of(
        &mut self,
        type_: &Type,
    ) -> StorageType {
        match type_ {
            Type::I8 => StorageType::I8,
            Type::I16 => StorageType::I16,
            _ => StorageType::Val(self.valtype_of(type_)),
        }
    }

    /// Handles valtype of.
    pub(crate) fn valtype_of(
        &mut self,
        type_: &Type,
    ) -> ValType {
        match type_ {
            Type::Any => ValType::Ref(RefType::ANYREF),
            Type::I8 | Type::I16 => ValType::I32,
            Type::I32 => ValType::I32,
            Type::I64 => ValType::I64,
            Type::F32 => ValType::F32,
            Type::F64 => ValType::F64,
            Type::Struct(items) => {
                ValType::Ref(RefType {
                    nullable: true,
                    heap_type: HeapType::Concrete(self.new_struct(items)),
                })
            }
            Type::Array(inner) => {
                ValType::Ref(RefType {
                    nullable: true,
                    heap_type: HeapType::Concrete(self.new_array(inner)),
                })
            }
            Type::Function {
                parameters,
                results,
            } => {
                ValType::Ref(RefType {
                    nullable: true,
                    heap_type: HeapType::Concrete(self.new_function(parameters, results)),
                })
            }
        }
    }
}

impl wasm_encoder::Encode for TypeSection {
    /// Handles encode.
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        let mut ts = wasm_encoder::TypeSection::new();
        for t in &self.type_section {
            match t {
                ConcreteType::Function(func_type) => ts.ty().func_type(func_type),
                ConcreteType::Array(ArrayType(FieldType {
                    element_type,
                    mutable,
                })) => ts.ty().array(element_type, *mutable),
                ConcreteType::StructType(field_types) => ts.ty().struct_(field_types.clone()),
            }
        }
        ts.encode(sink);
    }
}

impl wasm_encoder::Section for TypeSection {
    /// Returns the identifier for this value.
    fn id(&self) -> u8 {
        1
    }
}

/// Handles default value.
pub(crate) fn default_value(valtype: &ValType) -> ConstExpr {
    match valtype {
        ValType::I32 => ConstExpr::i32_const(0),
        ValType::I64 => ConstExpr::i64_const(0),
        ValType::F32 => ConstExpr::f32_const(0.0.into()),
        ValType::F64 => ConstExpr::f64_const(0.0.into()),
        ValType::V128 => ConstExpr::v128_const(0),
        ValType::Ref(RefType { heap_type, .. }) => ConstExpr::ref_null(*heap_type),
    }
}
