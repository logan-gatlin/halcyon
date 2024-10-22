use crate::{
  BinaryOp, Expression, ExpressionKind, Immediate, Parameter, Statement,
  StatementKind, UnaryOp,
};

use super::primitives::*;
use crate::err::*;

#[derive(Debug, Clone)]
pub enum Type {
  Ambiguous,
  Nothing,
  Prim(Primitive),
  Struct(Vec<Parameter>),
  Function {
    params: Vec<Type>,
    returns: Box<Type>,
  },
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type::*;
    match (self, other) {
      (Ambiguous, Ambiguous) => true,
      (Prim(p1), Prim(p2)) if p1 == p2 => true,
      (Struct(p1), Struct(p2)) => p1
        .iter()
        .map(|p| p.type_actual.clone())
        .eq(p2.iter().map(|p| p.type_actual.clone())),
      (
        Function {
          params: p1,
          returns: r1,
        },
        Function {
          params: p2,
          returns: r2,
        },
      ) => p1.iter().eq(p2.iter()) && r1 == r2,
      (Nothing, Nothing) => true,
      _ => false,
    }
  }
}

impl Type {
  pub fn from_string(value: &str) -> Self {
    if let Some(p) = Primitive::from_string(value) {
      Type::Prim(p)
    } else {
      Type::Ambiguous
    }
  }

  pub fn binary_op(lhs: &Type, op: BinaryOp, rhs: &Type) -> Result<Type> {
    use Type as t;
    let e = error().reason(format!(
      "Binary {op} is not defined for {lhs:?} and {rhs:?}",
    ));
    match (lhs, rhs) {
      (t::Prim(a), t::Prim(b)) => {
        let p = Primitive::binary_op(*a, op, *b)?;
        Ok(t::Prim(p))
      },
      _ => e,
    }
  }

  pub fn unary_op(op: UnaryOp, child: &Type) -> Result<Type> {
    use Type as t;
    if let t::Prim(p) = child {
      let p = Primitive::unary_op(op, *p)?;
      Ok(t::Prim(p))
    } else {
      error().reason(format!("Unary {op} is not defined for {child:?}"))
    }
  }

  fn coerce(expect: &Type, actual: &Type) -> Option<Type> {
    use Primitive as p;
    use Type::*;
    match (expect, actual) {
      (Ambiguous, Ambiguous) => None,
      (Ambiguous, Prim(p::integer_ambiguous)) => Some(Prim(p::integer)),
      (Ambiguous, Prim(p::real_ambiguous)) => Some(Prim(p::real)),
      (Ambiguous, t) => Some(t.clone()),
      (Prim(p1), Prim(p2)) => {
        let (p1, p2) = Primitive::coerce_ambiguous(*p1, *p2);
        if p1 != p2 { None } else { Some(Type::Prim(p1)) }
      },
      _ => None,
    }
  }
}

#[derive(Debug, Clone)]
enum Symbol {
  Var(String, Type, bool),
  Type(String, Type),
  BlockStart,
}

#[derive(Debug, Clone)]
struct SymbolTable {
  syms: Vec<Symbol>,
}

impl SymbolTable {
  fn define_var(&mut self, name: String, type_: Type, mutable: bool) {
    self.syms.push(Symbol::Var(name, type_, mutable));
  }

  fn define_type(&mut self, name: String, type_: Type) {
    self.syms.push(Symbol::Type(name, type_));
  }

  fn start_block(&mut self) {
    self.syms.push(Symbol::BlockStart);
  }

  fn end_block(&mut self) {
    while !self.syms.is_empty() {
      if let Some(Symbol::BlockStart) = self.syms.pop() {
        return;
      }
    }
    unreachable!("Tried to exit global scope in symbol table")
  }

  fn get_var(&self, name: &str) -> Result<(Type, bool)> {
    for s in self.syms.iter().rev() {
      if let Symbol::Var(name2, type_, mutable) = s {
        if name == name2 {
          return Ok((type_.clone(), *mutable));
        }
      }
    }
    error().reason(format!("Identifier {name} is not defined"))
  }

  fn get_type(&self, name: &str) -> Result<Type> {
    for s in self.syms.iter().rev() {
      if let Symbol::Type(name2, t) = s {
        if name == name2 {
          return Ok(t.clone());
        }
      }
    }
    if let Some(p) = Primitive::from_string(name) {
      return Ok(Type::Prim(p));
    }
    error().reason(format!("Type {name} is not defined"))
  }
}

