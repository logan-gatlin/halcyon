use super::*;
use crate::{parse::*, semantic::ir::ConstValue};

use super::operators::OpDef;
use NodeKind as n;

impl Analyzer {
  pub fn analyze_expression(&mut self, expr: Expression) -> Result<Node> {
    use ExpressionKind as e;
    let mut type_ = Type::Ambiguous;
    let kind = match expr.kind {
      e::Immediate(immediate) => {
        type_ = immediate.type_of().promote();
        let const_val = match immediate {
          Immediate::Unit => ConstValue::Nothing,
          Immediate::Integer(val, base) => ConstValue::Integer(
            parse_int_literal(&val, base as u32).span(&expr.span)?,
          ),
          Immediate::Real(val) => {
            ConstValue::Real(parse_real_literal(&val).span(&expr.span)?)
          },
          Immediate::String(val) => {
            let bytes = val.into_bytes();
            let length = bytes.len();
            let address = self.allocate(&bytes);
            ConstValue::String { length, address }
          },
          Immediate::Glyph(val) => ConstValue::Glyph(val),
          Immediate::Boolean(val) => ConstValue::Boolean(val),
        };
        n::ConstValue(const_val)
      },
      e::Identifier { name } => {
        let symbol = self.name_to_symbol(&name).span(&expr.span)?;
        let mangle = symbol.mangle.clone();
        n::Identifier {
          name,
          constant: symbol.is_constant,
          mangle,
        }
      },
      e::Binary { op, left, right } => {
        let left = self.analyze_expression(*left)?.into();
        let right = self.analyze_expression(*right)?.into();
        n::BinaryOp {
          op,
          opdef: OpDef::default(),
          left,
          right,
        }
      },
      e::Unary { op, child } => {
        let child = self.analyze_expression(*child)?.into();
        n::UnaryOp {
          op,
          opdef: OpDef::default(),
          child,
        }
      },
      e::Parenthesis(expression) => {
        return self.analyze_expression(*expression);
      },
      e::FunctionDef {
        params,
        returns,
        body,
      } => {
        self.enscope();
        self.start_function();
        let mut param_names = Vec::with_capacity(params.arity);
        let mut param_mangles = Vec::with_capacity(params.arity);
        let mut param_types = Vec::with_capacity(params.arity);
        for i in 0..params.arity {
          let e::Identifier { name } = &params.names[i].kind else {
            return error!("Function parameter name must be an identifier")
              .span(&expr.span);
          };
          let mangle = self.define_name(name, false)?;
          let type_ = self.analyze_expression(params.types[i].clone())?;
          param_names.push(name.to_string());
          param_mangles.push(mangle);
          param_types.push(type_);
        }
        let returns = if let Some(returns) = returns {
          self.analyze_expression(*returns)?
        } else {
          Node {
            span: expr.span,
            type_: Type::Type,
            kind: n::ConstValue(ConstValue::Nothing),
          }
        }
        .into();
        let mangle = self.define_anonymous();
        let type_ = Type::Function {
          param_types: vec![Type::Ambiguous; params.arity],
          return_type: Type::Ambiguous.into(),
        };
        let nodes = self.analyze_scope(body.into_iter())?.into();
        self.descope();
        self.constants.insert(
          mangle.clone(),
          Node {
            span: expr.span,
            type_: type_.clone(),
            kind: n::Function {
              mangle: mangle.clone(),
              param_mangles,
              param_types,
              returns,
              nodes,
            },
          },
        );
        n::ConstValue(ConstValue::Function(mangle))
      },
      e::FunctionCall { callee, args } => {
        let callee = self.analyze_expression(*callee)?.into();
        let params = args
          .into_iter()
          .map(|a| self.analyze_expression(a))
          .try_collect::<Vec<_>>()?;
        n::Call { callee, params }
      },
      e::StructDef(params) => {
        let mut member_names = vec![];
        let mut member_types = vec![];
        for i in 0..params.arity {
          let e::Identifier { name } = params.names[i].kind.clone() else {
            return error!("Struct parameter name must be an identifer");
          };
          member_names.push(name);
          let type_ = self.analyze_expression(params.types[i].clone())?;
          member_types.push(type_);
        }
        let mangle = self.define_anonymous();
        self.constants.insert(
          mangle.clone(),
          Node {
            span: expr.span,
            type_: type_.clone(),
            kind: n::StructDef {
              mangle: mangle.clone(),
              member_names,
              member_types,
            },
          },
        );
        type_ = Type::Type;
        n::Identifier {
          name: "<anonymous struct>".into(),
          constant: true,
          mangle,
        }
      },
      e::StructLiteral { struct_t, params } => {
        let struct_t = self.analyze_expression(*struct_t)?;
        let mut param_names = vec![];
        let mut param_values = vec![];
        for i in 0..params.arity {
          let e::Identifier { name } = params.names[i].kind.clone() else {
            return error!("Struct literal field name must be an identifier")
              .span(&expr.span);
          };
          param_names.push(name);
          let value = self.analyze_expression(params.types[i].clone())?;
          param_values.push(value);
        }
        n::StructLiteral {
          struct_t: struct_t.into(),
          param_names,
          param_values,
        }
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
        n::Field {
          namespace: namespace.into(),
          index,
        }
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
        n::If {
          predicate,
          then,
          else_,
        }
      },
      e::Loop { params, body } => {
        let mut names = vec![];
        let mut initials = vec![];
        for i in 0..params.arity {
          let e::Identifier { name } = params.names[i].kind.clone() else {
            return error!("Loop parameter name must be an identifier");
          };
          names.push(name);
          let initial = self.analyze_expression(params.types[i].clone())?;
          initials.push(initial);
        }
        self.enscope();
        let names = names
          .into_iter()
          .map(|n| self.define_name(n, false))
          .try_collect::<Vec<_>>()
          .span(&expr.span)?;
        let body = self.analyze_scope(body.into_iter())?;
        self.descope();
        n::Loop {
          names,
          initials,
          body: body.into(),
        }
      },
      e::Break { expr } => {
        let expr = self.analyze_expression(*expr)?;
        type_ = Primitive::never.promote();
        n::Break { expr: expr.into() }
      },
    };
    Ok(Node {
      span: expr.span,
      type_,
      kind,
    })
  }
}
