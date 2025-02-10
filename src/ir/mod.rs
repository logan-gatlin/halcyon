pub mod types;

use types::Type;

use crate::{
  Span,
  assembly::operators::OpDef,
  naming::Mangle,
  parse::{BinaryOp, UnaryOp},
};

/// Reference to another IR node
pub type IrPtr = usize;

#[derive(Debug, Clone)]
pub enum Block {
  Terminal,
  Unreachable,
  Basic {
    body: Vec<Ir>,
    next: IrPtr,
  },
  Branch {
    predicate_mangle: Mangle,
    when_true: IrPtr,
    when_false: IrPtr,
  },
}

impl Block {
  pub fn basic() -> Self {
    Self::Basic {
      body: vec![],
      next: 0,
    }
  }

  pub fn into_body(self) -> Vec<Ir> {
    if let Block::Basic { body, .. } = self {
      body
    } else {
      panic!("Tried to access body of {self:?}")
    }
  }

  pub fn push(&mut self, ir: Ir) {
    if let Block::Basic { body, .. } = self {
      body.push(ir)
    } else {
      panic!("Tried to append instruction to {self:?}")
    }
  }

  pub fn set_next(&mut self, new_next: IrPtr) {
    if let Block::Basic { next, .. } = self {
      *next = new_next
    } else {
      panic!("Tried to set next on {self:?}")
    }
  }

  pub fn is_terminal(&self) -> bool {
    if let Block::Terminal | Block::Unreachable = self {
      true
    } else {
      false
    }
  }
}

impl Default for Block {
  fn default() -> Self {
    Self::basic()
  }
}

#[derive(Debug, Clone)]
pub struct Ir {
  pub kind: IrKind,
  pub type_: Type,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum IrKind {
  /// Push a constant value
  Const(ConstValue),
  /// Pop 1 value, assign the value to a name
  Set(Mangle),
  /// Push a named value
  Get(Mangle),
  /// Pop 2 values, apply a binary operator, push the result
  BinaryOp { kind: BinaryOp, def: OpDef },
  /// Pop 1 value, apply a unary operator, push the result
  UnaryOp { kind: UnaryOp, def: OpDef },
  /// Pop 1 value, push the named field value
  Field(String),
  /// Pop N values, construct a new value and push it
  StructLiteral { param_names: Vec<String> },
  /// Pop N type values, push a new struct definition
  StructDef { param_names: Vec<String> },
  /// Pop 1 type, assert that the next value on the stack is
  /// of that type. Keep this second value on the stack
  TypeAssert,
  /// Pop 1 function, pop N argument values, call the
  /// function and push its return value
  Call { arity: usize },
  /// Inserts a scope guard, prevents popping values pushed
  /// before this point
  Enscope,
  /// Clear the stack frame save for one value. If no value
  /// exists before an enscope, push 'nothing' value
  Descope,
}

#[derive(Clone, Debug)]
pub enum ConstValue {
  Nothing,
  Integer(i64),
  Real(f64),
  Boolean(bool),
  String {
    address: usize,
    length: usize,
  },
  Glyph(char),
  Function(Mangle),
  StructLiteral {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
  },
  Type(Type),
}
