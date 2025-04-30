use super::*;

pub fn build_mlir(hlir: &HlIrModule) -> MlIrModule {
  let mut ir = vec![];
  let mut source_map = HashMap::new();
  lower(hlir, 0, &mut ir, &mut source_map);
  MlIrModule { ir, source_map }
}

fn lower(
  hlir: &HlIrModule,
  ptr: IrPtr,
  instrs: &mut Vec<MlIrNode>,
  map: &mut HashMap<IrPtr, MlIrSpan>,
) {
  let node = &hlir.nodes[ptr];
  let new = |kind| MlIrNode {
    span: node.span,
    kind,
  };
  let start_span = instrs.len();
  use HlIrKind::*;
  use MlIrKind as m;
  match &node.kind {
    Declaration {
      assignee,
      is_constant,
      value,
    } => {
      lower(hlir, *value, instrs, map);
      instrs.push(new(m::Set(assignee.clone())));
    },
    Immediate(const_value) => instrs.push(new(m::Const(const_value.clone()))),
    Block(items) => {
      for item in items {
        lower(hlir, *item, instrs, map);
      }
    },
    Identifier(mangle) => instrs.push(new(m::Get(mangle.clone()))),
    Tuple(items) => {
      for item in items {
        lower(hlir, *item, instrs, map);
      }
      instrs.push(new(m::Tuple(items.len())));
    },
    StructDef {
      field_names,
      field_types,
    } => {
      for field in field_types {
        lower(hlir, *field, instrs, map);
      }
      instrs.push(new(m::StructDef(field_names.clone())));
    },
    StructLiteral {
      struct_t,
      field_names,
      field_values,
    } => {
      for field in field_values {
        lower(hlir, *field, instrs, map);
      }
      instrs.push(new(m::StructLiteral(field_names.clone())));
    },
    Field { of, index } => {
      lower(hlir, *of, instrs, map);
      instrs.push(new(m::Field(index.clone())));
    },
    Binary {
      op,
      opdef,
      left,
      right,
    } => {
      lower(hlir, *left, instrs, map);
      lower(hlir, *right, instrs, map);
      instrs.push(new(m::BinaryOp(*op)))
    },
    Unary { op, opdef, child } => {
      lower(hlir, *child, instrs, map);
      instrs.push(new(m::UnaryOp(*op)));
    },
    FunctionDef {
      name,
      parameter_names,
      parameter_spans,
      body,
    } => {
      instrs.push(new(m::Function(name.clone())));
      for name in parameter_names {
        instrs.push(new(m::Set(name.clone())));
      }
      lower(hlir, *body, instrs, map);
      instrs.push(new(m::Return));
    },
    FunctionCall {
      callee,
      callee_name,
      arguments,
    } => {
      lower(hlir, *callee, instrs, map);
      instrs.push(new(m::Set(callee_name.clone())));
      for arg in arguments {
        lower(hlir, *arg, instrs, map);
      }
      instrs.push(new(m::Get(callee_name.clone())));
      instrs.push(new(m::Call(arguments.len())));
    },
    If {
      predicate,
      then,
      else_,
    } => {
      lower(hlir, *predicate, instrs, map);
      instrs.push(new(m::If));
      lower(hlir, *then, instrs, map);
      instrs.push(new(m::Else));
      if let Some(else_) = else_ {
        lower(hlir, *else_, instrs, map);
      }
      instrs.push(new(m::End));
    },
    Match {
      on,
      patterns,
      branches,
    } => todo!(),
    Loop {
      parameter_names,
      parameter_values,
      parameter_spans,
      body,
    } => todo!(),
    Break(_) => todo!(),
  }
  let end_span = instrs.len();
  map.insert(ptr, MlIrSpan(start_span, end_span - start_span));
}
