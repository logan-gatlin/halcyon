use std::collections::{HashMap, HashSet};

use crate::{hlir::*, operator::*};

#[derive(Debug, Clone)]
pub struct TypeConstraint(Type, Type);

#[derive(Debug, Clone)]
pub struct Substitution(TypeVariable, Type);

impl TypeConstraint {
  pub fn substitute(&mut self, tv: TypeVariable, type_: &Type) {
    self.0.substitute(tv, type_);
    self.1.substitute(tv, type_);
  }

  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    self.0.contains_type_var(tv) || self.1.contains_type_var(tv)
  }
}

pub fn type_solve(module: &mut HlIrModule) {
  let mut type_var = 0;
  let mut new_type_var = move || {
    type_var += 1;
    Type::TypeVariable(type_var - 1)
  };

  let mut constraints = vec![];
  type_inference(
    &mut module.nodes,
    0,
    &mut HashMap::new(),
    &mut new_type_var,
    &mut constraints,
  );
  let solution = unification(&constraints);
  apply_solution(&mut module.nodes, 0, solution);
}

fn type_inference(
  nodes: &mut [HlIrNode],
  ptr: IrPtr,
  name_map: &mut HashMap<Mangle, Type>,
  new_type_var: &mut impl FnMut() -> Type,
  constraints: &mut Vec<TypeConstraint>,
) -> Type {
  use HlIrKind::*;
  let type_ = match nodes[ptr].kind.clone() {
    Declaration {
      assignee,
      is_constant,
      value,
      in_,
    } => {
      if is_constant {
        let tv = new_type_var();
        name_map.insert(assignee, tv.clone());
        let t =
          type_inference(nodes, value, name_map, new_type_var, constraints);
        constraints.push(TypeConstraint(t, tv));
      } else {
        let t =
          type_inference(nodes, value, name_map, new_type_var, constraints);
        name_map.insert(assignee, t);
      }

      if let Some(in_) = in_ {
        type_inference(nodes, in_, name_map, new_type_var, constraints)
      } else {
        Type::Primitive(Primitive::nothing)
      }
    },
    Immediate(c) => Type::Primitive(match c {
      ConstValue::Nothing => Primitive::nothing,
      ConstValue::Never => Primitive::never,
      ConstValue::Integer(_) => Primitive::integer,
      ConstValue::Real(_) => Primitive::real,
      ConstValue::Boolean(_) => Primitive::boolean,
      ConstValue::String { .. } => Primitive::string,
      ConstValue::Glyph(_) => Primitive::glyph,
      _ => unreachable!(),
    }),
    Block(items) => items
      .into_iter()
      .fold(Type::Primitive(Primitive::nothing), |t, i| {
        type_inference(nodes, i, name_map, new_type_var, constraints)
      }),
    Identifier(i) => name_map.get(&i).unwrap().clone(),
    Tuple(items) => Type::Product(
      items
        .into_iter()
        .map(|i| type_inference(nodes, i, name_map, new_type_var, constraints))
        .collect(),
    ),
    StructDef {
      field_names,
      field_types,
    } => todo!(),
    StructLiteral {
      struct_t,
      field_names,
      field_values,
    } => Type::Struct {
      member_names: field_names,
      member_types: field_values
        .into_iter()
        .map(|v| type_inference(nodes, v, name_map, new_type_var, constraints))
        .collect(),
    },
    Field { of, index } => {
      type_inference(nodes, of, name_map, new_type_var, constraints);
      new_type_var()
    },
    Binary { op, left, right } => {
      let tv = new_type_var();
      let left_t =
        type_inference(nodes, left, name_map, new_type_var, constraints);
      let right_t =
        type_inference(nodes, right, name_map, new_type_var, constraints);
      constraints.push(TypeConstraint(left_t.clone(), right_t.clone()));
      use BinaryOp::*;
      match op {
        Star | Slash | Percent | Plus | Minus => {
          constraints.push(TypeConstraint(left_t.clone(), tv.clone()));
          constraints.push(TypeConstraint(right_t.clone(), tv.clone()));
        },
        And | Nand | Or | Xor | Xnor | DoubleEqual | Less | LessEqual
        | Greater | GreaterEqual | BangEqual => {
          constraints.push(TypeConstraint(
            tv.clone(),
            Type::Primitive(Primitive::boolean),
          ));
        },
        _ => todo!(),
      }
      tv
    },
    Unary { op, child } => {
      let child_t =
        type_inference(nodes, child, name_map, new_type_var, constraints);
      use UnaryOp::*;
      match op {
        Ampersand => Type::Ambiguous,
        Tilda => Type::Primitive(Primitive::nothing),
        Break => Type::Primitive(Primitive::never),
        Minus | Not => child_t,
      }
    },
    FunctionDef {
      name,
      parameter_names,
      parameter_spans,
      body,
    } => {
      let arity = parameter_names.len();
      let parameter_types =
        (0..arity).map(|_| new_type_var()).collect::<Vec<_>>();
      parameter_names
        .into_iter()
        .zip(parameter_types.clone())
        .for_each(|(n, t)| {
          name_map.insert(n, t);
        });
      let return_type =
        type_inference(nodes, body, name_map, new_type_var, constraints);
      Type::Function {
        param_types: parameter_types,
        return_type: return_type.into(),
      }
    },
    FunctionCall {
      callee,
      callee_name,
      arguments,
    } => {
      let tv = new_type_var();
      let callee_t =
        type_inference(nodes, callee, name_map, new_type_var, constraints);
      let param_types: Vec<_> = if arguments.len() == 1
        && let HlIrKind::Immediate(ConstValue::Nothing) =
          nodes[arguments[0]].kind
      {
        nodes[arguments[0]].type_ = Type::Primitive(Primitive::nothing);
        vec![]
      } else {
        arguments
          .into_iter()
          .map(|a| {
            type_inference(nodes, a, name_map, new_type_var, constraints)
          })
          .collect()
      };
      println!("{:?}", param_types);
      constraints.push(TypeConstraint(
        Type::Function {
          param_types,
          return_type: tv.clone().into(),
        },
        callee_t,
      ));
      tv
    },
    If {
      predicate,
      then,
      else_,
    } => {
      let tv = new_type_var();
      let pred_t =
        type_inference(nodes, predicate, name_map, new_type_var, constraints);
      let then_t =
        type_inference(nodes, then, name_map, new_type_var, constraints);
      let else_t = if let Some(else_) = else_ {
        type_inference(nodes, else_, name_map, new_type_var, constraints)
      } else {
        Type::Primitive(Primitive::nothing)
      };
      constraints.extend_from_slice(&[
        TypeConstraint(pred_t, Type::Primitive(Primitive::boolean)),
        TypeConstraint(then_t, tv.clone()),
        TypeConstraint(else_t, tv.clone()),
      ]);
      tv
    },
  };
  nodes[ptr].type_ = type_.clone();
  type_
}

