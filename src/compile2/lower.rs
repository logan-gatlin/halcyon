use crate::operator::*;

use super::*;

fn unwrap_const(
  c: ConstValue,
  func: &mut FunctionEncoder,
  state: &mut ModuleState,
) {
  use Instruction as i;
  match c {
    ConstValue::Nothing => {},
    ConstValue::Never => func.instr(i::Unreachable),
    ConstValue::Integer(i) => func.instr(i::I64Const(i)),
    ConstValue::Real(r) => func.instr(i::F64Const((r).into())),
    ConstValue::Boolean(b) => func.instr(i::I32Const(b as i32)),
    ConstValue::String { address, length } => {
      func.instr(i::I32Const(address as i32));
      func.instr(i::I32Const(length as i32));
      let string_t = state
        .get_type_id(&Type::Primitive(Primitive::string))
        .unwrap();
      func.instr(i::StructNew(string_t))
    },
    ConstValue::Glyph(g) => func.instr(i::I32Const(g as i32)),
    ConstValue::Function {
      func_index,
      type_index,
    } => {
      func.instr(i::RefFunc(func_index));
    },
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
        .for_each(|v| unwrap_const(v, func, state));
      func.instr(i::StructNew(type_id));
    },
    ConstValue::Type(_) => todo!(),
  }
}