pub fn typecheck(program: Vec<Statement>) -> Vec<Statement> {
  use StatementKind as s;
  let mut table = SymbolTable { syms: vec![] };
  let mut ret = vec![];
  for s in program {
    let span = s.span;
    ret.push(match statement(s.into(), &mut table) {
      Ok(s) => *s,
      Err(e) => Statement {
        kind: s::Error(e),
        span,
      },
    })
  }
  ret
}

fn statement(
  mut stmt: Box<Statement>,
  table: &mut SymbolTable,
) -> Result<Box<Statement>> {
  use Primitive as p;
  use StatementKind as s;
  match stmt.kind {
    s::Declaration {
      name,
      type_str,
      value,
      mutable,
      ..
    } => {
      let type_expect = match type_str {
        Some(ref s) => table.get_type(s).span(&stmt.span)?,
        None => Type::Ambiguous,
      };
      let value = expression(value.into(), table)?;
      let type_actual =
        Type::coerce(&type_expect, &value.type_).reason(format!(
          "Expected type '{:?}', found type '{:?}'",
          type_expect, value.type_
        ))?;
      // Check that structs are const
      if let Type::Struct(_) = type_actual {
        if mutable {
          return error()
            .reason("Struct declarations must be immutable")
            .span(&stmt.span);
        }
        table.define_type(name.clone(), type_actual.clone());
      }
      // Check that functions are const
      else if let Type::Function { .. } = type_actual {
        if mutable {
          return error()
            .reason("Function declarations must be immutable")
            .span(&stmt.span);
        }
        table.define_var(name.clone(), type_actual.clone(), false);
      } else {
        table.define_var(name.clone(), type_actual.clone(), mutable);
      }
      stmt.kind = s::Declaration {
        name,
        type_str,
        type_actual,
        value: *value,
        mutable,
      };
    },
    s::Assignment { name, value } => {
      let (type_, mutable) = table.get_var(&name).span(&stmt.span)?;
      // Check that it is mutable
      if !mutable {
        return error()
          .reason(format!("Cannot assign to immutable '{}'", name))
          .span(&stmt.span);
      }
      let value = *expression(value.into(), table)?;
      // Check for correct type
      if type_ != value.type_ {
        return error().reason(format!(
          "Attempted to assign '{:?}' to '{type_:?}'",
          value.type_
        ));
      }
      stmt.kind = s::Assignment { name, value };
    },
    s::If {
      predicate,
      block,
      else_,
    } => {
      table.start_block();
      let predicate = *expression(predicate.into(), table)?;
      if predicate.type_ != Type::Prim(p::boolean) {
        return error()
          .reason(format!(
            "Predicate of if statement must be a boolean, found {:?}",
            predicate.type_
          ))
          .span(&predicate.span);
      }
      let mut new_block = vec![];
      for stmt in block {
        new_block.push(*statement(stmt.into(), table)?);
      }
      let else_ = if let Some(else_) = else_ {
        Some(statement(else_, table)?)
      } else {
        None
      };
      stmt.kind = s::If {
        predicate,
        block: new_block,
        else_,
      };
      table.end_block();
    },
    s::While { predicate, block } => {
      table.start_block();
      let predicate = *expression(predicate.into(), table)?;
      if predicate.type_ != Type::Prim(p::boolean) {
        return error()
          .reason(format!(
            "Predicate of while statement must be a boolean, found {:?}",
            predicate.type_
          ))
          .span(&predicate.span);
      }
      let mut new_block = vec![];
      for stmt in block {
        new_block.push(*statement(stmt.into(), table)?);
      }
      stmt.kind = s::While {
        predicate,
        block: new_block,
      };
      table.end_block();
    },
    s::Print(e) => {
      stmt.kind = s::Print(*expression(e.into(), table)?);
    },
    s::Expression(mut e) => {
      use ExpressionKind as e;
      let is_func = if let e::Function { params, .. } = &mut e.kind {
        // TODO: start/end function instead
        table.start_block();
        for p in params {
          p.type_actual = table.get_type(&p.type_str)?;
          table.define_var(p.name.clone(), p.type_actual.clone(), false);
        }
        true
      } else {
        false
      };
      stmt.kind = s::Expression(*expression(e.into(), table)?);
      if is_func {
        table.end_block();
      }
    },
    s::Block(block) => {
      table.start_block();
      let mut new_block = vec![];
      for stmt in block {
        new_block.push(*statement(stmt.into(), table)?);
      }
      stmt.kind = s::Block(new_block);
      table.end_block();
    },
    s::Error(e) => return Err(e),
  }
  Ok(stmt)
}

