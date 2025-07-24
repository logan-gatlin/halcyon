mod build_ir;
pub mod constant;
mod namespace;
pub mod printing;
pub mod types;

use std::collections::HashMap;

use crate::{
  builtin::Builtin, lint::*, operator::*, semantic::ModuleInterface,
};

pub use build_ir::*;
pub use constant::*;
use namespace::*;
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

pub fn mangle_global(
  module_path: &[&str],
  name: impl std::fmt::Display,
) -> Mangle {
  let mut path = module_path.to_vec();
  let name = format!("{name}");
  path.push(&name);
  path.join(":")
}

#[derive(Debug, Clone)]
pub enum IrKind {
  Declaration {
    assignee: Mangle,
    value: IrPtr,
    in_: Option<IrPtr>,
  },
  RecursiveDeclaration {
    assignee: Mangle,
    parameter_name: Option<Mangle>,
    parameter_span: Span,
    parameter_type: Option<TypeRef>,
    captures: Vec<Mangle>,
    capture_types: Vec<TypeRef>,
    function_type: TypeRef,
    body: IrPtr,
    in_: Option<IrPtr>,
  },
  Immediate(ConstValue),
  Identifier(Mangle),
  Tuple(Vec<IrPtr>),
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
    parameter_name: Option<Mangle>,
    parameter_span: Span,
    parameter_type: Option<TypeRef>,
    captures: Vec<Mangle>,
    capture_types: Vec<TypeRef>,
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
  ImportedSymbol(Mangle, TypeRef),
}

#[derive(Debug, Clone)]
pub struct IrNode {
  pub kind: IrKind,
  pub span: Span,
  pub type_: TypeRef,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
  Let(String, IrPtr),
  Type(String, TypeRef),
}

#[derive(Debug, Clone)]
pub struct IrModule {
  pub module_name: String,
  pub universe: HashMap<Mangle, TypeRef>,
  pub items: Vec<ModuleItem>,
  pub nodes: Vec<IrNode>,
}

impl IrModule {
  pub fn ir_range(&self, start: IrPtr) -> std::ops::Range<IrPtr> {
    let mut current = start;
    loop {
      use IrKind::*;
      current = *match &self[current].kind {
        Declaration { value, in_, .. } => {
          if let Some(in_) = in_ {
            in_
          } else {
            value
          }
        },
        RecursiveDeclaration { body, in_, .. } => {
          if let Some(in_) = in_ {
            in_
          } else {
            body
          }
        },
        FunctionCall {
          argument: arguments,
          ..
        } => arguments,
        StructLiteral {
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
        ImportedSymbol(..) | Immediate(..) | Identifier(..) => break,
      }
    }
    start..current
  }
}

impl std::ops::Index<usize> for IrModule {
  type Output = IrNode;

  fn index(&self, index: usize) -> &Self::Output {
    &self.nodes[index]
  }
}

impl std::ops::IndexMut<usize> for IrModule {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.nodes[index]
  }
}

type FunctionDepth = usize;

#[derive(Debug, Clone)]
enum Scope {
  Value {
    clean: String,
    old: Option<(Mangle, FunctionDepth)>,
  },
  Type {
    clean: String,
    old: Option<Mangle>,
  },
}
