use super::{
  Analyzer,
  ir::{Node, NodeKind},
};
use crate::{
  err::*,
  error,
  semantic::{Type, primitives::Primitive},
};
use std::collections::HashSet;

impl Analyzer {
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
      n::BinaryOp {
        op, left, right, ..
      } => {
        let left = self.type_bottom_up(*left)?;
        let right = self.type_bottom_up(*right)?;
        let opdef = self
          .op_table
          .try_binary(op, &left.type_, &right.type_)
          .span(span)?;
        node.type_ = opdef.produces.clone();
        n::BinaryOp {
          op,
          opdef,
          left: left.into(),
          right: right.into(),
        }
      },
      n::UnaryOp { op, child, .. } => {
        let opdef = self.op_table.try_unary(op, &child.type_).span(span)?;
        node.type_ = opdef.produces.clone();
        let child = self.type_bottom_up(*child)?.into();
        n::UnaryOp { op, opdef, child }
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
        global,
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
          global,
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
        param_mangles: arguments,
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
          param_mangles: arguments,
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
}
