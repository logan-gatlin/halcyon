use std::collections::{HashMap, HashSet};

use super::{Module, Node};
use crate::{
  err::*,
  error,
  semantic::{Mangle, NodeKind},
};

type Dependencies = HashSet<Mangle>;

pub fn type_module(mut module: Module) -> Result<Module> {
  let mut dependency_map: HashMap<Mangle, Dependencies> = HashMap::new();
  // Sort out dependencies for const values
  for (mangle, node) in module.constants.iter() {
    dependency_map.insert(
      mangle.clone(),
      find_dependencies(&module.constants, node, HashSet::new()),
    );
  }
  // Check for circular dependencies
  for (mangle, deps) in dependency_map.iter() {
    if deps.contains(mangle) {
      let span = module.constants.get(mangle).unwrap().span;
      return error!("This expression contains a circular dependency").span(&span);
    }
  }
  // Resolve constant expressions
  let mut v = dependency_map.iter().collect::<Vec<_>>();
  v.sort_by(|(_, set1), (_, set2)| set1.len().cmp(&set2.len()));
  println!("{v:?}");
  Ok(module)
}

/// For a given node, determine the complete set of constant values which must
/// be known to resolve it
fn find_dependencies(
  map: &HashMap<Mangle, Node>,
  current: &Node,
  mut deps: Dependencies,
) -> Dependencies {
  let find = |n, deps| find_dependencies(map, n, deps);
  match &current.kind {
    NodeKind::Loop { initials, body, .. } => {
      for i in initials {
        deps.extend(find(i, deps.clone()));
      }
      deps.extend(find(body, deps.clone()));
    }
    NodeKind::Break { expr } => deps.extend(find(expr, deps.clone())),
    NodeKind::ConstValue(_) => {}
    NodeKind::Identifier {
      mangle,
      constant: global,
      ..
    } => {
      if !deps.contains(mangle) && *global {
        deps.insert(mangle.clone());
        deps.extend(find(map.get(mangle).unwrap(), deps.clone()));
      }
    }
    NodeKind::StructDef { member_types, .. } => {
      for t in member_types {
        deps.extend(find(t, deps.clone()))
      }
    }
    NodeKind::StructLiteral {
      struct_t,
      param_values,
      ..
    } => {
      deps.extend(find(struct_t, deps.clone()));
      for v in param_values {
        deps.extend(find(v, deps.clone()));
      }
    }
    NodeKind::BinaryOp { left, right, .. } => {
      deps.extend(find(left, deps.clone()));
      deps.extend(find(right, deps.clone()));
    }
    NodeKind::UnaryOp { child, .. } => {
      deps.extend(find(child, deps.clone()));
    }
    NodeKind::Field { namespace, index } => {
      deps.extend(find(namespace, deps.clone()));
    }
    NodeKind::If {
      predicate,
      then,
      else_,
    } => {
      deps.extend(find(predicate, deps.clone()));
      deps.extend(find(then, deps.clone()));
      if let Some(else_) = else_ {
        deps.extend(find(else_, deps.clone()));
      }
    }
    NodeKind::Call { callee, params, .. } => {
      deps.extend(find(callee, deps.clone()));
      for param in params {
        deps.extend(find(param, deps.clone()));
      }
    }
    NodeKind::Function {
      param_types,
      returns,
      nodes,
      ..
    } => {
      for type_ in param_types {
        deps.extend(find(type_, deps.clone()));
      }
      deps.extend(find(returns, deps.clone()));
      deps.extend(find(nodes, deps.clone()));
    }
    NodeKind::Declaration {
      type_assert, value, ..
    } => {
      if let Some(type_assert) = type_assert {
        deps.extend(find(type_assert, deps.clone()));
      }
      deps.extend(find(value, deps.clone()));
    }
    NodeKind::Block { nodes } => {
      for n in nodes {
        deps.extend(find(n, deps.clone()));
      }
    }
    NodeKind::Remainder { node } => {
      deps.extend(find(node, deps.clone()));
    }
    NodeKind::Lifted => {}
  }
  deps
}
