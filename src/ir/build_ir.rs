use std::{collections::HashSet, rc::Rc};

use super::*;
use crate::{lint::*, parse::*};

pub fn build_ir(
  module: ParsedModule,
  context: &HashMap<String, ModuleInterface>,
) -> Result<IrModule> {
  let mut items = vec![];
  let mut ir = vec![];
  let mut value_ns = NameSpace::new(module.name.clone());
  let mut type_ns = NameSpace::new(module.name.clone());
  let mut ns = NameSpace::new(module.name.clone());
  for expr in module.contents {
    use ModuleExpressionKind as e;
    match expr.kind {
      e::Let {
        assignee,
        assignee_span,
        value,
      } => {
        if &assignee != "_" && !names.insert(assignee.clone()) {
          return Err(lint(
            NameLint::NameRedefinition,
            assignee_span,
            [assignee.clone()],
          ));
        };
        let value = value_expr(&mut ir, &mut ns, *value)?;
        let mangle = if &assignee != "_" {
          ns.new_global(assignee)
        } else {
          let mangle = mangle_name(vec!["_".to_string()], &ns.salt.to_string());
          ns.salt += 1;
          mangle
        };
        items.push(ModuleItem::Let(mangle, value));
      },
      e::Type {
        assignee,
        assignee_span,
        value,
      } => {
        if !names.insert(assignee.clone()) {
          return Err(lint(
            NameLint::NameRedefinition,
            assignee_span,
            [assignee.clone()],
          ));
        };
        // Recursive type
        if matches!(value.kind, TypeDefinitionKind::Sum { .. }) {
          let mangle = ns.push_type(assignee, Type::TypeVariable(0).into());
          let type_ = type_def(&mut ns, *value)?;
          let weak = Type::Weak(Rc::downgrade(&type_));
          type_.borrow_mut().substitute(0, &weak);
          ns.update_type(mangle.clone(), type_.clone());
          items.push(ModuleItem::Type(mangle, type_));
        }
        // Non-recursive type
        else {
          let type_ = type_def(&mut ns, *value)?;
          let mangle = ns.push_type(assignee, type_.clone());
          items.push(ModuleItem::Type(mangle, type_));
        }
      },
      e::Import { name } => {
        let interface = context.get(&name).ok_or(lint(
          NameLint::NoSuchModule,
          expr.span,
          [name.clone()],
        ))?;
        ns.module_table.insert(name, interface.clone());
      },
    }
  }
  Ok(IrModule {
    module_name: module.name,
    universe: ns.type_table,
    items,
    nodes: ir,
  })
}

