use std::collections::HashSet;

use crate::{BinaryOp, Immediate, Span, UnaryOp};
use crate::{err::*, error};

use super::*;

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
    mangle: Mangle,
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
    arguments: Vec<Mangle>,
    nodes: Box<Node>,
  },
  Declaration {
    name: String,
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
  fn resolve_type(
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

  pub(crate) fn type_bottom_up(&mut self, mut node: Node) -> Result<Node> {
    use NodeKind as n;
    let span = &node.span;
    node.type_ = self.resolve_type(node.type_, HashSet::new()).span(span)?;
    node.kind = match node.kind {
      n::StructLiteral { names, values } => {
        node.type_ = node.type_.unwrap_type_name().span(span)?;
        n::StructLiteral {
          names,
          values: values
            .into_iter()
            .map(|n| self.type_bottom_up(n))
            .collect::<Result<Vec<_>>>()
            .span(span)?,
        }
      },
      n::BinaryOp { op, left, right } => {
        let left = self.type_bottom_up(*left)?;
        let right = self.type_bottom_up(*right)?;
        node.type_ = self
          .op_table
          .try_binary(op, &left.type_, &right.type_)
          .span(span)?;
        n::BinaryOp {
          op,
          left: left.into(),
          right: right.into(),
        }
      },
      n::UnaryOp { op, child } => {
        node.type_ = self.op_table.try_unary(op, &child.type_).span(span)?;
        let child = self.type_bottom_up(*child)?.into();
        n::UnaryOp { op, child }
      },
      n::Field { namespace, index } => {
        let namespace = self.type_bottom_up(*namespace)?;
        let Type::Struct {
          name,
          member_names,
          member_types,
          ..
        } = namespace.type_.clone()
        else {
          return error!(
            "The type '{}' does not contain fields",
            namespace.type_
          )
          .span(span);
        };
        node.type_ = member_names
          .iter()
          .zip(member_types.iter())
          .flat_map(|(name, type_)| {
            if name == &index {
              Some(type_.clone())
            } else {
              None
            }
          })
          .next()
          .reason(format!(
            "The struct '{}' does not contain field '{index}'",
            name.unwrap_or("anonymous struct".into())
          ))
          .span(span)?
          .unwrap_type_name()
          .span(span)?;
        n::Field {
          namespace: namespace.into(),
          index,
        }
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
      n::Call { callee, params, .. } => {
        let callee = self.type_bottom_up(*callee)?;
        let mangle = if let Type::Function {
          mangle,
          return_type,
          ..
        } = &callee.type_
        {
          node.type_ = return_type.clone().unwrap_type_name().span(span)?;
          mangle.clone()
        } else {
          return error!("Cannot call type {}", callee.type_);
        };
        let params = params
          .into_iter()
          .map(|p| self.type_bottom_up(p))
          .collect::<Result<Vec<_>>>()?;
        n::Call {
          mangle,
          callee: callee.into(),
          params,
        }
      },
      n::Declaration {
        name: clean_name,
        mangle,
        is_constant,
        type_assert,
        value,
      } => {
        let mut value = self.type_bottom_up(*value)?;
        // Name struct if it is anonymous
        value.type_ = if let Ok(Type::Struct {
          name: None,
          mangle,
          member_names,
          member_types,
        }) = value.type_.clone().unwrap_type_name()
        {
          Type::Type(
            Type::Struct {
              name: Some(clean_name.clone()),
              mangle,
              member_names,
              member_types,
            }
            .into(),
          )
        } else {
          value.type_
        };
        if type_assert.is_none() {
          *self.mangle_to_type_mut(&mangle).span(span)? = value.type_.clone();
        }
        n::Declaration {
          name: clean_name,
          mangle,
          is_constant,
          type_assert,
          value: value.into(),
        }
      },
      n::Block { nodes } => {
        let nodes = nodes
          .into_iter()
          .map(|n| self.type_bottom_up(n))
          .collect::<Result<Vec<_>>>()?;
        node.type_ = if let Some(Node {
          kind: n::Remainder { node: inner },
          ..
        }) = nodes.last()
        {
          inner.type_.clone()
        } else {
          Primitive::nothing.promote()
        };
        n::Block { nodes }
      },
      n::Immediate(_) => node.kind,
      n::Identifier { .. } => node.kind,
      n::Remainder { node: inner_node } => {
        let inner_node = self.type_bottom_up(*inner_node)?;
        node.type_ = inner_node.type_.clone();
        n::Remainder {
          node: inner_node.into(),
        }
      },
      n::Function {
        mangle,
        arguments,
        nodes,
      } => {
        node.type_ = self.resolve_type(
          self.mangle_to_type(&mangle).span(span)?.clone(),
          HashSet::new(),
        )?;
        *self.mangle_to_type_mut(&mangle).span(span)? = node.type_.clone();
        for mangle in &arguments {
          let type_ = self.mangle_to_type(mangle).span(span)?;
          *self.mangle_to_type_mut(mangle).span(span)? = self
            .resolve_type(type_.clone(), HashSet::new())
            .span(span)?;
        }
        let nodes = self.type_bottom_up(*nodes)?.into();
        n::Function {
          mangle,
          arguments,
          nodes,
        }
      },
      n::Loop {
        names,
        initials,
        body,
      } => {
        let mut initials = initials
          .into_iter()
          .map(|n| self.type_bottom_up(n))
          .try_collect::<Vec<_>>()?;
        for (mangle, type_) in
          names.iter().zip(initials.iter_mut().map(|n| &mut n.type_))
        {
          println!("{mangle:?} {type_:?}");
          *type_ = self
            .resolve_type(type_.clone(), HashSet::new())
            .span(span)?;
          *self.mangle_to_type_mut(mangle)? = type_.clone();
        }
        let body = self.type_bottom_up(*body)?;
        let Some(break_type) = self.check_breaks(&body) else {
          return error!(
            "This loop will never terminate, provide at least one 'break'"
          )
          .span(span);
        };
        node.type_ = break_type;
        n::Loop {
          names,
          initials,
          body: body.into(),
        }
      },
      n::Break { expr } => {
        let expr = self.type_bottom_up(*expr)?;
        n::Break { expr: expr.into() }
      },
    };
    if let Type::Ambiguous = node.type_ {
      return error!("Failed to determine type of expression").span(span);
    }
    Ok(node)
  }

  fn check_breaks(&mut self, node: &Node) -> Option<Type> {
    use NodeKind as n;
    match &node.kind {
      n::Break { expr } => Some(expr.type_.clone()),
      n::StructLiteral { values, .. } => {
        values.into_iter().flat_map(|n| self.check_breaks(n)).next()
      },
      n::BinaryOp { left, right, .. } => {
        self.check_breaks(left).or(self.check_breaks(right))
      },
      n::UnaryOp { child, .. } => self.check_breaks(child),
      n::If {
        predicate,
        then,
        else_,
      } => self.check_breaks(predicate).or(self.check_breaks(then)).or(
        if let Some(else_) = else_ {
          self.check_breaks(else_)
        } else {
          None
        },
      ),
      n::Call { callee, params, .. } => self
        .check_breaks(callee)
        .or(params.into_iter().flat_map(|n| self.check_breaks(n)).next()),
      n::Declaration { value, .. } => self.check_breaks(value),
      n::Block { nodes } => {
        nodes.into_iter().flat_map(|n| self.check_breaks(n)).next()
      },
      n::Remainder { node } => self.check_breaks(node),
      _ => None,
    }
  }

  pub fn type_top_down(&mut self, expects: Type, node: Node) -> Result<Node> {
    use NodeKind as n;
    let span = &node.span;
    if node.type_ != expects {
      return error!("Expected type '{expects}', found '{}'", node.type_)
        .span(span);
    }
    let kind = match node.kind {
      n::Immediate(im) => n::Immediate(im),
      n::Identifier { name, mangle } => n::Identifier { name, mangle },
      n::StructLiteral { names, mut values } => {
        let Type::Struct {
          member_names,
          member_types,
          ..
        } = node.type_.clone()
        else {
          return error!(
            "Type '{}' cannot be instantiated with a struct literal expression",
            node.type_
          )
          .span(span);
        };
        let mut member_name_set = HashSet::new();
        for id in 0..names.len() {
          let name = &names[id];
          if !member_names.contains(name) {
            return error!("'{}' does not have member '{name}'", node.type_)
              .span(span);
          }
          if !member_name_set.insert(name) {
            return error!("Struct member '{name}' has already been provided")
              .span(span);
          }
          values[id] = self.type_top_down(
            member_types[id].clone().unwrap_type_name().span(span)?,
            values[id].clone(),
          )?;
        }
        let member_set_actual: HashSet<_> = member_names.iter().collect();
        let missing: String = member_set_actual
          .difference(&member_name_set)
          .cloned()
          .cloned()
          .collect::<Vec<_>>()
          .join(", ");
        if missing.len() != 0 {
          return error!("Missing struct members: {missing}");
        }
        n::StructLiteral { names, values }
      },
      n::Field { namespace, index } => {
        let namespace =
          self.type_top_down(namespace.type_.clone(), *namespace)?;
        n::Field {
          namespace: namespace.into(),
          index,
        }
      },
      n::BinaryOp { op, left, right } => {
        let left = self.type_top_down(left.type_.clone(), *left)?;
        let right = self.type_top_down(right.type_.clone(), *right)?;
        n::BinaryOp {
          op,
          left: left.into(),
          right: right.into(),
        }
      },
      n::UnaryOp { op, child } => {
        let child = self.type_top_down(child.type_.clone(), *child)?;
        n::UnaryOp {
          op,
          child: child.into(),
        }
      },
      n::If {
        predicate,
        then,
        else_,
      } => {
        let predicate =
          self.type_top_down(Primitive::boolean.promote(), *predicate)?;
        let then = self.type_top_down(then.type_.clone(), *then)?;
        let else_ = if let Some(else_) = else_ {
          Some(self.type_top_down(else_.type_.clone(), *else_)?.into())
        } else {
          None
        };
        n::If {
          predicate: predicate.into(),
          then: then.into(),
          else_,
        }
      },
      n::Call {
        mangle,
        callee,
        params,
      } => {
        let callee = self.type_top_down(callee.type_.clone(), *callee)?;
        let Type::Function { param_types, .. } = callee.type_.clone() else {
          panic!("This should never happen")
        };
        if params.len() != param_types.len() {
          return error!(
            "This function expects {} arguments, but {} were provided",
            param_types.len(),
            params.len()
          )
          .span(span);
        }
        let params = param_types
          .into_iter()
          .map(|t| t.unwrap_type_name())
          .try_collect::<Vec<_>>()
          .span(span)?
          .into_iter()
          .zip(params.into_iter())
          .map(|(type_, node)| self.type_top_down(type_, node))
          .try_collect::<Vec<_>>()?;
        n::Call {
          mangle,
          callee: callee.into(),
          params,
        }
      },
      n::Declaration {
        name: clean_name,
        mangle,
        is_constant,
        type_assert,
        value,
      } => {
        let value = if let Some(type_) = &type_assert {
          let type_ = self
            .resolve_type(type_.clone(), HashSet::new())
            .span(span)?
            .unwrap_type_name()
            .span(span)?;
          self.type_top_down(type_.clone(), *value)?
        } else {
          self.type_top_down(value.type_.clone(), *value)?
        };
        n::Declaration {
          name: clean_name,
          mangle,
          is_constant,
          type_assert,
          value: value.into(),
        }
      },
      n::Block { nodes } => {
        let nodes = nodes
          .into_iter()
          .map(|n| self.type_top_down(n.type_.clone(), n))
          .try_collect::<Vec<_>>()?;
        n::Block { nodes }
      },
      n::Remainder { node } => n::Remainder {
        node: self.type_top_down(node.type_.clone(), *node)?.into(),
      },
      n::Function {
        mangle,
        arguments,
        nodes,
      } => {
        let Type::Function { return_type, .. } = node.type_ else {
          panic!("This should never happen")
        };
        let nodes =
          self.type_top_down((*return_type).unwrap_type_name()?, *nodes)?;
        n::Function {
          mangle,
          arguments,
          nodes: nodes.into(),
        }
      },
      n::Loop {
        names,
        initials,
        body,
      } => {
        let initials = initials
          .into_iter()
          .map(|n| self.type_top_down(n.type_.clone(), n))
          .try_collect::<Vec<_>>()?;
        if initials.len() > 1 {
          return error!(
            "Multiple loop parameters are not currently supported"
          );
        }
        let loop_expects = initials
          .first()
          .map(|n| n.type_.clone())
          .unwrap_or(Primitive::nothing.promote());
        let body = self.type_top_down(loop_expects, *body)?;
        n::Loop {
          names,
          initials,
          body: body.into(),
        }
      },
      n::Break { expr } => {
        let expr = self.type_top_down(expr.type_.clone(), *expr)?;
        n::Break { expr: expr.into() }
      },
    };
    Ok(Node {
      span: node.span,
      type_: expects,
      kind,
    })
  }
}
