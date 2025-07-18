use std::collections::HashMap;

use super::*;
use crate::{builtin::Builtin, lint::*, parse::*};

#[derive(Debug, Clone)]
struct Scope {
  clean: String,
  old: Option<(Mangle, FunctionDepth)>,
}

type FunctionDepth = usize;

#[derive(Debug, Clone)]
pub struct NameSpace {
  name_table: HashMap<String, (Mangle, FunctionDepth)>,
  builtins: HashMap<String, Mangle>,
  salt: usize,
  scopes: Vec<Scope>,
  captures: Vec<Vec<Mangle>>,
}

impl NameSpace {
  pub fn new() -> Self {
    let mut builtins = HashMap::new();
    Type::primitives().into_iter().for_each(|(_, name)| {
      builtins.insert(name.to_string(), mangle_builtin(name));
    });
    Builtin::ALL.into_iter().for_each(|bt| {
      builtins.insert(bt.to_string(), bt.get_mangle());
    });
    Self {
      name_table: HashMap::new(),
      builtins,
      salt: 0,
      scopes: vec![],
      captures: vec![],
    }
  }

  pub fn push(&mut self, name: String) -> Mangle {
    let mangle = mangle_name(vec![name.clone()], &format!("{}", self.salt));
    self.salt += 1;
    self.scopes.push(Scope {
      clean: name.clone(),
      old: self
        .name_table
        .insert(name.clone(), (mangle.clone(), self.captures.len())),
    });
    mangle
  }

  pub fn pop(&mut self) {
    let Scope { clean, old, .. } = self.scopes.pop().unwrap();
    match old {
      Some(old) => self.name_table.insert(clean, old),
      None => self.name_table.remove(&clean),
    };
  }

  pub fn get(&mut self, name: &String) -> Option<Mangle> {
    match self.name_table.get(name) {
      Some((mangle, depth)) => {
        for capture in (*depth)..(self.captures.len()) {
          self.captures[capture].push(mangle.clone());
        }
        Some(mangle.clone())
      },
      None => self.builtins.get(name).cloned(),
    }
  }

  pub fn new_func(&mut self) {
    self.captures.push(vec![]);
  }

  pub fn end_func(&mut self) -> Vec<Mangle> {
    self.captures.pop().unwrap()
  }
}

pub fn build_hlir(e: Expression) -> Result<HlIrModule> {
  let mut module = HlIrModule { nodes: vec![] };
  let mut ns = NameSpace::new();
  expr(&mut module, &mut ns, e)?;
  Ok(module)
}

