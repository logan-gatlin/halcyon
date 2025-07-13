use super::*;

use Instruction as i;

fn unary_op(
  state: &mut ModuleState,
  type_: Type,
  instructions: Vec<Instruction<'static>>,
) -> u32 {
  let f = state.make_function(
    &Type::Function {
      param_types: vec![type_.clone()],
      return_type: type_.clone().into(),
    },
    ["a".into()],
  );
  let tid = state.get_type_id(&type_);
  instructions
    .into_iter()
    .for_each(|i| state.func(f).instr(i));
  f
}

pub fn make_unary_operators(state: &mut ModuleState) -> HashMap<UnaryOp, u32> {
  use UnaryOp::*;
  let mut op_map = HashMap::new();
  [(
    Minus,
    Type::Integer,
    vec![i::I64Const(0), i::LocalGet(0), i::I64Sub],
  )]
  .into_iter()
  .for_each(|(op, t, i)| {
    op_map.insert(op, unary_op(state, t, i));
  });
  op_map
}

fn binary_op(
  state: &mut ModuleState,
  type_: Type,
  op: Instruction<'static>,
) -> u32 {
  let f = state.make_function(
    &Type::Function {
      param_types: vec![type_.clone(), type_.clone()],
      return_type: type_.clone().into(),
    },
    ["a".into(), "b".into()],
  );
  let tid = state.get_type_id(&type_);
  [
    i::LocalGet(0),
    i::StructGet {
      struct_type_index: tid,
      field_index: 0,
    },
    i::LocalGet(1),
    i::StructGet {
      struct_type_index: tid,
      field_index: 0,
    },
    op,
    i::StructNew(tid),
  ]
  .into_iter()
  .for_each(|i| state.func(f).instr(i));
  f
}

pub fn make_binary_operators(
  state: &mut ModuleState,
) -> HashMap<BinaryOp, u32> {
  use BinaryOp::*;
  let mut op_map = HashMap::new();
  [
    // Integer ops
    (Plus, Type::Integer, i::I64Add),
    (Minus, Type::Integer, i::I64Sub),
    (Star, Type::Integer, i::I64Mul),
    (Slash, Type::Integer, i::I64DivS),
    // Real ops
    (PlusDot, Type::Real, i::F64Add),
    (MinusDot, Type::Real, i::F64Sub),
    (StarDot, Type::Real, i::F64Mul),
    (SlashDot, Type::Real, i::F64Div),
    // Boolean ops
    (And, Type::Boolean, i::I32And),
    (Or, Type::Boolean, i::I32Or),
    (Xor, Type::Boolean, i::I32Xor),
  ]
  .into_iter()
  .for_each(|(op, t, i)| {
    op_map.insert(op, binary_op(state, t, i));
  });
  op_map
}
