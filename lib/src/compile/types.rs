use crate::semantic::{Type, Universe};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReducedType {
    AnyRef,
    Sum,
    I64,
    F64,
    I32,
    I8,
    Struct(Vec<ReducedType>),
    Array(Box<ReducedType>),
    Function(Box<ReducedType>, Box<ReducedType>),
    Indirection(usize),
}

impl ReducedType {
    fn capture() -> Self {
        Self::Array(Self::AnyRef.into())
    }
}

#[derive(Debug, Clone)]
enum RegisteredType {
    Function(FuncType),
    Array(StorageType),
    Struct(Vec<StorageType>),
}

#[derive(Debug, Clone)]
pub struct TypeEncoder {
    lowering_map: HashMap<(Path, Vec<Type>), usize>,
    indirect_map: Vec<ReducedType>,

    id_map: HashMap<ReducedType, u32>,
    value_map: HashMap<ReducedType, ValType>,
    type_section: Vec<RegisteredType>,
}

impl Encode<ReducedType> for TypeEncoder {
    fn encode(&mut self, type_: ReducedType) -> &mut Self {
        use ReducedType::*;
        let rt = match type_.clone() {
            AnyRef => {
                self.value_map.insert(type_, ValType::Ref(RefType::ANYREF));
                return self;
            },
            Sum => todo!(),
            I64 => todo!(),
            F64 => todo!(),
            I32 => todo!(),
            I8 => todo!(),
            Struct(reduced_types) => todo!(),
            Array(reduced_type) => todo!(),
            Function(reduced_type, reduced_type1) => todo!(),
            Indirection(_) => todo!(),
        }
    }
}

impl TypeEncoder {
    fn lower_type(&mut self, type_: Type) -> ReducedType {
        use ReducedType::*;
        match type_ {
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
            | Type::Product(items) => {
                Struct(items.into_iter().map(|t| self.lower_type(t)).collect())
            }
            Type::Sum { .. } => Sum,
            Type::Function(a, b) => {
                Function(self.lower_type(*a).into(), self.lower_type(*b).into())
            }
            Type::_ClosureCapture => Array(AnyRef.into()),
            Type::Instantiation(path, items) => {
                let instantiated = Universe::get().get_named_type(&path).instantiate(&items);
                let key = (path, items);
                if let Some(id) = self.lowering_map.get(&key) {
                    Indirection(*id)
                } else {
                    let id = self.indirect_map.len();
                    self.lowering_map.insert(key, id);
                    let rt = self.lower_type(instantiated);
                    self.indirect_map.push(rt);
                    Indirection(id)
                }
            }
        }
    }

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

    /*
    fn lower_type(&mut self, type_: Type) -> StorageType {
        use RegisteredType as rt;
        use StorageType as st;
        use ValType as vt;
        if let Some(id) = self.id_map.get(&type_) {
            return st::Val(vt::Ref(RefType {
                nullable: false,
                heap_type: HeapType::Concrete(*id),
            }));
        }
        let registered_type = match type_.clone() {
            Type::Any => todo!(),
            Type::_ClosureCapture => todo!(),
            Type::Unit => rt::Struct(vec![]),
            Type::Integer => rt::Struct(vec![st::Val(vt::I64)]),
            Type::Real => rt::Struct(vec![st::Val(vt::F64)]),
            Type::Boolean => rt::Struct(vec![st::I8]),
            Type::String => rt::Array(st::I8),
            Type::Glyph => rt::Struct(vec![st::Val(vt::I32)]),
            Type::Variable(_) => {
                self.val_map.insert(type_.clone(), vt::Ref(RefType::ANYREF));
                return st::Val(vt::Ref(RefType::ANYREF));
            }
            Type::Product(items)
            | Type::Struct {
                member_types: items,
                ..
            } => rt::Struct(items.into_iter().map(|t| self.lower_type(t)).collect()),
            Type::Sum { .. } => {
                rt::Struct(vec![st::Val(vt::I32), st::Val(vt::Ref(RefType::ANYREF))])
            }
            Type::Function(_, _) => rt::Struct(vec![
                st::Val(vt::I32),
                self.lower_type(Type::_ClosureCapture),
            ]),
            Type::Instantiation(path, types) => {
                return self.lower_type(Universe::get().get_named_type(&path).instantiate(&types));
            }
        };
        self.add_type_to_registry(type_, registered_type)
    }


    }
    */
}
