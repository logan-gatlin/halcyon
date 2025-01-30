use std::collections::{HashMap, HashSet};

use super::{
  ir::{Node, NodeKind},
  operators::OpDef,
  Analyzer, Mangle,
};
use crate::{err::*, error, BinaryOp, UnaryOp};

pub fn parse_int_literal(value: &str, base: u32) -> Result<i64> {
  i64::from_str_radix(value, base).reason(format!("Failed to parse integer literal '{value}'"))
}

pub fn parse_real_literal(value: &str) -> Result<f64> {
  value
    .parse()
    .ok()
    .reason(format!("Failed to parse real literal '{value}'"))
}

#[derive(Clone, Debug)]
pub enum ConstValue {
  Nothing,
  Integer(i64),
  Real(f64),
  Boolean(bool),
  String(String),
  Glyph(char),
  Struct {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
  },
  BinaryProduct {
    left: Box<ConstValue>,
    right: Box<ConstValue>,
    op: OpDef,
  },
  UnaryProduct {
    on: Box<ConstValue>,
    op: OpDef,
  },
  Field {
    namespace: Box<ConstValue>,
    index: String,
  },
  Identifier(Mangle),
  Function {
    mangle: Mangle,
    param_mangles: Vec<Mangle>,
    nodes: Box<Node>,
  },
}

impl Analyzer {
  pub fn consteval_pass(&mut self, scope: Vec<Node>) -> Result<Vec<Node>> {
    let (mangles, values): (Vec<_>, Vec<_>) = scope
      .clone()
      .into_iter()
      .flat_map(|n| {
        if let NodeKind::Declaration {
          mangle,
          is_constant: true,
          value,
          ..
        } = n.kind
        {
          Some((mangle, value))
        } else {
          None
        }
      })
      .unzip();
    let values = values
      .into_iter()
      .map(|v| self.consteval(&v))
      .try_collect::<Vec<_>>()?;
    let mut val_map: HashMap<Mangle, ConstValue> =
      mangles.into_iter().zip(values.into_iter()).collect();
    for (key, val) in val_map.clone().into_iter() {
      val_map.insert(
        key.clone(),
        Self::resolve(&val, &val_map, HashSet::from([key]))?,
      );
    }
    Ok(scope)
  }

  fn resolve(
    value: &ConstValue,
    map: &HashMap<Mangle, ConstValue>,
    mut history: HashSet<Mangle>,
  ) -> Result<ConstValue> {
    use ConstValue::*;
    match value {
      Function { .. } | Nothing | Integer(_) | Real(_) | Boolean(_) | String(_) | Glyph(_) => {
        Ok(value.clone())
      }
      Struct {
        member_names,
        member_values,
      } => Ok(Struct {
        member_names: member_names.clone(),
        member_values: member_values
          .iter()
          .map(|v| Self::resolve(v, map, history.clone()))
          .try_collect()?,
      }),
      BinaryProduct { left, right, op } => todo!(),
      UnaryProduct { on, op } => todo!(),
      Field { namespace, index } => {
        let ConstValue::Struct {
          member_names,
          member_values,
        } = Self::resolve(namespace, map, history)?
        else {
          panic!("Non-struct index failed to be caught by type checker")
        };
        Ok(
          member_names
            .into_iter()
            .zip(member_values.into_iter())
            .collect::<HashMap<_, _>>()
            .get(index)
            .unwrap()
            .clone(),
        )
      }
      Identifier(mangle) => {
        if !history.insert(mangle.clone()) {
          return error!("Detected circular dependency when evaluating constant expression");
        }
        Self::resolve(map.get(mangle).unwrap(), map, history)
      }
    }
  }

  fn consteval(&mut self, node: &Node) -> Result<ConstValue> {
    let err = |e: &str| error!("Cannot evaluate {e} expression at compile time");
    use NodeKind::*;
    match &node.kind {
      Declaration {
        name,
        global,
        mangle,
        is_constant,
        type_assert,
        value,
      } => err("declaration"),
      If {
        predicate,
        then,
        else_,
      } => err("if"),
      Loop { .. } => err("loop").span(&node.span),
      Break { .. } => err("break").span(&node.span),
      Call {
        mangle,
        callee,
        params,
      } => err("call"),
      Block { nodes } => err("block"),
      Remainder { node } => err("remainder"),
      Immediate(immediate) => Ok(match immediate {
        crate::Immediate::Unit => ConstValue::Nothing,
        crate::Immediate::Integer(val, base) => {
          ConstValue::Integer(parse_int_literal(&val, *base as u32).span(&node.span)?)
        }
        crate::Immediate::Real(val) => ConstValue::Real(parse_real_literal(&val).span(&node.span)?),
        crate::Immediate::String(val) => ConstValue::String(val.clone()),
        crate::Immediate::Glyph(val) => ConstValue::Glyph(*val),
        crate::Immediate::Boolean(val) => ConstValue::Boolean(*val),
      }),
      Identifier { mangle, .. } => Ok(ConstValue::Identifier(mangle.clone())),
      StructLiteral { names, values } => {
        let member_names = names.clone();
        let member_values = values
          .into_iter()
          .map(|v| self.consteval(v))
          .try_collect::<Vec<_>>()?;
        Ok(ConstValue::Struct {
          member_names,
          member_values,
        })
      }
      BinaryOp {
        opdef, left, right, ..
      } => Ok(ConstValue::BinaryProduct {
        left: self.consteval(left)?.into(),
        right: self.consteval(right)?.into(),
        op: opdef.clone(),
      }),
      UnaryOp { opdef, child, .. } => Ok(ConstValue::UnaryProduct {
        on: self.consteval(child)?.into(),
        op: opdef.clone(),
      }),
      Field { namespace, index } => Ok(ConstValue::Field {
        index: index.clone(),
        namespace: self.consteval(namespace)?.into(),
      }),
      Function {
        mangle,
        param_mangles,
        nodes,
      } => Ok(ConstValue::Function {
        mangle: mangle.clone(),
        param_mangles: param_mangles.clone(),
        nodes: nodes.clone(),
      }),
    }
  }
}
