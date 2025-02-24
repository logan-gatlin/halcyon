use std::collections::HashSet;

use crate::{err::*, error, ir::types::Primitive};

use super::solver::{Solver, StackValue};
use super::{Block, ConstValue, IrPtr, types::Type};

impl Solver {
  pub(super) fn pop(&mut self) -> ConstValue {
    match self.value_stack.pop() {
      Some(StackValue::Value(v)) => v,
      Some(StackValue::OldValue(v)) => {
        panic!()
      },
      Some(StackValue::Guard) => {
        self.value_stack.push(StackValue::Guard);
        ConstValue::Nothing
      },
      None => ConstValue::Nothing,
    }
  }

  pub(super) fn push(&mut self, val: ConstValue) {
    self.value_stack.push(StackValue::Value(val));
  }

  pub(super) fn save_state(&mut self) {
    self.value_stack.extend(
      self
        .rt_value_map
        .clone()
        .into_iter()
        .map(|v| StackValue::OldValue(v)),
    );
    self.rt_value_map.clear();
  }

  pub(super) fn retrieve_state(&mut self) {
    self.rt_value_map.clear();
    while let Some(StackValue::OldValue((mangle, val))) =
      self.value_stack.last()
    {
      self.rt_value_map.insert(mangle.clone(), val.clone());
      self.value_stack.pop();
    }
  }

  pub(super) fn start_scope_v(&mut self) {
    self.value_stack.push(StackValue::Guard);
  }

  pub(super) fn end_scope_v(&mut self) {
    let mut last = ConstValue::Nothing;
    while let Some(StackValue::Value(v)) = self.value_stack.last() {
      last = v.clone();
      self.value_stack.pop();
    }
    if let Some(StackValue::Guard) = self.value_stack.last() {
      self.value_stack.pop();
    } else {
      panic!("Unguard without corresponding guard");
    }
    self.push(last);
  }

  pub fn conseval_module(&mut self) -> Result<()> {
    let mut deps = self
      .dependency_graph
      .clone()
      .into_iter()
      .collect::<Vec<_>>();
    deps
      .sort_unstable_by(|(_, deps1), (_, deps2)| deps1.len().cmp(&deps2.len()));
    let mut resolved = HashSet::new();
    println!("{deps:#?}");
    // Iterate constants from least to most dependencies
    for (mangle, deps) in deps {
      self.rt_value_map.clear();
      self.value_stack.clear();
      resolved.insert(mangle.clone());
      // If constant is a function, type it
      if let Some(func) = self.module.functions.get(&mangle).cloned() {
        let mut param_types = vec![];
        for (id, m) in func.parameter_mangles.into_iter().enumerate() {
          let block = self.module.type_assertions.get(&m).unwrap();
          self.evaluate_block(*block)?;
          let ConstValue::Type(t) = self.pop().clone() else {
            return error!(
              "The type assertion for parameter {} is a term, not a type",
              id + 1
            );
          };
          param_types.push(t);
        }
        let return_type = if let Some(r) = func.returns_mangle {
          let block = self.module.type_assertions.get(&r).unwrap();
          self.evaluate_block(*block)?;
          let ConstValue::Type(t) = self.pop().clone() else {
            return error!(
              "The type assertion for this function's return type is a term, \
               not a type",
            );
          };
          t
        } else {
          Primitive::nothing.promote()
        };
        self.type_map.insert(
          func.mangle,
          Type::Function {
            param_types,
            return_type: return_type.into(),
          },
        );
      }
      // Otherwise, resolve the constant
      else if let Some(const_block) =
        self.module.constants.get(&mangle).cloned()
      {
        self.evaluate_block(const_block)?;
        let value = self.pop();
        self
          .type_map
          .insert(mangle.clone(), self.type_of_const(&value));
      } else if let Some(assert_block) =
        self.module.type_assertions.get(&mangle).cloned()
      {
        self.evaluate_block(assert_block)?;
        let top = self.pop();
        let ConstValue::Type(assert) = top else {
          return error!(
            "Type assertion expects type, found value '{top}' instead"
          );
        };
        if let Some(existing_type) = self.type_map.get(&mangle) {
          if existing_type != &assert {
            return error!(
              "Asserted type is '{assert}', but declaration was assigned \
               '{existing_type}'"
            );
          }
        }
        self.assert_map.insert(mangle, assert);
      }
    }
    println!(
      "{:#?}",
      self
        .assert_map
        .iter()
        .filter(|n| !n.0.starts_with("_"))
        .collect::<Vec<_>>()
    );
    Ok(())
  }

  pub fn type_of_const(&self, val: &ConstValue) -> Type {
    use Primitive as p;
    match val {
      ConstValue::Nothing => p::nothing.promote(),
      ConstValue::Integer(_) => p::integer.promote(),
      ConstValue::Real(_) => p::real.promote(),
      ConstValue::Boolean(_) => p::boolean.promote(),
      ConstValue::String {
        virtual_address: address,
        length,
      } => p::string.promote(),
      ConstValue::Glyph(_) => p::glyph.promote(),
      ConstValue::Function(mangle) => {
        self.type_map.get(mangle).unwrap().clone()
      },
      ConstValue::StructLiteral {
        member_names,
        member_values,
      } => {
        let member_types = member_values
          .into_iter()
          .map(|v| self.type_of_const(v))
          .collect::<Vec<_>>();
        Type::Struct {
          member_names: member_names.clone(),
          member_types,
        }
      },
      ConstValue::Type(_) => Type::Type,
    }
  }

  pub fn check_asserts(&self, mut block: IrPtr) -> Result<()> {
    let mut visited: HashSet<IrPtr> = HashSet::new();
    let mut to_visit = vec![];
    loop {
      visited.insert(block);
      match &self.module.blocks[block] {
        Block::Terminal | Block::Unreachable => {},
        Block::Basic { body, next, typed } => {
          to_visit.push(next);
          for ir in body {}
        },
        Block::Branch {
          span,
          when_true,
          when_false,
        } => {
          to_visit.push(when_true);
          to_visit.push(when_false);
        },
      }
    }
    todo!()
  }
}
