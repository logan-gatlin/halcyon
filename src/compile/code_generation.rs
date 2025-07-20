use crate::builtin::Builtin;

use super::*;

#[allow(dead_code)]
impl ModuleEncoder {
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

  pub fn get_unary_operator(&mut self, op: UnaryOp) -> u32 {
    if let Some(func) = self.unary_operator_map.get(&op) {
      *func
    } else {
      let func = self.generate_unary_operator(op);
      self.unary_operator_map.insert(op, func);
      func
    }
  }

  pub fn get_binary_operator(&mut self, op: BinaryOp) -> u32 {
    if let Some(func) = self.binary_operator_map.get(&op) {
      *func
    } else {
      let func = self.generate_binary_operator(op);
      self.binary_operator_map.insert(op, func);
      func
    }
  }

  pub fn get_symbol(&mut self, current_function: u32, mangle: &Mangle) {
    if self.func(current_function).has_local(mangle) {
      self.func(current_function).get_local(mangle);
    } else {
      let bt = Builtin::from_mangle(mangle).unwrap();
      let func = if let Some(func) = self.builtin_map.get(&bt) {
        *func
      } else {
        let func = self.generate_builtin(bt);
        self.builtin_map.insert(bt, func);
        func
      };
      self.push(current_function, Instruction::I32Const(func as i32));
      self.new_capture(current_function, 0);
      self.new_struct(current_function, &bt.get_type());
    }
  }

  pub fn new_array(&mut self, function: u32, type_: &Type, size: u32) {
    let array_t = self.get_type_id(type_, false);
    self.push(
      function,
      Instruction::ArrayNewFixed {
        array_type_index: array_t,
        array_size: size,
      },
    );
  }

  pub fn new_capture(&mut self, function: u32, size: u32) {
    self.new_array(function, &Type::_ClosureCapture, size);
  }

  pub fn new_struct(&mut self, function: u32, type_: &Type) {
    let type_id = self.get_type_id(type_, false);
    self.push(function, Instruction::StructNew(type_id));
  }

  pub fn func(&mut self, index: u32) -> &mut FunctionEncoder {
    let FunctionKind::Native(index) = self.elements_section[index as usize] else {
      panic!()
    };
    &mut self.code_section[index as usize]
  }

  pub fn call_raw_function(&mut self, function: u32, callee_id: u32, callee_type: &Type) {
    self.push(function, I32Const(callee_id as i32));
    let callee_type = self.get_type_id(callee_type, true);
    self.push(
      function,
      CallIndirect {
        type_index: callee_type,
        table_index: 0,
      },
    );
  }

  pub fn push_constant(&mut self, function: u32, c: ConstValue) {
    match c {
      ConstValue::Unit => {
        let tid = self.get_type_id(&Type::Unit, false);
        self.push(function, StructNew(tid));
      }
      ConstValue::Integer(i) => {
        self.push(function, I64Const(i));
        let tid = self.get_type_id(&Type::Integer, false);
        self.push(function, StructNew(tid));
      }
      ConstValue::Real(r) => {
        self.push(function, F64Const((r).into()));
        let tid = self.get_type_id(&Type::Real, false);
        self.push(function, StructNew(tid));
      }
      ConstValue::Boolean(b) => {
        self.push(function, I32Const(b as i32));
        let tid = self.get_type_id(&Type::Boolean, false);
        self.push(function, StructNew(tid));
      }
      ConstValue::String(s) => {
        for b in s.bytes() {
          self.push(function, I32Const(b as i32));
        }
        let array_type_index = self.get_type_id(&Type::String, false) as u32;
        self.push(
          function,
          ArrayNewFixed {
            array_type_index,
            array_size: s.len() as u32,
          },
        );
      }
      ConstValue::Glyph(g) => {
        self.push(function, I32Const(g as i32));
        let tid = self.get_type_id(&Type::Glyph, false);
        self.push(function, StructNew(tid));
      }
      ConstValue::Tuple {
        members: values,
        type_id,
      }
      | ConstValue::StructLiteral {
        member_values: values,
        type_id,
        ..
      } => {
        values
          .into_iter()
          .for_each(|v| self.push_constant(function, v));
        self.push(function, StructNew(type_id));
      }
      ConstValue::Type(_) => todo!(),
      ConstValue::Function { .. } => todo!(),
    }
  }

  pub fn unwrap_primitive(&mut self, function: u32, type_: &Type) {
    match type_ {
      Type::Integer | Type::Real | Type::Boolean | Type::String | Type::Glyph => {
        let type_id = self.get_type_id(type_, false);
        self.push(
          function,
          StructGet {
            struct_type_index: type_id,
            field_index: 0,
          },
        );
      }
      _ => todo!(),
    }
  }
}
