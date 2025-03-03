use crate::{
  assembly::operators::OpTable,
  err::*,
  error,
  ir::{
    ConstValue, IrPtr, Module,
    solver::Solution,
    types::{Primitive, Type},
  },
  naming::{CanonKind, CanonizedModule},
};

use CanonKind::*;
use Primitive as p;

use super::TypeChecker;

impl TypeChecker {
  pub(super) fn new(module: CanonizedModule, solution: Solution) -> Self {
    Self {
      module,
      solution,
      break_stack: vec![],
    }
  }

  pub(super) fn const_type(&self, const_value: &ConstValue) -> Type {
    match const_value {
      ConstValue::Nothing => p::nothing.promote(),
      ConstValue::Never => p::unreachable.promote(),
      ConstValue::Integer(_) => p::integer.promote(),
      ConstValue::Real(_) => p::real.promote(),
      ConstValue::Boolean(_) => p::boolean.promote(),
      ConstValue::String { .. } => p::string.promote(),
      ConstValue::Glyph(_) => p::glyph.promote(),
      ConstValue::Function(mangle) => {
        self.solution.assertions.get(mangle).unwrap().clone()
      },
      ConstValue::StructLiteral {
        member_names,
        member_values,
      } => Type::Struct {
        member_names: member_names.clone(),
        member_types: member_values
          .into_iter()
          .map(|v| self.const_type(v))
          .collect(),
      },
      ConstValue::Type(_) => Type::Type,
    }
  }

  #[allow(unused_variables)]
  fn consteval(&mut self, node: IrPtr) -> Result<Option<ConstValue>> {
    let span = self.module.nodes[node].span;
    Ok(match self.module.nodes[node].kind.clone() {
      Remainder(n) => self.consteval(n)?,
      Declaration {
        assignee,
        is_constant,
        type_assert,
        value,
      } => None,
      Immediate(const_value) => Some(const_value),
      Block(items) => None,
      Identifier(name) => self.solution.constants.get(&name).cloned(),
      StructDef { fields, types } => {
        let Some(types) = types
          .into_iter()
          .map(|t| match self.consteval(t) {
            Ok(Some(ConstValue::Type(t))) => Ok(Some(t)),
            Ok(Some(c)) => error!(
              "Structure definition expects type, but recieved term '{c}'"
            )
            .span(&span),

            Ok(None) => Ok(None),
            Err(e) => Err(e),
          })
          .try_collect::<Vec<_>>()?
          .into_iter()
          .try_collect::<Vec<_>>()
        else {
          return Ok(None);
        };
        Some(ConstValue::Type(Type::Struct {
          member_names: fields,
          member_types: types,
        }))
      },
      StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => None,
      Field { of, index } => None,
      Binary {
        op,
        opdef,
        left,
        right,
      } => None,
      Unary { op, opdef, child } => None,
      FunctionDef {
        name,
        parameter_names,
        parameter_types,
        returns,
        body,
      } => Some(ConstValue::Function(name)),
      FunctionCall {
        callee,
        callee_name,
        arguments,
      } => None,
      If {
        predicate,
        then,
        else_,
      } => None,
      Loop {
        parameter_names,
        parameter_values,
        body,
      } => None,
      Break(_) => None,
    })
  }

