use crate::err::*;
use std::collections::HashMap;

use crate::{
  Expression, ExpressionKind, Statement, StatementKind, Token, TokenKind,
};

#[derive(Debug, Clone)]
enum Value {
  Integer(i64),
  Real(f64),
  String(String),
  Boolean(bool),
  Undefined,
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Value::*;
    match self {
      Integer(i) => write!(f, "{i}"),
      Real(r) => write!(f, "{r}"),
      String(s) => write!(f, "{s}"),
      Boolean(b) => write!(f, "{b}"),
      Undefined => write!(f, "undefined"),
    }
  }
}

#[derive(Debug, Clone)]
struct Scope {
  outer: Option<Box<Scope>>,
  declarations: HashMap<String, Value>,
}

impl Scope {
  fn new() -> Self {
    Self {
      outer: None,
      declarations: Default::default(),
    }
  }

  fn enscope(&mut self) -> &mut Self {
    *self = Self {
      outer: Some(Box::new(self.clone())),
      declarations: HashMap::new(),
    };
    self
  }

  fn descope(&mut self) -> &mut Self {
    if let Some(outer) = &self.outer {
      *self = *outer.clone();
    }
    self
  }

  fn declare(&mut self, key: String) -> Result<()> {
    if self.declarations.contains_key(&key) {
      return error()
        .reason(format!("Re-declaration of '{key}' in same scope"));
    }
    self.declarations.insert(key, Value::Undefined);
    Ok(())
  }

  fn assign(&mut self, key: String, value: Value) -> Result<()> {
    if !self.declarations.contains_key(&key) {
      if let Some(outer) = &mut self.outer {
        return outer.assign(key, value);
      }
      return error()
        .reason(format!("Assignemnt to '{key}' before declaration"));
    }
    self.declarations.insert(key, value);
    Ok(())
  }

  fn access(&self, key: String) -> Result<Value> {
    match self.declarations.get(&key) {
      Some(v) => Ok(v.clone()),
      None => {
        if let Some(outer) = &self.outer {
          outer.access(key)
        } else {
          error().reason(format!("'{key}' was never declared"))
        }
      },
    }
  }
}

pub struct Interpreter<I: Iterator<Item = Statement>> {
  scope: Scope,
  iter: I,
}

impl<I: Iterator<Item = Statement>> Interpreter<I> {
  pub fn new(iter: I) -> Self {
    Self {
      scope: Scope::new(),
      iter,
    }
  }

  fn evaluate_unary(
    &mut self,
    token: TokenKind,
    child: Expression,
  ) -> Result<Value> {
    use TokenKind as t;
    use Value as v;
    let val = self.evaluate(child)?;
    Ok(match val {
      v::Integer(i) => v::Integer(match token {
        t::Plus => i,
        t::Minus => -i,
        _ => {
          return error()
            .reason(format!("Unary {token:?} is undefined for integers"));
        },
      }),
      v::Real(r) => v::Real(match token {
        t::Plus => r,
        t::Minus => -r,
        _ => {
          return error()
            .reason(format!("Unary {token:?} is undefined for reals"));
        },
      }),
      v::Boolean(b) => v::Boolean(match token {
        t::Not => !b,
        _ => {
          return error()
            .reason(format!("Unary {token:?} is undefined for booleans"));
        },
      }),
      _ => {
        return error()
          .reason(format!("Binary {token:?} is undefined for {val:?}",));
      },
    })
  }

