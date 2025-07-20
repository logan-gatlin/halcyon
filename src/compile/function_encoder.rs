use super::*;

#[derive(Debug, Clone)]
pub struct FunctionEncoder {
  local_names: HashMap<Mangle, u32>,
  parameter: Option<ValType>,
  closure_capture: bool,
  locals: Vec<ValType>,
  instrs: Vec<Instruction<'static>>,
}

impl FunctionEncoder {
  fn local_offset(&self) -> u32 {
    (self.parameter.is_some() as u32) + (self.closure_capture as u32)
  }

  pub fn new_local(&mut self, name: Mangle, type_: ValType) -> u32 {
    let id = self.locals.len() as u32 + self.local_offset();
    self.locals.push(type_);
    self.local_names.insert(name, id);
    id
  }

  pub fn get_local_id(&mut self, mangle: impl Into<String>) -> u32 {
    let mangle: String = mangle.into();
    self.local_names.get(&mangle).unwrap().clone()
  }

  pub fn has_local(&self, mangle: impl Into<String>) -> bool {
    let mangle: String = mangle.into();
    self.local_names.contains_key(&mangle)
  }

  pub fn get_local(&mut self, mangle: impl Into<String>) {
    let mangle: String = mangle.into();
    let local = self.local_names.get(&mangle).unwrap().clone();
    self.push(Instruction::LocalGet(local));
  }

  pub fn new_temporary(&mut self, type_: ValType) -> u32 {
    let id = self.locals.len() as u32 + self.local_offset();
    self.locals.push(type_);
    id
  }

  fn push(&mut self, instr: Instruction<'static>) {
    self.instrs.push(instr);
  }

  pub fn extend(&mut self, other: &[Instruction<'static>]) {
    self.instrs.extend_from_slice(other);
  }

  pub fn encode_name_map(&self) -> NameMap {
    self
      .local_names
      .iter()
      .fold(NameMap::new(), |mut map, (name, id)| {
        map.append(*id, name);
        map
      })
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

impl ModuleEncoder {
  pub fn new_main_function(&mut self) -> u32 {
    self.function_section.push(self.type_section.len() as u32);
    self
      .type_section
      .push(RegisteredType::Function(FuncType::new([], [])));
    let code_id = self.code_section.len() as u32;
    self.code_section.push(FunctionEncoder {
      local_names: HashMap::new(),
      parameter: None,
      closure_capture: false,
      locals: vec![],
      instrs: vec![],
    });
    let element_id = self.elements_section.len() as u32;
    self.elements_section.push(FunctionKind::Native(code_id));
    element_id
  }

  pub fn new_function(
    &mut self,
    type_: &Type,
    parameter_name: Mangle,
    capture_names: Vec<Mangle>,
    capture_types: Vec<Type>,
  ) -> u32 {
    let Type::Function(parameter_type, _) = type_ else {
      panic!()
    };
    let parameter_type = self.get_valtype(parameter_type, false);
    let closure_type_id = self.get_type_id(&Type::_ClosureCapture, false);
    let capture_types = capture_types.iter().map(|t| self.get_valtype(t, false));
    let mut code = FunctionEncoder {
      local_names: [(parameter_name, 0)].into_iter().collect(),
      parameter: Some(parameter_type),
      closure_capture: true,
      locals: vec![],
      instrs: vec![],
    };
    capture_names
      .into_iter()
      .zip(capture_types)
      .enumerate()
      .for_each(|(id, (c, t))| {
        use Instruction as i;
        let local = code.new_local(c, t);
        code.push(i::LocalGet(1));
        code.push(i::I32Const(id as i32));
        code.push(i::ArrayGet(closure_type_id));
        let ValType::Ref(RefType { heap_type, .. }) = t else {
          panic!()
        };
        code.push(i::RefCastNonNull(heap_type));
        code.push(i::LocalSet(local));
      });
    let type_id = self.get_type_id(&type_, true);
    self.function_section.push(type_id);
    let code_id = self.code_section.len() as u32;
    self.code_section.push(code);
    let element_id = self.elements_section.len() as u32;
    self.elements_section.push(FunctionKind::Native(code_id));
    element_id
  }

  pub fn push(&mut self, function: u32, instruction: Instruction<'static>) {
    self.func(function).push(instruction);
  }

  pub fn new_curried_function(
    &mut self,
    parameter_names: Vec<Mangle>,
    parameter_types: Vec<Type>,
    return_type: Type,
    capture_names: Vec<Mangle>,
    capture_types: Vec<Type>,
  ) -> (u32, u32) {
    let mut capture_names_list = vec![];
    let mut capture_types_list = vec![];
    parameter_names
      .clone()
      .into_iter()
      .zip(parameter_types.clone())
      .fold(
        (vec![], vec![]),
        |(mut name_list, mut type_list), (name, type_)| {
          capture_names_list.push(name_list.clone());
          capture_types_list.push(type_list.clone());
          name_list.push(name);
          type_list.push(type_);
          (name_list, type_list)
        },
      );
    capture_names_list
      .iter_mut()
      .for_each(|names| names.extend_from_slice(&capture_names));
    capture_types_list
      .iter_mut()
      .for_each(|types| types.extend(capture_types.clone()));
    let mut tail = 0;
    let head = parameter_names
      .into_iter()
      .zip(parameter_types)
      .zip(capture_names_list)
      .zip(capture_types_list)
      .rev()
      .fold(
        (Option::<u32>::None, return_type),
        |(last_function, return_type),
         (((parameter_name, parameter_type), capture_names), capture_types)| {
          let new_return_type = Type::func(parameter_type, return_type.clone());
          let next_function = self.new_function(
            &new_return_type,
            parameter_name.clone(),
            capture_names.clone(),
            capture_types.clone(),
          );
          if let Some(last_function) = last_function {
            use Instruction as i;
            self
              .func(next_function)
              .push(i::I32Const(last_function as i32));
            self.func(next_function).get_local(&parameter_name);
            for capture in &capture_names {
              self.func(last_function).get_local(capture);
            }
            let capture_array_type = self.get_type_id(&Type::_ClosureCapture, false);
            self.func(next_function).push(i::ArrayNewFixed {
              array_type_index: capture_array_type,
              array_size: (capture_names.len() + 1) as u32,
            });
            let function_type = self.get_type_id(&return_type, false);
            self.func(next_function).push(i::StructNew(function_type));
          } else {
            tail = next_function;
          }
          (Some(next_function), new_return_type)
        },
      );
    (head.0.unwrap(), tail)
  }
}
