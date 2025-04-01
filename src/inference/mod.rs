use std::collections::HashMap;

use crate::{hlir::*, operator::*, span::*};

#[derive(Debug, Clone)]
pub enum TypeConstraint {
  // For direct type inference
  Equals(Type, Type),
  BinaryResult(Type, BinaryOp, Type, Type),
  UnaryResult(Type, UnaryOp, Type),
}

pub struct ConstraintSolver<'a> {
  hlir: &'a mut HlIrModule,
  pub constraints: Vec<TypeConstraint>,
  environment: HashMap<Mangle, Type>,
  binary_defs: Vec<(BinaryOp, Type, Type, Type)>,
  unary_defs: Vec<(UnaryOp, Type, Type)>,
  type_var_counter: usize,
}

impl<'a> ConstraintSolver<'a> {
  pub fn new(hlir: &'a mut HlIrModule) -> Self {
    let optable = OpTable::new();
    Self {
      hlir,
      constraints: vec![],
      environment: Builtin::ALL
        .into_iter()
        .map(|b| (b.to_mangle(), b.type_()))
        .collect(),
      binary_defs: optable.get_binary_defs(),
      unary_defs: optable.get_unary_defs(),
      type_var_counter: 0,
    }
  }

  pub fn new_type_var(&mut self) -> Type {
    let tv = self.type_var_counter;
    self.type_var_counter += 1;
    Type::Polymorphic(tv)
  }

  pub fn solve(&mut self, node_ptr: IrPtr) -> Type {
    use HlIrKind as h;
    let node = &self.hlir.nodes[node_ptr];
    let type_ = match node.kind.clone() {
      h::Declaration {
        assignee,
        is_constant,
        value,
      } => {
        if !is_constant {
          let value_type = self.solve(value);
          self.environment.insert(assignee.clone(), value_type);
        } else {
          let type_var = self.new_type_var();
          self.environment.insert(assignee.clone(), type_var.clone());
          let value_type = self.solve(value);
          self
            .constraints
            .push(TypeConstraint::Equals(type_var, value_type));
        }
        Type::Primitive(Primitive::nothing)
      },
      h::Immediate(const_value) => const_value.type_of(),
      h::Block(items) => items
        .into_iter()
        .fold(Type::Ambiguous, |last, item| self.solve(item)),
      h::Identifier(mangle) => self.environment.get(&mangle).unwrap().clone(),
      h::Tuple(items) => todo!(),
      h::StructDef {
        field_names,
        field_types,
      } => Type::Type,
      h::StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => todo!(),
      h::Field { of, index } => todo!(),
      h::Binary {
        op,
        opdef,
        left,
        right,
      } => {
        let left_t = self.solve(left);
        let right_t = self.solve(right);
        let tv = self.new_type_var();
        self.constraints.push(TypeConstraint::BinaryResult(
          tv.clone(),
          op,
          left_t,
          right_t,
        ));
        tv
      },
      h::Unary { op, opdef, child } => {
        let on_t = self.solve(child);
        let tv = self.new_type_var();
        self.constraints.push(TypeConstraint::UnaryResult(
          tv.clone(),
          op,
          on_t,
        ));
        tv
      },
      h::FunctionDef {
        name,
        parameter_names,
        parameter_spans,
        body,
      } => todo!(),
      h::FunctionCall {
        callee,
        callee_name,
        arguments,
      } => todo!(),
      h::If {
        predicate,
        then,
        else_,
      } => todo!(),
      h::Match {
        on,
        patterns,
        branches,
      } => todo!(),
      h::Loop {
        parameter_names,
        parameter_values,
        parameter_spans,
        body,
      } => todo!(),
      h::Break(_) => todo!(),
    };
    self.hlir.nodes.get_mut(node_ptr).unwrap().type_ = type_.clone();
    type_
  }
}
