mod externals;
mod lower;

use std::collections::HashMap;
use wasm_encoder::*;

use crate::hlir::*;

pub fn compile(mut hlir: HlIrModule) -> Vec<u8> {
  let mut state = ModuleState::new();
  let main = state.make_main_function();
  lower::lower(&mut hlir, 0, &mut state, main);
  state.func(main).instr(Instruction::Drop);
  state.encode()
}

#[derive(Debug, Clone)]
pub struct FunctionEncoder {
  local_names: HashMap<Mangle, u32>,
  parameters: Vec<ValType>,
  locals: Vec<ValType>,
  instrs: Vec<Instruction<'static>>,
}

impl FunctionEncoder {
  pub fn new(parameters: impl IntoIterator<Item = ValType>) -> Self {
    Self {
      local_names: HashMap::new(),
      parameters: parameters.into_iter().collect(),
      locals: vec![],
      instrs: vec![],
    }
  }

  pub fn local(&mut self, name: Mangle, type_: ValType) -> u32 {
    self.locals.push(type_);
    let id = (self.parameters.len() + self.locals.len() - 1) as u32;
    self.local_names.insert(name, id);
    id
  }

  pub fn temporary(&mut self, type_: ValType) -> u32 {
    self.locals.push(type_);
    (self.parameters.len() + self.locals.len() - 1) as u32
  }

  pub fn instr(&mut self, instr: Instruction<'static>) {
    self.instrs.push(instr);
  }

  pub fn encode(self) -> Function {
    let mut func = Function::new_with_locals_types(self.locals);
    self
      .instrs
      .into_iter()
      .fold(&mut func, |func, i| func.instruction(&i))
      .instructions()
      .end();
    func
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

#[derive(Debug, Clone)]
struct ModuleState {
  type_map: HashMap<Type, u32>,
  type_section: Vec<RegisteredType>,
  function_section: Vec<u32>,
  code_section: Vec<FunctionEncoder>,
  import_section: Vec<ForeignFunction>,
}

impl ModuleState {
  pub fn new() -> Self {
    Self {
      type_map: HashMap::new(),
      type_section: vec![],
      function_section: vec![],
      code_section: vec![],
      import_section: vec![],
    }
  }

  pub fn make_main_function(&mut self) -> u32 {
    self.function_section.push(self.type_section.len() as u32);
    self
      .type_section
      .push(RegisteredType::Function(FuncType::new([], [])));
    self.code_section.push(FunctionEncoder::new([]));
    (self.import_section.len() + self.code_section.len() - 1) as u32
  }

  pub fn make_function(
    &mut self,
    type_: &Type,
    parameter_names: impl IntoIterator<Item = Mangle>,
  ) -> u32 {
    let Type::Function { param_types, .. } = type_ else {
      panic!()
    };
    let mut name_map = HashMap::new();
    let mut index = 0;
    for name in parameter_names.into_iter() {
      name_map.insert(name, index);
      index += 1;
    }
    let mut code = FunctionEncoder::new(
      param_types
        .into_iter()
        .map(|t| self.get_type(t))
        .map(storage_to_valtype),
    );
    code.local_names = name_map;
    let id = self.get_type_id(type_);
    self.function_section.push(id);
    self.code_section.push(code);
    (self.import_section.len() + self.code_section.len() - 1) as u32
  }

  pub fn func(&mut self, index: u32) -> &mut FunctionEncoder {
    &mut self.code_section[index as usize - self.import_section.len()]
  }

  pub fn encode(mut self) -> Vec<u8> {
    let import_section = self
      .import_section
      .clone()
      .into_iter()
      .fold(&mut ImportSection::new(), |s, i| {
        s.import(
          &i.major,
          &i.minor,
          EntityType::Function(self.get_type_id(&i.type_)),
        )
      })
      .clone();
    let start_func = self.import_section.len();
    let no_funcs = (self.import_section.len() + self.function_section.len()) as u32;
    Module::new()
      // Type section
      .section(&self.make_type_section())
      // Import section
      .section(&import_section)
      // Function section
      .section(
        &*self
          .function_section
          .into_iter()
          .fold(&mut FunctionSection::new(), |f, t| f.function(t)),
      )
      // Table section
      .section(TableSection::new().table(TableType {
        element_type: RefType::FUNCREF,
        table64: false,
        minimum: no_funcs as u64,
        maximum: Some(no_funcs as u64),
        shared: false,
      }))
      // Start section
      .section(&StartSection {
        function_index: start_func as u32,
      })
      // Elements
      .section(ElementSection::new().segment(ElementSegment {
        mode: ElementMode::Active {
          table: None,
          offset: &ConstExpr::i32_const(0),
        },
        elements: Elements::Functions(std::borrow::Cow::from(
          &(0..no_funcs).into_iter().collect::<Vec<_>>(),
        )),
      }))
      // Code section
      .section(
        &*self
          .code_section
          .into_iter()
          .fold(&mut CodeSection::new(), |s, c| s.function(&c.encode())),
      )
      // Finalize
      .clone()
      .finish()
  }

  pub fn make_type_section(&self) -> TypeSection {
    let mut ts = TypeSection::new();
    for t in &self.type_section {
      match t {
        RegisteredType::Function(func_type) => ts.ty().func_type(func_type),
        RegisteredType::Array(storage_type) => ts.ty().array(storage_type, true),
        RegisteredType::Struct(storage_types) => {
          ts.ty()
            .struct_(storage_types.into_iter().map(|t| FieldType {
              element_type: *t,
              mutable: true,
            }))
        }
      }
    }
    ts
  }

  pub fn get_type_id(&mut self, t: &Type) -> u32 {
    match self.get_type(t) {
      StorageType::Val(ValType::Ref(RefType {
        heap_type: HeapType::Concrete(id),
        ..
      })) => id,
      _ => panic!(),
    }
  }

  pub fn get_type(&mut self, t: &Type) -> StorageType {
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
          self,
          t.clone(),
          match p {
            Primitive::nothing => RegisteredType::Struct(vec![]),
            Primitive::integer => RegisteredType::Struct(vec![StorageType::Val(ValType::I64)]),
            Primitive::real => RegisteredType::Struct(vec![StorageType::Val(ValType::F64)]),
            Primitive::boolean => return StorageType::I8,
            Primitive::string => RegisteredType::Array(StorageType::I8),
            Primitive::glyph => RegisteredType::Struct(vec![StorageType::Val(ValType::I32)]),
          },
        );
      }
      Type::Struct { member_types, .. } => {
        RegisteredType::Struct(member_types.into_iter().map(|t| self.get_type(t)).collect())
      }
      Type::Function {
        param_types,
        return_type,
      } => RegisteredType::Function(FuncType::new(
        param_types
          .into_iter()
          .map(|t| self.get_type(t))
          .map(|t| storage_to_valtype(t))
          .collect::<Vec<_>>(),
        [storage_to_valtype(self.get_type(return_type))],
      )),
      Type::Product(items) => {
        RegisteredType::Struct(items.into_iter().map(|t| self.get_type(t)).collect())
      }
      Type::Sum(_) => todo!(),
      Type::Type => todo!(),
    };
    register(self, t.clone(), rt)
  }
}
