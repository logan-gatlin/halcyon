mod consteval;
pub mod generate;
mod interpret;
pub mod solver;

use std::collections::HashMap;

use crate::{Span, graph::Graph, hlir::*, operator::*, parse::*};

pub use generate::*;
pub use solver::*;

#[derive(Debug, Clone)]
pub enum Block {
  Terminal,
  Unreachable,
  Basic {
    body: Vec<MlIrNode>,
    next: IrPtr,
  },
  Branch {
    span: Span,
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

  pub fn push(&mut self, ir: MlIrNode) {
    if let Block::Basic { body, .. } = self {
      body.push(ir)
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
pub struct MlIrModule {
  pub heap: Vec<Vec<u8>>,
  pub constants: HashMap<Mangle, IrPtr>,
  pub functions: HashMap<Mangle, FunctionInfo>,
  pub type_assertions: HashMap<Mangle, IrPtr>,
  pub blocks: Vec<Block>,
}

impl MlIrModule {
  pub fn to_json(&self) -> String {
    let mut graph = Graph::new();
    for (i, b) in self.blocks.iter().enumerate() {
      match b {
        Block::Terminal => {
          graph.new_node(i.to_string(), "TERMINAL".to_string());
        }
        Block::Unreachable => {
          graph.new_node(i.to_string(), "UNREACHABLE".to_string());
        }
        Block::Basic { body, next } => {
          let mut body = body
            .into_iter()
            .map(|ir| format!("{ir:?}"))
            .collect::<Vec<_>>()
            .join("\\n");
          if body.is_empty() {
            body = "(empty)".to_string();
          }
          graph.new_node(i.to_string(), format!("{body}"));
          graph.new_edge(i.to_string(), next.to_string());
        }
        Block::Branch {
          when_true,
          when_false,
          ..
        } => {
          graph.new_node(i.to_string(), "BRANCH".into());
          graph.new_edge(i.to_string(), when_true.to_string());
          graph.new_edge(i.to_string(), when_false.to_string());
        }
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
  pub returns_mangle: Option<Mangle>,
  pub block: IrPtr,
}

#[derive(Clone)]
pub struct MlIrNode {
  pub span: Span,
  pub kind: MlIrKind,
}

impl std::fmt::Debug for MlIrNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self.kind,)
  }
}

#[derive(Clone)]
pub enum MlIrKind {
  /// Push a constant value
  Const(ConstValue),
  /// Pop 1 value, assign the value to a name
  Set(Mangle),
  /// Push a named value
  Get(Mangle),
  /// Pop 2 values, apply a binary operator, push the result
  BinaryOp { kind: BinaryOp },
  /// Pop 1 value, apply a unary operator, push the result
  UnaryOp { kind: UnaryOp },
  /// Pop 1 value, push the named field value
  Field(String),
  /// Pop N values, construct a new value and push it
  StructLiteral { param_names: Vec<String> },
  /// Pop N type values, push a new struct definition
  StructDef { fields: Vec<String> },
  /// Pop 1 type, assert that the next value on the stack is
  /// of that type. Keep this second value on the stack
  TypeAssert(Option<Mangle>),
  /// Pop 1 function, pop N argument values, call the
  /// function and push its return value
  Call { arity: usize },
  /// Clear the stack of any values up to the last enscope
  Drop,
  /*
  // Pop 1 boolean, branch to `then_ptr` if it is true, `else_ptr` otherwise
  Branch {
    then_ptr: IrPtr,
    else_ptr: IrPtr,
  },
  // Jump to relative position
  Jump(i64),
  */
  /// Inserts a scope guard, prevents popping values pushed
  /// before this point
  StartScope,
  /// Remove a previously placed scope guard, leaving any
  /// remaining values on the stack
  EndScope,
}

impl std::fmt::Debug for MlIrKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MlIrKind::Const(const_value) => write!(f, "push {const_value}"),
      MlIrKind::Set(mangle) => write!(f, "set {mangle}"),
      MlIrKind::Get(mangle) => write!(f, "get {mangle}"),
      MlIrKind::BinaryOp { kind } => write!(f, "binary {kind}"),
      MlIrKind::UnaryOp { kind } => write!(f, "unary {kind}"),
      MlIrKind::Field(name) => write!(f, "field {name}"),
      MlIrKind::StructLiteral { param_names } => {
        write!(f, "struct literal {}", param_names.len())
      }
      MlIrKind::StructDef {
        fields: param_names,
      } => {
        write!(f, "struct definition {}", param_names.len())
      }
      MlIrKind::TypeAssert(mangle) => write!(
        f,
        "type assert{}",
        if let Some(mangle) = mangle {
          format!(" ({mangle})")
        } else {
          format!("")
        }
      ),
      MlIrKind::Call { arity } => write!(f, "call {arity}"),
      MlIrKind::Drop => write!(f, "drop"),
      MlIrKind::StartScope => write!(f, "start scope"),
      MlIrKind::EndScope => write!(f, "end scope"),
    }
  }
}
