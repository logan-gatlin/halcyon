use super::Compiler;
use crate::semantic::ir::{Node, NodeKind};

// Moves all functions to global scope, leave anonymous
// identifiers in their place. This is because WAST does not
// allow for nested function definitions

impl Compiler {
  pub fn flatten_functions(
    &self,
    mut node: Node,
    global_scope: &mut Vec<Node>,
    depth: usize,
  ) -> Node {
    use NodeKind::*;
    node.kind = match node.kind {
      Immediate(_) => node.kind,
      Identifier { .. } => node.kind,
      StructLiteral { names, values } => StructLiteral {
        names,
        values: values
          .into_iter()
          .map(|v| self.flatten_functions(v, global_scope, depth))
          .collect(),
      },
      BinaryOp {
        op,
        opdef,
        left,
        right,
      } => BinaryOp {
        op,
        opdef,
        left: self.flatten_functions(*left, global_scope, depth).into(),
        right: self.flatten_functions(*right, global_scope, depth).into(),
      },
      UnaryOp { op, opdef, child } => UnaryOp {
        op,
        opdef,
        child: self.flatten_functions(*child, global_scope, depth).into(),
      },
      Field { namespace, index } => Field {
        namespace: self
          .flatten_functions(*namespace, global_scope, depth)
          .into(),
        index,
      },
      If {
        predicate,
        then,
        else_,
      } => If {
        predicate: self
          .flatten_functions(*predicate, global_scope, depth)
          .into(),
        then: self.flatten_functions(*then, global_scope, depth).into(),
        else_: if let Some(else_) = else_ {
          Some(self.flatten_functions(*else_, global_scope, depth).into())
        } else {
          None
        },
      },
      Call {
        mangle,
        callee,
        params,
      } => Call {
        mangle,
        callee: self.flatten_functions(*callee, global_scope, depth).into(),
        params: params
          .into_iter()
          .map(|p| self.flatten_functions(p, global_scope, depth))
          .collect(),
      },
      Function {
        mangle,
        param_mangles: arguments,
        nodes,
      } => {
        let nodes = self.flatten_functions(*nodes, global_scope, depth + 1);
        if depth != 0 {
          let func = Node {
            kind: Function {
              mangle: mangle.clone(),
              param_mangles: arguments,
              nodes: nodes.into(),
            },
            span: node.span,
            type_: node.type_.clone(),
          };
          global_scope.push(func);
          Identifier {
            name: "anonymous function".into(),
            mangle,
            global: true,
          }
        } else {
          Function {
            mangle,
            param_mangles: arguments,
            nodes: nodes.into(),
          }
        }
      },
      Declaration {
        name,
        global,
        mangle,
        is_constant,
        type_assert,
        value,
      } => Declaration {
        name,
        global,
        mangle,
        is_constant,
        type_assert,
        value: self.flatten_functions(*value, global_scope, depth).into(),
      },
      Block { nodes } => Block {
        nodes: nodes
          .into_iter()
          .map(|n| self.flatten_functions(n, global_scope, depth))
          .collect(),
      },
      Remainder { node } => Remainder {
        node: self.flatten_functions(*node, global_scope, depth).into(),
      },
      Loop {
        names,
        initials,
        body,
      } => Loop {
        names,
        initials: initials
          .into_iter()
          .map(|i| self.flatten_functions(i, global_scope, depth))
          .collect(),
        body,
      },
      Break { expr } => Break {
        expr: self.flatten_functions(*expr, global_scope, depth).into(),
      },
    };
    node
  }
}
