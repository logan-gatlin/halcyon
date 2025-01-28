use std::collections::HashSet;

use operators::OpDef;

use crate::{BinaryOp, Immediate, Span, UnaryOp};
use crate::{err::*, error};

use super::*;

#[derive(Debug, Clone)]
pub struct Module {
  pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
  Loop {
    names: Vec<Mangle>,
    initials: Vec<Node>,
    body: Box<Node>,
  },
  Break {
    expr: Box<Node>,
  },
  Immediate(Immediate),
  Identifier {
    name: String,
    global: bool,
    mangle: Mangle,
  },
  StructLiteral {
    names: Vec<String>,
    values: Vec<Node>,
  },
  BinaryOp {
    op: BinaryOp,
    opdef: OpDef,
    left: Box<Node>,
    right: Box<Node>,
  },
  UnaryOp {
    op: UnaryOp,
    opdef: OpDef,
    child: Box<Node>,
  },
  Field {
    namespace: Box<Node>,
    index: String,
  },
  If {
    predicate: Box<Node>,
    then: Box<Node>,
    else_: Option<Box<Node>>,
  },
  Call {
    mangle: Mangle,
    callee: Box<Node>,
    params: Vec<Node>,
  },
  Function {
    mangle: Mangle,
    param_mangles: Vec<Mangle>,
    nodes: Box<Node>,
  },
  Declaration {
    name: String,
    global: bool,
    mangle: Mangle,
    is_constant: bool,
    type_assert: Option<Type>,
    value: Box<Node>,
  },
  Block {
    nodes: Vec<Node>,
  },
  Remainder {
    node: Box<Node>,
  },
}

#[derive(Debug, Clone)]
pub struct Node {
  pub span: Span,
  pub type_: Type,
  pub kind: NodeKind,
}

impl Analyzer {
  pub fn resolve_type(
    &self,
    type_: Type,
    mut history: HashSet<Mangle>,
  ) -> Result<Type> {
    match type_ {
      Type::Type(t) => Ok(Type::Type(self.resolve_type(*t, history)?.into())),
      Type::SameAs(mangle) => {
        let t = self.mangle_to_type(&mangle)?;
        if !history.insert(mangle) {
          return error!("Cannot determine type, found circular dependency");
        }
        self.resolve_type(t.clone(), history)
      },
      Type::IsType(mangle) => {
        let t = self.mangle_to_type(&mangle)?;
        if !history.insert(mangle) {
          return error!("Cannot determine type, found circular dependency");
        }
        self.resolve_type(t.clone(), history)?.unwrap_type_name()
      },
      Type::Struct {
        name,
        mangle,
        member_names,
        member_types,
      } => Ok(Type::Struct {
        name,
        mangle,
        member_names,
        member_types: member_types
          .into_iter()
          .map(|t| self.resolve_type(t, history.clone()))
          .try_collect::<Vec<_>>()?
          .into_iter()
          .map(|t| t.expect_type_name())
          .try_collect()?,
      }),
      Type::Function {
        mangle,
        param_names,
        param_types,
        return_type,
      } => Ok(Type::Function {
        mangle,
        param_names,
        param_types: param_types
          .into_iter()
          .map(|t| self.resolve_type(t, history.clone()))
          .try_collect::<Vec<_>>()?
          .into_iter()
          .map(|t| t.expect_type_name())
          .try_collect()?,
        return_type: self
          .resolve_type(*return_type, history.clone())?
          .expect_type_name()?
          .into(),
      }),
      //Type::Ambiguous => error!("Cannot determine type")?,
      t => Ok(t),
    }
  }
}
