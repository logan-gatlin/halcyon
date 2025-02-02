use std::collections::HashSet;

use crate::{
  err::*,
  error,
  semantic::{Type, ir::Node, primitives::Primitive},
};

use super::{Analyzer, ir::NodeKind};

impl Analyzer {
  pub fn type_top_down(&mut self, expects: Type, node: Node) -> Result<Node> {
    use NodeKind as n;
    let span = &node.span;
    if node.type_ != expects {
      return error!("Expected type '{expects}', found '{}'", node.type_)
        .span(span);
    }
    let kind = match node.kind {
      n::Immediate(im) => n::Immediate(im),
      n::Identifier {
        name,
        global,
        mangle,
      } => n::Identifier {
        name,
        global,
        mangle,
      },
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
      n::BinaryOp {
        op,
        opdef,
        left,
        right,
      } => {
        let left = self.type_top_down(left.type_.clone(), *left)?;
        let right = self.type_top_down(right.type_.clone(), *right)?;
        n::BinaryOp {
          op,
          opdef,
          left: left.into(),
          right: right.into(),
        }
      },
      n::UnaryOp { op, opdef, child } => {
        let child = self.type_top_down(child.type_.clone(), *child)?;
        n::UnaryOp {
          op,
          opdef,
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
        global,
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
          global,
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
        param_mangles: arguments,
        nodes,
      } => {
        let Type::Function { return_type, .. } = node.type_ else {
          panic!("This should never happen")
        };
        let nodes =
          self.type_top_down((*return_type).unwrap_type_name()?, *nodes)?;
        n::Function {
          param_mangles: arguments,
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
