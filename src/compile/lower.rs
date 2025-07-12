use crate::operator::*;

use super::*;

fn unwrap_const(c: ConstValue, f: u32, state: &mut ModuleState) {
  use Instruction as i;
  match c {
    ConstValue::Nothing => {
      let tid = state.get_type_id(&Primitive::nothing.into());
      state.func(f).instr(i::StructNew(tid));
    }
    ConstValue::Integer(i) => {
      state.func(f).instr(i::I64Const(i));
      let tid = state.get_type_id(&Primitive::integer.into());
      state.func(f).instr(i::StructNew(tid));
    }
    ConstValue::Real(r) => {
      state.func(f).instr(i::F64Const((r).into()));
      let tid = state.get_type_id(&Primitive::real.into());
      state.func(f).instr(i::StructNew(tid));
    }
    ConstValue::Boolean(b) => {
      state.func(f).instr(i::I32Const(b as i32));
      let tid = state.get_type_id(&Primitive::boolean.into());
      state.func(f).instr(i::StructNew(tid));
    }
    ConstValue::String(s) => {
      for b in s.bytes() {
        state.func(f).instr(i::I32Const(b as i32));
      }
      let array_type_index = state.get_type_id(&Primitive::string.promote()) as u32;
      state.func(f).instr(i::ArrayNewFixed {
        array_type_index,
        array_size: s.len() as u32,
      });
    }
    ConstValue::Glyph(g) => {
      state.func(f).instr(i::I32Const(g as i32));
      let tid = state.get_type_id(&Primitive::glyph.into());
      state.func(f).instr(i::StructNew(tid));
    }
    ConstValue::Function { func_index, .. } => {
      state.func(f).instr(i::RefFunc(func_index));
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
      values.into_iter().for_each(|v| unwrap_const(v, f, state));
      state.func(f).instr(i::StructNew(type_id));
    }
    ConstValue::Type(_) => todo!(),
  }
}

pub fn lower(nodes: &mut HlIrModule, ptr: IrPtr, state: &mut ModuleState, f: u32) {
  use Instruction as i;
  let nk = nodes[ptr].kind.clone();
  let this_t = nodes[ptr].type_.clone();
  use HlIrKind as h;
  match nk {
    // Type declarations don't need to be compiled
    h::Declaration {
      is_type: true, in_, ..
    } => {
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      }
    }
    h::Declaration {
      assignee,
      is_type: false,
      value,
      in_,
      ..
    } => {
      let type_ = state.get_type(&nodes[value].type_);
      let local = state.func(f).local(assignee, storage_to_valtype(type_));
      lower(nodes, value, state, f);
      state.func(f).instr(i::LocalSet(local));
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      }
    }
    h::Immediate(const_value) => unwrap_const(const_value, f, state),
    h::Block(items) => {
      if items.len() == 0 {
        panic!();
      }
      for i in 0..(items.len() - 1) {
        lower(nodes, items[i], state, f);
        state.func(f).instr(i::Drop);
      }
      lower(nodes, items[items.len() - 1], state, f);
    }
    h::Identifier(mangle) => {
      if let Some(id) = state.func(f).local_names.get(&mangle).cloned() {
        state.func(f).instr(i::LocalGet(id));
      }
    }
    h::Tuple(items)
    | h::StructLiteral {
      field_values: items,
      ..
    } => {
      items.into_iter().for_each(|i| lower(nodes, i, state, f));
      let tid = state.get_type_id(&this_t);
      state.func(f).instr(i::StructNew(tid));
    }
    h::Field { of, index } => {
      lower(nodes, of, state, f);
      let struct_t = &nodes[of].type_;
      let field_id = struct_t.field_index(&index).unwrap();
      let struct_t = state.get_type_id(&struct_t);
      state.func(f).instr(i::StructGet {
        struct_type_index: struct_t,
        field_index: field_id,
      })
    }
    h::Binary { op, left, right } => {
      lower(nodes, left, state, f);
      lower(nodes, right, state, f);
      todo!()
    }
    h::Unary { op, child } => {
      lower(nodes, child, state, f);
      let child_t = &nodes[child].type_;
      let f = state.func(f);
      todo!()
    }
    h::If {
      predicate,
      then,
      else_,
    } => {
      lower(nodes, predicate, state, f);
      let block_type = BlockType::Result(storage_to_valtype(state.get_type(&this_t)));
      state.func(f).instr(i::If(block_type));
      lower(nodes, then, state, f);
      if let Some(else_) = else_ {
        state.func(f).instr(i::Else);
        lower(nodes, else_, state, f);
      }
      state.func(f).instr(i::End);
    }
    h::FunctionDef {
      parameter_names,
      body,
      ..
    } => {
      let new_func = state.make_function(&this_t, parameter_names);
      lower(nodes, body, state, new_func);
      state.func(f).instr(i::RefFunc(new_func));
    }
    h::FunctionCall {
      callee, arguments, ..
    } => {
      let type_ = storage_to_valtype(state.get_type(&nodes[callee].type_));
      let type_id = state.get_type_id(&nodes[callee].type_);
      let temp = state.func(f).temporary(type_);
      lower(nodes, callee, state, f);
      state.func(f).instr(i::LocalSet(temp));
      arguments
        .into_iter()
        .for_each(|a| lower(nodes, a, state, f));
      state.func(f).instr(i::LocalGet(temp));
      state.func(f).instr(i::CallRef(type_id));
    }
    h::StructDef { .. } => todo!(),
  }
}
