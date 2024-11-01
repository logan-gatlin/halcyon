use super::{Analyzer, Type};
use crate::{
  Expression, ExpressionKind, Statement, StatementKind, err::*,
  semantic::Primitive,
};

impl Analyzer {
  pub fn top_down_stmt(
    &mut self,
    mut stmt: Box<Statement>,
  ) -> Result<Box<Statement>> {
    use StatementKind as s;
    stmt.kind = match stmt.kind {
      s::Declaration {
        name,
        type_str,
        type_actual,
        mut value,
        mutable,
        uid,
      } => {
        value = *self.top_down_expr(value.into(), &type_actual)?;
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
        let type_ = self.table.resolve_type(&uid).span(&stmt.span)?;
        value = *self.top_down_expr(value.into(), &type_)?;
        s::Assignment { name, value, uid }
      },
      s::If {
        mut predicate,
        mut block,
        mut else_,
      } => {
        predicate = *self
          .top_down_expr(predicate.into(), &Type::Prim(Primitive::boolean))?;
        for s in &mut block {
          *s = *self.top_down_stmt(s.clone().into())?;
        }
        else_ = if let Some(else_) = else_ {
          Some(self.top_down_stmt(else_)?)
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
        predicate = *self
          .top_down_expr(predicate.into(), &Type::Prim(Primitive::boolean))?;
        for s in &mut block {
          *s = *self.top_down_stmt(s.clone().into())?;
        }
        s::While { predicate, block }
      },
      s::Print(mut expr) => {
        let expr_t = expr.type_.clone();
        expr = *self.top_down_expr(expr.into(), &expr_t)?;
        s::Print(expr)
      },
      s::Expression(mut expr) => {
        let expr_t = expr.type_.clone();
        expr = *self.top_down_expr(expr.into(), &expr_t)?;
        s::Expression(expr)
      },
      s::Block(mut block) => {
        for s in &mut block {
          *s = *self.top_down_stmt(s.clone().into())?;
        }
        s::Block(block)
      },
      s::Return(expr) => {
        if let Some(mut expr) = expr {
          let expr_t = expr.type_.clone();
          expr = *self.top_down_expr(expr.into(), &expr_t)?;
          s::Return(Some(expr))
        } else {
          s::Return(None)
        }
      },
      s::Error(diagnostic) => s::Error(diagnostic),
    };
    Ok(stmt)
  }

  pub fn top_down_expr(
    &mut self,
    mut expr: Box<Expression>,
    expect: &Type,
  ) -> Result<Box<Expression>> {
    use ExpressionKind as e;
    expr.type_ = expr.type_.coerce(expect).span(&expr.span)?;
    expr.kind = match expr.kind {
      e::Immediate(i) => e::Immediate(i),
      e::Identifier(i, m) => e::Identifier(i, m),
      e::Binary {
        op,
        mut left,
        mut right,
      } => {
        left = self.top_down_expr(left, expect)?;
        right = self.top_down_expr(right, expect)?;
        e::Binary { op, left, right }
      },
      e::Unary { op, mut child } => {
        child = self.top_down_expr(child, expect)?;
        e::Unary { op, child }
      },
      e::Parenthesis(mut expression) => {
        expression = self.top_down_expr(expression, expect)?;
        e::Parenthesis(expression)
      },
      e::FunctionDef {
        params,
        returns_str,
        returns_actual,
        mut body,
        id,
      } => {
        for s in &mut body {
          *s = *self.top_down_stmt(s.clone().into())?;
        }
        e::FunctionDef {
          params,
          returns_str,
          returns_actual,
          body,
          id,
        }
      },
      e::FunctionCall {
        callee,
        mut args,
        is_reference,
        id,
      } => {
        let func_def = if let Type::Function(fid) = callee.type_ {
          self.table.functions[fid].clone()
        } else {
          unreachable!()
        };
        if func_def.params.len() != args.len() {
          return error()
            .reason(format!(
              "Wrong number of arguments; found {}, expected {}",
              args.len(),
              func_def.params.len()
            ))
            .span(&expr.span);
        }
        for (param, arg) in func_def.params.iter().zip(args.iter_mut()) {
          let type_ = self.table.resolve_type(param).span(&arg.span)?;
          *arg = *self.top_down_expr(arg.clone().into(), &type_)?;
        }
        e::FunctionCall {
          callee,
          args,
          is_reference,
          id,
        }
      },
      e::StructDef(params, sid) => e::StructDef(params, sid),
      e::StructLiteral { name, mut args, id } => {
        let sid = if let Type::Struct(sid) = expr.type_ {
          sid
        } else {
          return error()
            .reason(format!("Cannot create struct from type {}", expr.type_))
            .span(&expr.span);
        };
        if self.table.structs[sid].0.len() != args.len() {
          return error()
            .reason(format!(
              "Wrong number of parameters; found {}, expected {}",
              args.len(),
              self.table.structs[sid].0.len()
            ))
            .span(&expr.span);
        }
        let mut declared_params: Vec<String> = vec![];
        for (name, arg) in args.iter_mut() {
          if declared_params.contains(name) {
            return error()
              .reason(format!("Field {name} initialized more than once"))
              .span(&arg.span);
          }
          declared_params.push(name.clone());
          let param_t = self
            .table
            .get_field(sid, &name)
            .span(&arg.span)?
            .is_alias()
            .span(&arg.span)?;
          *arg = *self.top_down_expr(arg.clone().into(), &param_t)?;
        }
        e::StructLiteral { name, args, id }
      },
      e::Field {
        namespace,
        field,
        uid,
      } => e::Field {
        namespace,
        field,
        uid,
      },
    };
    Ok(expr)
  }
}
