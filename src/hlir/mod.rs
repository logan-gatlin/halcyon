mod build_hlir;
pub mod builtins;
pub mod constant;
pub mod printing;
pub mod types;

use std::collections::HashMap;

use crate::{lint::*, operator::*, parse::*};

pub use build_hlir::*;
pub use builtins::*;
pub use constant::*;
pub use types::*;

pub type IrPtr = usize;
pub type Mangle = String;

pub fn build_hlir(expr: Expression) -> Result<HlIrModule> {
  let canon = Canonizer::new();
  canon.canonize_expr(expr)
}

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
  Block(Vec<IrPtr>),
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
    id: u32,
    export_name: Option<String>,
    parameter_names: Vec<Mangle>,
    parameter_spans: Vec<Span>,
    parameter_types: Vec<Option<IrPtr>>,
    body: IrPtr,
  },
  FunctionCall {
    callee: IrPtr,
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
