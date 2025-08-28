use crate::{
    WithSpan,
    semantic::{Type, Universe},
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReducedType {
    AnyRef,
    Sum,
    I64,
    F64,
    I32,
    I8,
    Function,
    Struct(Vec<ReducedType>),
    Array(Box<ReducedType>),
}

impl Type {
    fn reduce(self) -> ReducedType {
        use ReducedType::*;
        match self {
            Type::Any => panic!(),
            Type::Unit => Struct(vec![]),
            Type::Integer => I64,
            Type::Real => F64,
            Type::Boolean => I8,
            Type::String => Array(I8.into()),
            Type::Glyph => I32,
            Type::Variable(_) => AnyRef,
            Type::Struct {
                member_types: items,
                ..
            }
            | Type::Product(items) => Struct(items.into_iter().map(|t| t.reduce()).collect()),
            Type::Sum { .. } => Sum,
            Type::Function(..) => Function,
            // All type recursion must pass through a sum type, and sum types are not
            // recursive at the WASM level. Therefore no rist of infinite recursion here
            Type::Instantiation(path, items) => Universe::get()
                .get_named_type(&path)
                .instantiate(&items)
                .reduce(),
        }
    }
}

impl ReducedType {
    pub fn capture() -> Self {
        Self::Array(Self::AnyRef.into())
    }
}

#[derive(Debug, Clone)]
enum RegisteredType {
    Function(FuncType),
    Array(StorageType),
    Struct(Vec<StorageType>),
}

impl Encode<ReducedType> for TypeEncoder {
    fn encode(&mut self, obj: ReducedType) -> &mut Self {
        self.make_storage_type(obj);
        self
    }
}

impl Encode<Type> for TypeEncoder {
    fn encode(&mut self, type_: Type) -> &mut Self {
        if self.type_map.get(&type_).is_none() {
            let rtype = type_.clone().reduce();
            self.type_map.insert(type_, rtype.clone());
            self.make_storage_type(rtype);
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct TypeEncoder {
    pub type_map: HashMap<Type, ReducedType>,
    pub id_map: HashMap<ReducedType, u32>,
    pub value_map: HashMap<ReducedType, ValType>,
    type_section: Vec<RegisteredType>,
    pub function_map: HashMap<(ReducedType, ReducedType), u32>,
    function_section: Vec<u32>,
}

impl TypeEncoder {
    fn add_type_to_registry(&mut self, type_: ReducedType, rt: RegisteredType) -> StorageType {
        let id = self.type_section.len() as u32;
        self.type_section.push(rt);
        let valtype = ValType::Ref(RefType {
            nullable: false,
            heap_type: HeapType::Concrete(id),
        });
        self.id_map.insert(type_.clone(), id);
        self.value_map.insert(type_, valtype);
        StorageType::Val(valtype)
    }

    fn make_function_type(&mut self, parameter_type: ReducedType, return_type: ReducedType) -> u32 {
        let id = self.type_section.len() as u32;
        self.make_storage_type(parameter_type.clone());
        self.make_storage_type(return_type.clone());
        self.make_storage_type(ReducedType::capture());
        let parameter_valtype = self.value_map.get(&parameter_type).cloned().unwrap();
        let capture_valtype = self
            .value_map
            .get(&ReducedType::capture())
            .cloned()
            .unwrap();
        let return_valtype = self.value_map.get(&return_type).cloned().unwrap();
        RegisteredType::Function(FuncType::new(
            [parameter_valtype, capture_valtype],
            [return_valtype],
        ));
        self.function_map
            .insert((parameter_type, return_type), id as u32);
        id
    }

    fn make_storage_type(&mut self, type_: ReducedType) -> StorageType {
        use ReducedType::*;
        use RegisteredType as rt;
        use StorageType as st;
        use ValType as vt;
        if let Some(id) = self.id_map.get(&type_) {
            return st::Val(vt::Ref(RefType {
                nullable: false,
                heap_type: HeapType::Concrete(*id),
            }));
        }
        let rt = match type_.clone() {
            AnyRef => {
                let vt = ValType::Ref(RefType::ANYREF);
                self.value_map.insert(type_, vt.clone());
                return st::Val(vt);
            }
            Sum => RegisteredType::Struct(vec![
                StorageType::Val(ValType::I32),
                StorageType::Val(ValType::Ref(RefType::ANYREF)),
            ]),
            I64 => rt::Struct(vec![st::Val(vt::I64)]),
            F64 => rt::Struct(vec![st::Val(vt::F64)]),
            I8 | I32 => rt::Struct(vec![st::Val(vt::I32)]),
            Struct(types) => rt::Struct(
                types
                    .into_iter()
                    .map(|t| self.make_storage_type(t))
                    .collect(),
            ),
            Array(type_) if type_ == I8.into() => rt::Array(st::I8),
            Array(type_) => rt::Array(self.make_storage_type(*type_)),
            Function => rt::Struct(vec![
                st::Val(vt::Ref(RefType::FUNCREF)),
                self.make_storage_type(ReducedType::capture()),
            ]),
        };
        self.add_type_to_registry(type_, rt)
    }
}
