use crate::{compile::*, hlir::*, lint::*, mlir::*};

pub struct VirtualMachine {
  pub stack: Vec<WasmValue>,
}

impl ConstValue {
  pub fn to_wasm_value(self) -> Vec<WasmValue> {
    use WasmValue as w;
    match self {
      ConstValue::Nothing => vec![],
      ConstValue::Never => vec![],
      ConstValue::Integer(val) => vec![w::I64(val)],
      ConstValue::Real(val) => vec![w::F64(val)],
      ConstValue::Boolean(val) => vec![w::I32(val as i32)],
      ConstValue::String {
        virtual_address: address,
        length,
      } => {
        vec![w::I32(address as i32), w::I32(length as i32)]
      }
      ConstValue::Glyph(val) => vec![w::I32(val as i32)],
      ConstValue::Function { name, .. } => vec![w::FuncRef(name)],
      ConstValue::StructLiteral { member_values, .. } => member_values
        .into_iter()
        .rev()
        .flat_map(|c| c.to_wasm_value())
        .collect(),
      ConstValue::Type(_) => panic!("Type 'Type' has no WASM representation"),
    }
  }
}

// Dumb repetitive code, kinda has to be this way though
impl VirtualMachine {
  pub fn run(initial_stack: Vec<ConstValue>, ops: Vec<Wasm>, expects: Type) -> Result<ConstValue> {
    use WasmValue as w;
    let mut this = Self {
      stack: initial_stack
        .into_iter()
        .flat_map(|c| c.to_wasm_value())
        .collect(),
    };
    for op in ops {
      this.exec(op)?;
    }
    let err = Err(lint_nospan(0_usize));
    match expects {
      Type::Primitive(primitive) => match primitive {
        Primitive::nothing => Ok(ConstValue::Nothing),
        Primitive::unreachable => {
          panic!("Cannot construct never primitive from wasm")
        }
        Primitive::integer => {
          let Some(w::I64(val)) = this.stack.pop() else {
            return err;
          };
          Ok(ConstValue::Integer(val))
        }
        Primitive::real => {
          let Some(w::F64(val)) = this.stack.pop() else {
            return err;
          };
          Ok(ConstValue::Real(val))
        }
        Primitive::boolean => {
          let Some(w::I32(val)) = this.stack.pop() else {
            return err;
          };
          Ok(ConstValue::Boolean(val != 0))
        }
        Primitive::string => {
          let Some(w::I32(len)) = this.stack.pop() else {
            return err;
          };
          let Some(w::I32(ptr)) = this.stack.pop() else {
            return err;
          };
          Ok(ConstValue::String {
            virtual_address: ptr as usize,
            length: len as usize,
          })
        }
        Primitive::glyph => {
          let Some(w::I32(val)) = this.stack.pop() else {
            return err;
          };
          Ok(ConstValue::Glyph(
            char::from_u32(val as u32).lint(EvalLint::GlyphOutOfRange)?,
          ))
        }
      },
      Type::Struct { .. } => todo!(),
      Type::Function { .. } => todo!(),
      Type::Reference(_) => todo!(),
      Type::Ambiguous => todo!(),
      Type::Type => panic!("Type 'Type' has no WASM representation"),
    }
  }

  #[allow(unused_variables)]
  pub fn exec(&mut self, instr: Wasm) -> Result<()> {
    use WasmType as t;
    use WasmValue as v;
    let mut pop = || self.stack.pop().expect("Popped an empty stack");
    // TODO fix ignored op type
    match instr {
      Wasm::Constant(wasm_value) => self.stack.push(wasm_value),
      Wasm::Add(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right, wasm_type) {
          (v::I32(l), v::I32(r), t::I32) => v::I32(l + r),
          (v::I64(l), v::I64(r), t::I64) => v::I64(l + r),
          (v::F32(l), v::F32(r), t::F32) => v::F32(l + r),
          (v::F64(l), v::F64(r), t::F64) => v::F64(l + r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Subtract(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l - r),
          (v::I64(l), v::I64(r)) => v::I64(l - r),
          (v::F32(l), v::F32(r)) => v::F32(l - r),
          (v::F64(l), v::F64(r)) => v::F64(l - r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Multiply(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l * r),
          (v::I64(l), v::I64(r)) => v::I64(l * r),
          (v::F32(l), v::F32(r)) => v::F32(l * r),
          (v::F64(l), v::F64(r)) => v::F64(l * r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Divide(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l / r),
          (v::I64(l), v::I64(r)) => v::I64(l / r),
          (v::F32(l), v::F32(r)) => v::F32(l / r),
          (v::F64(l), v::F64(r)) => v::F64(l / r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Remainder(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l % r),
          (v::I64(l), v::I64(r)) => v::I64(l % r),
          (v::F32(l), v::F32(r)) => v::F32(l % r),
          (v::F64(l), v::F64(r)) => v::F64(l % r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::And(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l & r),
          (v::I64(l), v::I64(r)) => v::I64(l & r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Or(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l | r),
          (v::I64(l), v::I64(r)) => v::I64(l | r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Xor(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l ^ r),
          (v::I64(l), v::I64(r)) => v::I64(l ^ r),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Equal(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l == r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l == r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l == r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l == r) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Unequal(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l != r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l != r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l != r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l != r) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::GreaterSigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l > r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l > r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l > r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l > r) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::GreaterUnsigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((*l as u32 > *r as u32) as i32),
          (v::I64(l), v::I64(r)) => v::I32((*l as u32 > *r as u32) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::LesserSigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l < r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l < r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l < r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l < r) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::LesserUnsigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(((*l as u32) < (*r as u32)) as i32),
          (v::I64(l), v::I64(r)) => v::I32(((*l as u32) < (*r as u32)) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::GreaterEqualSigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l >= r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l >= r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l >= r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l >= r) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::GreaterEqualUnsigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((*l as u32 >= *r as u32) as i32),
          (v::I64(l), v::I64(r)) => v::I32((*l as u32 >= *r as u32) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::LesserEqualSigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l <= r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l <= r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l <= r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l <= r) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::LesserEqualUnsigned(wasm_type) => {
        let right = pop();
        let left = pop();
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((*l as u32 <= *r as u32) as i32),
          (v::I64(l), v::I64(r)) => v::I32((*l as u32 <= *r as u32) as i32),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Negate(wasm_type) => {
        let left = pop();
        let result = match &left {
          v::I32(l) => v::I32(-l),
          v::I64(l) => v::I64(-l),
          v::F32(l) => v::F32(-l),
          v::F64(l) => v::F64(-l),
          _ => unreachable!(),
        };
        self.stack.push(result);
      }
      Wasm::Drop => {
        pop();
      }
      Wasm::Comment(_) | Wasm::Nop => {}
      Wasm::Unreachable => {
        return Err(lint_nospan(EvalLint::Unreachable));
      }
      Wasm::Import { .. }
      | Wasm::Local(_, _)
      | Wasm::LocalSet(_)
      | Wasm::LocalGet(_)
      | Wasm::Function { .. }
      | Wasm::If
      | Wasm::Else
      | Wasm::Loop(_)
      | Wasm::Block(_)
      | Wasm::Branch(_)
      | Wasm::Call(_)
      | Wasm::Custom(_)
      | Wasm::Memory { .. }
      | Wasm::Data { .. }
      | Wasm::Return
      | Wasm::End
      | Wasm::Start(_) => todo!(),
    }
    Ok(())
  }
}
