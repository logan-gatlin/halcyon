use std::collections::{HashMap, HashSet};

use super::{ConstValue, Module, Node};
use crate::{
  err::*,
  error,
  semantic::{Mangle, NodeKind},
};

type Dependencies = HashSet<Mangle>;

fn find_dependencies(
  map: &HashMap<Mangle, Node>,
  node: &Node,
  set: &mut HashSet<Mangle>,
) {
  node
    .map(&mut |n| {
      if let NodeKind::Identifier {
        mangle,
        constant: true,
        name,
        ..
      } = &n.kind
      {
        if !set.contains(mangle) {
          set.insert(mangle.clone());
          find_dependencies(
            map,
            map
              .get(mangle)
              .reason(format!("Cannot resolve constant identifier '{name}'"))
              .span(&n.span)?,
            set,
          );
        }
      }
      Ok(())
    })
    .unwrap();
}

fn resolve_constants(
  resolved: &HashMap<Mangle, ConstValue>,
  node: &mut Node,
) -> Result<()> {
  if let NodeKind::Identifier {
    constant: true,
    ref mangle,
    ..
  } = node.kind
  {
    node.kind = NodeKind::ConstValue(resolved.get(mangle).unwrap().clone());
  }
  Ok(())
}

fn evaluate_expression(node: &Node) -> ConstValue {
  todo!()
}

pub fn type_module(mut module: Module) -> Result<Module> {
  let mut dependency_map: HashMap<Mangle, Dependencies> = HashMap::new();
  // Sort out dependencies for const values
  for (mangle, node) in module.constants.iter() {
    let mut deps = HashSet::new();
    find_dependencies(&module.constants, node, &mut deps);
    dependency_map.insert(mangle.clone(), deps);
  }
  // Check for circular dependencies
  for (mangle, deps) in dependency_map.iter() {
    if deps.contains(mangle) {
      let span = module.constants.get(mangle).unwrap().span;
      return error!(
        "Found a circular dependency while evaluating this constant expression"
      )
      .span(&span);
    }
  }
  // Resolve constant expressions
  let mut to_resolve = dependency_map.into_iter().collect::<Vec<_>>();
  to_resolve.sort_by(|(_, set1), (_, set2)| set1.len().cmp(&set2.len()));
  println!("{to_resolve:#?}");
  let mut resolved = HashMap::new();
  for (mangle, node) in to_resolve {
    let node = module.constants.get_mut(&mangle).unwrap();
    node
      .map_mut(&mut |n| resolve_constants(&resolved, n))
      .unwrap();
    let const_value = evaluate_expression(node);
    resolved.insert(mangle, const_value);
  }
  Ok(module)
}
