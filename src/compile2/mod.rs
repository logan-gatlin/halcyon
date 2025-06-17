use crate::hlir::*;

impl Type {
  fn to_valtype(&self) -> Vec<wasm_encoder::ValType> {
    use wasm_encoder::ValType;
    match self {
      Type::Ambiguous | Type::TypeVariable(_) => panic!(),
      Type::Primitive(p) => match p {
        Primitive::nothing => vec![],
        Primitive::never => vec![],
        Primitive::integer => vec![ValType::I32],
        Primitive::real => vec![ValType::F32],
        Primitive::boolean => vec![ValType::I32],
        Primitive::string => vec![ValType::I32, ValType::I32],
        Primitive::glyph => vec![ValType::I32],
      },
      Type::Product(types)
      | Type::Struct {
        member_types: types,
        ..
      } => types.into_iter().flat_map(|t| t.to_valtype()).collect(),
      Type::Sum(hash_set) => todo!(),
      Type::Function {
        param_types,
        return_type,
      } => {
        vec![ValType::FUNCREF]
      }
      Type::Type => todo!(),
    }
  }
}

pub fn compile(hlir: HlIrModule) {
  use wasm_encoder::*;
  let mut module = Module::new();
  todo!()
}
