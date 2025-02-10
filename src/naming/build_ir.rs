use crate::{
  diagnostic,
  err::*,
  error,
  ir::{Block, ConstValue, Ir, IrKind, IrPtr, types::Primitive},
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
  fn analyze_block(
    &mut self,
    stmts: impl IntoIterator<Item = Statement>,
  ) -> Result<Option<IrPtr>> {
    let mut tail: Option<IrPtr> = None;
    for s in stmts {
      if let Some((new_head, new_tail)) = self.analyze_stmt(s)? {
        if let Some(tail) = tail {
          self.blocks[tail].set_next(new_head);
        }
        tail = Some(new_tail);
      };
    }
    Ok(tail)
  }

  fn unreachable_error() -> Diagnostic {
    diagnostic!("This expression is unreachable")
  }

  // Every statement is at least one block. Returns the head
  // and tail block address
  fn analyze_stmt(
    &mut self,
    stmt: Statement,
  ) -> Result<Option<(IrPtr, IrPtr)>> {
    let ir = |kind| Ir {
      kind,
      type_: Default::default(),
      span: stmt.span,
    };
    let head = self.new_block();
    let mut tail = head;
    match stmt.kind {
      StatementKind::Declaration {
        name,
        type_,
        value,
        is_constant,
      } => {
        tail = self.analyze_expr(value, tail)?;
        if let Some(type_) = type_ {
          tail = self.analyze_expr(type_, tail)?;
        };
        let mangle = self.define_name(name, is_constant).span(&stmt.span)?;
        self.push(tail, ir(i::Set(mangle.clone())));
        if is_constant {
          self.constants.insert(mangle, head);
          return Ok(None);
        }
      },
      StatementKind::Expression(expression) => {
        tail = self.analyze_expr(expression, tail)?;
      },
      StatementKind::Remainder(expression) => {
        tail = self.analyze_expr(expression, tail)?;
      },
      StatementKind::Error(diagnostic) => return Err(diagnostic),
    };
    Ok(Some((head, tail)))
  }

  // Returns the tail block
  fn analyze_expr(
    &mut self,
    expr: Expression,
    mut block: IrPtr,
  ) -> Result<IrPtr> {
    use ExpressionKind as e;
    let ir = |kind| Ir {
      kind,
      type_: Default::default(),
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
              address,
              length: bytes.len(),
            })
          },
          Immediate::Glyph(val) => i::Const(ConstValue::Glyph(val)),
          Immediate::Boolean(val) => i::Const(ConstValue::Boolean(val)),
        };
        self.push(block, ir(kind));
      },
      e::Identifier { name } => {
        let symbol = self.name_to_symbol(&name).span(&expr.span)?.clone();
        self.push(block, ir(i::Get(symbol.mangle)));
      },
      e::Binary { op, left, right } => {
        block = self.analyze_expr(*left, block)?;
        block = self.analyze_expr(*right, block)?;
        self.push(
          block,
          ir(i::BinaryOp {
            kind: op,
            def: Default::default(),
          }),
        )
      },
      e::Unary { op, child } => {
        block = self.analyze_expr(*child, block)?;
        self.push(
          block,
          ir(i::UnaryOp {
            kind: op,
            def: Default::default(),
          }),
        )
      },
      e::Parenthesis(expression) => {
        block = self.analyze_expr(*expression, block)?;
      },
      e::FunctionDef {
        params,
        returns,
        body,
      } => {
        todo!()
      },
      e::FunctionCall { callee, args } => {
        let arity = args.len();
        block = self.analyze_expr(*callee, block)?;
        let callee_mangle = self.define_unique("callee");
        self.push(block, ir(i::Set(callee_mangle.clone())));
        for a in args {
          block = self.analyze_expr(a, block)?;
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
          block = self.analyze_expr(t, block)?;
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
        block = self.analyze_expr(*struct_t, block)?;
        for t in params.types {
          block = self.analyze_expr(t, block)?;
        }
        self.push(block, ir(i::StructDef { param_names }));
      },
      e::Field { namespace, field } => {
        let e::Identifier { name: field_name } = field.kind else {
          return error!("Field must be an identifier").span(&field.span);
        };
        block = self.analyze_expr(*namespace, block)?;
        self.push(block, ir(i::Field(field_name)));
      },
      e::Block(statements) => {
        self.push(block, ir(i::Enscope));
        if let Some(new_tail) = self.analyze_block(statements)? {
          block = new_tail;
        }
        self.push(block, ir(i::Descope));
      },
      e::If {
        predicate,
        then,
        else_,
      } => {
        // Capture predicate
        block = self.analyze_expr(*predicate, block)?;
        let predicate_mangle = self.define_unique("predicate");
        self.push(block, ir(i::Set(predicate_mangle.clone())));
        // Hook branch block
        let branch_block = self.new_block();
        self.blocks[block].set_next(branch_block);
        // Analyze then and else blocks
        let then_block_head = self.new_block();
        let then_block_tail = self.analyze_expr(*then, then_block_head)?;
        let else_block_head = self.new_block();
        let else_block_tail = if let Some(else_) = else_ {
          self.analyze_expr(*else_, else_block_head)?
        } else {
          self.push(else_block_head, ir(i::Const(ConstValue::Nothing)));
          else_block_head
        };
        // Hook then and else blocks
        self.blocks[branch_block] = Block::Branch {
          predicate_mangle,
          when_true: then_block_head,
          when_false: else_block_head,
        };
        // If both branches already converge at some point
        if then_block_tail == else_block_tail {
          block = then_block_tail
        }
        // If at least one branch does not diverge
        else if !self.blocks[then_block_tail].is_terminal()
          || !self.blocks[else_block_tail].is_terminal()
        {
          let converge_block = self.new_block();
          if self.blocks[then_block_tail].is_terminal() {
            self.blocks[then_block_tail].set_next(converge_block);
          }
          if self.blocks[else_block_tail].is_terminal() {
            self.blocks[else_block_tail].set_next(converge_block);
          }
          block = converge_block;
        }
        // If both blocks diverge
        else {
          block = Self::TERMINUS
        }
      },
      e::Loop { params, body } => {
        // Create loop target
        let loop_head = self.new_block();
        self.blocks[block].set_next(loop_head);
        // Set up break target
        let break_target = self.new_block();
        self.blocks[break_target] = Block::Terminal;
        self.break_targets.push(break_target);
        let loop_tail = self.analyze_expr(*body, loop_head)?;
        self.break_targets.pop().unwrap();
        // If the loop iterates at least once
        if !self.blocks[loop_tail].is_terminal() {
          self.blocks[loop_tail].set_next(loop_tail);
          block = break_target;
          self.blocks[break_target] = Block::basic();
        }
        // If the loop does not iterate or diverge
        else if loop_tail == break_target {
          block = break_target;
          self.blocks[break_target] = Block::basic();
        }
        // If loop always diverges
        else {
          block = loop_tail;
        }
      },
      e::Break { expr } => {
        let span = expr.span;
        block = self.analyze_expr(*expr, block)?;
        if self.blocks[block].is_terminal() {
          return Err(Self::unreachable_error()).span(&span);
        }
        block = if let Some(target) = self.break_targets.last() {
          *target
        } else {
          return error!("A 'break' must be inside of a loop").span(&span);
        };
      },
    }
    Ok(block)
  }
}
