use super::Analyzer;
use crate::{Expression, ExpressionKind, Statement, StatementKind, err::*};

use super::Type;

impl Analyzer {
  pub fn bottom_up_stmt(
    &mut self,
    mut stmt: Box<Statement>,
  ) -> Result<Box<Statement>> {
    use StatementKind as s;
    stmt.kind = match stmt.kind {
      s::Declaration {
        name,
        type_str,
        mut type_actual,
        mut value,
        mutable,
        uid,
      } => {
        value = *self.bottom_up_expr(value.into())?;
        if let Some(ref uid) = type_str {
          type_actual =
            self.table.resolve_type(uid)?.is_alias().span(&stmt.span)?;
        }
        value.type_ = value
          .type_
          .deduce(&type_actual)
          .span(&value.span)?
          .promote();
        type_actual = value.type_.clone();
        if let Type::Ambiguous = type_actual {
          return error()
            .reason(format!("Cannot deduce type of {name}"))
            .span(&stmt.span);
        }
        self.table.modify_ident(
          uid.clone(),
          None,
          Some(type_actual.clone()),
          None,
          true,
        )?;
        s::Declaration {
          name,
          type_str,
          type_actual,
          value,
          mutable,
          uid,
        }
      },
      s::Assignment {
        name,
        mut value,
        uid,
      } => {
        value = *self.bottom_up_expr(value.into())?;
        self.table.modify_ident(
          uid.clone(),
          None,
          Some(value.type_.clone()),
          Some(true),
          true,
        )?;
        s::Assignment { name, value, uid }
      },
      s::If {
        mut predicate,
        mut block,
        mut else_,
      } => {
        predicate = *self.bottom_up_expr(predicate.into())?;
        self.table.start_block();
        for s in &mut block {
          *s = *self.bottom_up_stmt(s.clone().into())?;
        }
        self.table.end_block();
        else_ = if let Some(else_) = else_ {
          Some(self.bottom_up_stmt(else_)?)
        } else {
          None
        };
        s::If {
          predicate,
          block,
          else_,
        }
      },
      s::While {
        mut predicate,
        mut block,
      } => {
        predicate = *self.bottom_up_expr(predicate.into())?;
        self.table.start_block();
        for s in &mut block {
          *s = *self.bottom_up_stmt(s.clone().into())?;
        }
        self.table.end_block();
        s::While { predicate, block }
      },
      s::Print(mut expression) => {
        expression = *self.bottom_up_expr(expression.into())?;
        s::Print(expression)
      },
      s::Expression(mut expression) => {
        expression = *self.bottom_up_expr(expression.into())?;
        s::Expression(expression)
      },
      s::Block(mut block) => {
        for s in &mut block {
          *s = *self.bottom_up_stmt(s.clone().into())?;
        }
        s::Block(block)
      },
      s::Return(mut expression) => {
        expression = if let Some(e) = expression {
          Some(*self.bottom_up_expr(e.into())?)
        } else {
          None
        };
        s::Return(expression)
      },
      s::Error(diagnostic) => s::Error(diagnostic),
    };
    Ok(stmt)
  }

  // Bottom up type inference
  pub fn bottom_up_expr(
    &mut self,
    mut expr: Box<Expression>,
  ) -> Result<Box<Expression>> {
    use ExpressionKind as e;
    expr.kind = match expr.kind {
      //  Type assigned in naming step
      e::Immediate(immediate) => e::Immediate(immediate),
      e::Identifier(name, mangle) => {
        expr.type_ = self
          .table
          .resolve_type(&mangle)
          .trace_span(expr.span, format!("While resolving type of '{name}'"))?;
        e::Identifier(name, mangle)
      },
      e::Binary {
        op,
        mut left,
        mut right,
      } => {
        left = self.bottom_up_expr(left)?;
        right = self.bottom_up_expr(right)?;
        expr.type_ =
          Type::binary_op(&left.type_, op, &right.type_).span(&expr.span)?;
        e::Binary { op, left, right }
      },
      e::Unary { op, mut child } => {
        child = self.bottom_up_expr(child)?;
        expr.type_ = Type::unary_op(op, &child.type_)?;
        e::Unary { op, child }
      },
      e::Parenthesis(mut expression) => {
        expression = self.bottom_up_expr(expression)?;
        expr.type_ = expression.type_.clone();
        e::Parenthesis(expression)
      },
      e::FunctionDef {
        mut params,
        returns_str,
        mut returns_actual,
        mut body,
        id,
      } => {
        let funcdef = self.table.functions[id].clone();
        self.table.start_function();
        for (uid, param) in funcdef.params.iter().zip(params.iter_mut()) {
          let type_ = self.table.resolve_type(uid).span(&expr.span)?;
          param.type_actual = type_;
        }
        for s in &mut body {
          *s = *self.bottom_up_stmt(s.clone().into())?;
        }
        self.table.end_function();
        returns_actual =
          self.table.resolve_type(&funcdef.returns).span(&expr.span)?;
        e::FunctionDef {
          params,
          returns_str,
          returns_actual,
          body,
          id,
        }
      },
      e::FunctionCall {
        mut callee,
        mut args,
        is_reference,
        id,
      } => {
        callee = self.bottom_up_expr(callee)?;
        match callee.type_ {
          Type::Function(fid) => {
            expr.type_ = self
              .table
              .resolve_type(&self.table.functions[fid].returns)
              .span(&callee.span)?
              .is_alias()
              .span(&expr.span)?
          },
          _ => {
            return error()
              .reason(format!("Cannot call type {}", callee.type_))
              .span(&expr.span);
          },
        }
        for a in &mut args {
          *a = *self.bottom_up_expr(a.clone().into())?;
        }
        e::FunctionCall {
          callee,
          args,
          is_reference,
          id,
        }
      },
      e::StructDef(mut params, sid) => {
        let struct_ = &self.table.structs[sid].0;
        for ((_, uid), p) in struct_.iter().zip(params.iter_mut()) {
          p.type_actual = self
            .table
            .resolve_type(uid)
            .trace_span(expr.span, "For struct field")?;
        }
        e::StructDef(params, sid)
      },
      e::StructLiteral { name, mut args, id } => {
        expr.type_ = self
          .table
          .resolve_type(&id)
          .span(&expr.span)?
          .is_alias()
          .span(&expr.span)?;
        for (_name, arg) in &mut args {
          *arg = *self.bottom_up_expr(arg.clone().into())?;
        }
        e::StructLiteral { name, args, id }
      },
      e::Field {
        mut namespace,
        field,
        uid,
      } => {
        namespace = self.bottom_up_expr(namespace)?;
        if let Type::Struct(s) = namespace.type_ {
          if let e::Identifier(name, _) = &field.kind {
            expr.type_ = self
              .table
              .get_field(s, name)
              .span(&expr.span)?
              .is_alias()
              .reason("Expected type, found value")
              .span(&expr.span)?;
          } else {
            return error().reason("Field must be identifier").span(&expr.span);
          }
        } else {
          return error()
            .reason(format!(
              "Type '{}' does not contain fields",
              namespace.type_
            ))
            .span(&expr.span);
        }
        // TODO handle name mangle
        e::Field {
          namespace,
          field,
          uid,
        }
      },
    };
    Ok(expr)
  }
}
