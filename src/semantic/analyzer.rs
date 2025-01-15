use std::collections::HashMap;

use crate::{
  Expression, ExpressionKind, Immediate, Span, Statement, StatementKind,
  diagnostic, err::*, error, semantic::primitives::Primitive,
};

use super::{Mangle, Type, ir::*, mangle_builtin};
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
  Modify {
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
    let mut this = Self {
      scope_depth: 0,
      salt: 0,
      path: vec![],
      _name_to_symbol: HashMap::new(),
      _mangle_to_type: HashMap::new(),
      event_stack: vec![],
    };
    this.prelude();
    this
  }

  pub fn prelude(&mut self) {
    let mut define_type = |name: String, type_: Type| {
      let mangle = mangle_builtin(&name);
      self._name_to_symbol.insert(name, Symbol {
        mangle: mangle.clone(),
        scope_depth: 0,
        is_constant: true,
        consumed_at: None,
      });
      self
        ._mangle_to_type
        .insert(mangle, Type::Type(type_.into()));
    };
    for p in Primitive::ALL {
      define_type(p.to_string(), Type::Prim(p));
    }
  }

  pub fn print_table(&self) {
    let new_table = self
      ._mangle_to_type
      .clone()
      .into_iter()
      .filter(|(a, _)| !a.starts_with("$") && !a.starts_with("4anon"))
      .collect::<HashMap<_, _>>();
    println!("{new_table:#?}");
  }

  pub(crate) fn name_to_symbol(&self, name: &str) -> Result<&Symbol> {
    self
      ._name_to_symbol
      .get(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  pub(crate) fn name_to_symbol_mut(
    &mut self,
    name: &str,
  ) -> Result<&mut Symbol> {
    self
      ._name_to_symbol
      .get_mut(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  pub(crate) fn mangle_to_type(&self, mangle: &str) -> Result<&Type> {
    self._mangle_to_type.get(mangle).ok_or(diagnostic!(
      "This error should never occur! Mangle {mangle} is untyped"
    ))
  }

  pub(crate) fn mangle_to_type_mut(
    &mut self,
    mangle: &str,
  ) -> Result<&mut Type> {
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
    let event = Event::Modify {
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

  fn define_anonymous(&mut self, type_: Type) -> Mangle {
    let name = String::from("anon");
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = super::mangle_name(path, &salt);
    self._mangle_to_type.insert(mangle.clone(), type_);
    mangle
  }

  fn enscope(&mut self, event: Event) {
    assert!(match event {
      Event::FunctionStart => true,
      Event::BlockStart => true,
      Event::Modify { .. } => false,
    });
    self.event_stack.push(event);
    self.scope_depth += 1;
  }

  fn descope(&mut self) {
    while let Some(e) = self.event_stack.pop() {
      match e {
        Event::FunctionStart | Event::BlockStart => {
          self.scope_depth -= 1;
          break;
        },
        Event::Modify { name, old_value } => {
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
        self.event_stack.push(Event::Modify {
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
    let nodes = block
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
      .try_collect::<Vec<Node>>()?
      .into_iter()
      .map(|n| self.type_bottom_up(n))
      .try_collect::<Vec<_>>()?;
    Ok(Node {
      type_: Type::Prim(Primitive::nothing),
      kind: n::Block { nodes },
    })
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
        self.enscope(Event::FunctionStart);
        self.start_function();
        let mut param_types = Vec::with_capacity(params.arity);
        let mut arguments = Vec::with_capacity(params.arity);
        for i in 0..params.arity {
          let name = &params.names[i];
          let mangle = self.define_name(name, false)?;
          let type_name = &params.type_names[i];
          let type_actual =
            Type::Unresolved(self.name_to_symbol(type_name)?.mangle.clone());
          *self.mangle_to_type_mut(&mangle)? = type_actual.clone();
          param_types.push(type_actual.clone());
          arguments.push(mangle);
        }
        let param_types = params
          .type_names
          .into_iter()
          .map(|name| {
            self
              .name_to_symbol(&name)
              .map(|s| Type::Unresolved(s.mangle.clone()))
          })
          .try_collect::<Vec<_>>()?;

        let return_type = if let Some(returns) = returns_str {
          Type::Unresolved(self.name_to_symbol(&returns)?.mangle.clone())
        } else {
          Type::Type(Type::Prim(Primitive::nothing).into())
        }
        .into();
        let mangle = self.define_anonymous(Type::Ambiguous);
        let type_ = Type::Function {
          mangle: mangle.clone(),
          param_names: params.names,
          param_types,
          return_type,
        };
        *self.mangle_to_type_mut(&mangle)? = type_.clone();
        let nodes = self.analyze_scope(body.into_iter())?.into();
        self.descope();
        Node {
          type_,
          kind: n::Function {
            mangle,
            arguments,
            nodes,
          },
        }
      },
      e::FunctionCall { callee, args } => {
        let callee = self.analyze_expression(*callee)?.into();
        let params = args
          .into_iter()
          .map(|a| self.analyze_expression(a))
          .try_collect::<Vec<_>>()?;
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
        let mangle = self.define_anonymous(Type::Ambiguous);
        let type_ = Type::Type(
          Type::Struct {
            mangle: mangle.clone(),
            member_names: params.names,
            member_types,
          }
          .into(),
        );
        *self.mangle_to_type_mut(&mangle)? = type_.clone();
        Node {
          type_,
          kind: n::Identifier(mangle),
        }
      },
      e::StructLiteral { name, args } => {
        let (names, values): (Vec<_>, Vec<_>) = args.into_iter().unzip();
        Node {
          type_: Type::Unresolved(self.name_to_symbol(&name)?.mangle.clone()),
          kind: n::StructLiteral {
            names,
            values: values
              .into_iter()
              .map(|v| self.analyze_expression(v))
              .try_collect::<Vec<_>>()?,
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
      e::Block(block) => {
        self.enscope(Event::BlockStart);
        let block = self.analyze_scope(block.into_iter())?;
        self.descope();
        block
      },
      e::If {
        predicate,
        block,
        else_,
      } => {
        let predicate = self.analyze_expression(*predicate)?.into();
        self.enscope(Event::BlockStart);
        let then = self.analyze_scope(block.into_iter())?.into();
        self.descope();
        let else_ = if let Some(else_) = else_ {
          self.enscope(Event::BlockStart);
          let else_ = Some(self.analyze_expression(*else_)?.into());
          self.descope();
          else_
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
