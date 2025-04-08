mod constraints;

use std::collections::HashMap;

use crate::{hlir::*, operator::*, span::*};
use constraints::*;

pub struct ConstraintSolver<'a> {
  hlir: &'a mut HlIrModule,
  pub constraints: Vec<TypeConstraint>,
  pub solutions: HashMap<TypeVariable, Type>,
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
      solutions: HashMap::new(),
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
        let prod_t = self.new_type_var();
        use BinaryOp::*;
        match op {
          Star | Slash | Percent | Plus | Minus => {
            self
              .constraints
              .push(TypeConstraint::Equals(left_t.clone(), right_t.clone()));
            self
              .constraints
              .push(TypeConstraint::Equals(left_t, prod_t.clone()));
            self
              .constraints
              .push(TypeConstraint::Equals(right_t, prod_t.clone()));
          },
          And | Nand | Or | Xor | Xnor | DoubleEqual | Less | LessEqual
          | Greater | GreaterEqual | BangEqual => {
            self
              .constraints
              .push(TypeConstraint::Equals(left_t, right_t));
            self.constraints.push(TypeConstraint::Equals(
              prod_t.clone(),
              Type::Primitive(Primitive::boolean),
            ));
          },
          _ => todo!(),
        }
        prod_t
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
      } => {
        let parameter_types = parameter_names
          .iter()
          .map(|n| {
            println!("{n}");
            let tv = self.new_type_var();
            self.environment.insert(n.clone(), tv.clone());
            tv
          })
          .collect::<Vec<_>>();
        let return_t = self.solve(body);
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
        let callee_t = self.solve(callee);
        let param_types = arguments
          .into_iter()
          .map(|a| self.solve(a))
          .collect::<Vec<_>>();
        self.constraints.push(TypeConstraint::Equals(
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
        let predicate_t = self.solve(predicate);
        self.constraints.push(TypeConstraint::Equals(
          predicate_t,
          Type::Primitive(Primitive::boolean),
        ));
        let tv = self.new_type_var();
        let then_t = self.solve(then);
        let else_t = if let Some(else_) = else_ {
          self.solve(else_)
        } else {
          Type::Primitive(Primitive::nothing)
        };
        self
          .constraints
          .push(TypeConstraint::Equals(tv.clone(), then_t));
        self
          .constraints
          .push(TypeConstraint::Equals(tv.clone(), else_t));
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

  pub fn simplify_constraints(&mut self) {
    while let Some(constraint) = self.constraints.pop() {
      println!("Solutions: {:?}", self.solutions);
      println!("Constraint: {constraint:?}\n");
      match constraint {
        TypeConstraint::Equals(t1, t2) => {
          match (t1, t2) {
            (t1, t2) if t1 == t2 => {},
            // Function decomposition
            (
              Type::Function {
                param_types: p1,
                return_type: r1,
              },
              Type::Function {
                param_types: p2,
                return_type: r2,
              },
            ) => {
              if p1.len() != p2.len() {
                // Type (arity) error
                panic!("Wrong function arity");
              }
              p1.into_iter().zip(p2.into_iter()).for_each(|(p1, p2)| {
                self.constraints.push(TypeConstraint::Equals(p1, p2))
              });
              self.constraints.push(TypeConstraint::Equals(*r1, *r2));
            },
            // Polymorphic value
            (Type::Polymorphic(tv), t) | (t, Type::Polymorphic(tv)) => {
              self.solutions.insert(tv, t.clone());
              for node in &mut self.hlir.nodes {
                if let Type::Polymorphic(this_tv) = node.type_.clone()
                  && tv == this_tv
                {
                  node.type_ = t.clone();
                }
              }

              self
                .constraints
                .iter_mut()
                .for_each(|c| c.substitute(tv, t.clone()));
            },
            _ => todo!(),
          }
        },
        TypeConstraint::UnaryResult(prod, unary_op, t1) => todo!(),
      }
    }
  }
}
