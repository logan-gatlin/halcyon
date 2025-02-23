use std::collections::{HashMap, HashSet};

use crate::naming::{Mangle, mangle_builtin};

use super::{
  Block, ConstValue, IrKind, IrPtr, Module,
  types::{Primitive, Type},
};

#[derive(Debug, Clone)]
pub(super) enum StackValue {
  Value(ConstValue),
  OldValue((Mangle, ConstValue)),
  Guard,
}

#[derive(Debug, Clone)]
pub struct Solver {
  pub module: Module,
  pub dependency_graph: HashMap<Mangle, HashSet<Mangle>>,
  pub type_map: HashMap<Mangle, Type>,
  pub const_value_map: HashMap<Mangle, ConstValue>,
  pub rt_value_map: HashMap<Mangle, ConstValue>,
  pub(super) value_stack: Vec<StackValue>,
}

impl Solver {
  pub fn new(module: Module) -> Self {
    let function_dependencies =
      module.functions.iter().map(|(mangle, func)| {
        let mut deps = HashSet::new();
        for p in &func.parameter_mangles {
          deps = deps
            .union(
              &module
                .find_type_dependencies(*module.parameters.get(p).unwrap()),
            )
            .cloned()
            .collect();
        }
        if let Some(r) = &func.returns_mangle {
          deps = deps
            .union(
              &module
                .find_type_dependencies(*module.parameters.get(r).unwrap()),
            )
            .cloned()
            .collect();
        }

        (mangle.clone(), deps)
      });
    let constant_dependencies = module.constants.iter().map(|(mangle, ptr)| {
      (mangle.clone(), module.find_type_dependencies(*ptr))
    });
    let dependency_graph =
      function_dependencies.chain(constant_dependencies).collect();

    // Prelude
    let mut const_value_map = HashMap::new();
    let mut type_map = HashMap::new();
    for p in Primitive::ALL {
      let mangle = mangle_builtin(p.to_string());
      const_value_map.insert(mangle.clone(), ConstValue::Type(p.promote()));
      type_map.insert(mangle, Type::Type);
    }
    let type_mangle = mangle_builtin(format!("{}", Type::Type));
    const_value_map.insert(type_mangle.clone(), ConstValue::Type(Type::Type));
    type_map.insert(type_mangle, Type::Type);
    Self {
      module,
      dependency_graph,
      type_map,
      const_value_map,
      rt_value_map: Default::default(),
      value_stack: Default::default(),
    }
  }
}

impl Module {
  /// Depth first search that finds all direct and
  /// indirect dependencies needed to determine the
  /// type of every expression in the block,
  /// and every block it touches
  fn find_type_dependencies(&self, block: IrPtr) -> HashSet<Mangle> {
    let mut deps = HashSet::new();
    let mut visited = HashSet::new();
    let mut to_visit = vec![];
    let mut current_block = block;
    loop {
      visited.insert(current_block);
      match &self.blocks[current_block] {
        Block::Terminal | Block::Unreachable => {},
        Block::Basic { body, next, typed } => {
          to_visit.push(*next);
          body.into_iter().for_each(|ir| {
            if let IrKind::Get(ident) = &ir.kind
              && let Some(block) = self.constants.get(ident)
            {
              deps.insert(ident.clone());
              to_visit.push(*block);
            } else if let IrKind::Const(ConstValue::Function(mangle)) = &ir.kind
            {
              let func = self.functions.get(mangle).unwrap();
              for p in &func.parameter_mangles {
                to_visit.push(*self.parameters.get(p).unwrap());
              }
              if let Some(mangle) = &func.returns_mangle {
                to_visit.push(*self.parameters.get(mangle).unwrap());
              }
              deps.insert(mangle.clone());
              //to_visit.push(func.block);
            }
          });
        },
        Block::Branch {
          when_true,
          when_false,
          ..
        } => {
          to_visit.push(*when_true);
          to_visit.push(*when_false);
        },
      }
      loop {
        if let Some(ptr) = to_visit.pop() {
          if !visited.contains(&ptr) {
            current_block = ptr;
            break;
          }
        } else {
          return deps;
        }
      }
    }
  }
}