  fn evaluate_binary(
    &mut self,
    token: TokenKind,
    left: Expression,
    right: Expression,
  ) -> Result<Value> {
    use TokenKind as t;
    use Value::*;
    let left = self.evaluate(left)?;
    let right = self.evaluate(right)?;
    Ok(match (left.clone(), right.clone()) {
      (Integer(l), Integer(r)) => match token {
        t::Plus => Integer(l + r),
        t::Minus => Integer(l - r),
        t::Star => Integer(l * r),
        t::Slash => Integer(l / r),
        t::Percent => Integer(l % r),
        t::DoubleEqual => Boolean(l == r),
        t::Less => Boolean(l < r),
        t::Greater => Boolean(l > r),
        t::LessEqual => Boolean(l <= r),
        t::GreaterEqual => Boolean(l >= r),
        t => {
          return error()
            .reason(format!("Binary {t:?} is undefined for integers"));
        },
      },
      (Real(l), Real(r)) => Real(match token {
        t::Plus => l + r,
        t::Minus => l - r,
        t::Star => l * r,
        t::Slash => l / r,
        t => {
          return error()
            .reason(format!("Binary {t:?} is undefined for reals"));
        },
      }),
      _ => {
        return error().reason(format!(
          "Binary {:?} is undefined for {:?} and {:?}",
          token, left, right
        ));
      },
    })
  }

  fn evaluate(&mut self, expr: Expression) -> Result<Value> {
    use ExpressionKind as e;
    match expr.kind {
      e::Integer(i) => Ok(Value::Integer(i)),
      e::Real(r) => Ok(Value::Real(r)),
      e::String(s) => Ok(Value::String(s)),
      e::Boolean(b) => Ok(Value::Boolean(b)),
      e::Identifier(i) => self.scope.access(i),
      e::Binary { token, left, right } => {
        self.evaluate_binary(token, *left, *right)
      },
      e::Unary { token, child } => self.evaluate_unary(token, *child),
      e::Parenthesis(e) => self.evaluate(*e),
      e::Call { callee, args } => todo!(),
      e::Field { namespace, field } => todo!(),
      e::Function {
        params,
        returns,
        body,
      } => todo!(),
    }
    .span(&expr.span)
  }

  pub fn execute(&mut self, statement: Statement) -> Result<()> {
    use StatementKind as s;
    match statement.kind {
      s::Mutable { name, value, .. } => {
        self.scope.declare(name.clone())?;
        if let Some(value) = value {
          let value = self.evaluate(value)?;
          self.scope.assign(name, value)?;
        }
      },
      s::Immutable { name, value, .. } => {
        self.scope.declare(name.clone())?;
        let value = self.evaluate(value)?;
        self.scope.assign(name, value)?;
      },
      s::Assignment { name, value } => {
        let span = value.span;
        let value = self.evaluate(value).span(&span)?;
        self.scope.assign(name, value).span(&span)?;
      },
      s::Print(e) => {
        let e = self.evaluate(e)?;
        println!("{e}");
      },
      s::Expression(e) => {
        self.evaluate(e)?;
      },
      s::Block(block) => self.block(block)?,
      s::If {
        predicate,
        block,
        else_,
      } => {
        let span = predicate.span;
        let value = self.evaluate(predicate)?;
        if let Value::Boolean(b) = value {
          if b {
            self.block(block)?;
          } else if let Some(else_) = else_ {
            self.execute(*else_)?;
          }
        } else {
          return error()
            .reason("Predicate for 'if' statement must be a boolean")
            .span(&span);
        }
      },
      s::While { predicate, block } => {
        let span = predicate.span;
        loop {
          match self.evaluate(predicate.clone())? {
            Value::Boolean(true) => self.block(block.clone())?,
            Value::Boolean(false) => break,
            _ => {
              return error()
                .reason("Predicate for 'while' statement must be a boolean")
                .span(&span);
            },
          }
        }
      },
      s::Error(e) => {
        eprintln!("{e}");
        panic!();
      },
    }
    Ok(())
  }

  fn block(&mut self, block: Vec<Statement>) -> Result<()> {
    self.scope.enscope();
    for s in block.into_iter() {
      let span = s.span;
      self.execute(s).span(&span)?;
    }
    self.scope.descope();
    Ok(())
  }

  pub fn run(&mut self) -> Result<()> {
    loop {
      let next = match self.iter.next() {
        Some(n) => n,
        None => break,
      };
      let span = next.span;
      self.execute(next).span(&span)?;
    }
    Ok(())
  }
}
