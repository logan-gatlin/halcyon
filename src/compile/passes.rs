use super::*;

pub fn generate_types(
  module: &HlIrModule,
  type_map: &mut HashMap<Type, u32>,
) -> TypeSection {
  let mut type_section = vec![];
  pub fn register_type(
    t: &Type,
    type_map: &mut HashMap<Type, u32>,
    type_section: &mut Vec<RegisteredType>,
  ) -> StorageType {
    let register = |type_map: &mut HashMap<Type, u32>,
                    type_section: &mut Vec<RegisteredType>,
                    t: Type,
                    rt: RegisteredType| {
      let id = type_section.len() as u32;
      type_section.push(rt);
      type_map.insert(t, id);
      StorageType::Val(ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::Concrete(id),
      }))
    };

    if let Some(t) = type_map.get(t) {
      return StorageType::Val(ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::Concrete(*t),
      }));
    }
    let rt = match t {
      Type::Ambiguous => panic!(),
      Type::TypeVariable(_) => panic!(),
      Type::Primitive(p) => {
        return register(
          type_map,
          type_section,
          t.clone(),
          match p {
            Primitive::nothing => RegisteredType::Struct(vec![]),
            Primitive::integer => {
              RegisteredType::Struct(vec![StorageType::Val(ValType::I64)])
            },
            Primitive::real => {
              RegisteredType::Struct(vec![StorageType::Val(ValType::F64)])
            },
            Primitive::boolean => return StorageType::I8,
            Primitive::string => RegisteredType::Array(StorageType::I8),
            Primitive::glyph => {
              RegisteredType::Struct(vec![StorageType::Val(ValType::I32)])
            },
          },
        );
      },
      Type::Struct { member_types, .. } => RegisteredType::Struct(
        member_types
          .into_iter()
          .map(|t| register_type(t, type_map, type_section))
          .collect(),
      ),
      Type::Function {
        param_types,
        return_type,
      } => RegisteredType::Function(FuncType::new(
        param_types
          .into_iter()
          .map(|t| register_type(t, type_map, type_section))
          .map(|t| storage_to_valtype(t))
          .collect::<Vec<_>>(),
        [storage_to_valtype(register_type(
          return_type,
          type_map,
          type_section,
        ))],
      )),
      Type::Product(items) => RegisteredType::Struct(
        items
          .into_iter()
          .map(|t| register_type(t, type_map, type_section))
          .collect(),
      ),
      Type::Sum(_) => todo!(),
      Type::Type => todo!(),
    };
    register(type_map, type_section, t.clone(), rt)
  }
  module.nodes.iter().map(|n| &n.type_).for_each(|t| {
    register_type(t, type_map, &mut type_section);
  });
  let mut ts = TypeSection::new();
  type_section.into_iter().for_each(|t| match t {
    RegisteredType::Function(func_type) => ts.ty().func_type(&func_type),
    RegisteredType::Array(storage_type) => ts.ty().array(&storage_type, true),
    RegisteredType::Struct(storage_types) => {
      ts.ty()
        .struct_(storage_types.into_iter().map(|t| FieldType {
          element_type: t,
          mutable: true,
        }))
    },
  });
  ts
}
