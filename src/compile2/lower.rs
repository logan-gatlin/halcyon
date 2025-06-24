use crate::operator::*;

use super::*;

fn unwrap_const(c: ConstValue, instr: &mut Vec<Instruction>) {
  use Instruction as i;
  match c {
    ConstValue::Nothing => {},
    ConstValue::Never => instr.push(i::Unreachable),
    ConstValue::Integer(i) => instr.push(i::I64Const(i)),
    ConstValue::Real(r) => instr.push(i::F64Const((r).into())),
    ConstValue::Boolean(b) => instr.push(i::I32Const(b as i32)),
    ConstValue::String { address, length } => {
      instr.push(i::I32Const(address as i32));
      instr.push(i::I32Const(length as i32));
    },
    ConstValue::Glyph(g) => instr.push(i::I32Const(g as i32)),
    ConstValue::Function(id) => instr.push(i::I32Const(id as i32)),
    ConstValue::Tuple(values)
    | ConstValue::StructLiteral {
      member_values: values,
      ..
    } => {
      values.into_iter().for_each(|v| unwrap_const(v, instr));
    },
    ConstValue::Type(_) => todo!(),
  }
}

#[derive(Debug, Clone)]
pub struct Context {
  nodes: Vec<HlIrNode>,
  func_index: HashMap<Mangle, u32>,
  type_index: Vec<wasm_encoder::FuncType>,
}

impl Context {
  pub fn new(module: HlIrModule) -> Self {
    let nodes = module.nodes;
    todo!()
  }
}

pub fn lower(module: &mut Context, ptr: IrPtr, func: &mut Function) {
  let node = module.nodes[ptr].clone();
  use HlIrKind::*;
  use Instruction as i;
  match &node.kind {
    Declaration {
      assignee,
      is_constant,
      value,
      in_,
    } => todo!(),
    Immediate(value) => {
      let mut temp = vec![];
      unwrap_const(value.clone(), &mut temp);
      temp.into_iter().for_each(|i| func.instr(i));
    },
    StructLiteral {
      field_values: items,
      ..
    }
    | Tuple(items)
    | Block(items) => items.into_iter().for_each(|p| lower(module, *p, func)),
    Identifier(_) => todo!(),
    StructDef {
      field_names,
      field_types,
    } => todo!(),
    Field { of, index } => todo!(),
    Binary { op, left, right } => {
      let left_t = &module.nodes[*left].type_;
      let right_t = &module.nodes[*right].type_;
      use BinaryOp::*;
      use Primitive::*;
      match (op, left_t, right_t) {
        (op, Type::Primitive(p1), Type::Primitive(p2)) => match (op, p1, p2) {
          (Plus, integer, integer) => func.instr(i::I64Add),
          (Plus, real, real) => func.instr(i::F64Add),
          (Minus, integer, integer) => func.instr(i::I64Sub),
          (Minus, real, real) => func.instr(i::F64Sub),
          (Star, integer, integer) => func.instr(i::I64Mul),
          (Star, real, real) => func.instr(i::F64Mul),
          (Slash, integer, integer) => func.instr(i::I64DivS),
          (Slash, real, real) => func.instr(i::F64Div),
          (Percent, integer, integer) => func.instr(i::I64RemS),
          _ => panic!(),
        },
        _ => panic!(),
      }
    },
    Unary { op, child } => todo!(),
    FunctionDef {
      name,
      parameter_names,
      parameter_spans,
      body,
    } => todo!(),
    FunctionCall {
      callee,
      callee_name,
      arguments,
    } => todo!(),
    If {
      predicate,
      then,
      else_,
    } => todo!(),
  }
}
