use std::collections::HashSet;

use crate::{BinaryOp, Immediate, UnaryOp};
use crate::{err::*, error};

use super::*;

#[derive(Debug, Clone)]
pub enum NodeKind {
  Immediate(Immediate),
  Identifier(Mangle),
  Return {
    node: Box<Node>,
  },
  StructLiteral {
    names: Vec<String>,
    values: Vec<Node>,
  },
  BinaryOp {
    op: BinaryOp,
    left: Box<Node>,
    right: Box<Node>,
  },
  UnaryOp {
    op: UnaryOp,
    child: Box<Node>,
  },
  Field {
    namespace: Box<Node>,
    index: Box<Node>,
  },
  If {
    predicate: Box<Node>,
    then: Box<Node>,
    else_: Option<Box<Node>>,
  },
  Call {
    callee: Box<Node>,
    params: Vec<Node>,
  },
  Function {
    mangle: Mangle,
    arguments: Vec<Mangle>,
    nodes: Box<Node>,
  },
  Declaration {
    mangle: Mangle,
    is_constant: bool,
    type_assert: Option<Type>,
    value: Box<Node>,
  },
  Block {
    nodes: Vec<Node>,
  },
}

#[derive(Debug, Clone)]
pub struct Node {
  // Type of this node
  pub remainder: Option<Type>,
  pub returns: Option<Type>,
  pub type_: Type,
  pub kind: NodeKind,
}

impl Analyzer {
  fn resolve_type(
    &self,
    type_: Type,
    mut history: HashSet<Mangle>,
  ) -> Result<Type> {
    match type_ {
      Type::Type(t) => Ok(Type::Type(self.resolve_type(*t, history)?.into())),
      Type::Unresolved(mangle) => {
        let t = self.mangle_to_type(&mangle)?;
        if !history.insert(mangle) {
          return error!("Cannot determine type, found circular dependency");
        }
        self.resolve_type(t.clone(), history)
      },
      Type::Struct {
        mangle,
        member_names,
        member_types,
      } => Ok(Type::Struct {
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
      t => Ok(t),
    }
  }

  pub(crate) fn type_bottom_up(&mut self, mut node: Node) -> Result<Node> {
    use NodeKind as n;
    node.type_ = self.resolve_type(node.type_, HashSet::new())?;
    node.remainder = if let Some(rem) = node.remainder {
      Some(self.resolve_type(rem, HashSet::new())?)
    } else {
      None
    };
    node.returns = if let Some(ret) = node.returns {
      Some(self.resolve_type(ret, HashSet::new())?)
    } else {
      None
    };
    node.kind = match node.kind {
      n::StructLiteral { names, values } => n::StructLiteral {
        names,
        values: values
          .into_iter()
          .map(|n| self.type_bottom_up(n))
          .collect::<Result<Vec<_>>>()?,
      },
      n::BinaryOp { op, left, right } => {
        let left = self.type_bottom_up(*left)?;
        let right = self.type_bottom_up(*right)?;
        node.type_ = self.op_table.try_binary(op, &left.type_, &right.type_)?;
        n::BinaryOp {
          op,
          left: left.into(),
          right: right.into(),
        }
      },
      n::UnaryOp { op, child } => {
        node.type_ = self.op_table.try_unary(op, &child.type_)?;
        let child = self.type_bottom_up(*child)?.into();
        n::UnaryOp { op, child }
      },
      n::Field { namespace, index } => {
        let namespace = self.type_bottom_up(*namespace)?.into();
        let index = self.type_bottom_up(*index)?.into();
        n::Field { namespace, index }
      },
      n::If {
        predicate,
        then,
        else_,
      } => {
        let predicate = self.type_bottom_up(*predicate)?.into();
        let then = self.type_bottom_up(*then)?;
        let then_t = then.type_.clone();
        let (else_t, else_) = if let Some(else_) = else_ {
          let node = self.type_bottom_up(*else_)?;
          (node.type_.clone(), Some(node.into()))
        } else {
          (Primitive::nothing.promote(), None)
        };
        if then_t != else_t {
          return error!(
            "Branches of this 'if' expression produce different types \
             ('{then_t}' and '{else_t}')"
          );
        }
        node.type_ = then_t;
        n::If {
          predicate,
          then: then.into(),
          else_,
        }
      },
      n::Call { callee, params } => {
        let callee = self.type_bottom_up(*callee)?;
        if let Type::Function { return_type, .. } = &callee.type_ {
          node.type_ = return_type.clone().unwrap_type_name()?;
        } else {
          return error!("Cannot call type {}", callee.type_);
        }
        let params = params
          .into_iter()
          .map(|p| self.type_bottom_up(p))
          .collect::<Result<Vec<_>>>()?;
        n::Call {
          callee: callee.into(),
          params,
        }
      },
      n::Function {
        mangle,
        arguments,
        nodes,
      } => {
        node.type_ = self.resolve_type(
          self.mangle_to_type(&mangle)?.clone(),
          HashSet::new(),
        )?;
        *self.mangle_to_type_mut(&mangle)? = node.type_.clone();
        for mangle in &arguments {
          let type_ = self.mangle_to_type(mangle)?;
          *self.mangle_to_type_mut(mangle)? =
            self.resolve_type(type_.clone(), HashSet::new())?;
        }
        let nodes = self.type_bottom_up(*nodes)?.into();
        n::Function {
          mangle,
          arguments,
          nodes,
        }
      },
      n::Declaration {
        mangle,
        is_constant,
        type_assert,
        value,
      } => {
        let value = self.type_bottom_up(*value)?;
        if type_assert.is_none() {
          *self.mangle_to_type_mut(&mangle)? = value.type_.clone();
        }
        n::Declaration {
          mangle,
          is_constant,
          type_assert,
          value: value.into(),
        }
      },
      n::Block { nodes } => n::Block {
        nodes: nodes
          .into_iter()
          .map(|n| self.type_bottom_up(n))
          .collect::<Result<Vec<_>>>()?,
      },
      n::Immediate(_) => node.kind,
      n::Identifier(_) => node.kind,
      n::Return { node } => n::Return {
        node: self.type_bottom_up(*node)?.into(),
      },
    };
    Ok(node)
  }

  pub(crate) fn type_top_down(
    &mut self,
    mut node: Node,
    expects: Type,
    returns: Type,
  ) -> Result<Node> {
    todo!()
  }
}
