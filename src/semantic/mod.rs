mod inference;
mod unification;

use inference::*;
use unification::*;

use std::collections::{HashMap, HashSet};

use crate::{builtin::Builtin, hlir::*, lint::*, operator::*};

pub struct Environment {
  let_bound_map: HashMap<Mangle, bool>,
  type_map: HashMap<Mangle, Type>,
  value_map: HashMap<Mangle, ConstValue>,
  type_variable: TypeVariable,
}

impl Environment {
  pub fn new() -> Self {
    let mut let_bound_map = HashMap::new();
    let mut type_map = HashMap::new();
    let mut value_map = HashMap::new();
    Type::primitives().into_iter().for_each(|(prim, name)| {
      let mangle = mangle_builtin(name);
      let_bound_map.insert(mangle.clone(), false);
      type_map.insert(mangle.clone(), Type::Type);
      value_map.insert(mangle.clone(), ConstValue::Type(prim));
    });
    Builtin::ALL.into_iter().for_each(|bt| {
      let mangle = bt.get_mangle();
      let_bound_map.insert(mangle.clone(), true);
      type_map.insert(mangle.clone(), bt.get_type());
    });
    Self {
      let_bound_map,
      type_map,
      value_map,
      type_variable: 0,
    }
  }

  pub fn fresh_type_var(&mut self) -> Type {
    let tv = self.type_variable;
    self.type_variable += 1;
    Type::TypeVariable(tv)
  }

  fn map_fresh_type_variables(
    &mut self,
    t: &Type,
    map: &mut HashMap<TypeVariable, TypeVariable>,
  ) {
    match t {
      Type::Any
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
      Type::Sum(hash_set) => hash_set
        .into_iter()
        .for_each(|t| self.map_fresh_type_variables(t, map)),
      Type::Product(items)
      | Type::Struct {
        member_types: items,
        ..
      } => items
        .into_iter()
        .for_each(|t| self.map_fresh_type_variables(t, map)),
      Type::Function(param_type, return_type) => {
        self.map_fresh_type_variables(param_type, map);
        self.map_fresh_type_variables(return_type, map);
      },
    }
  }

  pub fn get_type(&mut self, mangle: &Mangle) -> Type {
    if *self.let_bound_map.get(mangle).unwrap() {
      let mut t = self.type_map.get(mangle).unwrap().clone();
      let mut fresh = HashMap::new();
      self.map_fresh_type_variables(&t, &mut fresh);
      fresh
        .into_iter()
        .for_each(|(old, new)| t.substitute(old, &Type::TypeVariable(new)));
      t
    } else {
      self.type_map.get(mangle).unwrap().clone()
    }
  }

  pub fn insert_type(&mut self, mangle: Mangle, type_: Type, let_bound: bool) {
    self.type_map.insert(mangle.clone(), type_);
    self.let_bound_map.insert(mangle, let_bound);
  }

  pub fn get_value(&self, mangle: &Mangle) -> Result<&ConstValue> {
    self
      .value_map
      .get(mangle)
      .ok_or(lint_nospan(TypeLint::NotAvailable))
  }

  pub fn insert_value(&mut self, mangle: Mangle, value: ConstValue) {
    self.value_map.insert(mangle, value);
  }
}

#[derive(Debug, Clone)]
pub struct TypeConstraint(Type, Type, Span);

impl std::fmt::Display for TypeConstraint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}) == ({})", self.0, self.1)
  }
}

#[derive(Debug, Clone)]
pub struct Substitution(TypeVariable, Type);

pub fn type_solve(module: &mut HlIrModule) -> Result<()> {
  let mut env = Environment::new();
  let mut constraints = vec![];
  type_inference(module, 0, &mut env, &mut constraints)?;
  let solution = unification(&constraints)?;
  apply_solution(module, 0, solution);
  Ok(())
}

pub fn parse_type(
  nodes: &HlIrModule,
  node: IrPtr,
  env: &Environment,
) -> Result<Type> {
  let span = nodes[node].span;
  use HlIrKind::*;
  Ok(match &nodes[node].kind {
    Identifier(mangle) => match env.get_value(mangle) {
      Ok(ConstValue::Type(t)) => t.clone(),
      _ => return Err(lint_nospan(TypeLint::NotAvailable)).span(span),
    },
    StructDef {
      field_names,
      field_types,
    } => Type::Struct {
      member_names: field_names.clone(),
      member_types: field_types
        .into_iter()
        .map(|t| parse_type(nodes, *t, env))
        .try_collect()?,
    },
    Binary { op, left, right } => match op {
      BinaryOp::Plus => {
        parse_type(nodes, *left, env)? + parse_type(nodes, *right, env)?
      },
      BinaryOp::Star => {
        parse_type(nodes, *left, env)? * parse_type(nodes, *right, env)?
      },
      BinaryOp::Arrow => Type::Function(
        parse_type(nodes, *left, env)?.into(),
        parse_type(nodes, *right, env)?.into(),
      ),
      _ => {
        return Err(lint_nospan(TypeLint::BinaryOpUndefined))
          .context(format!("{op}"))
          .context(format!("{}", Type::Type))
          .context(format!("{}", Type::Type))
          .span(span);
      },
    },
    Unary { op, .. } => {
      return Err(lint_nospan(TypeLint::UnaryOpUndefined))
        .context(format!("{op}"))
        .context(format!("{}", Type::Type))
        .span(span);
    },
    Immediate(ConstValue::Unit) => Type::Unit,
    _ => return Err(lint_nospan(TypeLint::NotAType)).span(span),
  })
}
