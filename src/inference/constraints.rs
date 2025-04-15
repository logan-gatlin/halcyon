use std::collections::{HashMap, HashSet};

use crate::{hlir::*, operator::*, span::*};

#[derive(Debug, Clone)]
pub struct TypeConstraint(pub Type, pub Type);

impl PartialEq for TypeConstraint {
  fn eq(&self, other: &Self) -> bool {
    self.0 == self.1
  }
}

impl TypeConstraint {
  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    self.0.contains_type_var(tv) || self.1.contains_type_var(tv)
  }

  pub fn substitute(&mut self, tv: TypeVariable, type_: Type) {
    self.0.substitute(tv, type_.clone());
    self.1.substitute(tv, type_);
  }
}

impl Type {
  fn contains_type_var(&self, tv: TypeVariable) -> bool {
    match self {
      Type::Undetermined(t) => tv == *t,
      Type::Struct {
        member_names,
        member_types,
      } => member_types
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Tuple(items) => items
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Variant(hash_set) => hash_set
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .into_iter()
          .fold(false, |accum, x| accum || x.contains_type_var(tv))
          || return_type.contains_type_var(tv)
      },
      Type::Reference(t) => t.contains_type_var(tv),
      Type::Type => false,
      Type::Dependent(_) => false,
      Type::Ambiguous => false,
      Type::Primitive(primitive) => false,
    }
  }

  fn substitute(&mut self, tv: TypeVariable, type_: Type) {
    match self {
      Type::Ambiguous => {},
      Type::Undetermined(t) => {
        if *t == tv {
          *self = type_;
        }
      },
      Type::Primitive(primitive) => {},
      Type::Struct {
        member_names,
        member_types,
      } => {
        member_types
          .iter_mut()
          .for_each(|t| t.substitute(tv, type_.clone()));
      },
      Type::Tuple(items) => items
        .iter_mut()
        .for_each(|i| i.substitute(tv, type_.clone())),
      Type::Variant(hash_set) => {
        *self = Type::Variant(
          hash_set
            .clone()
            .into_iter()
            .map(|mut t| {
              t.substitute(tv, type_.clone());
              t
            })
            .collect::<HashSet<_>>(),
        );
      },
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .iter_mut()
          .for_each(|t| t.substitute(tv, type_.clone()));
        return_type.substitute(tv, type_);
      },
      Type::Reference(r) => r.substitute(tv, type_),
      Type::Type => {},
      Type::Dependent(_) => {},
    }
  }
}

struct ConstraintSolver<'a> {
  hlir: &'a mut HlIrModule,
  constraints: Vec<TypeConstraint>,
  environment: HashMap<Mangle, Type>,
  type_var_counter: usize,
}

impl<'a> ConstraintSolver<'a> {
  pub fn new_type_var(&mut self) -> Type {
    let tv = self.type_var_counter;
    self.type_var_counter += 1;
    Type::Undetermined(tv)
  }

