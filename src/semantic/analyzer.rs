use std::collections::HashMap;

use crate::{
  Expression, ExpressionKind, Immediate, Span, Statement, StatementKind,
  diagnostic, err::*, error, semantic::primitives::Primitive,
};

use super::{Mangle, Type, ir::*};
use NodeKind as n;

#[derive(Debug, Clone)]
pub struct Symbol {
  mangle: Mangle,
  scope_depth: usize,
  is_constant: bool,
  consumed_at: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum Event {
  FunctionStart,
  BlockStart,
  Declare {
    name: String,
    old_value: Option<Symbol>,
  },
}

pub struct Analyzer {
  scope_depth: usize,
  salt: usize,
  path: Vec<String>,
  _name_to_symbol: HashMap<String, Symbol>,
  _mangle_to_type: HashMap<Mangle, Type>,
  event_stack: Vec<Event>,
}

impl Analyzer {
  pub fn new() -> Self {
    Self {
      scope_depth: 0,
      salt: 0,
      path: vec![],
      _name_to_symbol: HashMap::new(),
      _mangle_to_type: HashMap::new(),
      event_stack: vec![],
    }
  }

  fn name_to_symbol(&self, name: &str) -> Result<&Symbol> {
    self
      ._name_to_symbol
      .get(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  fn name_to_symbol_mut(&mut self, name: &str) -> Result<&mut Symbol> {
    self
      ._name_to_symbol
      .get_mut(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  fn mangle_to_type(&self, mangle: &str) -> Result<&Type> {
    self._mangle_to_type.get(mangle).ok_or(diagnostic!(
      "This error should never occur! Mangle {mangle} is untyped"
    ))
  }

  fn mangle_to_type_mut(&mut self, mangle: &str) -> Result<&mut Type> {
    self._mangle_to_type.get_mut(mangle).ok_or(diagnostic!(
      "This error should never occur! Mangle {mangle} is untyped"
    ))
  }

  fn next_salt(&mut self) -> String {
    let returned_salt = self.salt.to_string();
    self.salt += 1;
    returned_salt
  }

  fn define_name(
    &mut self,
    name: impl Into<String>,
    is_constant: bool,
  ) -> Result<Mangle> {
    let name = name.into();
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = super::mangle_name(path, &salt);
    let old_value = self.name_to_symbol(&name).ok().cloned();
    if let Some(old) = &old_value {
      if old.scope_depth == self.scope_depth && is_constant && old.is_constant {
        return error!("Conflicting definitions of '{name}' in the same scope");
      }
    }
    let event = Event::Declare {
      old_value,
      name: name.clone(),
    };
    self.event_stack.push(event);
    self._name_to_symbol.insert(name.clone(), Symbol {
      mangle: mangle.clone(),
      scope_depth: self.scope_depth,
      consumed_at: None,
      is_constant,
    });
    self._mangle_to_type.insert(mangle.clone(), Type::Ambiguous);
    Ok(mangle)
  }

  fn define_anonymous(
    &mut self,
    name_hint: impl Into<String>,
    type_: Type,
  ) -> Mangle {
    let name = name_hint.into();
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = super::mangle_name(path, &salt);
    self._mangle_to_type.insert(mangle.clone(), type_).unwrap();
    mangle
  }

  fn unwind_scope(&mut self) {
    while let Some(e) = self.event_stack.pop() {
      match e {
        Event::FunctionStart | Event::BlockStart => {
          self.scope_depth -= 1;
          break;
        },
        Event::Declare { name, old_value } => {
          if let Some(old) = old_value {
            self._name_to_symbol.insert(name, old);
          } else {
            self._name_to_symbol.remove(&name);
          }
        },
      }
    }
  }

  fn start_function(&mut self) {
    let mut to_reset = vec![];
    for (name, symbol) in self._name_to_symbol.iter() {
      if !symbol.is_constant {
        self.event_stack.push(Event::Declare {
          name: name.clone(),
          old_value: Some(symbol.clone()),
        });
        to_reset.push(name.clone())
      }
    }
    for name in to_reset {
      self._name_to_symbol.remove(&name);
    }
  }

  pub fn analyze_scope(
    &mut self,
    block: impl Iterator<Item = Statement>,
  ) -> Result<Node> {
    // Construct IR
    let mut block = block
      .map(|stmt| {
        if let StatementKind::Declaration {
          name,
          is_constant: true,
          ..
        } = &stmt.kind
        {
          self.define_name(name, true).span(&stmt.span).map(|_| stmt)
        } else {
          Ok(stmt)
        }
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .map(|stmt| self.analyze_statement(stmt))
      .collect::<Result<Vec<Node>>>()?;
    // Up-propogate types
    todo!()
  }

  fn analyze_statement(&mut self, stmt: Statement) -> Result<Node> {
    use StatementKind as s;
    Ok(match stmt.kind {
      s::Declaration {
        name,
        type_str,
        value,
        is_constant,
      } => {
        let value = self.analyze_expression(value)?;
        let mangle = if !is_constant {
          self.define_name(name, true)
        } else {
          self.name_to_symbol(&name).map(|s| s.mangle.clone())
        }
        .span(&stmt.span)?;
        let type_assert = if let Some(t) = type_str {
          let type_actual = Type::Unresolved(t);
          *self.mangle_to_type_mut(&mangle)? = type_actual.clone();
          Some(type_actual)
        } else {
          None
        };
        if type_assert.is_none() {
          *self.mangle_to_type_mut(&mangle)? = value.type_.clone();
        }
        Node {
          type_: Type::Prim(Primitive::nothing),
          kind: n::Declaration {
            mangle,
            is_constant,
            type_assert,
            value: value.into(),
          },
        }
      },
      s::Expression(expression) => self.analyze_expression(expression)?,
      s::Remainder(expression) => self.analyze_expression(expression)?,
      s::Return(expression) => todo!(),
      s::Error(diagnostic) => return Err(diagnostic),
    })
  }

  fn analyze_expression(&mut self, expr: Expression) -> Result<Node> {
    use ExpressionKind as e;
    Ok(match expr.kind {
      e::Immediate(immediate) => Node {
        type_: Type::Prim(immediate.type_of()),
        kind: n::Immediate(immediate),
      },
      e::Identifier { name } => {
        let mangle = self.name_to_symbol(&name)?.mangle.clone();
        let type_ = Type::Unresolved(mangle.clone());
        Node {
          type_,
          kind: n::Identifier(mangle),
        }
      },
      e::Binary { op, left, right } => {
        let left = self.analyze_expression(*left)?.into();
        let right = self.analyze_expression(*right)?.into();
        Node {
          type_: Type::Ambiguous,
          kind: n::BinaryOp { op, left, right },
        }
      },
      e::Unary { op, child } => {
        let child = self.analyze_expression(*child)?.into();
        Node {
          type_: Type::Ambiguous,
          kind: n::UnaryOp { op, child },
        }
      },
      e::Parenthesis(expression) => self.analyze_expression(*expression)?,
      e::FunctionDef {
        params,
        returns_str,
        body,
      } => {
        let param_types = params
          .type_names
          .into_iter()
          .map(|name| {
            self
              .name_to_symbol(&name)
              .map(|s| Type::Unresolved(s.mangle.clone()))
          })
          .collect::<Result<Vec<_>>>()?;
        let return_type = if let Some(returns) = returns_str {
          Type::Unresolved(self.name_to_symbol(&returns)?.mangle.clone())
        } else {
          Type::Prim(Primitive::nothing)
        }
        .into();
        let type_ = Type::Function {
          param_names: params.names,
          param_types,
          return_type,
        };
        let mangle = self.define_anonymous("function", type_.clone());
        // TODO enscope
        let nodes = self.analyze_scope(body.into_iter())?.into();
        // TODO descope
        Node {
          type_,
          kind: n::Function { mangle, nodes },
        }
      },
      e::FunctionCall { callee, args } => {
        let callee = self.analyze_expression(*callee)?.into();
        let params = args
          .into_iter()
          .map(|a| self.analyze_expression(a))
          .collect::<Result<Vec<_>>>()?;
        Node {
          type_: Type::Ambiguous,
          kind: n::Call { callee, params },
        }
      },
      e::StructDef(params) => {
        let member_types = params
          .type_names
          .into_iter()
          .map(|name| {
            self
              .name_to_symbol(&name)
              .map(|s| Type::Unresolved(s.mangle.clone()))
          })
          .collect::<Result<Vec<_>>>()?;
        let type_ = Type::Struct {
          member_names: params.names,
          member_types,
        };
        let mangle = self.define_anonymous("struct", type_.clone());
        Node {
          type_,
          kind: n::Identifier(mangle),
        }
      },
      e::StructLiteral { name, args } => {
        let (names, values): (Vec<_>, Vec<_>) = args.into_iter().unzip();
        Node {
          type_: Type::Unresolved(self.name_to_symbol(&name)?.mangle.clone()),
          kind: n::StructLiteal {
            names,
            values: values
              .into_iter()
              .map(|v| self.analyze_expression(v))
              .collect::<Result<Vec<_>>>()?,
          },
        }
      },
      e::Field { namespace, field } => {
        let namespace = self.analyze_expression(*namespace)?.into();
        let index = self.analyze_expression(*field)?.into();
        Node {
          type_: Type::Ambiguous,
          kind: n::Field { namespace, index },
        }
      },
      e::Block(block) => self.analyze_scope(block.into_iter())?,
      e::If {
        predicate,
        block,
        else_,
      } => {
        let predicate = self.analyze_expression(*predicate)?.into();
        let then = self.analyze_scope(block.into_iter())?.into();
        let else_ = if let Some(else_) = else_ {
          Some(self.analyze_expression(*else_)?.into())
        } else {
          None
        };
        Node {
          type_: Type::Ambiguous,
          kind: n::If {
            predicate,
            then,
            else_,
          },
        }
      },
    })
  }
}