pub fn value_expr(
  module: &mut Vec<IrNode>,
  ns: &mut NameSpace,
  expr: ValueExpression,
) -> Result<IrPtr> {
  use IrKind as ir;
  use ValueExpressionKind::*;
  let span = expr.span;
  let mut ptr = module.len();
  module.push(IrNode {
    kind: IrKind::Immediate(ConstValue::Unit),
    span,
    type_: Default::default(),
  });
  let kind = match expr.kind {
    Literal(literal) => {
      fn int(value: &str, base: u32) -> Result<i64> {
        i64::from_str_radix(value, base).lint(TokenLint::InvalidInteger)
      }
      fn real(value: &str) -> Result<f64> {
        value.parse().lint(TokenLint::InvalidReal)
      }

      ir::Immediate(match literal {
        crate::parse::Literal::Unit => ConstValue::Unit,
        crate::parse::Literal::Integer(i, base) => {
          ConstValue::Integer(int(&i, base as u32).span(span)?)
        },
        crate::parse::Literal::Real(r) => {
          ConstValue::Real(real(&r).span(span)?)
        },
        crate::parse::Literal::String(s) => ConstValue::String(s),
        crate::parse::Literal::Glyph(g) => ConstValue::Glyph(g),
        crate::parse::Literal::Boolean(b) => ConstValue::Boolean(b),
      })
    },
    Identifier(name) => ir::Identifier(ns.get_value(&name).ok_or(lint(
      NameLint::UndefinedName,
      span,
      [name],
    ))?),
    Binary { op, left, right } => ir::Binary {
      op,
      left: value_expr(module, ns, *left)?,
      right: value_expr(module, ns, *right)?,
    },
    Unary { op, child } => ir::Unary {
      op,
      child: value_expr(module, ns, *child)?,
    },
    // TODO revisit this
    FunctionDef {
      mut arguments,
      mut argument_spans,
      mut types,
      body,
    } => {
      if arguments.len() == 0 {
        ns.new_func();
        let parameter_span = span;
        let body = value_expr(module, ns, *body)?;
        let captures = ns.end_func();
        let capture_types = vec![Type::Any.to_ref(); captures.len()];
        ir::FunctionDef {
          parameter_name: None,
          parameter_span,
          parameter_type: None,
          captures,
          capture_types,
          body,
        }
      } else {
        ns.new_func();
        let (argument, new_arguments) = arguments.split_first().unwrap();
        let parameter_name = ns.push_value(argument.clone());
        arguments = new_arguments.to_vec();
        let (parameter_span, new_spans) = argument_spans.split_first().unwrap();
        let parameter_span = parameter_span.clone();
        argument_spans = new_spans.to_vec();
        let (type_, new_type_s) = types.split_first().unwrap();
        let parameter_type = if let Some(type_) = type_.clone() {
          Some(type_expr(ns, type_)?)
        } else {
          None
        };
        types = new_type_s.to_vec();
        let body = if arguments.len() == 0 {
          value_expr(module, ns, *body)?
        } else {
          value_expr(
            module,
            ns,
            Expression {
              kind: FunctionDef {
                arguments,
                argument_spans,
                types,
                body,
              },
              span,
            },
          )?
        };
        let captures = ns.end_func();
        let capture_types = vec![Type::Any.to_ref(); captures.len()];
        ir::FunctionDef {
          parameter_name: Some(parameter_name),
          parameter_span,
          parameter_type,
          captures,
          capture_types,
          body,
        }
      }
    },
    // Recursive let
    Let {
      assignee,
      value:
        value @ box Expression {
          kind: FunctionDef { .. },
          ..
        },
      in_,
      ..
    } => {
      module.pop();
      let assignee = ns.push_value(assignee);
      let value = value_expr(module, ns, *value)?;
      ptr = value;
      ns.pop();
      let ir::FunctionDef {
        parameter_name,
        parameter_span,
        parameter_type,
        captures,
        capture_types,
        body,
      } = module[value].kind.clone()
      else {
        panic!()
      };
      let in_ = if let Some(in_) = in_ {
        Some(value_expr(module, ns, *in_)?)
      } else {
        None
      };
      ir::RecursiveDeclaration {
        assignee,
        parameter_name,
        parameter_span,
        parameter_type,
        captures,
        function_type: Type::Any.to_ref(),
        capture_types,
        body,
        in_,
      }
    },
    Let {
      assignee,
      value,
      in_,
      ..
    } => {
      let value = value_expr(module, ns, *value)?;
      let assignee = ns.push_value(assignee);
      let in_ = if let Some(in_) = in_ {
        Some(value_expr(module, ns, *in_)?)
      } else {
        None
      };
      ns.pop();
      ir::Declaration {
        assignee,
        value,
        in_,
      }
    },
    FunctionCall { callee, argument } => {
      let callee = value_expr(module, ns, *callee)?;
      let argument = value_expr(module, ns, *argument)?;
      ir::FunctionCall { callee, argument }
    },
    If {
      predicate,
      then,
      else_,
    } => ir::If {
      predicate: value_expr(module, ns, *predicate)?,
      then: value_expr(module, ns, *then)?,
      else_: if let Some(else_) = else_ {
        Some(value_expr(module, ns, *else_)?)
      } else {
        None
      },
    },
    Tuple(expressions) => ir::Tuple(
      expressions
        .into_iter()
        .map(|e| value_expr(module, ns, e))
        .try_collect()?,
    ),
    StructureLiteral { lhs, rhs } => ir::StructLiteral {
      field_names: lhs,
      field_values: rhs
        .into_iter()
        .map(|e| value_expr(module, ns, e))
        .try_collect()?,
    },
    Field { lhs, rhs } => ir::Field {
      of: value_expr(module, ns, *lhs)?,
      index: rhs,
    },
    ModuleField { lhs, rhs } => {
      fn flatten_module_path(
        ns: &NameSpace,
        expr: ValueExpression,
        mut path: Vec<String>,
        span: Span,
      ) -> Result<IrKind> {
        match expr.kind {
          ModuleField { lhs, rhs } => {
            path.push(rhs);
            flatten_module_path(ns, *lhs, path, span + expr.span)
          },
          Identifier(base) => {
            path.push(base);
            path.reverse();
            let (mangle, type_) =
              ns.resolve_module_value_path(&path).span(span)?;
            Ok(ir::ImportedSymbol(mangle, type_))
          },
          _ => todo!(),
        }
      }
      flatten_module_path(ns, *lhs, vec![rhs], expr.span)?
    },
  };
  module[ptr].kind = kind;
  Ok(ptr)
}

pub fn type_def(ns: &mut NameSpace, expr: TypeDefinition) -> Result<TypeRef> {
  use TypeDefinitionKind::*;
  Ok(match expr.kind {
    Structure { lhs, rhs } => Type::Struct {
      member_names: lhs,
      member_types: rhs.into_iter().map(|e| type_expr(ns, e)).try_collect()?,
    }
    .to_ref(),
    Sum {
      variant_names,
      variant_types,
    } => Type::Sum {
      variant_names,
      variant_types: variant_types
        .into_iter()
        .map(|e| type_expr(ns, e))
        .try_collect()?,
    }
    .to_ref(),
    Expression(expression) => type_expr(ns, expression)?,
  })
}

pub fn type_expr(ns: &mut NameSpace, expr: TypeExpression) -> Result<TypeRef> {
  use TypeExpressionKind::*;
  Ok(match expr.kind {
    Identifier(name) => ns.get_type(&name).ok_or(lint(
      NameLint::UndefinedName,
      expr.span,
      [name],
    ))?,
    Product(expressions) => Type::Product(
      expressions
        .into_iter()
        .map(|e| type_expr(ns, e))
        .try_collect()?,
    )
    .to_ref(),
    ModulePath(items) => ns.resolve_module_type_path(&items)?,
  })
}
