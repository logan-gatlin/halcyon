use std::collections::{HashMap, HashSet};

use super::{
  Analyzer, Mangle, Type,
  ir::{Node, NodeKind},
  operators::OpDef,
};
use crate::{BinaryOp, UnaryOp, err::*, error};

pub fn parse_int_literal(value: &str, base: u32) -> Result<i64> {
  i64::from_str_radix(value, base)
    .reason(format!("Failed to parse integer literal '{value}'"))
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
  String {
    address: usize,
    length: usize,
  },
  Glyph(char),
  Struct {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
  },
}

impl Node {
  pub fn constant_evaluate(&self) -> Result<ConstValue> {
    match &self.kind {
      NodeKind::Immediate(immediate) => Ok(immediate.clone()),
      NodeKind::StructLiteral { names, values } => {
        let Type::Struct {
          member_names: ordered_names,
          ..
        } = &self.type_
        else {
          panic!("Struct does not have struct type");
        };
        let mut sorted_names = vec![];
        let mut sorted_values = vec![];
        for name in ordered_names {
          let pos = names.iter().position(|n| n == name).unwrap();
          sorted_names.push(names[pos].clone());
          let val = values[pos].constant_evaluate()?;
          sorted_values.push(val);
        }
        Ok(ConstValue::Struct {
          member_names: sorted_names,
          member_values: sorted_values,
        })
      },
      _ => error!("Only literals are allowed in constant expressions for now")
        .span(&self.span),
    }
  }
}
