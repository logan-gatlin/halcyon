use super::*;

use Instruction as i;

fn unary_op(
  state: &mut ModuleEncoder,
  type_: Type,
  instructions: Vec<Instruction<'static>>,
) -> u32 {
  todo!()
}

pub fn make_unary_operators(
  state: &mut ModuleEncoder,
) -> HashMap<UnaryOp, u32> {
  use UnaryOp::*;
  todo!()
}

fn binary_op(
  state: &mut ModuleEncoder,
  type_: Type,
  op: Instruction<'static>,
) -> u32 {
  let param_type = type_.clone() * type_.clone();
  let f = state.new_function(
    &Type::Function(param_type.clone().into(), type_.clone().into()),
    "a".to_string(),
    vec![],
    vec![],
  );
  let param_type_id = state.get_type_id(&param_type, false);
  let return_type_id = state.get_type_id(&type_, false);
  state.func(f).extend(&[
    i::LocalGet(0),
    i::StructGet {
      struct_type_index: param_type_id,
      field_index: 0,
    },
    i::StructGet {
      struct_type_index: return_type_id,
      field_index: 0,
    },
    i::LocalGet(0),
    i::StructGet {
      struct_type_index: param_type_id,
      field_index: 1,
    },
    i::StructGet {
      struct_type_index: return_type_id,
      field_index: 0,
    },
    op,
    i::StructNew(return_type_id),
  ]);
  f
}

pub fn make_binary_operators(
  state: &mut ModuleEncoder,
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
    /*
     */
  ]
  .into_iter()
  .for_each(|(op, t, i)| {
    op_map.insert(op, binary_op(state, t, i));
  });
  op_map
}
