mod build_hlir;
pub mod constant;
pub mod printing;
pub mod types;

use crate::{lint::*, operator::*};

pub use build_hlir::*;
pub use constant::*;
pub use types::*;

pub type IrPtr = usize;
pub type Mangle = String;

/// Name mangle syntax:
/// mangle ::= "$" path salt
/// path ::= {path-element}*
/// path-element ::= length ident
/// ident ::= _a-zA-Z {_a-zA-Z0-9}*
/// length ::= {0-9}+
/// salt ::= {a-zA-Z}*
pub fn mangle_name(path: Vec<String>, salt: &str) -> Mangle {
  let mut buf: Vec<u8> = vec![];
  for p in path {
    let puny = punycode::encode(&p).unwrap();
    let bytes = format!("{}{puny}", puny.len());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}

/// Builtin mangle syntax:
/// "$" {ident}
pub fn mangle_builtin(name: impl std::fmt::Display) -> Mangle {
  format!("_{name}")
}

#[derive(Debug, Clone)]
pub enum HlIrKind {
  Declaration {
    assignee: Mangle,
    is_type: bool,
    is_recursive: bool,
    value: IrPtr,
    in_: Option<IrPtr>,
  },
  Immediate(ConstValue),
  Identifier(Mangle),
  Tuple(Vec<IrPtr>),
  StructDef {
    field_names: Vec<String>,
    field_types: Vec<IrPtr>,
  },
  StructLiteral {
    field_names: Vec<String>,
    field_values: Vec<IrPtr>,
  },
  Field {
    of: IrPtr,
    index: String,
  },
  Binary {
    op: BinaryOp,
    left: IrPtr,
    right: IrPtr,
  },
  Unary {
    op: UnaryOp,
    child: IrPtr,
  },
  FunctionDef {
    parameter_name: Mangle,
    parameter_span: Span,
    parameter_type: Option<IrPtr>,
    captures: Vec<Mangle>,
    capture_types: Vec<Type>,
    body: IrPtr,
  },
  FunctionCall {
    callee: IrPtr,
    argument: IrPtr,
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
}

impl HlIrModule {
  pub fn ir_range(&self, start: IrPtr) -> std::ops::Range<IrPtr> {
    let mut current = start;
    loop {
      use HlIrKind::*;
      current = *match &self[current].kind {
        Declaration { value, in_, .. } => {
          if let Some(in_) = in_ {
            in_
          } else {
            value
          }
        },
        FunctionCall {
          argument: arguments,
          ..
        } => arguments,
        StructDef {
          field_types: items, ..
        }
        | StructLiteral {
          field_values: items,
          ..
        }
        | Tuple(items) => {
          if let Some(last) = items.last() {
            last
          } else {
            break;
          }
        },
        FunctionDef { body: last, .. }
        | Binary { right: last, .. }
        | Unary { child: last, .. }
        | Field { of: last, .. } => last,
        If { then, else_, .. } => {
          if let Some(else_) = else_ {
            else_
          } else {
            then
          }
        },
        Immediate(_) | Identifier(_) => break,
      }
    }
    start..current
  }
}

impl std::ops::Index<usize> for HlIrModule {
  type Output = HlIrNode;

  fn index(&self, index: usize) -> &Self::Output {
    &self.nodes[index]
  }
}

impl std::ops::IndexMut<usize> for HlIrModule {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.nodes[index]
  }
}
