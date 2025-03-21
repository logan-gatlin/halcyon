use super::*;

#[derive(Debug, Clone)]
pub enum HlIrKind {
  Declaration {
    assignee: Mangle,
    is_constant: bool,
    type_assert: Option<IrPtr>,
    value: IrPtr,
  },
  Immediate(ConstValue),
  Block(Vec<IrPtr>),
  Identifier(Mangle),
  StructDef {
    fields: Vec<String>,
    types: Vec<IrPtr>,
  },
  StructLiteral {
    struct_t: Option<(IrPtr, Mangle)>,
    field_names: Vec<String>,
    field_values: Vec<IrPtr>,
  },
  Field {
    of: IrPtr,
    index: String,
  },
  Binary {
    op: BinaryOp,
    opdef: OpDef,
    left: IrPtr,
    right: IrPtr,
  },
  Unary {
    op: UnaryOp,
    opdef: OpDef,
    child: IrPtr,
  },
  FunctionDef {
    name: Mangle,
    parameter_names: Vec<Mangle>,
    parameter_types: Vec<IrPtr>,
    returns: Option<(IrPtr, Mangle)>,
    body: IrPtr,
  },
  FunctionCall {
    callee: IrPtr,
    callee_name: Mangle,
    arguments: Vec<IrPtr>,
  },
  If {
    predicate: IrPtr,
    then: IrPtr,
    else_: Option<IrPtr>,
  },
  Loop {
    parameter_names: Vec<Mangle>,
    parameter_values: Vec<IrPtr>,
    body: IrPtr,
  },
  Break(Option<IrPtr>),
}

#[derive(Debug, Clone)]
pub struct HlIrNode {
  pub kind: HlIrKind,
  pub span: Span,
  pub type_: Type,
}

#[derive(Debug, Clone)]
pub struct HlIrModule {
  pub nodes: Vec<HlIrNode>,
  pub constants: HashMap<Mangle, IrPtr>,
  pub type_map: HashMap<Mangle, Type>,
  pub heap: Vec<Vec<u8>>,
  pub main: Option<Mangle>,
}

impl HlIrModule {
  pub fn type_of(&self, node: IrPtr) -> Type {
    self.nodes[node].type_.clone()
  }

  pub fn value_span(&self, node: IrPtr) -> Span {
    let mut n = &self.nodes[node];
    loop {
      match &n.kind {
        HlIrKind::Block(vec) => {
          if let Some(last) = vec.last() {
            n = &self.nodes[*last];
          } else {
            return n.span;
          }
        }
        HlIrKind::If { then, .. } => n = &self.nodes[*then],
        _ => return n.span,
      }
    }
  }
}
