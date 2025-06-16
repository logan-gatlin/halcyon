use std::collections::{HashMap, HashSet};

use crate::{hlir::*, operator::*};

#[derive(Debug, Clone)]
pub struct TypeConstraint(Type, Type);

impl TypeConstraint {
  pub fn substitute(&mut self, tv: TypeVariable, type_: &Type) {
    self.0.substitute(tv, type_);
    self.1.substitute(tv, type_);
  }

  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    self.0.contains_type_var(tv) || self.1.contains_type_var(tv)
  }
}

pub fn hindley_milner_inference(
  module: &mut HlIrModule,
) -> Vec<TypeConstraint> {
  infer_types(
    &mut Context {
      module,
      type_var: &mut 0,
      name_map: HashMap::new(),
    },
    &[],
    0,
  )
  .1
}

struct Context<'a> {
  module: &'a mut HlIrModule,
  type_var: &'a mut usize,
  name_map: HashMap<Mangle, Type>,
}

fn infer_types(
  ctx: &mut Context,
  given_constraints: &[TypeConstraint],
  node: IrPtr,
) -> (Type, Vec<TypeConstraint>) {
  let new_type_var = |t: &mut Context| {
    let temp = *t.type_var;
    *t.type_var += 1;
    Type::TypeVariable(temp)
  };
  let mut constraints = vec![];
  let type_ = match ctx.module.nodes.get(node).unwrap().kind.clone() {
    HlIrKind::Declaration {
      assignee,
      is_constant,
      value,
      in_,
    } => todo!(),
    HlIrKind::Immediate(const_value) => Type::Primitive(match const_value {
      ConstValue::Nothing => Primitive::nothing,
      ConstValue::Never => Primitive::never,
      ConstValue::Integer(_) => Primitive::integer,
      ConstValue::Real(_) => Primitive::real,
      ConstValue::Boolean(_) => Primitive::boolean,
      ConstValue::String { .. } => Primitive::string,
      ConstValue::Glyph(_) => Primitive::glyph,
      _ => unreachable!(),
    }),
    HlIrKind::Block(items) => {
      let (t, cons) = items
        .into_iter()
        .fold((Type::Primitive(Primitive::nothing), vec![]), |t, i| {
          infer_types(ctx, given_constraints, i)
        });
      constraints.extend_from_slice(&cons);
      t
    },
    HlIrKind::Identifier(_) => todo!(),
    HlIrKind::Tuple(items) => {
      let (types, cons): (Vec<_>, Vec<_>) = items
        .into_iter()
        .map(|i| infer_types(ctx, given_constraints, i))
        .unzip();
      constraints
        .extend_from_slice(&cons.into_iter().flatten().collect::<Vec<_>>());
      Type::Product(types)
    },
    HlIrKind::StructDef {
      field_names,
      field_types,
    } => {
      let (field_t, cons): (Vec<_>, Vec<_>) = field_types
        .into_iter()
        .map(|t| infer_types(ctx, given_constraints, t))
        .unzip();
      constraints
        .extend_from_slice(&cons.into_iter().flatten().collect::<Vec<_>>());
      constraints.extend_from_slice(
        &field_t
          .into_iter()
          .map(|t| TypeConstraint(t, Type::Type))
          .collect::<Vec<_>>(),
      );
      Type::Type
    },
    HlIrKind::StructLiteral {
      struct_t,
      field_names,
      field_values,
    } => {
      let (val_t, val_cons): (Vec<_>, Vec<_>) = field_values
        .into_iter()
        .map(|n| infer_types(ctx, given_constraints, n))
        .unzip();
      constraints
        .extend_from_slice(&val_cons.into_iter().flatten().collect::<Vec<_>>());
      Type::Struct {
        member_names: field_names,
        member_types: val_t,
      }
    },
    HlIrKind::Field { of, index } => new_type_var(ctx),
    HlIrKind::Binary {
      op,
      opdef,
      left,
      right,
    } => {
      let tv = new_type_var(ctx);
      let (left_t, left_cons) = infer_types(ctx, &constraints, left);
      let (right_t, right_cons) = infer_types(ctx, &constraints, right);
      constraints.extend_from_slice(&left_cons);
      constraints.extend_from_slice(&right_cons);
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
    HlIrKind::Unary { op, opdef, child } => {
      let (child_t, child_cons) = infer_types(ctx, given_constraints, child);
      constraints.extend_from_slice(&child_cons);
      use UnaryOp::*;
      match op {
        Ampersand => Type::Ambiguous,
        Tilda => Type::Primitive(Primitive::nothing),
        Break => Type::Primitive(Primitive::never),
        Minus | Not => child_t,
      }
    },
    HlIrKind::FunctionDef {
      name,
      parameter_names,
      parameter_spans,
      body,
    } => todo!(),
    HlIrKind::FunctionCall {
      callee,
      callee_name,
      arguments,
    } => todo!(),
    HlIrKind::If {
      predicate,
      then,
      else_,
    } => {
      let tv = new_type_var(ctx);
      let (pred_t, pred_cons) = infer_types(ctx, given_constraints, predicate);
      let (then_t, then_cons) = infer_types(ctx, given_constraints, then);
      let (else_t, else_cons) = if let Some(else_) = else_ {
        infer_types(ctx, given_constraints, else_)
      } else {
        (Type::Primitive(Primitive::nothing), vec![])
      };
      constraints.extend_from_slice(&pred_cons);
      constraints.extend_from_slice(&then_cons);
      constraints.extend_from_slice(&else_cons);
      constraints.extend_from_slice(&[
        TypeConstraint(pred_t, Type::Primitive(Primitive::boolean)),
        TypeConstraint(then_t, tv.clone()),
        TypeConstraint(else_t, tv.clone()),
      ]);
      tv
    },
  };

  ctx.module.nodes.get_mut(node).unwrap().type_ = type_.clone();
  (type_, constraints)
}

pub fn unification(
  constraints: &[TypeConstraint],
) -> Vec<(TypeVariable, Type)> {
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
        solution.push((tv, t));
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
      _ => {
        panic!();
      },
    }
  }
  solution
}

pub fn apply_solution(
  module: &mut HlIrModule,
  mut node_ptr: IrPtr,
  solution: Vec<(TypeVariable, Type)>,
) {
  let mut visited = HashSet::new();
  let mut to_visit = vec![];
  'outer: loop {
    visited.insert(node_ptr);
    let node = module.nodes.get_mut(node_ptr).unwrap();
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
      HlIrKind::Identifier(_) => todo!(),
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
      HlIrKind::Binary {
        op,
        opdef,
        left,
        right,
      } => {
        to_visit.push(left);
        to_visit.push(right);
      },
      HlIrKind::Unary { op, opdef, child } => {
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
    let nt = &mut module.nodes.get_mut(n).unwrap().type_;
    solution.iter().for_each(|(tv, t)| nt.substitute(*tv, t));
  });
}
