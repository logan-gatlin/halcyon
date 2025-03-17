use std::collections::{HashMap, HashSet};

use crate::hlir::*;
use crate::lint::*;

use super::*;

pub const RECURSION_LIMIT: usize = 0x100;
pub const LOCAL_EVAL_LIMIT: usize = 0x1000;
pub const GLOBAL_EVAL_LIMIT: usize = 0x10000;

#[derive(Debug, Clone)]
pub(super) enum StackValue {
  Value(ConstValue),
  OldValue((Mangle, ConstValue)),
  Guard,
}
#[derive(Debug, Clone)]
pub(super) struct ReturnAddress {
  pub block: usize,
  pub ip: usize,
  pub expected_type: Type,
}

#[derive(Debug, Clone)]
pub struct Solver {
  pub module: Module,
  pub dependency_graph: HashMap<Mangle, HashSet<Mangle>>,
  pub type_map: HashMap<Mangle, Type>,
  pub assert_map: HashMap<Mangle, Type>,
  pub const_value_map: HashMap<Mangle, ConstValue>,
  pub rt_value_map: HashMap<Mangle, ConstValue>,
  pub(super) value_stack: Vec<StackValue>,
  pub(super) control_stack: Vec<ReturnAddress>,
  pub(super) ip: usize,
  pub(super) block: usize,
}

#[derive(Debug, Clone)]
pub struct Solution {
  pub constants: HashMap<Mangle, ConstValue>,
  pub assertions: HashMap<Mangle, Type>,
}

impl Solver {
  pub fn solve(module: Module) -> Result<Solution> {
    let mut this = Self::new(module);
    this.consteval_module()?;
    // Account for implicit function type assertions
    for (name, type_) in &this.type_map {
      if let Type::Function { .. } = type_ {
        this.assert_map.insert(name.clone(), type_.clone());
      }
    }
    this
      .assert_map
      .insert(mangle_builtin("print_string"), Type::Function {
        param_types: vec![Primitive::string.promote()],
        return_type: Box::new(Primitive::nothing.promote()),
      });
    Ok(Solution {
      constants: this.const_value_map.clone(),
      assertions: this.assert_map.clone(),
    })
  }

  fn new(module: Module) -> Self {
    let assertion_dependencies = module.type_assertions.iter().map(|(mangle, assert)| {
      let mut deps = module.find_type_dependencies(*assert);
      deps.remove(mangle);
      (mangle.clone(), deps)
    });
    let function_dependencies = module.functions.iter().map(|(mangle, func)| {
      let mut deps = HashSet::new();
      for p in &func.parameter_mangles {
        deps = deps
          .union(&module.find_type_dependencies(*module.type_assertions.get(p).unwrap()))
          .cloned()
          .collect();
      }
      if let Some(r) = &func.returns_mangle {
        deps = deps
          .union(&module.find_type_dependencies(*module.type_assertions.get(r).unwrap()))
          .cloned()
          .collect();
      }
      deps.remove(mangle);
      (mangle.clone(), deps)
    });
    let constant_dependencies = module.constants.iter().map(|(mangle, ptr)| {
      let mut deps = module.find_type_dependencies(*ptr);
      deps.remove(mangle);
      (mangle.clone(), deps)
    });
    let dependency_graph = assertion_dependencies
      .chain(function_dependencies)
      .chain(constant_dependencies)
      .collect();

    // Prelude
    let mut const_value_map = HashMap::new();
    let mut type_map = HashMap::new();
    for p in Primitive::ALL {
      let mangle = mangle_builtin(p.to_string());
      const_value_map.insert(mangle.clone(), ConstValue::Type(p.promote()));
      type_map.insert(mangle, Type::Type);
    }
    for b in Builtin::ALL {
      let mangle = mangle_builtin(b.to_string());
      const_value_map.insert(mangle.clone(), b.value());
      type_map.insert(mangle, b.type_());
    }
    Self {
      module,
      dependency_graph,
      type_map,
      const_value_map,
      assert_map: Default::default(),
      rt_value_map: Default::default(),
      value_stack: Default::default(),
      control_stack: Default::default(),
      ip: 0,
      block: 0,
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
        Block::Terminal | Block::Unreachable => {}
        Block::Basic { body, next } => {
          to_visit.push(*next);
          body.into_iter().for_each(|ir| {
            if let MlIrKind::Get(ident) = &ir.kind
              && let Some(block) = self.constants.get(ident)
            {
              deps.insert(ident.clone());
              to_visit.push(*block);
            } else if let MlIrKind::Const(ConstValue::Function(mangle)) = &ir.kind {
              let func = self.functions.get(mangle).unwrap();
              for p in &func.parameter_mangles {
                to_visit.push(*self.type_assertions.get(p).unwrap());
              }
              if let Some(mangle) = &func.returns_mangle {
                to_visit.push(*self.type_assertions.get(mangle).unwrap());
              }
              deps.insert(mangle.clone());
              to_visit.push(func.block);
            }
          });
        }
        Block::Branch {
          when_true,
          when_false,
          ..
        } => {
          to_visit.push(*when_true);
          to_visit.push(*when_false);
        }
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
