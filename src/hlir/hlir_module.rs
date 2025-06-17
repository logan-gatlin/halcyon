use super::*;

#[derive(Debug, Clone)]
pub enum PatternKind {
  Const(ConstValue),
  Wildcard(String),
  Tuple(Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub struct Pattern {
  pub kind: PatternKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HlIrKind {
  Declaration {
    assignee: Mangle,
    is_constant: bool,
    value: IrPtr,
    in_: Option<IrPtr>,
  },
  Immediate(ConstValue),
  Block(Vec<IrPtr>),
  Identifier(Mangle),
  Tuple(Vec<IrPtr>),
  StructDef {
    field_names: Vec<String>,
    field_types: Vec<IrPtr>,
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
    parameter_spans: Vec<Span>,
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
  pub heap: Memory,
}

impl HlIrModule {
  pub fn get_node(&self, node: IrPtr) -> &HlIrNode {
    &self.nodes[node]
  }

  pub fn type_of(&self, node: IrPtr) -> Type {
    self.get_node(node).type_.clone()
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
