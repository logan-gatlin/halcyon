use super::Analyzer;
use crate::{
  Expression, ExpressionKind, Immediate, Statement, StatementKind, err::*,
  semantic::Primitive,
};

use super::Type;

impl Analyzer {
  pub fn naming_pass_stmt(
    &mut self,
    mut stmt: Box<Statement>,
  ) -> Result<Box<Statement>> {
    use StatementKind as s;
    stmt.kind = match stmt.kind {
      s::Declaration {
        name,
        mut type_str,
        type_actual,
        mut value,
        mutable,
        mut uid,
      } => {
        value = *self.naming_pass_expr(value.into())?;
        let type_ = if let Some(ref s) = type_str {
          let s = self.table.reference_ident(s);
          type_str = Some(s.uid.clone());
          s.type_.is_alias().unwrap_or(Type::Ambiguous)
        } else {
          value.type_.clone()
        };
        let symbol = self.table.define_ident(&name, type_, mutable)?;
        uid = symbol.uid;
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
        mut uid,
      } => {
        uid = self.table.reference_ident(&name).uid;
        value = *self.naming_pass_expr(value.into())?;
        // Hint ident is rhs type and mutable
        self.table.modify_ident(
          uid.clone(),
          None,
          Some(value.type_.clone()),
          Some(true),
          false,
          false,
        )?;
        s::Assignment { name, value, uid }
      },
      s::If {
        mut predicate,
        mut block,
        mut else_,
      } => {
        predicate = *self.naming_pass_expr(predicate.into())?;
        self.table.start_block();
        for s in &mut block {
          *s = *self.naming_pass_stmt(s.clone().into())?;
        }
        self.table.end_block();
        else_ = if let Some(else_) = else_ {
          Some(self.naming_pass_stmt(else_)?)
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
        predicate = *self.naming_pass_expr(predicate.into())?;
        self.table.start_block();
        for s in &mut block {
          *s = *self.naming_pass_stmt(s.clone().into())?;
        }
        self.table.end_block();
        s::While { predicate, block }
      },
      s::Print(mut expression) => {
        expression = *self.naming_pass_expr(expression.into())?;
        s::Print(expression)
      },
      s::Expression(mut expression) => {
        expression = *self.naming_pass_expr(expression.into())?;
        s::Expression(expression)
      },
      s::Block(mut block) => {
        self.table.start_block();
        for s in &mut block {
          *s = *self.naming_pass_stmt(s.clone().into())?;
        }
        self.table.end_block();
        s::Block(block)
      },
      s::Return(mut expression) => {
        if let Some(e) = expression {
          expression = Some(*self.naming_pass_expr(e.into())?);
        }
        s::Return(expression)
      },
      s::Error(diagnostic) => s::Error(diagnostic),
    };
    Ok(stmt)
  }

  pub fn naming_pass_expr(
    &mut self,
    mut expr: Box<Expression>,
  ) -> Result<Box<Expression>> {
    use ExpressionKind as e;
    expr.kind = match expr.kind {
      e::Immediate(immediate) => {
        expr.type_ = Type::Prim(match &immediate {
          Immediate::Integer(_, _) => Primitive::integer_ambiguous,
          Immediate::Real(_) => Primitive::real_ambiguous,
          Immediate::String(_) => Primitive::string,
          Immediate::Glyph(_) => Primitive::glyph,
          Immediate::Boolean(_) => Primitive::boolean,
        });
        e::Immediate(immediate)
      },
      e::Identifier(name, mut mangle) => {
        let sym = self.table.reference_ident(&name);
        mangle = sym.uid;
        expr.type_ = sym.type_;
        e::Identifier(name, mangle)
      },
      e::Binary {
        op,
        mut left,
        mut right,
      } => {
        left = self.naming_pass_expr(left)?;
        right = self.naming_pass_expr(right)?;
        e::Binary { op, left, right }
      },
      e::Unary { op, mut child } => {
        child = self.naming_pass_expr(child)?;
        e::Unary { op, child }
      },
      e::Parenthesis(mut expression) => {
        expression = self.naming_pass_expr(expression)?;
        e::Parenthesis(expression)
      },
      e::FunctionDef {
        params,
        returns_str,
        returns_actual,
        mut body,
        mut id,
      } => {
        // Starts function
        id = self
          .table
          .create_function(params.clone(), returns_str.clone())
          .span(&expr.span)?;
        expr.type_ = Type::Function(id.clone());
        for s in &mut body {
          *s = *self.naming_pass_stmt(s.clone().into())?;
        }
        self.table.end_function();
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
        callee = self.naming_pass_expr(callee)?;
        for a in &mut args {
          *a = *self.naming_pass_expr(a.clone().into())?;
        }
        e::FunctionCall {
          callee,
          args,
          is_reference,
          id,
        }
      },
      e::StructDef(params, mut sid) => {
        sid = self.table.create_struct(params.clone());
        expr.type_ = Type::Alias(Box::new(Type::Struct(sid)));
        e::StructDef(params, sid)
      },
      e::StructLiteral {
        name,
        mut args,
        mut id,
      } => {
        id = self.table.reference_ident(&name).uid;
        for (_, a) in &mut args {
          *a = *self.naming_pass_expr(a.clone().into())?;
        }
        e::StructLiteral { name, args, id }
      },
      e::Field {
        mut namespace,
        field,
        uid,
      } => {
        namespace = self.naming_pass_expr(namespace)?;
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
