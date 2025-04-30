pub mod build_mlir;
//pub mod dependencies;
pub mod evaluate;

pub use build_mlir::*;
//pub use dependencies::*;
pub use evaluate::*;

use std::collections::{HashMap, HashSet};

use crate::{hlir::*, lint::*, memory::*, operator::*, parse::*};

/// Start, Length
#[derive(Clone, Copy, Debug)]
pub struct MlIrSpan(IrPtr, usize);

#[derive(Debug, Clone)]
pub struct MlIrModule {
  ir: Vec<MlIrNode>,
  pub source_map: HashMap<IrPtr, MlIrSpan>,
}

impl std::fmt::Display for MlIrModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for i in &self.ir {
      write!(f, "{i}\n")?;
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MlIrNode {
  pub span: Span,
  pub kind: MlIrKind,
}

impl std::fmt::Display for MlIrNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.kind,)
  }
}

#[derive(Debug, Clone)]
pub enum MlIrKind {
  /// Push a constant value
  Const(ConstValue),
  /// Pop 1 value, assign the value to a name
  Set(Mangle),
  /// Push a named value
  Get(Mangle),
  /// Pop 2 values, apply a binary operator, push the result
  BinaryOp(BinaryOp),
  /// Pop 1 value, apply a unary operator, push the result
  UnaryOp(UnaryOp),
  /// Pop 1 value, push the named field value
  Field(String),
  /// Pop N values, construct a new value and push it
  StructLiteral(Vec<String>),
  /// Pop N type values, push a new struct definition
  StructDef(Vec<String>),
  /// Pop N values, construct a new tuple and push it
  Tuple(usize),
  /// Pop 1 type, assert that the next value on the stack is
  /// of that type. Keep this second value on the stack
  TypeAssert,
  /// Pop 1 function, pop N argument values, call the
  /// function and push its return value
  Call(usize),
  /// Clear the stack of any values up to the last enscope
  Drop,
  /// Pop 1 boolean, if true continue, otherwise go to else
  If,
  Else,
  End,
  Function(Mangle),
  Return,
  Nop,
}

impl std::fmt::Display for MlIrKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MlIrKind::Const(const_value) => write!(f, "push {const_value}"),
      MlIrKind::Set(mangle) => write!(f, "set {mangle}"),
      MlIrKind::Get(mangle) => write!(f, "get {mangle}"),
      MlIrKind::BinaryOp(kind) => write!(f, "operator {kind}"),
      MlIrKind::UnaryOp(kind) => write!(f, "operator {kind}"),
      MlIrKind::Field(name) => write!(f, "field {name}"),
      MlIrKind::StructLiteral(fields) => {
        write!(f, "struct literal {}", fields.len())
      },
      MlIrKind::StructDef(fields) => {
        write!(f, "struct definition {}", fields.len())
      },
      MlIrKind::Tuple(size) => write!(f, "tuple {size}"),
      MlIrKind::TypeAssert => write!(f, "type assert",),
      MlIrKind::Function(mangle) => write!(f, "function {mangle}"),
      MlIrKind::Call(arity) => write!(f, "call {arity}"),
      MlIrKind::Drop => write!(f, "drop"),
      MlIrKind::If => write!(f, "if"),
      MlIrKind::Else => write!(f, "else"),
      MlIrKind::Return => write!(f, "return"),
      MlIrKind::End => write!(f, "end"),
      MlIrKind::Nop => write!(f, "nop"),
    }
  }
}
