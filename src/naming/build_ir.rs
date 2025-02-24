use crate::{
  diagnostic,
  err::*,
  error,
  ir::{Block, ConstValue, FunctionInfo, Ir, IrKind, IrPtr},
  naming::Symbol,
  parse::{Expression, ExpressionKind, Immediate, Statement, StatementKind},
};

use super::Analyzer;

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

use IrKind as i;

impl Analyzer {
  pub(crate) fn analyze_block(
    &mut self,
    stmts: impl IntoIterator<Item = Statement>,
    block: IrPtr,
  ) -> Result<IrPtr> {
    stmts
      .into_iter()
      .map(|s| {
        if let StatementKind::Declaration {
          is_constant: true,
          name,
          ..
        } = &s.kind
        {
          let span = s.span;
          self.define_name(name, true).map(|_| s).span(&span)
        } else {
          Ok(s)
        }
      })
      .try_collect::<Vec<_>>()?
      .into_iter()
      .try_fold(block, |block, stmt| self.analyze_stmt(stmt, block, false))
  }

  fn unreachable_error() -> Diagnostic {
    diagnostic!("This expression is unreachable")
  }

  // Every statement is at least one block. Returns the head
  // and tail block address
  fn analyze_stmt(
    &mut self,
    stmt: Statement,
    mut block: IrPtr,
    is_typecheck: bool,
  ) -> Result<IrPtr> {
    let ir = |kind| Ir {
      kind,
      typecheck_only: is_typecheck,
      span: stmt.span,
    };
    let tir = |kind| Ir {
      kind,
      typecheck_only: true,
      span: stmt.span,
    };
    match stmt.kind {
      StatementKind::Declaration {
        name,
        type_,
        value,
        is_constant,
      } => {
        // Prevent constants from interfering with their surrounding
        // control flow
        let old_breaks = self.break_targets.clone();
        let mut new_block = if is_constant {
          self.break_targets = vec![];
          self.new_block()
        } else {
          block
        };
        let head = new_block;
        // Analyze RHS expression
        new_block = self.analyze_expr(value, new_block, is_typecheck)?;
        let mangle = if is_constant {
          self.name_to_symbol(&name)?.mangle.clone()
        } else {
          self.define_name(name, is_constant).span(&stmt.span)?
        };
        // Analyze type hint
        if let Some(type_) = type_ {
          // Place the hint inline for constant declarations
          if is_constant {
            new_block = self.analyze_expr(type_, new_block, !is_constant)?;
            self.push(new_block, ir(i::TypeAssert(Some(mangle.clone()))));
          }
          // Break the hint out to its own block for runtime
          // declarations
          else {
            let hint_block = self.new_block();
            self.type_assertions.insert(mangle.clone(), hint_block);
            self.analyze_expr(type_, hint_block, false)?;
          }
        }
        self.push(new_block, ir(i::Set(mangle.clone())));
        self.push(new_block, ir(i::Drop));
        if is_constant {
          self.break_targets = old_breaks;
          self.constants.insert(mangle, head);
        }
      },
      StatementKind::Expression(expression) => {
        block = self.analyze_expr(expression, block, is_typecheck)?;
        if !self.blocks[block].is_terminal() {
          self.push(block, ir(i::Drop));
        }
      },
      StatementKind::Remainder(expression) => {
        block = self.analyze_expr(expression, block, is_typecheck)?;
      },
      StatementKind::Error(diagnostic) => return Err(diagnostic),
    };
    Ok(block)
  }

