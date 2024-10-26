use crate::{
  Expression, ExpressionKind, Statement, err::*, semantic::SymbolTable,
};

use super::Type;

pub struct Analyzer {
  table: SymbolTable,
}

impl Analyzer {
  pub fn typecheck(statements: Vec<Statement>) -> Vec<Statement> {
    /*
    let mut this = Self {
      table: SymbolTable::new(),
    };
    this.block(statements)
    */
    statements
  }

  // Bottom up type inference
  fn propogate_up(
    &mut self,
    mut expr: Box<Expression>,
  ) -> Result<Box<Expression>> {
    use ExpressionKind as e;
    expr.kind = match expr.kind {
      e::Immediate(imm) => e::Immediate(imm),
      e::Identifier(id, mut mangle) => {
        let symbol = self.table.lookup(&id).span(&expr.span)?;
        mangle = symbol.uid;
        expr.type_ = symbol.type_;
        e::Identifier(id, mangle)
      },
      e::Binary {
        op,
        mut left,
        mut right,
      } => {
        left = self.propogate_up(left)?;
        right = self.propogate_up(right)?;
        expr.type_ =
          Type::binary_op(&left.type_, op, &right.type_).span(&expr.span)?;
        e::Binary { op, left, right }
      },
      e::Unary { op, mut child } => {
        child = self.propogate_up(child)?;
        expr.type_ = Type::unary_op(op, &child.type_).span(&expr.span)?;
        e::Unary { op, child }
      },
      e::Parenthesis(mut inner) => {
        inner = self.propogate_up(inner)?;
        expr.type_ = inner.type_.clone();
        e::Parenthesis(inner)
      },
      // Define anonymous function, do not evaluate body yet
      e::FunctionDef {
        mut params,
        returns_str,
        mut returns_actual,
        body,
        mut id,
      } => {
        for p in &mut params {
          p.type_actual =
            self.table.lookup_typedef(&p.type_str).span(&expr.span)?;
        }
        let param_types: Vec<_> =
          params.iter().map(|p| p.type_actual.clone()).collect();
        returns_actual = match returns_str {
          Some(ref s) => self.table.lookup_typedef(s).span(&expr.span)?,
          None => Type::Nothing,
        };
        id = self
          .table
          .start_func(param_types.clone(), returns_actual.clone());
        expr.type_ = Type::FunctionDef {
          params: param_types,
          returns: returns_actual.clone().into(),
          id: id.clone(),
        };
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
        mut is_reference,
        mut id,
      } => {
        callee = self.propogate_up(callee)?;
        let (params, returns) = match callee.type_.clone() {
          Type::FunctionRef {
            id: uid,
            params,
            returns,
          } => {
            is_reference = true;
            id = uid;
            (params, returns)
          },
          Type::FunctionDef {
            id: uid,
            params,
            returns,
          } => {
            is_reference = false;
            id = uid;
            (params, returns)
          },
          _ => {
            return error()
              .reason(format!("Cannot call type '{}'", callee.type_))
              .span(&expr.span);
          },
        };
        expr.type_ = *returns;
        // Check that args are correct
        if args.len() != params.len() {
          return error().reason(format!(
            "Expected {} arguments, found {}",
            params.len(),
            args.len()
          ));
        }
        for (index, a) in args.iter_mut().enumerate() {
          *a = *self.propogate_up(a.clone().into())?;
          *a = *self.propogate_down(a.clone().into(), params[index].clone())?;
        }
        e::FunctionCall {
          callee,
          args,
          is_reference,
          id,
        }
      },
      e::StructDef(vec) => todo!(),
      e::StructLiteral { ref name, mut args } => {
        self.table.lookup_typedef(name).span(&expr.span)?;
        todo!()
      },
      e::Field {
        namespace,
        field,
        uid,
      } => todo!(),
    };
    Ok(expr)
  }

  // Top down type assertion
  fn propogate_down(
    &mut self,
    expr: Box<Expression>,
    expect: Type,
  ) -> Result<Box<Expression>> {
    todo!()
  }

