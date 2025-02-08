use crate::{
  Span,
  parse::{BinaryOp, UnaryOp},
  semantic::{Mangle, Type, operators::OpDef},
};

pub enum Block {
  Terminus,
  Basic {
    body: Vec<Ir>,
    next: IrPtr,
  },
  Branch {
    predicate_mangle: Mangle,
    body: Vec<Ir>,
    when_true: IrPtr,
    when_false: IrPtr,
  },
  Loop {
    param_mangles: Vec<Mangle>,
    body: Vec<Ir>,
    next: IrPtr,
  },
  Function {
    param_names: Vec<Mangle>,
    param_types: Vec<Type>,
    returns: Type,
  },
}

pub struct Ir {
  pub kind: IrKind,
  pub type_: Type,
  pub span: Span,
}

pub enum IrKind {
  Const(ConstValue),
  Set { mangle: Mangle, type_: Type },
  Get { constant: bool, mangle: Mangle },
  BinaryOp { kind: BinaryOp, def: OpDef },
  UnaryOp { kind: UnaryOp, def: OpDef },
  Field(String),
  StructLiteral { param_names: Vec<String> },
  StructDef { param_names: Vec<String> },
  TypeAssert,
  Call,
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

/// Reference to another IR node
pub type IrPtr = usize;
