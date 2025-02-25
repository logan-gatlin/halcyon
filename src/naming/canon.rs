use std::collections::HashSet;

use super::Canonizer;
use crate::{
  Span,
  err::*,
  error,
  ir::{ConstValue, IrPtr, types::Type},
  naming::{CanonKind, CanonNode, Symbol},
  parse::{Expression, ExpressionKind, Immediate, Statement, StatementKind},
};

pub fn parse_int_literal(value: &str, base: u32) -> Result<i64> {
  i64::from_str_radix(value, base)
    .reason(format!("Failed to parse integer literal '{value}'"))
}

pub fn parse_real_literal(value: &str) -> Result<f64> {
  value
    .parse()
    .ok()
    .reason(format!("Failed to parse real literal '{value}'"))
}

impl Canonizer {
  fn extract_names<'a>(
    &mut self,
    error_hint: &str,
    iter: impl Iterator<Item = &'a Expression>,
  ) -> Result<Vec<String>> {
    let (names, spans): (Vec<String>, Vec<Span>) = iter
      .enumerate()
      .map(|(id, n)| {
        if let ExpressionKind::Identifier { name } = &n.kind {
          Ok((self.define_name(name, false)?, n.span))
        } else {
          error!("{error_hint} parameter {} must be an identifier", id + 1)
            .span(&n.span)
        }
      })
      .try_collect::<Vec<_>>()?
      .into_iter()
      .unzip();
    if let Some(pos) = {
      let mut unique = HashSet::new();
      let mut set = names.iter();
      set.position(move |x| !unique.insert(x))
    } {
      return error!(
        "{error_hint} parameter {} must have a unique name",
        pos + 1
      )
      .span(&spans[pos]);
    }
    Ok(names)
  }

  pub(super) fn canon_block(
    &mut self,
    stmts: Vec<Statement>,
  ) -> Result<Vec<IrPtr>> {
    stmts
      .iter()
      .map(|s| {
        if let StatementKind::Declaration {
          name,
          is_constant: true,
          ..
        } = &s.kind
        {
          self.define_name(name, true).map(|_| {})
        } else {
          Ok(())
        }
      })
      .try_collect::<Vec<_>>()?;
    let v = stmts
      .into_iter()
      .map(|s| self.canon_statement(s))
      .try_collect::<Vec<_>>();
    v
  }

  fn canon_statement(&mut self, stmt: Statement) -> Result<IrPtr> {
    use CanonKind as k;
    let node = self.new_node();
    let kind = match stmt.kind {
      StatementKind::Declaration {
        name,
        type_,
        value,
        is_constant,
      } => {
        let assignee = if is_constant {
          self.name_to_symbol(&name).span(&stmt.span)?.mangle.clone()
        } else {
          self.define_name(name, is_constant).span(&stmt.span)?
        };
        let type_assert = if let Some(type_) = type_ {
          Some(self.canon_expr(type_)?)
        } else {
          None
        };
        let value = self.canon_expr(value)?;
        k::Declaration {
          assignee,
          is_constant,
          type_assert,
          value,
        }
      },
      StatementKind::Expression(expression) => {
        self.ir.pop();
        return self.canon_expr(expression);
      },
      StatementKind::Remainder(expression) => {
        k::Remainder(self.canon_expr(expression)?)
      },
      StatementKind::Error(diagnostic) => return Err(diagnostic),
    };
    self.set_node(
      node,
      CanonNode {
        kind,
        span: stmt.span,
        type_: Type::default(),
      },
    );
    Ok(node)
  }

  fn canon_expr(&mut self, expr: Expression) -> Result<IrPtr> {
    use CanonKind as k;
    use ExpressionKind as e;
    let node = self.new_node();
    let kind = match expr.kind {
      e::Immediate(immediate) => match immediate {
        Immediate::Unit => k::Immediate(ConstValue::Nothing),
        Immediate::Integer(val, base) => k::Immediate(ConstValue::Integer(
          parse_int_literal(&val, base as u32)?,
        )),
        Immediate::Real(val) => {
          k::Immediate(ConstValue::Real(parse_real_literal(&val)?))
        },
        Immediate::String(val) => {
          let bytes = val.into_bytes();
          let address = self.allocate(&bytes);
          k::Immediate(ConstValue::String {
            virtual_address: address,
            length: bytes.len(),
          })
        },
        Immediate::Glyph(val) => k::Immediate(ConstValue::Glyph(val)),
        Immediate::Boolean(val) => k::Immediate(ConstValue::Boolean(val)),
      },
      e::Identifier { name } => {
        let Symbol { mangle, .. } =
          self.name_to_symbol(&name).span(&expr.span)?.clone();
        k::Identifier(mangle)
      },
      e::Binary { op, left, right } => {
        let left = self.canon_expr(*left)?;
        let right = self.canon_expr(*right)?;
        k::Binary { op, left, right }
      },
      e::Unary { op, child } => {
        let child = self.canon_expr(*child)?;
        k::Unary { op, child }
      },
      e::Parenthesis(expression) => {
        self.ir.pop();
        return self.canon_expr(*expression);
      },
      e::FunctionDef {
        params,
        returns,
        body,
      } => {
        self.start_function();
        let function_mangle = self.define_unique("function");
        self.enscope();
        let parameter_names =
          self.extract_names("Function", params.names.iter())?;
        let parameter_types = params
          .types
          .into_iter()
          .map(|e| self.canon_expr(e))
          .try_collect::<Vec<_>>()?;
        let returns = if let Some(returns) = returns {
          Some((
            self.canon_expr(*returns)?,
            self.define_unique("return_type"),
          ))
        } else {
          None
        };
        let body = self.canon_expr(*body)?;
        self.descope();
        k::FunctionDef {
          name: function_mangle,
          parameter_names,
          parameter_types,
          returns,
          body,
        }
      },
      e::FunctionCall { callee, args } => {
        let callee = self.canon_expr(*callee)?;
        let arguments = args
          .into_iter()
          .map(|a| self.canon_expr(a))
          .try_collect::<Vec<_>>()?;
        k::FunctionCall {
          callee,
          callee_name: self.define_unique("callee"),
          arguments,
        }
      },
      e::StructDef(parameters) => {
        let fields = self
          .extract_names("Structure definition", parameters.names.iter())?;
        let types = parameters
          .types
          .into_iter()
          .map(|t| self.canon_expr(t))
          .try_collect::<Vec<_>>()?;
        k::StructDef { fields, types }
      },
      e::StructLiteral { struct_t, params } => {
        let struct_t = if let Some(struct_t) = struct_t {
          Some((
            self.canon_expr(*struct_t)?,
            self.define_unique("struct_type"),
          ))
        } else {
          None
        };
        let field_names =
          self.extract_names("Structure literal", params.names.iter())?;
        let field_values = params
          .types
          .into_iter()
          .map(|t| self.canon_expr(t))
          .try_collect::<Vec<_>>()?;
        k::StructLiteral {
          struct_t,
          field_names,
          field_values,
        }
      },
      e::Field { namespace, field } => {
        let of = self.canon_expr(*namespace)?;
        let e::Identifier { name: index } = field.kind else {
          return error!("Field must be an identifier").span(&field.span);
        };
        k::Field { of, index }
      },
      e::Block(statements) => {
        self.enscope();
        let body = self.canon_block(statements)?;
        self.descope();
        k::Block(body)
      },
      e::If {
        predicate,
        then,
        else_,
      } => {
        let predicate = self.canon_expr(*predicate)?;
        let then = self.canon_expr(*then)?;
        let else_ = if let Some(else_) = else_ {
          Some(self.canon_expr(*else_)?)
        } else {
          None
        };
        k::If {
          predicate,
          then,
          else_,
        }
      },
      e::Loop { params, body } => {
        self.enscope();
        let parameter_names =
          self.extract_names("Loop", params.names.iter())?;
        let parameter_values = params
          .types
          .into_iter()
          .map(|e| self.canon_expr(e))
          .try_collect::<Vec<_>>()?;
        let body = self.canon_expr(*body)?;
        self.descope();
        k::Loop {
          parameter_names,
          parameter_values,
          body,
        }
      },
      e::Break { expr } => {
        let value = if let Some(expr) = expr {
          Some(self.canon_expr(*expr)?)
        } else {
          None
        };
        k::Break(value)
      },
    };
    self.set_node(
      node,
      CanonNode {
        kind,
        span: expr.span,
        type_: Type::default(),
      },
    );
    Ok(node)
  }
}