  /*
  fn block(&mut self, block: Vec<Statement>) -> Vec<Statement> {
    let mut new_block = vec![];
    for st in block {
      let span = st.span;
      let st = match self.statement(st.into()) {
        Ok(st) => *st,
        Err(e) => Statement {
          kind: StatementKind::Error(e),
          span,
        },
      };
      new_block.push(st);
    }
    new_block
  }

  fn statement(&mut self, mut stmt: Box<Statement>) -> Result<Box<Statement>> {
    use Primitive as p;
    use StatementKind as s;
    match stmt.kind {
      // Variable declaration
      s::Declaration {
        name,
        type_str,
        value,
        mutable,
        ..
      } => {
        let type_lhs = if let Some(ref s) = type_str {
          self.table.get_type(s).span(&stmt.span)?
        } else {
          Type::Ambiguous
        };
        let type_hint = if let Type::Ambiguous = type_lhs {
          None
        } else {
          Some(type_lhs.clone())
        };
        let mut value = self.expression(value.into(), type_hint)?;
        let type_actual = Type::coerce(&type_lhs, &value.type_)
          .reason(format!(
            "Expected type '{:?}', found type '{:?}'",
            type_lhs, value.type_
          ))
          .span(&stmt.span)?;
        value.type_ = type_actual.clone();
        let varkind = self
          .table
          .define_symbol(name.clone(), type_actual.clone(), mutable)
          .span(&stmt.span)?;
        stmt.kind = s::Declaration {
          name,
          type_str,
          type_actual,
          value: *value,
          mutable,
          varkind,
        };
      },
      // Variable assignment
      s::Assignment { name, value, .. } => {
        let symbol = self.table.find_symbol(&name).span(&stmt.span)?;
        // Check that it is mutable
        if !symbol.mutable {
          return error()
            .reason(format!("Cannot assign to immutable '{}'", name))
            .span(&stmt.span);
        }
        let mut value =
          *self.expression(value.into(), Some(symbol.type_.clone()))?;
        let type_actual =
          Type::coerce(&symbol.type_, &value.type_).span(&stmt.span)?;
        value.type_ = type_actual;
        stmt.kind = s::Assignment {
          name,
          value,
          varkind: symbol.kind,
        };
      },
      s::If {
        predicate,
        block,
        else_,
      } => {
        self.table.start_block();
        let predicate =
          *self.expression(predicate.into(), Some(Type::Prim(p::boolean)))?;
        Type::coerce(&Type::Prim(p::boolean), &predicate.type_)
          .span(&predicate.span)?;
        let block = self.block(block);
        let else_ = if let Some(else_) = else_ {
          Some(self.statement(else_)?)
        } else {
          None
        };
        stmt.kind = s::If {
          predicate,
          block,
          else_,
        };
        self.table.end_block();
      },
      s::While { predicate, block } => {
        self.table.start_block();
        let predicate =
          *self.expression(predicate.into(), Some(Type::Prim(p::boolean)))?;
        Type::coerce(&Type::Prim(p::boolean), &predicate.type_)
          .span(&predicate.span)?;
        let block = self.block(block);
        stmt.kind = s::While { predicate, block };
        self.table.end_block();
      },
      s::Print(e) => {
        let mut e = *self.expression(e.into(), None)?;
        e.type_ = Type::coerce(&Type::Prim(p::integer), &e.type_)?;
        stmt.kind = s::Print(e);
      },
      s::Expression(e) => {
        let mut expr = *self.expression(e.into(), None)?;
        expr.type_ =
          Type::coerce(&Type::Ambiguous, &expr.type_).span(&expr.span)?;
        stmt.kind = s::Expression(expr);
      },
      s::Block(block) => {
        self.table.start_block();
        let block = self.block(block);
        stmt.kind = s::Block(block);
        self.table.end_block();
      },
      s::Error(e) => return Err(e),
      s::Return(mut expression) => {
        let return_type = self.table.get_return_type().span(&stmt.span)?;
        let type_ = match expression {
          Some(e) => {
            let mut e = self.expression(e.into(), Some(return_type.clone()))?;
            e.type_ = Type::coerce(&return_type, &e.type_).span(&e.span)?;
            let type_ = e.type_.clone();
            expression = Some(*e);
            type_
          },
          None => Type::Nothing,
        };
        Type::coerce(&return_type, &type_).span(&stmt.span)?;
        stmt.kind = s::Return(expression);
      },
    }
    Ok(stmt)
  }

  fn expression(
    &mut self,
    mut expr: Box<Expression>,
    type_hint: Option<Type>,
  ) -> Result<Box<Expression>> {
    // TODO implement type hinting
    use ExpressionKind as e;
    use Immediate as i;
    use Primitive as p;
    let type_ = match expr.kind {
      e::Immediate(ref i) => Type::Prim(match i {
        i::Integer(_, _) => p::integer_ambiguous,
        i::Real(_) => p::real_ambiguous,
        i::String(_) => p::string,
        i::Boolean(_) => p::boolean,
      }),
      e::Identifier(ref i, ref mut kind) => {
        let symbol = self.table.find_symbol(i)?;
        *kind = symbol.kind;
        self.table.find_symbol(i)?.type_
      },
      e::Binary { op, left, right } => {
        let left = self.expression(left, type_hint.clone())?;
        let right = self.expression(right, type_hint.clone())?;
        let type_ = Type::binary_op(&left.type_, op, &right.type_)?;
        expr.kind = e::Binary { left, right, op };
        type_
      },
      e::Unary { op, child } => {
        let child = self.expression(child, type_hint.clone())?;
        let type_ = Type::unary_op(op, &child.type_)?;
        expr.kind = e::Unary { child, op };
        type_
      },
      e::Parenthesis(inner) => {
        let inner = self.expression(inner, type_hint.clone())?;
        let type_ = inner.type_.clone();
        expr.kind = e::Parenthesis(inner);
        type_
      },
      e::FunctionDef {
        mut params,
        returns_str,
        mut returns_actual,
        body,
        id: _,
      } => {
        returns_actual = match &returns_str {
          Some(s) => self.table.get_type(s).span(&expr.span)?,
          None => Type::Nothing,
        };
        let id = self.table.start_func(returns_actual.clone());
        for p in &mut params {
          p.type_actual = self.table.get_type(&p.type_str).span(&expr.span)?;
          self
            .table
            .define_param(p.name.clone(), p.type_actual.clone())?;
        }
        let body = self.block(body);
        self.table.end_func();
        expr.kind = e::FunctionDef {
          params: params.clone(),
          returns_str,
          returns_actual: returns_actual.clone(),
          body,
          id,
        };
        Type::FunctionDef {
          params: params.into_iter().map(|p| p.type_actual).collect(),
          returns: returns_actual.into(),
          id,
        }
      },
      e::FunctionCall { callee, mut args } => {
        let callee = self.expression(callee, None)?;
        // Check that this is actually a function
        // TODO allow function references to be called
        let Type::FunctionDef {
          ref params,
          ref returns,
          ..
        } = callee.type_
        else {
          return error()
            .reason(format!("Cannot call type {:?}", callee.type_))
            .span(&callee.span);
        };
        // Check for correct number of args
        if params.len() != args.len() {
          return error()
            .reason(format!(
              "Wrong number of arguments, function expects {}, found {}",
              params.len(),
              args.len()
            ))
            .span(&callee.span);
        }
        // Check for correct arg types
        for (expect, actual) in params.iter().zip(args.iter_mut()) {
          *actual =
            *self.expression(actual.clone().into(), Some(expect.clone()))?;
          let coerced_type = Type::coerce(expect, &actual.type_);
          if let Ok(t) = coerced_type {
            actual.type_ = t;
          } else {
            return error()
              .reason(format!(
                "Expected type {expect:?}, found {:?}",
                actual.type_
              ))
              .span(&actual.span);
          }
        }
        let returns = *returns.clone();
        expr.kind = e::FunctionCall { callee, args };
        returns
      },
      e::StructDef(mut params) => {
        for p in &mut params {
          p.type_actual = self.table.get_type(&p.type_str).span(&expr.span)?;
        }
        expr.kind = e::StructDef(params.clone());
        Type::StructDef(params)
      },
      e::StructLiteral { name, args } => {
        let type_ = self.table.get_type(&name).span(&expr.span)?;
        let Type::Struct(params) = type_ else {
          return error().reason(format!(
            "Cannot construct type {:?} as struct literal",
            type_
          ));
        };
        if args.len() != params.len() {
          return error().reason(format!(
            "Incorrect number of parameters for struct '{}'; expected {}, \
             found {}",
            name,
            params.len(),
            args.len()
          ));
        }
        // TODO out of order params
        let mut new_args = vec![];
        for (
          (argname, argexpr),
          Parameter {
            name: pname,
            type_actual: ptype,
            ..
          },
        ) in args.iter().zip(params.iter())
        {
          if argname != pname {
            return error()
              .reason(format!(
                "In struct literal, expected parameter '{pname}', found \
                 '{argname}'"
              ))
              .span(&argexpr.span);
          }
          let argspan = argexpr.span;
          let mut arg = *self
            .expression(argexpr.clone().into(), Some(ptype.clone()))
            .trace_span(expr.span, "while parsing struct literal")?;
          let coerced_type = Type::coerce(ptype, &arg.type_);
          if let Ok(t) = coerced_type {
            arg.type_ = t;
          } else {
            return error()
              .reason(format!(
                "In struct literal, expected type '{ptype:?}', found '{:?}",
                arg.type_,
              ))
              .span(&argspan);
          }
          new_args.push((argname.clone(), arg));
        }
        expr.kind = e::StructLiteral {
          name,
          args: new_args,
        };
        Type::Struct(params)
      },
      e::Field { namespace, field } => {
        let namespace = self.expression(namespace, None)?;
        // Check that namespace is struct
        // TODO: fields in other types
        let Type::Struct(ref params) = namespace.type_ else {
          return error()
            .reason(format!("Type {:?} does not have fields", namespace.type_))
            .span(&namespace.span);
        };
        // Check that field is identifier
        // TODO: tuple fields?
        let e::Identifier(ref name, _) = field.kind else {
          return error()
            .reason("Field must be an identifier")
            .span(&field.span);
        };
        let mut type_ = None;
        for p in params {
          if &p.name == name {
            type_ = Some(p.type_actual.clone());
            break;
          }
        }
        let type_ = type_
          .reason(format!(
            "Type {:?} does not contain field {}",
            namespace, name
          ))
          .span(&field.span)?;
        expr.kind = e::Field { namespace, field };
        type_
      },
    };
    /*
    if let Some(expect) = &type_hint {
      if let Type::Ambiguous = expect {
      } else {
        expr.type_ = Type::coerce(expect, &expr.type_)?;
      }
    }
    */
    expr.type_ = type_;
    Ok(expr)
  }
  */
}
