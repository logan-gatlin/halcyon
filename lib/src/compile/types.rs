use wasm_encoder::FieldType;

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
    Function,
    Struct(Vec<ReducedType>),
    Array(Box<ReducedType>),
}

impl Type {
    pub fn reduce(self) -> ReducedType {
        use ReducedType::*;
        match self.clone() {
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
            Type::Function(_, _) => Struct(vec![Function, Array(AnyRef.into())]),
            Type::Array(t) => Array(t.reduce().into()),
            // All type recursion must pass through a sum type, and sum types are not
            // recursive at the WASM level. Therefore no rist of infinite recursion here
            Type::Instantiation(ref path, ref items) => Universe::get()
                .get_named_type(path)
                .instantiate(items)
                .unwrap()
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
    #[allow(clippy::map_entry)]
    fn encode(&mut self, mut type_: Type) -> &mut Self {
        type_.visit(|t: &mut Type| {
            let type_ = t.clone();
            let reduced_type = type_.clone().reduce();
            self.make_storage_type(reduced_type);
        });
        self
    }
}

impl Encode<ForeignFunctionType> for TypeEncoder {
    fn encode(&mut self, obj: ForeignFunctionType) -> &mut Self {
        if self.foreign_function_map.contains_key(&obj) {
            self
        } else {
            let id = self.type_section.len() as u32;
            self.type_section.push(RegisteredType::Function(obj.into()));
            self.foreign_function_map.insert(obj, id);
            self
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeEncoder {
    pub id_map: HashMap<ReducedType, u32>,
    pub value_map: HashMap<ReducedType, ValType>,
    pub foreign_function_map: HashMap<ForeignFunctionType, u32>,
    type_section: Vec<RegisteredType>,
    main_type_id: Option<u32>,
}

impl std::default::Default for TypeEncoder {
    fn default() -> Self {
        Self {
            id_map: Default::default(),
            value_map: Default::default(),
            foreign_function_map: Default::default(),
            type_section: Default::default(),
            main_type_id: Default::default(),
        }
    }
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

    pub fn main_fn_type_id(&mut self) -> u32 {
        match self.main_type_id {
            Some(id) => id,
            None => {
                let id = self.type_section.len() as u32;
                self.type_section
                    .push(RegisteredType::Function(FuncType::new([], [])));
                self.main_type_id = Some(id);
                id
            }
        }
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
        if AnyRef == type_
            && let Some(t) = self.value_map.get(&AnyRef).cloned()
        {
            return st::Val(t);
        }
        let rt = match type_.clone() {
            AnyRef => {
                let vt = ValType::Ref(RefType {
                    nullable: false,
                    heap_type: HeapType::ANY,
                });
                self.value_map.insert(type_, vt.clone());
                return st::Val(vt);
            }
            Sum => RegisteredType::Struct(vec![
                StorageType::Val(ValType::I32),
                StorageType::Val(ValType::Ref(RefType {
                    nullable: false,
                    heap_type: HeapType::ANY,
                })),
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
            // String optimization
            Array(type_) if type_ == I8.into() => rt::Array(st::I8),
            // Contravariance
            Array(t) => {
                self.make_storage_type(*t);
                rt::Array(StorageType::Val(ValType::Ref(RefType {
                    nullable: false,
                    heap_type: HeapType::ANY,
                })))
            }
            Function => {
                self.make_storage_type(ReducedType::capture());
                let capture_valtype = self.value_map.get(&ReducedType::capture()).unwrap().clone();
                let any_valtype = self.value_map.get(&ReducedType::AnyRef).unwrap().clone();
                rt::Function(FuncType::new([any_valtype, capture_valtype], [any_valtype]))
            }
        };
        self.add_type_to_registry(type_, rt)
    }

    pub fn finish(self) -> TypeSection {
        self.type_section
            .into_iter()
            .fold(TypeSection::new(), |mut ts, t| {
                match t {
                    RegisteredType::Array(storage_type) => ts.ty().array(&storage_type, true),
                    RegisteredType::Struct(storage_types) => {
                        ts.ty().struct_(storage_types.iter().map(|t| FieldType {
                            element_type: *t,
                            mutable: false,
                        }))
                    }
                    RegisteredType::Function(func_type) => ts.ty().func_type(&func_type),
                };
                ts
            })
    }
}

impl ModuleEncoder {
    pub fn foreign_function_type(&self, type_: ForeignFunctionType) -> u32 {
        *self
            .type_encoder
            .foreign_function_map
            .get(&type_)
            .unwrap_or_else(|| panic!("Foreign function type was not encoded: {type_:?}"))
    }

    pub fn function_type_id(&self) -> u32 {
        self.type_encoder
            .id_map
            .get(&ReducedType::Function)
            .unwrap_or_else(|| panic!("No function type id"))
            .clone()
    }

    pub fn reduced_valtype(&self, type_: &ReducedType) -> ValType {
        self.type_encoder
            .value_map
            .get(type_)
            .unwrap_or_else(|| panic!("No reduced valtype for {type_:?}"))
            .clone()
    }

    pub fn reduced_type_id(&self, type_: &ReducedType) -> u32 {
        self.type_encoder
            .id_map
            .get(type_)
            .unwrap_or_else(|| panic!("No reduced type ID for: {type_:?}"))
            .clone()
    }

    pub fn valtype(&self, type_: &Type) -> ValType {
        self.reduced_valtype(&type_.clone().reduce())
    }

    pub fn type_id(&self, type_: &Type) -> u32 {
        self.reduced_type_id(&type_.clone().reduce())
    }
}