fn expression(
  mut expr: Box<Expression>,
  table: &SymbolTable,
) -> Result<Box<Expression>> {
  use ExpressionKind as e;
  use Immediate as i;
  use Primitive as p;
  let type_ = match expr.kind {
    e::Immediate(ref i) => Type::Prim(match i {
      i::Integer(_) => p::integer_ambiguous,
      i::Real(_) => p::real_ambiguous,
      i::String(_) => p::string,
      i::Boolean(_) => p::boolean,
    }),
    e::Identifier(ref i) => table.get_var(i)?.0,
    e::Binary { op, left, right } => {
      let left = expression(left, table)?;
      let right = expression(right, table)?;
      let type_ = Type::binary_op(&left.type_, op, &right.type_)?;
      expr.kind = e::Binary { left, right, op };
      type_
    },
    e::Unary { op, child } => {
      let child = expression(child, table)?;
      let type_ = Type::unary_op(op, &child.type_)?;
      expr.kind = e::Unary { child, op };
      type_
    },
    e::Parenthesis(inner) => {
      let inner = expression(inner, table)?;
      let type_ = inner.type_.clone();
      expr.kind = e::Parenthesis(inner);
      type_
    },
    e::Function {
      mut params,
      returns_str,
      mut returns_actual,
      body,
    } => {
      for p in &mut params {
        p.type_actual = table.get_type(&p.type_str).span(&expr.span)?;
      }
      returns_actual = match &returns_str {
        Some(s) => table.get_type(s).span(&expr.span)?,
        None => Type::Nothing,
      };
      expr.kind = e::Function {
        params: params.clone(),
        returns_str,
        returns_actual: returns_actual.clone(),
        body,
      };
      Type::Function {
        params: params.into_iter().map(|p| p.type_actual).collect(),
        returns: returns_actual.into(),
      }
    },
    e::Struct(mut params) => {
      for p in &mut params {
        p.type_actual = table.get_type(&p.type_str).span(&expr.span)?;
      }
      expr.kind = e::Struct(params.clone());
      Type::Struct(params)
    },
    e::StructLiteral { name, mut args } => {
      let type_ = table.get_type(&name).span(&expr.span)?;
      let Type::Struct(params) = type_ else {
        return error().reason(format!(
          "Cannot construct type {:?} as struct literal",
          type_
        ));
      };
      if args.len() != params.len() {
        return error().reason(format!(
          "Incorrect number of parameters for struct '{}', expected {} and \
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
        let arg = *expression(argexpr.clone().into(), table)
          .trace_span(expr.span, "while parsing struct literal")?;
        if &arg.type_ != ptype {
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
    e::Call { callee, args } => {
      let callee = expression(callee, table)?;
      // Check that this is actually a function
      let Type::Function {
        ref params,
        ref returns,
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
            "Wrong number of arguments, function expects {}",
            params.len()
          ))
          .span(&callee.span);
      }
      // Check for correct arg types
      for (expect, actual) in params.iter().zip(args.iter()) {
        if *expect != actual.type_ {
          return error()
            .reason(format!(
              "Expected type {expect:?}, found {:?}",
              actual.type_
            ))
            .span(&actual.span);
        }
      }
      let returns = *returns.clone();
      expr.kind = e::Call { callee, args };
      returns
    },
    e::Field { namespace, field } => {
      let namespace = expression(namespace, table)?;
      // Check that namespace is struct
      // TODO: fields in other types
      let Type::Struct(ref params) = namespace.type_ else {
        return error()
          .reason(format!("Type {:?} does not have fields", namespace.type_))
          .span(&namespace.span);
      };
      // Check that field is identifier
      // TODO: tuple fields?
      let e::Identifier(ref name) = field.kind else {
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
  expr.type_ = type_;
  Ok(expr)
}
