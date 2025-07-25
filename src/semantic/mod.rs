mod check;
mod inference;
mod unification;

use check::*;
use inference::*;
use unification::*;

use std::collections::{HashMap, HashSet};

use crate::{ir::*, lint::*, operator::*};

pub struct Environment {
  scheme_map: HashMap<Mangle, bool>,
  value_map: HashMap<Mangle, TypeRef>,
  type_map: HashMap<Mangle, TypeRef>,
  type_variable: TypeVariable,
}

impl Environment {
  pub fn new() -> Self {
    let mut type_map = HashMap::new();
    Type::primitives().into_iter().for_each(|(prim, name)| {
      let mangle = mangle_builtin(name);
      type_map.insert(mangle.clone(), prim);
    });
    Self {
      scheme_map: HashMap::new(),
      value_map: HashMap::new(),
      type_map,
      type_variable: 0,
    }
  }

  pub fn fresh_type_var(&mut self) -> TypeRef {
    let tv = self.type_variable;
    self.type_variable += 1;
    Type::TypeVariable(tv).to_ref()
  }

  pub fn reset_type_variable(&mut self) {
    self.type_variable = 0;
  }

  fn map_fresh_type_variables(
    &mut self,
    t: &TypeRef,
    map: &mut HashMap<TypeVariable, TypeVariable>,
  ) {
    match &*(t.borrow()) {
      Type::Weak(_)
      | Type::Any
      | Type::_ClosureCapture
      | Type::Unit
      | Type::Integer
      | Type::Real
      | Type::Boolean
      | Type::String
      | Type::Glyph
      | Type::Type => {},
      Type::TypeVariable(tv) => {
        if !map.contains_key(tv) {
          let new_tv = self.type_variable;
          self.type_variable += 1;
          map.insert(*tv, new_tv);
        }
      },
      Type::Sum {
        variant_types: items,
        ..
      }
      | Type::Product(items)
      | Type::Struct {
        member_types: items,
        ..
      } => items
        .into_iter()
        .for_each(|t| self.map_fresh_type_variables(&t, map)),
      Type::Function(param_type, return_type) => {
        self.map_fresh_type_variables(param_type, map);
        self.map_fresh_type_variables(return_type, map);
      },
    }
  }

  pub fn define_type(&mut self, mangle: Mangle, type_: TypeRef) {
    self.type_map.insert(mangle, type_);
  }

  pub fn get_type(&self, mangle: &Mangle) -> TypeRef {
    self.type_map.get(mangle).unwrap().clone()
  }

  pub fn get_value_type(&mut self, mangle: &Mangle) -> TypeRef {
    if *self.scheme_map.get(mangle).unwrap() {
      let t = self.value_map.get(mangle).unwrap().clone();
      let mut fresh = HashMap::new();
      self.map_fresh_type_variables(&t, &mut fresh);
      fresh.into_iter().for_each(|(old, new)| {
        t.borrow_mut().substitute(old, &Type::TypeVariable(new))
      });
      t
    } else {
      self.value_map.get(mangle).unwrap().clone()
    }
  }

  pub fn insert_value_type(
    &mut self,
    mangle: Mangle,
    type_: TypeRef,
    let_bound: bool,
  ) {
    self.value_map.insert(mangle.clone(), type_);
    self.scheme_map.insert(mangle, let_bound);
  }
}

#[derive(Debug, Clone)]
pub struct TypeConstraint(TypeRef, TypeRef, Span);

impl std::fmt::Display for TypeConstraint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}) == ({})", self.0.borrow(), self.1.borrow())
  }
}

#[derive(Debug, Clone)]
pub struct Substitution(TypeVariable, TypeRef);

#[derive(Debug, Clone, Default)]
pub struct ModuleInterface {
  pub types: HashMap<String, TypeRef>,
  pub values: HashMap<String, TypeRef>,
}

pub fn type_solve(module: &mut IrModule) -> Result<ModuleInterface> {
  let mut interface = ModuleInterface::default();
  let mut env = Environment::new();
  for item in module.items.clone() {
    match item {
      ModuleItem::Let(name, ir) => {
        let mut constraints = vec![];
        type_inference(module, ir, &mut env, &mut constraints)?;
        let solution = unification(&constraints)?;
        apply_solution(module, ir, solution);
        let second_solution = unification(&check_structs(&env, module, ir)?)?;
        apply_solution(module, ir, second_solution);
        env.insert_value_type(name.clone(), module[ir].type_.clone(), true);
        env.reset_type_variable();
        interface.values.insert(name, module[ir].type_.clone());
      },
      ModuleItem::Type(name, type_) => {
        env.define_type(name.clone(), type_.clone());
        interface.types.insert(name, type_);
      },
      ModuleItem::CompilerBuiltin(name, type_, _) => {
        env.insert_value_type(name, type_, true);
      },
    }
  }
  Ok(interface)
}
