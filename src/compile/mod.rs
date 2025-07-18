mod function_encoder;
mod lower;
mod runtime;

use std::collections::HashMap;
use wasm_encoder::*;

use crate::{hlir::*, operator::*};

use function_encoder::*;
use runtime::*;

#[derive(Debug, Clone)]
struct Import {
  major: String,
  minor: String,
  entity: EntityType,
}

#[derive(Debug, Clone, Copy)]
enum FunctionKind {
  Native(u32),
  Import(u32),
}

pub fn compile(mut hlir: HlIrModule) -> Vec<u8> {
  let mut state = ModuleEncoder::new();
  state.new_import("sys", "print_integer", FuncType::new([ValType::I32], []));
  state.binary_operator_map = make_binary_operators(&mut state);
  state.unary_operator_map = make_unary_ops(&mut state);
  state.builtin_map = make_builtins(&mut state);
  let main = state.new_main_function();
  lower::lower(&mut hlir, 0, &mut state, main);
  state.func(main).push(Instruction::Drop);
  state.encode(main)
}

#[derive(Debug, Clone)]
enum RegisteredType {
  Function(FuncType),
  Array(StorageType),
  Struct(Vec<StorageType>),
}

#[derive(Debug, Clone, Default)]
struct ModuleEncoder {
  type_map: HashMap<Type, u32>,
  raw_type_map: HashMap<Type, u32>,
  import_section: Vec<Import>,
  type_section: Vec<RegisteredType>,
  function_section: Vec<u32>,
  elements_section: Vec<FunctionKind>,
  code_section: Vec<FunctionEncoder>,
  builtin_map: HashMap<Mangle, (Type, u32)>,
  binary_operator_map: HashMap<BinaryOp, u32>,
  unary_operator_map: HashMap<UnaryOp, u32>,
}

impl ModuleEncoder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_import(
    &mut self,
    major: impl Into<String>,
    minor: impl Into<String>,
    ft: FuncType,
  ) -> u32 {
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
    element_id
  }

  pub fn get_unary_operator(&self, op: UnaryOp) -> u32 {
    self.unary_operator_map.get(&op).unwrap().clone()
  }

  pub fn get_binary_operator(&self, op: BinaryOp) -> u32 {
    self.binary_operator_map.get(&op).unwrap().clone()
  }

  pub fn get_symbol(&mut self, current_function: u32, mangle: &Mangle) {
    if self.func(current_function).has_local(mangle) {
      self.func(current_function).get_local(mangle);
    } else {
      let (type_, func) = self.builtin_map.get(mangle).unwrap().clone();
      self
        .func(current_function)
        .push(Instruction::I32Const(func as i32));
      let closure_t = self.get_type_id(&Type::_ClosureCapture, false);
      self
        .func(current_function)
        .push(Instruction::ArrayNewFixed {
          array_type_index: closure_t,
          array_size: 0,
        });
      let builtin_t = self.get_type_id(&type_, false);
      self
        .func(current_function)
        .push(Instruction::StructNew(builtin_t));
    }
  }

  pub fn func(&mut self, index: u32) -> &mut FunctionEncoder {
    let FunctionKind::Native(index) = self.elements_section[index as usize] else {
      panic!()
    };
    &mut self.code_section[index as usize]
  }

  pub fn encode(self, main_func: u32) -> Vec<u8> {
    let FunctionKind::Native(main_func) = self.elements_section[main_func as usize] else {
      panic!()
    };
    let mut name_section = NameSection::new();
    name_section.types(
      &self
        .type_map
        .clone()
        .into_iter()
        .map(|(type_, id)| (id, format!("{type_}")))
        .chain(
          self
            .raw_type_map
            .clone()
            .into_iter()
            .map(|(type_, id)| (id, format!("(raw) {type_}"))),
        )
        .fold(NameMap::new(), |mut names, (id, type_)| {
          names.append(id, &type_);
          names
        }),
    );
    name_section.locals(
      &self
        .code_section
        .clone()
        .into_iter()
        .enumerate()
        .map(|(id, code)| (id, code.encode_name_map()))
        .fold(IndirectNameMap::new(), |mut indirect_map, (id, map)| {
          indirect_map.append(id as u32, &map);
          indirect_map
        }),
    );

    let no_imports = self.import_section.len() as u32;
    let no_funcs = self.function_section.len() as u32;

    let elements_section = ElementSection::new()
      .segment(ElementSegment {
        mode: ElementMode::Active {
          table: None,
          offset: &ConstExpr::i32_const(0),
        },
        elements: Elements::Functions(std::borrow::Cow::from(
          &self
            .elements_section
            .clone()
            .into_iter()
            .map(|e| match e {
              FunctionKind::Native(f) => f + no_imports,
              FunctionKind::Import(f) => f,
            })
            .collect::<Vec<_>>(),
        )),
      })
      .clone();

    Module::new()
      .section(&name_section)
      // Type section
      .section(&self.make_type_section())
      // Import section
      .section(
        &*self
          .import_section
          .clone()
          .into_iter()
          .fold(&mut ImportSection::new(), |section, import| {
            section.import(&import.major, &import.minor, import.entity)
          }),
      )
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
        minimum: (no_funcs + no_imports) as u64,
        maximum: Some((no_funcs + no_imports) as u64),
        shared: false,
      }))
      // Start section
      .section(&StartSection {
        function_index: (main_func + no_imports),
      })
      // Elements
      .section(&elements_section)
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
              mutable: false,
            }))
        }
      }
    }
    ts
  }

  pub fn get_type_id(&mut self, t: &Type, raw: bool) -> u32 {
    match self.get_storage_type(t, raw) {
      StorageType::Val(ValType::Ref(RefType {
        heap_type: HeapType::Concrete(id),
        ..
      })) => id,
      _ => panic!(),
    }
  }

  pub fn get_valtype(&mut self, t: &Type, raw: bool) -> ValType {
    match self.get_storage_type(t, raw) {
      StorageType::I8 | StorageType::I16 => ValType::I32,
      StorageType::Val(val_type) => val_type,
    }
  }

  pub fn get_storage_type(&mut self, t: &Type, raw: bool) -> StorageType {
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

    if let Some(t) = if raw {
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
    let rt = match t {
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
          .map(|t| self.get_storage_type(t, false))
          .collect(),
      ),
      Type::Function(_, _) if !raw => {
        //let raw_func_type = self.get_storage_type(t, true);
        let raw_func_type = StorageType::Val(ValType::I32);
        let capture_type = self.get_storage_type(&Type::_ClosureCapture, false);
        RegisteredType::Struct(vec![raw_func_type, capture_type])
      }
      Type::Function(a, b) => RegisteredType::Function(FuncType::new(
        [
          self.get_valtype(a, false),
          self.get_valtype(&Type::_ClosureCapture, false),
        ],
        [self.get_valtype(b, false)],
      )),
      Type::Product(items) => RegisteredType::Struct(
        items
          .into_iter()
          .map(|t| self.get_storage_type(t, false))
          .collect(),
      ),
      Type::Sum(_) => todo!(),
      Type::Type => todo!(),
    };
    register(self, t.clone(), rt)
  }
}