fn expr(
  module: &mut HlIrModule,
  ns: &mut NameSpace,
  ex: Expression,
) -> Result<IrPtr> {
  use ExpressionKind as e;
  use HlIrKind as h;
  let span = ex.span;
  let ptr = module.nodes.len();
  module.nodes.push(HlIrNode {
    kind: h::Immediate(ConstValue::Unit),
    span,
    type_: Default::default(),
  });
  let kind = match ex.kind {
    e::Let {
      is_type,
      is_recursive: true,
      assignee,
      value,
      in_,
      ..
    } => {
      let assignee = ns.push(assignee);
      let value = expr(module, ns, *value)?;
      if !matches!(module[value].kind, h::FunctionDef { .. }) {
        panic!()
      }
      let in_ = if let Some(in_) = in_ {
        Some(expr(module, ns, *in_)?)
      } else {
        None
      };
      ns.pop();
      h::Declaration {
        assignee,
        is_type,
        is_recursive: true,
        value,
        in_,
      }
    },
    e::Let {
      is_type,
      is_recursive: false,
      assignee,
      value,
      in_,
      ..
    } => {
      let value = expr(module, ns, *value)?;
      let assignee = ns.push(assignee);
      let in_ = if let Some(in_) = in_ {
        Some(expr(module, ns, *in_)?)
      } else {
        None
      };
      h::Declaration {
        assignee,
        is_type,
        is_recursive: false,
        value,
        in_,
      }
    },
    e::Literal(literal) => {
      fn int(value: &str, base: u32) -> Result<i64> {
        i64::from_str_radix(value, base).lint(TokenLint::InvalidInteger)
      }
      fn real(value: &str) -> Result<f64> {
        value.parse().lint(TokenLint::InvalidReal)
      }

      h::Immediate(match literal {
        Literal::Unit => ConstValue::Unit,
        Literal::Integer(i, base) => {
          ConstValue::Integer(int(&i, base as u32).span(span)?)
        },
        Literal::Real(r) => ConstValue::Real(real(&r).span(span)?),
        Literal::String(s) => ConstValue::String(s),
        Literal::Glyph(g) => ConstValue::Glyph(g),
        Literal::Boolean(b) => ConstValue::Boolean(b),
      })
    },
    e::Identifier(name) => {
      let mangle =
        ns.get(&name)
          .ok_or(lint(NameLint::UndefinedName, span, &[name]))?;
      h::Identifier(mangle)
    },
    e::Binary { op, left, right } => h::Binary {
      op,
      left: expr(module, ns, *left)?,
      right: expr(module, ns, *right)?,
    },
    e::Unary { op, child } => h::Unary {
      op,
      child: expr(module, ns, *child)?,
    },
    e::FunctionDef {
      export_name: _,
      mut arguments,
      mut argument_spans,
      mut types,
      body,
    } => {
      if arguments.len() == 0 {
        ns.new_func();
        let parameter_span = span;
        let body = expr(module, ns, *body)?;
        let captures = ns.end_func();
        let capture_types = vec![Type::Any; captures.len()];
        h::FunctionDef {
          parameter_name: None,
          parameter_span,
          parameter_type: None,
          captures,
          capture_types,
          body,
        }
      } else {
        ns.new_func();
        let (argument, new_arguments) = arguments.split_first().unwrap();
        let parameter_name = ns.push(argument.clone());
        arguments = new_arguments.to_vec();
        let (parameter_span, new_spans) = argument_spans.split_first().unwrap();
        let parameter_span = parameter_span.clone();
        argument_spans = new_spans.to_vec();
        let (type_, new_type_s) = types.split_first().unwrap();
        let parameter_type = if let Some(type_) = type_.clone() {
          Some(expr(module, ns, type_)?)
        } else {
          None
        };
        types = new_type_s.to_vec();
        let body = if arguments.len() == 0 {
          expr(module, ns, *body)?
        } else {
          expr(
            module,
            ns,
            Expression {
              kind: e::FunctionDef {
                export_name: None,
                arguments,
                argument_spans,
                types,
                body,
              },
              span,
            },
          )?
        };
        let captures = ns.end_func();
        let capture_types = vec![Type::Any; captures.len()];
        h::FunctionDef {
          parameter_name: Some(parameter_name),
          parameter_span,
          parameter_type,
          captures,
          capture_types,
          body,
        }
      }
    },
    e::FunctionCall { callee, arguments } => {
      let callee = expr(module, ns, *callee)?;
      let argument = expr(module, ns, *arguments)?;
      h::FunctionCall { callee, argument }
    },
    e::If {
      predicate,
      then,
      else_,
    } => {
      let predicate = expr(module, ns, *predicate)?;
      let then = expr(module, ns, *then)?;
      let else_ = if let Some(else_) = else_ {
        Some(expr(module, ns, *else_)?)
      } else {
        None
      };
      h::If {
        predicate,
        then,
        else_,
      }
    },
    e::Structure {
      is_definition: true,
      lhs,
      rhs,
    } => h::StructDef {
      field_names: lhs,
      field_types: rhs
        .into_iter()
        .map(|e| expr(module, ns, e))
        .try_collect()?,
    },
    e::Structure {
      is_definition: false,
      lhs,
      rhs,
    } => h::StructLiteral {
      field_names: lhs,
      field_values: rhs
        .into_iter()
        .map(|e| expr(module, ns, e))
        .try_collect()?,
    },
  };
  module[ptr].kind = kind;
  Ok(ptr)
}
