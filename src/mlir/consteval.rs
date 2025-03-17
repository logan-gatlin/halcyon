use std::collections::HashSet;

use crate::{hlir::*, lint::*};

use super::*;

impl Solver {
  pub(super) fn pop(&mut self) -> ConstValue {
    match self.value_stack.pop() {
      Some(StackValue::Value(v)) => v,
      Some(StackValue::OldValue(_)) => {
        panic!()
      }
      Some(StackValue::Guard) => {
        self.value_stack.push(StackValue::Guard);
        ConstValue::Nothing
      }
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
    while let Some(StackValue::OldValue((mangle, val))) = self.value_stack.last() {
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

  pub fn consteval_module(&mut self) -> Result<()> {
    let mut deps = self
      .dependency_graph
      .clone()
      .into_iter()
      .collect::<Vec<_>>();
    deps.sort_unstable_by(|(_, deps1), (_, deps2)| deps1.len().cmp(&deps2.len()));
    let mut resolved = HashSet::new();
    // Iterate constants from least to most dependencies
    for (mangle, _) in deps {
      self.rt_value_map.clear();
      self.value_stack.clear();
      resolved.insert(mangle.clone());
      // If constant is a function, type it
      if let Some(func) = self.module.functions.get(&mangle).cloned() {
        let mut param_types = vec![];
        for m in func.parameter_mangles.into_iter() {
          let block = self.module.type_assertions.get(&m).unwrap();
          self.evaluate_block(*block)?;
          let top = self.pop().clone();
          let ConstValue::Type(t) = top else {
            return Err(lint_nospan(TypeLint::TypeMismatch))
              .context("type")
              .context(format!("{}", self.type_of_const(&top)));
          };
          param_types.push(t);
        }
        let return_type = if let Some(r) = func.returns_mangle {
          let block = self.module.type_assertions.get(&r).unwrap();
          self.evaluate_block(*block)?;
          let top = self.pop().clone();
          let ConstValue::Type(t) = top else {
            return Err(lint_nospan(TypeLint::TypeMismatch))
              .context("type")
              .context(format!("{}", self.type_of_const(&top)));
          };
          t
        } else {
          Primitive::nothing.promote()
        };
        self.type_map.insert(func.mangle, Type::Function {
          param_types,
          return_type: return_type.into(),
        });
      }
      // Otherwise, resolve the constant
      else if let Some(const_block) = self.module.constants.get(&mangle).cloned() {
        self.evaluate_block(const_block)?;
        let value = self.pop();
        self
          .type_map
          .insert(mangle.clone(), self.type_of_const(&value));
      } else if let Some(assert_block) = self.module.type_assertions.get(&mangle).cloned() {
        self.evaluate_block(assert_block)?;
        let top = self.pop();
        let ConstValue::Type(assert) = top else {
          return Err(lint_nospan(TypeLint::TypeMismatch))
            .context("type")
            .context(format!("{}", self.type_of_const(&top)));
        };
        if let Some(existing_type) = self.type_map.get(&mangle) {
          if existing_type != &assert {
            return Err(lint_nospan(TypeLint::TypeMismatch))
              .context(format!("{assert}"))
              .context(format!("{existing_type}"));
          }
        }
        self.assert_map.insert(mangle, assert);
      }
    }
    Ok(())
  }

  pub fn type_of_const(&self, val: &ConstValue) -> Type {
    use Primitive as p;
    match val {
      ConstValue::Nothing => p::nothing.promote(),
      ConstValue::Never => p::unreachable.promote(),
      ConstValue::Integer(_) => p::integer.promote(),
      ConstValue::Real(_) => p::real.promote(),
      ConstValue::Boolean(_) => p::boolean.promote(),
      ConstValue::String { .. } => p::string.promote(),
      ConstValue::Glyph(_) => p::glyph.promote(),
      ConstValue::Function(mangle) => self.type_map.get(mangle).unwrap().clone(),
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
      }
      ConstValue::Type(_) => Type::Type,
    }
  }
}