  // Returns the tail block
  fn analyze_expr(
    &mut self,
    expr: Expression,
    mut block: IrPtr,
    is_typecheck: bool,
  ) -> Result<IrPtr> {
    use ExpressionKind as e;
    if self.blocks[block].is_terminal() {
      return Ok(block);
    }
    let ir = |kind| Ir {
      kind,
      typecheck_only: is_typecheck,
      span: expr.span,
    };
    let tir = |kind| Ir {
      kind,
      typecheck_only: true,
      span: expr.span,
    };
    match expr.kind {
      e::Immediate(immediate) => {
        let kind = match immediate {
          Immediate::Unit => i::Const(ConstValue::Nothing),
          Immediate::Integer(val, base) => {
            i::Const(ConstValue::Integer(parse_int_literal(&val, base as u32)?))
          },
          Immediate::Real(val) => {
            i::Const(ConstValue::Real(parse_real_literal(&val)?))
          },
          Immediate::String(val) => {
            let bytes = val.into_bytes();
            let address = self.allocate(&bytes);
            i::Const(ConstValue::String {
              virtual_address: address,
              length: bytes.len(),
            })
          },
          Immediate::Glyph(val) => i::Const(ConstValue::Glyph(val)),
          Immediate::Boolean(val) => i::Const(ConstValue::Boolean(val)),
        };
        self.push(block, ir(kind));
      },
      e::Identifier { name } => {
        let Symbol {
          mangle,
          is_constant,
          ..
        } = self.name_to_symbol(&name).span(&expr.span)?.clone();
        self.push(block, ir(i::Get(mangle)));
      },
      e::Binary { op, left, right } => {
        block = self.analyze_expr(*left, block, is_typecheck)?;
        block = self.analyze_expr(*right, block, is_typecheck)?;
        self.push(block, ir(i::BinaryOp { kind: op }))
      },
      e::Unary { op, child } => {
        block = self.analyze_expr(*child, block, is_typecheck)?;
        self.push(block, ir(i::UnaryOp { kind: op }))
      },
      e::Parenthesis(expression) => {
        block = self.analyze_expr(*expression, block, is_typecheck)?;
      },
      e::FunctionDef {
        params,
        returns,
        body,
      } => {
        self.start_function();
        let function_mangle = self.define_unique("function");
        let mut param_names = Vec::with_capacity(params.arity);
        let mut parameter_mangles = Vec::with_capacity(params.arity);
        for i in 0..params.arity {
          let e::Identifier { name } = &params.names[i].kind else {
            return error!("Function parameter name must be an identifier")
              .span(&expr.span);
          };
          if param_names.contains(name) {
            return error!("Multiple parameters have the same name: '{name}'")
              .span(&expr.span);
          }
          param_names.push(name.clone());
          let param_mangle = self.define_name(name, false)?;
          parameter_mangles.push(param_mangle.clone());
          let param_block = self.new_block();
          self.type_assertions.insert(param_mangle, param_block);
          self.analyze_expr(
            params.types[i].clone(),
            param_block,
            is_typecheck,
          )?;
        }
        let returns_mangle = if let Some(r) = returns {
          let returns_mangle = self.define_unique("return_type");
          let return_type_block = self.new_block();
          self
            .type_assertions
            .insert(returns_mangle.clone(), return_type_block);
          self.analyze_expr(*r, return_type_block, is_typecheck)?;
          Some(returns_mangle)
        } else {
          None
        };
        let func_block = self.new_block();
        self.analyze_expr(*body, func_block, is_typecheck)?;
        self.push(
          block,
          ir(i::Const(ConstValue::Function(function_mangle.clone()))),
        );
        self.functions.insert(
          function_mangle.clone(),
          FunctionInfo {
            mangle: function_mangle,
            arity: params.arity,
            parameter_mangles,
            returns_mangle,
            block: func_block,
          },
        );
      },
      e::FunctionCall { callee, args } => {
        let arity = args.len();
        block = self.analyze_expr(*callee, block, is_typecheck)?;
        let callee_mangle = self.define_unique("callee");
        self.push(block, ir(i::Set(callee_mangle.clone())));
        for a in args {
          block = self.analyze_expr(a, block, is_typecheck)?;
        }
        self.push(block, ir(i::Get(callee_mangle)));
        self.push(block, ir(i::Call { arity }));
      },
      e::StructDef(parameters) => {
        let param_names = parameters
          .names
          .into_iter()
          .map(|n| {
            if let e::Identifier { name } = n.kind {
              Ok(name)
            } else {
              error!("Structure field name must be an identifier").span(&n.span)
            }
          })
          .try_collect::<Vec<_>>()?;
        for t in parameters.types {
          block = self.analyze_expr(t, block, is_typecheck)?;
        }
        self.push(block, ir(i::StructDef { param_names }));
      },
      e::StructLiteral { struct_t, params } => {
        let param_names = params
          .names
          .into_iter()
          .map(|n| {
            if let e::Identifier { name } = n.kind {
              Ok(name)
            } else {
              error!("Structure field name must be an identifier").span(&n.span)
            }
          })
          .try_collect::<Vec<_>>()?;
        let struct_t_mangle = if let Some(struct_t) = struct_t {
          block = self.analyze_expr(*struct_t, block, true)?;
          let mangle = self.define_unique("struct_t");
          self.push(block, tir(i::Set(mangle.clone())));
          Some(mangle)
        } else {
          None
        };
        for t in params.types {
          block = self.analyze_expr(t, block, true)?;
        }
        self.push(block, ir(i::StructLiteral { param_names }));
        if let Some(struct_t_mangle) = struct_t_mangle {
          self.push(block, tir(i::Get(struct_t_mangle)));
          self.push(block, tir(i::TypeAssert(None)));
        }
      },
      e::Field { namespace, field } => {
        let e::Identifier { name: field_name } = field.kind else {
          return error!("Field must be an identifier").span(&field.span);
        };
        block = self.analyze_expr(*namespace, block, is_typecheck)?;
        self.push(block, ir(i::Field(field_name)));
      },
      e::Block(statements) => {
        self.enscope();
        self.push(block, ir(i::StartScope));
        block = self.analyze_block(statements, block)?;
        if !self.blocks[block].is_terminal() {
          self.push(block, ir(i::EndScope));
        }
        self.descope();
      },
      e::If {
        predicate,
        then,
        else_,
      } => {
        // Capture predicate
        block = self.analyze_expr(*predicate, block, is_typecheck)?;
        // Hook branch block
        let branch_block = self.new_block();
        self.blocks[block].set_next(branch_block);
        // Analyze then and else blocks
        let then_block_head = self.new_block();
        let then_block_tail =
          self.analyze_expr(*then, then_block_head, is_typecheck)?;
        let else_block_head = self.new_block();
        let else_block_tail = if let Some(else_) = else_ {
          self.analyze_expr(*else_, else_block_head, is_typecheck)?
        } else {
          else_block_head
        };
        // Hook then and else blocks
        self.blocks[branch_block] = Block::Branch {
          when_true: then_block_head,
          when_false: else_block_head,
          span: expr.span,
        };
        // If both branches already converge at some point
        if then_block_tail == else_block_tail {
          println!("Converge {then_block_tail}");
          block = then_block_tail
        }
        // If only the else block diverges
        else if !self.blocks[then_block_tail].is_terminal()
          && self.blocks[else_block_tail].is_terminal()
        {
          block = then_block_tail;
        }
        // If only the then block diverges
        else if !self.blocks[else_block_tail].is_terminal()
          && self.blocks[then_block_tail].is_terminal()
        {
          block = else_block_tail;
        }
        // If neither branch diverge
        else if !self.blocks[then_block_tail].is_terminal()
          && !self.blocks[else_block_tail].is_terminal()
        {
          let converge_block = self.new_block();
          self.blocks[then_block_tail].set_next(converge_block);
          self.blocks[else_block_tail].set_next(converge_block);
          block = converge_block;
        }
        // If both blocks diverge
        else {
          println!("Both diverge");
          block = Self::TERMINUS
        }
      },
      e::Loop { params, body } => {
        if params.arity > 1 {
          return error!("Only one loop parameter allowed for now")
            .span(&expr.span);
        }
        let mut param_names = Vec::with_capacity(params.arity);
        let mut param_mangles = Vec::with_capacity(params.arity);
        for p in 0..params.arity {
          let e::Identifier { name } = &params.names[p].kind else {
            return error!("Function parameter name must be an identifier")
              .span(&expr.span);
          };
          if param_names.contains(name) {
            return error!("Multiple parameters have the same name: '{name}'")
              .span(&expr.span);
          }
          param_names.push(name.clone());
          let param_mangle = self.define_name(name, false)?;
          param_mangles.push(param_mangle.clone());
          block =
            self.analyze_expr(params.types[p].clone(), block, is_typecheck)?;
          self.push(block, ir(i::Set(param_mangle)))
        }
        // Create loop target
        let loop_head = self.new_block();
        self.blocks[block].set_next(loop_head);
        // Set up break target
        let break_target = self.new_block();
        self.blocks[break_target] = Block::Terminal;
        self.break_targets.push(break_target);
        let loop_tail = self.analyze_expr(*body, loop_head, is_typecheck)?;
        self.break_targets.pop().unwrap();
        // Loop always diverges
        if loop_tail == Self::TERMINUS {
          for mangle in param_mangles {
            self.push(loop_tail, ir(i::Set(mangle)));
          }
          self.blocks[loop_tail].set_next(loop_head);
          self.blocks[break_target] = Block::Unreachable;
          block = Self::TERMINUS;
        }
        // Loop always breaks or breaks conditionally
        else if loop_tail == break_target {
          self.blocks[break_target] = Block::basic();
          block = break_target;
        } else if self.node_reaches(loop_head, break_target) {
          for mangle in param_mangles {
            self.push(loop_tail, ir(i::Set(mangle)));
          }
          self.blocks[loop_tail].set_next(loop_head);
          self.blocks[break_target] = Block::basic();
          block = break_target;
        }
        // Loop is infinite
        else {
          self.blocks[break_target] = Block::Unreachable;
          self.blocks[loop_tail].set_next(loop_head);
          block = Self::TERMINUS;
        }
      },
      e::Break { expr: expression } => {
        let span = expr.span;
        if let Some(expr) = expression {
          block = self.analyze_expr(*expr, block, is_typecheck)?;
          if self.blocks[block].is_terminal() {
            return Err(Self::unreachable_error()).span(&span);
          }
        }
        self.push(block, ir(i::EndScope));
        block = if let Some(target) = self.break_targets.last() {
          self.blocks[block].set_next(*target);
          *target
        } else {
          return error!("A 'break' must be inside of a loop").span(&span);
        };
      },
    }
    Ok(block)
  }
}
