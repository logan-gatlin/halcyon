use std::collections::HashMap;

use crate::{
  BinaryOp, Expression, ExpressionKind, Span, Statement, StatementKind,
  UnaryOp, diagnostic, err::*, error, semantic::primitives::Primitive,
};

use super::{Mangle, Type, ir::*, mangle_builtin, operators::OpTable};
use NodeKind as n;

#[derive(Debug, Clone)]
pub struct Symbol {
  mangle: Mangle,
  scope_depth: usize,
  is_constant: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
  ScopeStart,
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
  pub op_table: OpTable,
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
      op_table: OpTable::new(),
    };
    this.prelude();
    this
  }

  pub fn prelude(&mut self) {
    // Define primitive types
    let mut define_type = |name: String, type_: Type| {
      let mangle = mangle_builtin(&name);
      self._name_to_symbol.insert(name, Symbol {
        mangle: mangle.clone(),
        scope_depth: 0,
        is_constant: true,
      });
      self
        ._mangle_to_type
        .insert(mangle, Type::Type(type_.into()));
    };
    for p in Primitive::ALL {
      define_type(p.to_string(), Type::Prim(p));
    }
    // Define binary operators
    use Primitive::*;
    {
      let mut bin =
        |op: BinaryOp, t1: Primitive, t2: Primitive, produces: Primitive| {
          self
            .op_table
            .define_binary(op, t1.promote(), t2.promote(), produces.promote())
            .unwrap();
        };
      use BinaryOp::*;
      for op in [Plus, Minus, Star, Slash] {
        bin(op, integer, integer, integer);
        bin(op, real, real, real);
      }
      for op in [And, Nand, Xor, Xnor, Or, Nor] {
        bin(op, integer, integer, integer);
        bin(op, boolean, boolean, boolean);
      }
      for op in [DoubleEqual, LessEqual, GreaterEqual, Less, Greater] {
        bin(op, integer, integer, boolean);
        bin(op, real, real, boolean);
        bin(op, boolean, boolean, boolean);
      }
      bin(Percent, integer, integer, integer);
    }
    {
      let mut un = |op: UnaryOp, t: Primitive, produces: Primitive| {
        self
          .op_table
          .define_unary(op, t.promote(), produces.promote())
          .unwrap();
      };
      use UnaryOp::*;
      un(Minus, integer, integer);
      un(Minus, real, real);
      un(Not, integer, integer);
      un(Not, boolean, boolean);
      for t in Primitive::ALL {
        un(Plus, t, t);
        un(Tilda, t, t);
      }
    }
  }

  pub fn print_table(&self) {
    let new_table = self
      ._mangle_to_type
      .clone()
      .into_iter()
      .filter(|(a, _)| !a.starts_with("$") && !a.starts_with("5anon"))
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

  fn enscope(&mut self) {
    self.event_stack.push(Event::ScopeStart);
    self.scope_depth += 1;
  }

  fn descope(&mut self) {
    while let Some(e) = self.event_stack.pop() {
      match e {
        Event::ScopeStart => {
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

  pub fn typecheck_program(
    &mut self,
    block: impl Iterator<Item = Statement>,
  ) -> Result<Node> {
    let node = self.analyze_scope(block)?;
    let node = self.type_bottom_up(node)?;
    self.type_top_down(Primitive::nothing.promote(), node)
  }

  pub fn analyze_scope(
    &mut self,
    block: impl Iterator<Item = Statement>,
  ) -> Result<Node> {
    let nodes = block
      // Pass 1 - define constant names
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
      // Pass 2 - construct ir
      .map(|stmt| self.analyze_statement(stmt))
      .try_collect::<Vec<Node>>()?;
    let mut span = Span { row: 0, column: 0 };
    for n in nodes.iter() {
      span = span + n.span;
    }
    Ok(Node {
      span,
      type_: Type::Ambiguous,
      kind: NodeKind::Block { nodes },
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
          self.define_name(name.clone(), true)
        } else {
          self.name_to_symbol(&name).map(|s| s.mangle.clone())
        }
        .span(&stmt.span)?;
        let type_assert = if let Some(type_name) = type_str {
          let type_mangle = self
            .name_to_symbol(&type_name)
            .span(&stmt.span)?
            .mangle
            .clone();
          let type_actual = Type::SameAs(type_mangle);
          *self.mangle_to_type_mut(&mangle)? = type_actual.clone();
          Some(type_actual)
        } else {
          None
        };
        if type_assert.is_none() {
          *self.mangle_to_type_mut(&mangle)? = value.type_.clone();
        }
        Node {
          span: stmt.span,
          type_: Type::Prim(Primitive::nothing),
          kind: n::Declaration {
            name,
            mangle,
            is_constant,
            type_assert,
            value: value.into(),
          },
        }
      },
      s::Expression(expression) => self.analyze_expression(expression)?,
      s::Remainder(expression) => {
        let node = self.analyze_expression(expression)?;
        Node {
          span: stmt.span,
          type_: node.type_.clone(),
          kind: n::Remainder { node: node.into() },
        }
      },
      s::Error(diagnostic) => return Err(diagnostic),
    })
  }

  fn analyze_expression(&mut self, expr: Expression) -> Result<Node> {
    use ExpressionKind as e;
    let (type_, kind) = match expr.kind {
      e::Immediate(immediate) => {
        (Type::Prim(immediate.type_of()), n::Immediate(immediate))
      },
      e::Identifier { name } => {
        let mangle = self.name_to_symbol(&name)?.mangle.clone();
        let type_ = Type::SameAs(mangle.clone());
        (type_, n::Identifier { name, mangle })
      },
      e::Binary { op, left, right } => {
        let left = self.analyze_expression(*left)?.into();
        let right = self.analyze_expression(*right)?.into();
        (Type::Ambiguous, n::BinaryOp { op, left, right })
      },
      e::Unary { op, child } => {
        let child = self.analyze_expression(*child)?.into();
        (Type::Ambiguous, n::UnaryOp { op, child })
      },
      e::Parenthesis(expression) => {
        return self.analyze_expression(*expression);
      },
      e::FunctionDef {
        params,
        returns_str,
        body,
      } => {
        self.enscope();
        self.start_function();
        let mut param_types = Vec::with_capacity(params.arity);
        let mut arguments = Vec::with_capacity(params.arity);
        for i in 0..params.arity {
          let name = &params.names[i];
          let mangle = self.define_name(name, false)?;
          let type_name = &params.type_names[i];
          let type_actual =
            Type::IsType(self.name_to_symbol(type_name)?.mangle.clone());
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
              .map(|s| Type::SameAs(s.mangle.clone()))
          })
          .try_collect::<Vec<_>>()?;

        let return_type = if let Some(returns) = returns_str {
          Type::SameAs(self.name_to_symbol(&returns)?.mangle.clone())
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
        (type_, n::Function {
          mangle,
          arguments,
          nodes,
        })
      },
      e::FunctionCall { callee, args } => {
        let callee = self.analyze_expression(*callee)?.into();
        let params = args
          .into_iter()
          .map(|a| self.analyze_expression(a))
          .try_collect::<Vec<_>>()?;
        (Type::Ambiguous, n::Call {
          callee,
          params,
          mangle: "".into(),
        })
      },
      e::StructDef(params) => {
        let member_types = params
          .type_names
          .into_iter()
          .map(|name| {
            self
              .name_to_symbol(&name)
              .map(|s| Type::SameAs(s.mangle.clone()))
          })
          .collect::<Result<Vec<_>>>()?;
        let mangle = self.define_anonymous(Type::Ambiguous);
        let type_ = Type::Type(
          Type::Struct {
            name: None,
            mangle: mangle.clone(),
            member_names: params.names,
            member_types,
          }
          .into(),
        );
        *self.mangle_to_type_mut(&mangle)? = type_.clone();
        (type_, n::Identifier {
          name: "anonymous struct".into(),
          mangle,
        })
      },
      e::StructLiteral { name, args } => {
        let (names, values): (Vec<_>, Vec<_>) = args.into_iter().unzip();
        (
          Type::SameAs(self.name_to_symbol(&name)?.mangle.clone()),
          n::StructLiteral {
            names,
            values: values
              .into_iter()
              .map(|v| self.analyze_expression(v))
              .try_collect::<Vec<_>>()?,
          },
        )
      },
      e::Field { namespace, field } => {
        let namespace = self.analyze_expression(*namespace)?;
        let Expression {
          kind: e::Identifier { name: index },
          ..
        } = *field
        else {
          return error!("Index must be an identifier").span(&field.span);
        };
        (Type::Ambiguous, n::Field {
          namespace: namespace.into(),
          index,
        })
      },
      e::Block(block) => {
        self.enscope();
        let block = self.analyze_scope(block.into_iter())?;
        self.descope();
        return Ok(block);
      },
      e::If {
        predicate,
        block,
        else_,
      } => {
        let predicate = self.analyze_expression(*predicate)?.into();
        self.enscope();
        let then = self.analyze_scope(block.into_iter())?.into();
        self.descope();
        let else_ = if let Some(else_) = else_ {
          self.enscope();
          let else_ = Some(self.analyze_expression(*else_)?.into());
          self.descope();
          else_
        } else {
          None
        };
        (Type::Ambiguous, n::If {
          predicate,
          then,
          else_,
        })
      },
      e::Loop { names, exprs, body } => {
        let initials = exprs
          .into_iter()
          .map(|e| self.analyze_expression(e))
          .try_collect::<Vec<_>>()?;
        self.enscope();
        let names = names
          .into_iter()
          .map(|n| self.define_name(n, false))
          .try_collect::<Vec<_>>()
          .span(&expr.span)?;
        let body = self.analyze_scope(body.into_iter())?;
        self.descope();
        (Type::Ambiguous, n::Loop {
          names,
          initials,
          body: body.into(),
        })
      },
      e::Break { expr } => {
        let expr = self.analyze_expression(*expr)?;
        (Primitive::never.promote(), n::Break { expr: expr.into() })
      },
    };
    Ok(Node {
      span: expr.span,
      type_,
      kind,
    })
  }
}
