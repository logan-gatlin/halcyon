use std::collections::HashSet;

use crate::{hlir::*, lint::*, mlir::*};

pub fn typecheck(hl: &mut HlIrModule, ml: &MlIrModule, mangle: &Mangle) {
  let node = *hl.constants.get(mangle).unwrap();
}

fn type_node(hl: &mut HlIrModule, node: IrPtr) -> Result<Type> {
  use HlIrKind::*;
  let type_ = match hl.nodes[node].kind.clone() {
    Declaration {
      assignee,
      is_constant,
      type_assert,
      value,
    } => {
      type_node(hl, value)?;
      Primitive::nothing.promote()
    }
    Immediate(const_value) => const_value.type_of(),
    Block(items) => {
      for item in &items {
        type_node(hl, *item)?;
      }
      if let Some(last) = items.last() {
        hl.nodes[*last].type_.clone()
      } else {
        Primitive::nothing.promote()
      }
    }
    Identifier(mangle) => hl.type_map.get(&mangle).unwrap().clone(),
    StructDef { types, .. } => {
      for t in types {
        type_node(hl, t)?;
      }
      Type::Type
    }
    StructLiteral {
      struct_t,
      field_names,
      field_values,
    } => todo!(),
    Field { of, index } => {
      type_node(hl, of)?;
      todo!()
    }
    Binary {
      op,
      opdef,
      left,
      right,
    } => todo!(),
    Unary { op, opdef, child } => todo!(),
    FunctionDef {
      name,
      parameter_names,
      parameter_types,
      returns,
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
    Loop {
      parameter_names,
      parameter_values,
      body,
    } => todo!(),
    Break(_) => todo!(),
  };
  hl.nodes[node].type_ = type_.clone();
  Ok(type_)
}