  pub fn instantiate_type(
    &mut self,
    type_: &mut Type,
    replace_map: &mut HashMap<TypeVariable, TypeVariable>,
  ) {
    match type_ {
      Type::Ambiguous => {},
      Type::Undetermined(t) => {
        *type_ =
          Type::Undetermined(*replace_map.entry(*t).or_insert_with(|| {
            let tv = self.type_var_counter;
            self.type_var_counter += 1;
            tv
          }));
      },
      Type::Dependent(t) => {},
      Type::Primitive(primitive) => {},
      Type::Struct {
        member_names,
        member_types,
      } => member_types
        .iter_mut()
        .for_each(|t| self.instantiate_type(t, replace_map)),
      Type::Tuple(items) => items
        .iter_mut()
        .for_each(|t| self.instantiate_type(t, replace_map)),
      Type::Variant(hash_set) => {
        *type_ = Type::Variant(HashSet::from_iter(
          hash_set.clone().into_iter().map(|mut t| {
            self.instantiate_type(&mut t, replace_map);
            t
          }),
        ));
      },
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .iter_mut()
          .for_each(|t| self.instantiate_type(t, replace_map));
        self.instantiate_type(return_type, replace_map);
      },
      Type::Reference(t) => {
        self.instantiate_type(t, replace_map);
      },
      Type::Type => {},
    }
  }

  pub fn generate_constraints(&mut self, node_ptr: IrPtr) -> Type {
    use HlIrKind as h;
    let node = &self.hlir.nodes[node_ptr];
    let type_ = match node.kind.clone() {
      h::Declaration {
        assignee,
        is_constant,
        value,
      } => {
        if !is_constant {
          let value_type = self.generate_constraints(value);
          self.environment.insert(assignee.clone(), value_type);
        } else {
          let type_var = self.new_type_var();
          self.environment.insert(assignee.clone(), type_var.clone());
          let value_type = self.generate_constraints(value);
          self.constraints.push(TypeConstraint(type_var, value_type));
        }
        Type::Primitive(Primitive::nothing)
      },
      h::Immediate(const_value) => const_value.type_of(),
      h::Block(items) => {
        items.into_iter().fold(Type::Ambiguous, |last, item| {
          self.generate_constraints(item)
        })
      },
      h::Identifier(mangle) => {
        let mut type_ = self.environment.get(&mangle).unwrap().clone();
        self.instantiate_type(&mut type_, &mut HashMap::new());
        type_
      },
      h::Tuple(items) => Type::Tuple(
        items
          .into_iter()
          .map(|i| self.generate_constraints(i))
          .collect(),
      ),
      h::StructDef {
        field_names,
        field_types,
      } => Type::Type,
      h::StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => {
        let member_types = field_values
          .into_iter()
          .map(|v| self.generate_constraints(v))
          .collect::<Vec<_>>();
        Type::Struct {
          member_names: field_names,
          member_types,
        }
      },
      h::Field { of, index } => {
        self.generate_constraints(of);
        self.new_type_var()
      },
      h::Binary {
        op,
        opdef,
        left,
        right,
      } => {
        let left_t = self.generate_constraints(left);
        let right_t = self.generate_constraints(right);
        let prod_t = self.new_type_var();
        use BinaryOp::*;
        match op {
          Star | Slash | Percent | Plus | Minus => {
            self
              .constraints
              .push(TypeConstraint(left_t.clone(), right_t.clone()));
            self
              .constraints
              .push(TypeConstraint(left_t, prod_t.clone()));
            self
              .constraints
              .push(TypeConstraint(right_t, prod_t.clone()));
          },
          Colon => self
            .constraints
            .push(TypeConstraint(left_t, Type::Dependent(right))),
          And | Nand | Or | Xor | Xnor | DoubleEqual | Less | LessEqual
          | Greater | GreaterEqual | BangEqual => {
            self.constraints.push(TypeConstraint(left_t, right_t));
            self.constraints.push(TypeConstraint(
              prod_t.clone(),
              Type::Primitive(Primitive::boolean),
            ));
          },
          _ => todo!(),
        }
        prod_t
      },
      h::Unary { op, opdef, child } => {
        let on_t = self.generate_constraints(child);
        let tv = self.new_type_var();
        use UnaryOp::*;
        match op {
          Ampersand => Type::Reference(tv.into()),
          Tilda => Type::Primitive(Primitive::nothing),
          Minus => tv,
          Not => tv,
          Break => tv,
        }
      },
      h::FunctionDef {
        name,
        parameter_names,
        parameter_spans,
        body,
      } => {
        let parameter_types = parameter_names
          .iter()
          .map(|n| {
            let tv = self.new_type_var();
            self.environment.insert(n.clone(), tv.clone());
            tv
          })
          .collect::<Vec<_>>();
        let return_t = self.generate_constraints(body);
        Type::Function {
          param_types: parameter_types,
          return_type: return_t.into(),
        }
      },
      h::FunctionCall {
        callee,
        callee_name,
        arguments,
      } => {
        let tv = self.new_type_var();
        let callee_t = self.generate_constraints(callee);
        let param_types = arguments
          .into_iter()
          .map(|a| self.generate_constraints(a))
          .collect::<Vec<_>>();
        self.constraints.push(TypeConstraint(
          callee_t,
          Type::Function {
            param_types,
            return_type: tv.clone().into(),
          },
        ));
        tv
      },
      h::If {
        predicate,
        then,
        else_,
      } => {
        let predicate_t = self.generate_constraints(predicate);
        self.constraints.push(TypeConstraint(
          predicate_t,
          Type::Primitive(Primitive::boolean),
        ));
        let tv = self.new_type_var();
        let then_t = self.generate_constraints(then);
        let else_t = if let Some(else_) = else_ {
          self.generate_constraints(else_)
        } else {
          Type::Primitive(Primitive::nothing)
        };
        self.constraints.push(TypeConstraint(tv.clone(), then_t));
        self.constraints.push(TypeConstraint(tv.clone(), else_t));
        tv
      },
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

pub fn generate_constraints(hlir: &mut HlIrModule) -> Vec<TypeConstraint> {
  let mut cs = ConstraintSolver {
    hlir,
    constraints: vec![],
    environment: Builtin::ALL
      .into_iter()
      .map(|b| (b.to_mangle(), b.type_()))
      .collect(),
    type_var_counter: 0,
  };
  cs.generate_constraints(0);
  cs.constraints
}
