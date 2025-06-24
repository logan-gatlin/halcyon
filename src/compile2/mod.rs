mod lower;

use std::collections::HashMap;
use wasm_encoder::*;

use crate::hlir::*;

#[derive(Debug, Clone)]
pub struct Function {
  inputs: Vec<ValType>,
  outputs: Vec<ValType>,
  locals: Vec<ValType>,
  instrs: Vec<Instruction<'static>>,
}

impl Function {
  pub fn new(
    inputs: impl IntoIterator<Item = ValType>,
    outputs: impl IntoIterator<Item = ValType>,
  ) -> Self {
    Self {
      inputs: inputs.into_iter().collect(),
      outputs: outputs.into_iter().collect(),
      locals: vec![],
      instrs: vec![],
    }
  }

  pub fn local(&mut self, type_: ValType) -> usize {
    self.locals.push(type_);
    self.inputs.len() + self.locals.len() - 1
  }

  pub fn instr(&mut self, instr: Instruction<'static>) {
    self.instrs.push(instr);
  }
}

pub fn storage_to_valtype(s: StorageType) -> ValType {
  match s {
    StorageType::I8 | StorageType::I16 => ValType::I32,
    StorageType::Val(val_type) => val_type,
  }
}

#[derive(Debug, Clone)]
enum RegisteredType {
  Function(FuncType),
  Array(StorageType),
  Struct(Vec<StorageType>),
}

#[derive(Debug)]
struct ModuleState {
  type_map: HashMap<Type, u32>,
  type_section: Vec<RegisteredType>,
  function_index: HashMap<Mangle, u32>,
  function_types: HashMap<Mangle, u32>,
}

impl ModuleState {
  pub fn make_type_section(&self) -> TypeSection {
    let mut ts = TypeSection::new();
    for t in &self.type_section {
      match t {
        RegisteredType::Function(func_type) => ts.ty().func_type(func_type),
        RegisteredType::Array(storage_type) => {
          ts.ty().array(storage_type, true)
        },
        RegisteredType::Struct(storage_types) => {
          ts.ty()
            .struct_(storage_types.into_iter().map(|t| FieldType {
              element_type: *t,
              mutable: true,
            }))
        },
      }
    }
    ts
  }

  pub fn get_stype(&mut self, t: &Type) -> Option<StorageType> {
    let register = |this: &mut Self, t: Type, rt: RegisteredType| {
      let id = this.type_section.len() as u32;
      this.type_section.push(rt);
      this.type_map.insert(t, id);
      StorageType::Val(ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::Concrete(id),
      }))
    };

    if let Some(t) = self.type_map.get(t) {
      return Some(StorageType::Val(ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::Concrete(*t),
      })));
    }
    let rt = match t {
      Type::Ambiguous => panic!(),
      Type::TypeVariable(_) => panic!(),
      Type::Primitive(p) => {
        return match p {
          Primitive::nothing => None,
          Primitive::never => None,
          Primitive::integer => Some(StorageType::Val(ValType::I64)),
          Primitive::real => Some(StorageType::Val(ValType::F64)),
          Primitive::boolean => Some(StorageType::I8),
          Primitive::string => Some(register(
            self,
            t.clone(),
            RegisteredType::Array(StorageType::I8),
          )),
          Primitive::glyph => Some(StorageType::Val(ValType::I32)),
        };
      },
      Type::Struct {
        member_names,
        member_types,
      } => RegisteredType::Struct(
        member_types
          .into_iter()
          .flat_map(|t| self.get_stype(t))
          .collect(),
      ),
      Type::Function {
        param_types,
        return_type,
      } => RegisteredType::Function(FuncType::new(
        param_types
          .into_iter()
          .flat_map(|t| self.get_stype(t))
          .map(|t| storage_to_valtype(t))
          .collect::<Vec<_>>(),
        self.get_stype(return_type).map(|t| storage_to_valtype(t)),
      )),
      Type::Product(items) => RegisteredType::Struct(
        items.into_iter().flat_map(|t| self.get_stype(t)).collect(),
      ),
      Type::Sum(hash_set) => todo!(),
      Type::Type => todo!(),
    };
    Some(register(self, t.clone(), rt))
  }
}
