mod inference;
mod unification;

use inference::*;
use unification::*;

use std::collections::{HashMap, HashSet};

use crate::{hlir::*, lint::*, operator::*};

pub struct Environment {
  type_map: HashMap<Mangle, Type>,
  value_map: HashMap<Mangle, ConstValue>,
}

impl Environment {
  pub fn new() -> Self {
    let mut type_map = HashMap::new();
    let mut value_map = HashMap::new();
    for bt in Builtin::ALL {
      type_map.insert(bt.to_mangle(), bt.type_());
      value_map.insert(bt.to_mangle(), bt.value());
    }
    Self {
      type_map,
      value_map,
    }
  }

  pub fn get_type(&self, mangle: &Mangle) -> &Type {
    self.type_map.get(mangle).unwrap()
  }

  pub fn insert_type(&mut self, mangle: Mangle, type_: Type) {
    self.type_map.insert(mangle, type_);
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

#[derive(Debug, Clone)]
pub struct Substitution(TypeVariable, Type);

pub fn type_solve(module: &mut HlIrModule) -> Result<()> {
  let mut type_var = 0;
  let mut new_type_var = move || {
    type_var += 1;
    Type::TypeVariable(type_var - 1)
  };

  let mut constraints = vec![];
  type_inference(
    module,
    0,
    &mut Environment::new(),
    &mut new_type_var,
    &mut constraints,
  )?;
  let solution = unification(&constraints)?;
  apply_solution(module, 0, solution);
  Ok(())
}

pub fn parse_type(nodes: &HlIrModule, node: IrPtr, env: &Environment) -> Result<Type> {
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
      BinaryOp::Plus => parse_type(nodes, *left, env)? + parse_type(nodes, *right, env)?,
      BinaryOp::Star => parse_type(nodes, *left, env)? * parse_type(nodes, *right, env)?,
      BinaryOp::Arrow => Type::Function {
        param_types: vec![parse_type(nodes, *left, env)?],
        return_type: parse_type(nodes, *right, env)?.into(),
      },
      _ => {
        return Err(lint_nospan(TypeLint::BinaryOpUndefined))
          .context(format!("{op}"))
          .context(format!("{}", Type::Type))
          .context(format!("{}", Type::Type))
          .span(span);
      }
    },
    Unary { op, .. } => {
      return Err(lint_nospan(TypeLint::UnaryOpUndefined))
        .context(format!("{op}"))
        .context(format!("{}", Type::Type))
        .span(span);
    }
    Immediate(ConstValue::Nothing) => Type::Unit,
    _ => return Err(lint_nospan(TypeLint::NotAType)).span(span),
  })
}