fn unification(constraints: &[TypeConstraint]) -> Vec<Substitution> {
  let mut cons = constraints.to_vec();
  let mut solution = vec![];
  while let Some(con) = cons.pop() {
    if con.0.ambiguous() || con.1.ambiguous() {
      continue;
    }
    match (con.0, con.1) {
      (t1, t2) if t1 == t2 => {},
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
          panic!();
        }
        p1.into_iter()
          .zip(p2.into_iter())
          .for_each(|(t1, t2)| cons.push(TypeConstraint(t1, t2)));
        cons.push(TypeConstraint(*r1, *r2));
      },
      (Type::TypeVariable(tv), t) | (t, Type::TypeVariable(tv))
        if !t.contains_type_var(tv) =>
      {
        cons.iter_mut().for_each(|TypeConstraint(t1, t2)| {
          t1.substitute(tv, &t);
          t2.substitute(tv, &t);
        });
        solution.push(Substitution(tv, t));
      },
      (Type::Product(p1), Type::Product(p2)) if p1.len() == p2.len() => cons
        .extend_from_slice(
          &p1
            .into_iter()
            .zip(p2.into_iter())
            .map(|(t1, t2)| TypeConstraint(t1, t2))
            .collect::<Vec<_>>(),
        ),
      (
        Type::Struct {
          member_names: n1,
          member_types: t1,
        },
        Type::Struct {
          member_names: n2,
          member_types: t2,
        },
      ) if t1.len() == t2.len() => cons.extend_from_slice(
        &t1
          .into_iter()
          .zip(t2.into_iter())
          .map(|(t1, t2)| TypeConstraint(t1, t2))
          .collect::<Vec<_>>(),
      ),
      (t1, t2) => panic!("{t1} ;; {t2}"),
    }
  }
  solution
}

fn apply_solution(
  nodes: &mut [HlIrNode],
  mut node_ptr: IrPtr,
  solution: Vec<Substitution>,
) {
  let mut visited = HashSet::new();
  let mut to_visit = vec![];
  'outer: loop {
    visited.insert(node_ptr);
    let node = &mut nodes[node_ptr];
    match node.kind.clone() {
      HlIrKind::Declaration {
        assignee,
        is_constant,
        value,
        in_,
      } => {
        to_visit.push(value);
        if let Some(in_) = in_ {
          to_visit.push(in_);
        }
      },
      HlIrKind::Immediate(const_value) => {},
      HlIrKind::Block(items) => to_visit.extend_from_slice(&items),
      HlIrKind::Identifier(_) => {},
      HlIrKind::Tuple(items) => to_visit.extend_from_slice(&items),
      HlIrKind::StructDef {
        field_names,
        field_types,
      } => to_visit.extend_from_slice(&field_types),
      HlIrKind::StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => {
        if let Some(struct_t) = struct_t {
          to_visit.push(struct_t.0);
        }
        to_visit.extend_from_slice(&field_values);
      },
      HlIrKind::Field { of, index } => to_visit.push(of),
      HlIrKind::Binary { op, left, right } => {
        to_visit.push(left);
        to_visit.push(right);
      },
      HlIrKind::Unary { op, child } => {
        to_visit.push(child);
      },
      HlIrKind::FunctionDef {
        name,
        parameter_names,
        parameter_spans,
        body,
      } => {
        to_visit.push(body);
      },
      HlIrKind::FunctionCall {
        callee,
        callee_name,
        arguments,
      } => {
        to_visit.push(callee);
        to_visit.extend_from_slice(&arguments);
      },
      HlIrKind::If {
        predicate,
        then,
        else_,
      } => {
        to_visit.push(predicate);
        to_visit.push(then);
        if let Some(else_) = else_ {
          to_visit.push(else_);
        }
      },
    }
    while let Some(next) = to_visit.pop() {
      if !visited.contains(&next) {
        node_ptr = next;
        continue 'outer;
      }
    }
    break;
  }
  visited.into_iter().for_each(|n| {
    let nt = &mut nodes[n].type_;
    solution
      .iter()
      .for_each(|Substitution(tv, t)| nt.substitute(*tv, t));
  });
}
