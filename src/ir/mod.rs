pub mod consteval;
pub mod types;

use std::collections::HashMap;

use types::Type;

use crate::{
  Span,
  assembly::operators::OpDef,
  graph::Graph,
  naming::Mangle,
  parse::{BinaryOp, UnaryOp},
};

/// Reference to another IR node
pub type IrPtr = usize;

#[derive(Debug, Clone)]
pub enum Block {
  Terminal,
  Unreachable,
  Basic { body: Vec<Ir>, next: IrPtr },
  Branch { when_true: IrPtr, when_false: IrPtr },
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
pub struct Module {
  pub heap: Vec<Vec<u8>>,
  pub constants: HashMap<Mangle, IrPtr>,
  pub functions: HashMap<Mangle, FunctionInfo>,
  pub parameters: HashMap<Mangle, IrPtr>,
  pub blocks: Vec<Block>,
}

impl Module {
  pub fn to_json(&self) -> String {
    let mut graph = Graph::new();
    for (i, b) in self.blocks.iter().enumerate() {
      match b {
        Block::Terminal => {
          graph.new_node(i.to_string(), "TERMINAL".to_string());
        },
        Block::Unreachable => {
          graph.new_node(i.to_string(), "UNREACHABLE".to_string());
        },
        Block::Basic { body, next } => {
          let mut body = body
            .into_iter()
            .map(|b| format!("{b:?}"))
            .collect::<Vec<_>>()
            .join("\\n");
          if body.is_empty() {
            body = "(empty)".to_string();
          }
          graph.new_node(i.to_string(), format!("{body}"));
          graph.new_edge(i.to_string(), next.to_string());
        },
        Block::Branch {
          when_true,
          when_false,
        } => {
          graph.new_node(i.to_string(), "BRANCH".into());
          graph.new_edge(i.to_string(), when_true.to_string());
          graph.new_edge(i.to_string(), when_false.to_string());
        },
      };
    }
    graph.edges = graph
      .edges
      .into_iter()
      .filter(|e| e.target != "0".to_string())
      .collect();
    graph.nodes.remove(0);
    graph.to_json()
  }
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
  pub mangle: Mangle,
  pub arity: usize,
  pub parameter_mangles: Vec<Mangle>,
  pub block: IrPtr,
}

#[derive(Clone)]
pub struct Ir {
  pub type_: Type,
  pub span: Span,
  pub kind: IrKind,
}

impl std::fmt::Debug for Ir {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self.kind)
  }
}

#[derive(Clone)]
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
  /// Clear the stack of any values up to the last enscope
  Drop,
  /// Inserts a scope guard, prevents popping values pushed
  /// before this point
  Enscope,
  /// Remove a previously placed scope guard, leaving any
  /// remaining values on the stack
  Descope,
}

impl std::fmt::Debug for IrKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      IrKind::Const(const_value) => write!(f, "push {const_value}"),
      IrKind::Set(mangle) => write!(f, "set {mangle}"),
      IrKind::Get(mangle) => write!(f, "get {mangle}"),
      IrKind::BinaryOp { kind, def } => write!(f, "binary {kind}"),
      IrKind::UnaryOp { kind, def } => write!(f, "unary {kind}"),
      IrKind::Field(name) => write!(f, "field {name}"),
      IrKind::StructLiteral { param_names } => {
        write!(f, "struct literal {}", param_names.len())
      },
      IrKind::StructDef { param_names } => {
        write!(f, "struct definition {}", param_names.len())
      },
      IrKind::TypeAssert => write!(f, "type assert"),
      IrKind::Call { arity } => write!(f, "call {arity}"),
      IrKind::Drop => write!(f, "drop"),
      IrKind::Enscope => write!(f, "start scope"),
      IrKind::Descope => write!(f, "end scope"),
    }
  }
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

impl std::fmt::Display for ConstValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConstValue::Nothing => write!(f, "()"),
      ConstValue::String { address, length } => write!(f, "string {address}"),
      ConstValue::Function(val) => write!(f, "function {val}"),
      ConstValue::StructLiteral {
        member_names,
        member_values,
      } => write!(f, "struct"),
      ConstValue::Type(val) => write!(f, "{val}"),
      ConstValue::Integer(val) => write!(f, "{val}"),
      ConstValue::Real(val) => write!(f, "{val}"),
      ConstValue::Glyph(val) => write!(f, "{val}"),
      ConstValue::Boolean(val) => write!(f, "{val}"),
    }
  }
}