pub fn lower(
  nodes: &[HlIrNode],
  ptr: IrPtr,
  state: &mut ModuleState,
  f: &mut FunctionEncoder,
) {
  use Instruction as i;
  let nk = nodes[ptr].kind.clone();
  let this_t = nodes[ptr].type_.clone();
  use HlIrKind::*;
  match nk {
    Declaration {
      assignee,
      is_constant,
      value,
      in_,
    } => {
      if let Some(type_) = state.get_type(&nodes[value].type_) {
        let local = f.local(assignee, storage_to_valtype(type_));
        lower(nodes, value, state, f);
        f.instr(i::LocalSet(local));
      }
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      }
    },
    Immediate(const_value) => unwrap_const(const_value, f, state),
    Block(items) => {
      if items.len() == 0 {
        panic!();
      }
      for i in 0..(items.len() - 1) {
        lower(nodes, items[i], state, f);
        f.instr(i::Drop);
      }
      lower(nodes, items[items.len() - 1], state, f);
    },
    Identifier(mangle) => {
      if let Some(id) = f.local_names.get(&mangle) {
        f.instr(i::LocalGet(*id));
      }
    },
    Tuple(items)
    | StructLiteral {
      field_values: items,
      ..
    } => {
      items.into_iter().for_each(|i| lower(nodes, i, state, f));
      let tid = state.get_type_id(&this_t).unwrap();
      f.instr(i::StructNew(tid));
    },
    Field { of, index } => {
      lower(nodes, of, state, f);
      let struct_t = &nodes[of].type_;
      let struct_t = state.get_type_id(&struct_t).unwrap();
      f.instr(i::StructGet {
        struct_type_index: struct_t,
        field_index: 0,
      })
    },
    Binary { op, left, right } => {
      lower(nodes, left, state, f);
      lower(nodes, right, state, f);
      let left_t = &nodes[left].type_;
      let right_t = &nodes[right].type_;
      use BinaryOp::*;
      use Primitive::*;
      match (op, left_t, right_t) {
        (op, Type::Primitive(p1), Type::Primitive(p2)) => match (op, p1, p2) {
          // Arithmetic
          (Plus, integer, integer) => f.instr(i::I64Add),
          (Plus, real, real) => f.instr(i::F64Add),
          (Minus, integer, integer) => f.instr(i::I64Sub),
          (Minus, real, real) => f.instr(i::F64Sub),
          (Star, integer, integer) => f.instr(i::I64Mul),
          (Star, real, real) => f.instr(i::F64Mul),
          (Slash, integer, integer) => f.instr(i::I64DivS),
          (Slash, real, real) => f.instr(i::F64Div),
          (Percent, integer, integer) => f.instr(i::I64RemS),
          (And, integer, integer) => f.instr(i::I64And),
          (And, boolean, boolean) => f.instr(i::I32And),
          (Or, integer, integer) => f.instr(i::I64Or),
          (Or, boolean, boolean) => f.instr(i::I32Or),
          (Xor, integer, integer) => f.instr(i::I64Xor),
          (Xor, boolean, boolean) => {
            f.instr(i::I32Xor);
            f.instr(i::I32Const(0b1));
            f.instr(i::I32And);
          },
          // TODO other logical ops

          // Comparisons
          (DoubleEqual, integer, integer) => f.instr(i::I64Eq),
          (DoubleEqual, real, real) => f.instr(i::F64Eq),
          (DoubleEqual, glyph, glyph) | (DoubleEqual, boolean, boolean) => {
            f.instr(i::I32Eq)
          },
          (DoubleEqual, nothing, nothing) => f.instr(i::I32Const(1)),

          (BangEqual, integer, integer) => f.instr(i::I64Ne),
          (BangEqual, real, real) => f.instr(i::F64Ne),
          (BangEqual, glyph, glyph) | (BangEqual, boolean, boolean) => {
            f.instr(i::I32Ne)
          },
          (BangEqual, nothing, nothing) => f.instr(i::I32Const(0)),

          (Less, integer, integer) => f.instr(i::I64LtS),
          (Less, real, real) => f.instr(i::F64Lt),
          (Less, glyph, glyph) | (Less, boolean, boolean) => f.instr(i::I32LtU),

          (LessEqual, integer, integer) => f.instr(i::I64LeS),
          (LessEqual, real, real) => f.instr(i::F64Le),
          (LessEqual, glyph, glyph) | (LessEqual, boolean, boolean) => {
            f.instr(i::I32LeU)
          },

          (Greater, integer, integer) => f.instr(i::I64GtS),
          (Greater, real, real) => f.instr(i::F64Gt),
          (Greater, glyph, glyph) | (Greater, boolean, boolean) => {
            f.instr(i::I32GtU)
          },

          (GreaterEqual, integer, integer) => f.instr(i::I64GeS),
          (GreaterEqual, real, real) => f.instr(i::F64Ge),
          (GreaterEqual, glyph, glyph) | (GreaterEqual, boolean, boolean) => {
            f.instr(i::I32GeU)
          },
          // TODO string ops
          _ => panic!(),
        },
        _ => panic!(),
      }
    },
    Unary { op, child } => {
      lower(nodes, child, state, f);
      let child_t = &nodes[child].type_;
      use Primitive::*;
      use UnaryOp::*;
      match (op, child_t) {
        (op, Type::Primitive(p)) => match (op, p) {
          (Minus, integer) => {
            f.instr(i::I64Const(-1));
            f.instr(i::I64Mul);
          },
          (Minus, real) => {
            f.instr(i::F64Const(Ieee64::from(-1.0)));
            f.instr(i::F64Mul);
          },
          (Not, integer) => {
            f.instr(i::I64Const(-1));
            f.instr(i::I64Xor);
          },
          (Not, boolean) => {
            f.instr(i::I32Const(1));
            f.instr(i::I32Xor)
          },
          _ => panic!(),
        },
        _ => panic!(),
      }
    },
    If {
      predicate,
      then,
      else_,
    } => {
      lower(nodes, predicate, state, f);
      f.instr(i::If(
        match state.get_type(&this_t).map(storage_to_valtype) {
          Some(v) => BlockType::Result(v),
          None => BlockType::Empty,
        },
      ));
      lower(nodes, then, state, f);
      if let Some(else_) = else_ {
        f.instr(i::Else);
        lower(nodes, else_, state, f);
      }
      f.instr(i::End);
    },
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
    StructDef {
      field_names,
      field_types,
    } => todo!(),
  }
}
