use std::collections::{HashMap, HashSet};

use super::*;
use crate::{hlir::*, operator::*, span::*};

struct ConstraintSolver<'a> {
  hlir: &'a mut HlIrModule,
  constraints: Vec<TypeConstraint>,
  environment: HashMap<Mangle, Type>,
  is_let_bound: HashSet<Mangle>,
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
      Type::Ambiguous => {}
      Type::Undetermined(t) => {
        *type_ = Type::Undetermined(*replace_map.entry(*t).or_insert_with(|| {
          let tv = self.type_var_counter;
          self.type_var_counter += 1;
          tv
        }));
      }
      Type::Dependent(t) => {}
      Type::Primitive(primitive) => {}
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
        *type_ = Type::Variant(HashSet::from_iter(hash_set.clone().into_iter().map(
          |mut t| {
            self.instantiate_type(&mut t, replace_map);
            t
          },
        )));
      }
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .iter_mut()
          .for_each(|t| self.instantiate_type(t, replace_map));
        self.instantiate_type(return_type, replace_map);
      }
      Type::Reference(t) => {
        self.instantiate_type(t, replace_map);
      }
      Type::Type => {}
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
        in_,
      } => {
        self.is_let_bound.insert(assignee.clone());
        let value_type = self.generate_constraints(value);
        self.environment.insert(assignee.clone(), value_type);
        if let Some(in_) = in_ {
          self.generate_constraints(in_)
        } else {
          Type::Primitive(Primitive::nothing)
        }
      }
      h::Immediate(const_value) => const_value.type_of(),
      h::Block(items) => items.into_iter().fold(Type::Ambiguous, |last, item| {
        self.generate_constraints(item)
      }),
      h::Identifier(mangle) => {
        let mut type_ = self.environment.get(&mangle).unwrap().clone();
        if self.is_let_bound.contains(&mangle) {
          self.instantiate_type(&mut type_, &mut HashMap::new());
          type_
        } else {
          type_
        }
      }
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
      }
      h::Field { of, index } => {
        self.generate_constraints(of);
        self.new_type_var()
      }
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
          }
          Colon => self
            .constraints
            .push(TypeConstraint(left_t, Type::Dependent(right))),
          And | Nand | Or | Xor | Xnor | DoubleEqual | Less | LessEqual | Greater
          | GreaterEqual | BangEqual => {
            self.constraints.push(TypeConstraint(left_t, right_t));
            self.constraints.push(TypeConstraint(
              prod_t.clone(),
              Type::Primitive(Primitive::boolean),
            ));
          }
          _ => todo!(),
        }
        prod_t
      }
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
      }
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
      }
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
        self
          .constraints
          .push(TypeConstraint(callee_t, Type::Function {
            param_types,
            return_type: tv.clone().into(),
          }));
        tv
      }
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
      }
      h::Match {
        on,
        patterns,
        branches,
      } => todo!(),
    };
    self.hlir.nodes.get_mut(node_ptr).unwrap().type_ = type_.clone();
    type_
  }
  fn simplify_constraints(&mut self) {
    while let Some(TypeConstraint(t1, t2)) = self.constraints.pop() {
      println!("{:#?}", self.constraints);
      match (t1, t2) {
        (t1, t2) if t1 == t2 => {}
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
          p1.into_iter()
            .zip(p2.into_iter())
            .for_each(|(p1, p2)| self.constraints.push(TypeConstraint(p1, p2)));
          self.constraints.push(TypeConstraint(*r1, *r2));
        }
        // Polymorphic value
        (Type::Undetermined(tv), t) | (t, Type::Undetermined(tv)) => {
          println!("replace {tv} {t}");
          for node in &mut self.hlir.nodes {
            if let Type::Undetermined(this_tv) = node.type_.clone()
              && tv == this_tv
            {
              node.type_.substitute(tv, t.clone());
            }
          }

          self
            .constraints
            .iter_mut()
            .for_each(|c| c.substitute(tv, t.clone()));
        }
        _ => todo!(),
      }
    }
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
    is_let_bound: HashSet::new(),
    type_var_counter: 0,
  };
  cs.generate_constraints(0);
  cs.simplify_constraints();
  cs.constraints
}
