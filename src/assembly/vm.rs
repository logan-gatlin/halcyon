use super::{Wasm, WasmValue};
use crate::{err::*, error};

pub struct VirtualMachine {
  stack: Vec<WasmValue>,
}

// Dumb repetitive code, kinda has to be this way though
impl VirtualMachine {
  pub fn exec(&mut self, instr: Wasm) -> Result<()> {
    use WasmValue as v;
    let mut pop = || self.stack.pop().reason("Popped an empty stack");
    match instr {
      Wasm::Constant(wasm_value) => self.stack.push(wasm_value),
      Wasm::Add(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l + r),
          (v::I64(l), v::I64(r)) => v::I64(l + r),
          (v::F32(l), v::F32(r)) => v::F32(l + r),
          (v::F64(l), v::F64(r)) => v::F64(l + r),
          _ => return error!("Invalid add operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Subtract(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l - r),
          (v::I64(l), v::I64(r)) => v::I64(l - r),
          (v::F32(l), v::F32(r)) => v::F32(l - r),
          (v::F64(l), v::F64(r)) => v::F64(l - r),
          _ => return error!("Invalid sub operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Multiply(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l * r),
          (v::I64(l), v::I64(r)) => v::I64(l * r),
          (v::F32(l), v::F32(r)) => v::F32(l * r),
          (v::F64(l), v::F64(r)) => v::F64(l * r),
          _ => return error!("Invalid mul operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Divide(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l / r),
          (v::I64(l), v::I64(r)) => v::I64(l / r),
          (v::F32(l), v::F32(r)) => v::F32(l / r),
          (v::F64(l), v::F64(r)) => v::F64(l / r),
          _ => return error!("Invalid div operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Remainder(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l % r),
          (v::I64(l), v::I64(r)) => v::I64(l % r),
          (v::F32(l), v::F32(r)) => v::F32(l % r),
          (v::F64(l), v::F64(r)) => v::F64(l % r),
          _ => return error!("Invalid rem operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::And(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l & r),
          (v::I64(l), v::I64(r)) => v::I64(l & r),
          _ => return error!("Invalid and operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Or(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l | r),
          (v::I64(l), v::I64(r)) => v::I64(l | r),
          _ => return error!("Invalid or operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Xor(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(l ^ r),
          (v::I64(l), v::I64(r)) => v::I64(l ^ r),
          _ => return error!("Invalid xor operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Equal(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l == r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l == r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l == r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l == r) as i32),
          _ => return error!("Invalid eq operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Unequal(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l != r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l != r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l != r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l != r) as i32),
          _ => return error!("Invalid neq operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::GreaterSigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l > r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l > r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l > r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l > r) as i32),
          _ => return error!("Invalid gt_s operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::GreaterUnsigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((*l as u32 > *r as u32) as i32),
          (v::I64(l), v::I64(r)) => v::I32((*l as u32 > *r as u32) as i32),
          _ => return error!("Invalid gt_u operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::LesserSigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l < r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l < r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l < r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l < r) as i32),
          _ => return error!("Invalid lt_s operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::LesserUnsigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32(((*l as u32) < (*r as u32)) as i32),
          (v::I64(l), v::I64(r)) => v::I32(((*l as u32) < (*r as u32)) as i32),
          _ => return error!("Invalid lt_u operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::GreaterEqualSigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l >= r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l >= r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l >= r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l >= r) as i32),
          _ => return error!("Invalid ge_s operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::GreaterEqualUnsigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((*l as u32 >= *r as u32) as i32),
          (v::I64(l), v::I64(r)) => v::I32((*l as u32 >= *r as u32) as i32),
          _ => return error!("Invalid ge_u operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::LesserEqualSigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((l <= r) as i32),
          (v::I64(l), v::I64(r)) => v::I32((l <= r) as i32),
          (v::F32(l), v::F32(r)) => v::I32((l <= r) as i32),
          (v::F64(l), v::F64(r)) => v::I32((l <= r) as i32),
          _ => return error!("Invalid le_s operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::LesserEqualUnsigned(wasm_type) => {
        let left = pop()?;
        let right = pop()?;
        let result = match (&left, &right) {
          (v::I32(l), v::I32(r)) => v::I32((*l as u32 <= *r as u32) as i32),
          (v::I64(l), v::I64(r)) => v::I32((*l as u32 <= *r as u32) as i32),
          _ => return error!("Invalid le_u operands: {left:?}, {right:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Negate(wasm_type) => {
        let left = pop()?;
        let result = match &left {
          v::I32(l) => v::I32(-l),
          v::I64(l) => v::I64(-l),
          v::F32(l) => v::F32(-l),
          v::F64(l) => v::F64(-l),
          _ => return error!("Invalid neg operand: {left:?}"),
        };
        self.stack.push(result);
      },
      Wasm::Drop => {
        pop()?;
      },
      Wasm::Unreachable => return error!("Encountered unreachable"),
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
      | Wasm::Nop
      | Wasm::Custom(_)
      | Wasm::Memory { .. }
      | Wasm::Data { .. }
      | Wasm::Return
      | Wasm::End
      | Wasm::Comment(_)
      | Wasm::Start(_) => todo!(),
    }
    Ok(())
  }
}