  pub(super) fn check(&mut self, node: IrPtr) -> Result<Type> {
    let span = self.module.nodes[node].span;
    let type_ = match self.module.nodes[node].kind.clone() {
      Remainder(n) => self.check(n)?,
      Declaration {
        assignee,
        is_constant,
        value,
        ..
      } => {
        let value_t = self.check(value)?;
        if let Some(assert) = self.solution.assertions.get(&assignee) {
          if !value_t.ambiguous() && !is_constant && &value_t != assert {
            return error!(
              "Asserted type is {assert}, but expression has type {value_t}"
            )
            .span(&span);
          }
        }
        if !is_constant {
          self.solution.assertions.insert(assignee, value_t);
        }
        p::nothing.promote()
      },
      Immediate(const_value) => self.const_type(&const_value),
      Block(items) => {
        let mut never = false;
        let length = items.len();
        let mut type_ = None;
        for (id, item) in items.into_iter().enumerate() {
          let produces = self.check(item)?;
          if produces == p::unreachable.promote() {
            never = true;
          }
          if let Remainder(_) = &self.module.nodes[item].kind
            && id == length - 1
          {
            type_ = Some(produces);
          }
        }
        if never {
          p::unreachable.promote()
        } else {
          type_.unwrap_or(p::nothing.promote())
        }
      },
      Identifier(name) => {
        if let Some(c) = self.solution.constants.get(&name).cloned() {
          let type_ = self.const_type(&c);
          self.module.nodes[node].kind = Immediate(c);
          type_
        } else {
          self.solution.assertions.get(&name).unwrap().clone()
        }
      },
      StructDef { .. } => Type::Type,
      StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => {
        let value = if let Some((struct_t, _)) = struct_t {
          self.check(struct_t)?;
          if let Some(value) = self.consteval(struct_t)? {
            Some(value)
          } else {
            return Ok(Type::Ambiguous);
          }
        } else {
          None
        };
        let mut field_types = vec![];
        for value in field_values {
          let type_ = self.check(value)?;
          if type_.ambiguous() {
            return Ok(Type::Ambiguous);
          };
          field_types.push(type_);
        }
        let node_type = Type::Struct {
          member_names: field_names,
          member_types: field_types,
        };
        if let Some(value) = value {
          let ConstValue::Type(expected @ Type::Struct { .. }) = value else {
            return error!("Expected structure type, but found term {value}")
              .span(&span);
          };
          if expected != node_type {
            return error!(
              "Expected type '{expected}', but found '{node_type}'"
            )
            .span(&span);
          }
        }
        node_type
      },
      Field { of, index } => {
        let struct_t = self.check(of)?;
        if struct_t.ambiguous() {
          return Ok(Type::Ambiguous);
        }
        let Type::Struct {
          member_names,
          member_types,
        } = struct_t
        else {
          return error!("Cannot create struct literal from type {struct_t}");
        };
        let pos = member_names
          .iter()
          .position(|n| n == &index)
          .reason(
            "Structure does not contain field {index}, available fields are \
             {member_names:?}",
          )
          .span(&span)?;
        member_types[pos].clone()
      },
      Binary {
        op, left, right, ..
      } => {
        let left_t = self.check(left)?;
        let right_t = self.check(right)?;
        if left_t.ambiguous() || right_t.ambiguous() {
          return Ok(Type::Ambiguous);
        }
        let opdef = OpTable::new()
          .try_binary(op, &left_t, &right_t)
          .span(&span)?;
        self.module.nodes[node].kind = Binary {
          op,
          opdef: opdef.clone(),
          left,
          right,
        };
        opdef.produces
      },
      Unary { op, child, .. } => {
        let child_t = self.check(child)?;
        let opdef = OpTable::new().try_unary(op, &child_t).span(&span)?;
        self.module.nodes[node].kind = Unary {
          op,
          opdef: opdef.clone(),
          child,
        };
        opdef.produces
      },
      FunctionDef { name, body, .. } => {
        let func_type = self.solution.assertions.get(&name).cloned().unwrap();
        let Type::Function { return_type, .. } = &func_type else {
          panic!()
        };
        let body_t = self.check(body)?;
        if !body_t.ambiguous() && *return_type.clone() != body_t {
          return error!(
            "Function expects to return {return_type}, but evaluates to \
             {body_t} instead"
          )
          .span(&span);
        }
        func_type
      },
      FunctionCall {
        callee, arguments, ..
      } => {
        self.check(callee)?;
        let mut argument_types = vec![];
        for arg in arguments {
          let type_ = self.check(arg)?;
          // TODO change early return to post return
          if type_.ambiguous() {
            return Ok(Type::Ambiguous);
          }
          argument_types.push((type_, self.module.nodes[arg].span));
        }
        let Some(ConstValue::Function(mangle)) = self.consteval(callee)? else {
          return Ok(Type::Ambiguous);
        };
        let Type::Function {
          param_types,
          return_type,
        } = self.solution.assertions.get(&mangle).cloned().unwrap()
        else {
          panic!()
        };
        for (id, ((found, span), expects)) in
          argument_types.into_iter().zip(param_types).enumerate()
        {
          if found != expects {
            return error!(
              "Argument {} is expected to be of type '{expects}', but is \
               actually '{found}'",
              id + 1
            )
            .span(&span);
          }
        }
        *return_type
      },
      If {
        predicate,
        then,
        else_,
      } => {
        let predicate_t = self.check(predicate)?;
        let then_t = self.check(then)?;
        let else_t = if let Some(else_) = else_ {
          self.check(else_)?
        } else {
          p::nothing.promote()
        };
        if predicate_t.ambiguous() || then_t.ambiguous() || else_t.ambiguous() {
          return Ok(Type::Ambiguous);
        }
        if predicate_t != p::boolean.promote() {
          return error!(
            "Predicate of 'if' statement must be a boolean. Found type \
             '{predicate_t}'"
          )
          .span(&span);
        }
        use Type::Primitive as P;
        let result_t = match (then_t, else_t) {
          (P(p::unreachable), P(p::unreachable)) => P(p::unreachable),
          (P(p::unreachable), P(t)) | (P(t), P(p::unreachable)) => P(t),
          (then_t, else_t) => {
            return error!(
              "The 'then' branch produces the type '{then_t}', but the 'else' \
               branch produces the type '{else_t}'"
            )
            .span(&span);
          },
        };
        result_t
      },
      Loop {
        parameter_names,
        parameter_values,
        body,
      } => {
        let mut param_values = vec![];
        for (name, value) in parameter_names
          .into_iter()
          .zip(parameter_values.into_iter())
        {
          let value_t = self.check(value)?;
          param_values.push(value_t.clone());
          self.solution.assertions.insert(name, value_t);
        }
        self.break_stack.push(p::unreachable.promote());
        let body_t = self.check(body)?;
        self.break_stack.pop().unwrap()
      },
      Break(maybe_node) => {
        let type_ = if let Some(break_node) = maybe_node {
          self.check(break_node)?
        } else {
          p::nothing.promote()
        };
        let break_t = self.break_stack.last_mut().unwrap();
        if break_t != &p::unreachable.promote() && &type_ != break_t {
          return error!(
            "This loop produces the type '{break_t}', but this break \
             expression has the type '{type_}'"
          )
          .span(&span);
        }
        *self.break_stack.last_mut().unwrap() = type_;
        p::unreachable.promote()
      },
    };
    self.module.nodes[node].type_ = type_.clone();
    Ok(type_)
  }
}
