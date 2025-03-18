use std::collections::{HashMap, HashSet};

use crate::{hlir::*, lint::*};

use super::*;

pub struct Analyzer {
  nodes: Vec<HlIrNode>,
  blocks: Vec<Block>,
  break_stack: Vec<IrPtr>,
  functions: HashMap<Mangle, FunctionInfo>,
  constants: HashMap<Mangle, IrPtr>,
  type_assertions: HashMap<Mangle, IrPtr>,
}

impl Analyzer {
  const TERMINUS: IrPtr = 0;

  pub fn analyze(canon_mod: &HlIrModule) -> Result<MlIrModule> {
    let mut this = Self::new(canon_mod.nodes.clone());
    let block = this.new_block();
    this.analyze_node(0, block)?;
    let Analyzer {
      blocks,
      functions,
      constants,
      type_assertions,
      ..
    } = this;
    Ok(MlIrModule {
      heap: canon_mod.heap.clone(),
      constants,
      functions,
      type_assertions,
      blocks,
    })
  }

  fn new(module: Vec<HlIrNode>) -> Self {
    Self {
      nodes: module,
      blocks: vec![Block::Terminal],
      break_stack: Default::default(),
      functions: Default::default(),
      constants: Default::default(),
      type_assertions: Default::default(),
    }
  }

  fn new_block(&mut self) -> IrPtr {
    self.blocks.push(Block::Basic {
      body: vec![],
      next: 0,
    });
    self.blocks.len() - 1
  }

  fn node_reaches(&mut self, from: IrPtr, to: IrPtr) -> bool {
    let mut visited = HashSet::new();
    let mut to_visit = vec![];
    let mut current_node = from;
    loop {
      if current_node == to {
        return true;
      }
      visited.insert(current_node);
      match self.blocks.get(current_node) {
        Some(Block::Unreachable) | Some(Block::Terminal) => {}
        Some(Block::Basic { next, .. }) => {
          to_visit.push(next);
        }
        Some(Block::Branch {
          when_true,
          when_false,
          ..
        }) => {
          to_visit.push(when_true);
          to_visit.push(when_false);
        }
        None => {}
      };
      loop {
        if let Some(n) = to_visit.pop() {
          if !visited.contains(n) {
            current_node = *n;
            break;
          }
        } else {
          return false;
        }
      }
    }
  }

  fn push(&mut self, block: IrPtr, ir: MlIrNode) {
    self.blocks[block].push(ir);
  }

  fn analyze_node(&mut self, node: IrPtr, mut block: IrPtr) -> Result<IrPtr> {
    use HlIrKind as k;
    use MlIrKind as i;
    let node = self.nodes[node].clone();
    if self.blocks[block].is_terminal() {
      return Ok(block);
    }
    let ir = |kind| MlIrNode {
      kind,
      span: node.span,
    };
    match node.kind {
      k::Declaration {
        assignee,
        is_constant,
        type_assert,
        value,
      } => {
        let old_breaks = self.break_stack.clone();
        let mut new_block = if is_constant {
          self.break_stack = vec![];
          self.new_block()
        } else {
          block
        };
        let head = new_block;
        // Analyze RHS expression
        new_block = self.analyze_node(value, new_block)?;
        // Analyze type hint
        if let Some(type_) = type_assert {
          // Place the hint inline for constant declarations
          if is_constant {
            new_block = self.analyze_node(type_, new_block)?;
            self.push(new_block, ir(i::TypeAssert(Some(assignee.clone()))));
          }
          // Break the hint out to its own block for runtime
          // declarations
          else {
            let hint_block = self.new_block();
            self.type_assertions.insert(assignee.clone(), hint_block);
            self.analyze_node(type_, hint_block)?;
          }
        }
        self.push(new_block, ir(i::Set(assignee.clone())));
        if is_constant {
          self.break_stack = old_breaks;
          self.constants.insert(assignee, head);
        } else {
          block = new_block;
        }
      }
      k::Immediate(const_value) => {
        self.push(block, ir(i::Const(const_value)));
      }
      k::Block(items) => {
        self.push(block, ir(i::StartScope));
        let length = items.len();
        for (id, node) in items.into_iter().enumerate() {
          let new_block = self.analyze_node(node, block)?;
          if id != length - 1 {
            self.push(new_block, ir(i::Drop));
          }
          block = new_block;
        }
        self.push(block, ir(i::EndScope));
      }
      k::Identifier(mangle) => {
        self.push(block, ir(i::Get(mangle)));
      }
      k::Binary {
        op, left, right, ..
      } => {
        block = self.analyze_node(left, block)?;
        block = self.analyze_node(right, block)?;
        self.push(block, ir(i::BinaryOp { kind: op }));
      }
      k::Unary { op, child, .. } => {
        block = self.analyze_node(child, block)?;
        self.push(block, ir(i::UnaryOp { kind: op }));
      }
      k::FunctionDef {
        name,
        parameter_names,
        parameter_types,
        returns,
        body,
      } => {
        parameter_names
          .iter()
          .zip(parameter_types.into_iter())
          .map(|(n, t)| {
            let param_block = self.new_block();
            self.type_assertions.insert(n.clone(), param_block);
            self.analyze_node(t, param_block)
          })
          .try_collect::<Vec<_>>()?;
        let returns_mangle = if let Some((return_type, returns_mangle)) = returns {
          let return_type_block = self.new_block();
          self
            .type_assertions
            .insert(returns_mangle.clone(), return_type_block);
          self.analyze_node(return_type, return_type_block)?;
          Some(returns_mangle)
        } else {
          None
        };
        let func_block = self.new_block();
        self.analyze_node(body, func_block)?;
        self.push(block, ir(i::Const(ConstValue::Function(name.clone()))));
        self.functions.insert(name.clone(), FunctionInfo {
          mangle: name,
          arity: parameter_names.len(),
          parameter_mangles: parameter_names,
          returns_mangle,
          block: func_block,
        });
      }
      k::FunctionCall {
        callee,
        callee_name,
        arguments,
      } => {
        let arity = arguments.len();
        block = self.analyze_node(callee, block)?;
        self.push(block, ir(i::Set(callee_name.clone())));
        for a in arguments {
          block = self.analyze_node(a, block)?;
        }
        self.push(block, ir(i::Get(callee_name)));
        self.push(block, ir(i::Call { arity }));
      }
      k::If {
        predicate,
        then,
        else_,
      } => {
        // Capture predicate
        block = self.analyze_node(predicate, block)?;
        // Hook branch block
        let branch_block = self.new_block();
        self.blocks[block].set_next(branch_block);
        // Analyze then and else blocks
        let then_block_head = self.new_block();
        let then_block_tail = self.analyze_node(then, then_block_head)?;
        let else_block_head = self.new_block();
        let else_block_tail = if let Some(else_) = else_ {
          self.analyze_node(else_, else_block_head)?
        } else {
          else_block_head
        };
        // Hook then and else blocks
        self.blocks[branch_block] = Block::Branch {
          when_true: then_block_head,
          when_false: else_block_head,
          span: node.span,
        };
        // If both branches already converge at some point
        if then_block_tail == else_block_tail {
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
          block = Self::TERMINUS
        }
      }
      k::StructDef { fields, types } => {
        block = types
          .into_iter()
          .try_fold(block, |block, type_| self.analyze_node(type_, block))?;
        self.push(block, ir(i::StructDef { fields }));
      }
      k::StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => {
        let struct_t_mangle = if let Some((struct_t, struct_name)) = struct_t {
          block = self.analyze_node(struct_t, block)?;
          self.push(block, ir(i::Set(struct_name.clone())));
          Some(struct_name)
        } else {
          None
        };
        block = field_values
          .into_iter()
          .try_fold(block, |block, value| self.analyze_node(value, block))?;
        self.push(
          block,
          ir(i::StructLiteral {
            param_names: field_names,
          }),
        );
        if let Some(struct_t_mangle) = struct_t_mangle {
          self.push(block, ir(i::Get(struct_t_mangle)));
          self.push(block, ir(i::TypeAssert(None)));
        }
      }
      k::Field { of, index } => {
        block = self.analyze_node(of, block)?;
        self.push(block, ir(i::Field(index)));
      }
      k::Loop {
        parameter_names,
        parameter_values,
        body,
      } => {
        let arity = parameter_names.len();
        if arity > 1 {
          return Err(lint(NameLint::MultipleLoopParams, node.span, &[]));
        }
        for p in 0..arity {
          block = self.analyze_node(parameter_values[p], block)?;
          self.push(block, ir(i::Set(parameter_names[p].clone())))
        }
        // Create loop target
        let loop_head = self.new_block();
        self.blocks[block].set_next(loop_head);
        // Set up break target
        let break_target = self.new_block();
        self.blocks[break_target] = Block::Terminal;
        self.break_stack.push(break_target);
        let loop_tail = self.analyze_node(body, loop_head)?;
        self.break_stack.pop().unwrap();
        // Loop always diverges
        if loop_tail == Self::TERMINUS {
          for name in parameter_names {
            self.push(loop_tail, ir(i::Set(name)));
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
          for name in parameter_names {
            self.push(loop_tail, ir(i::Set(name)));
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
      }
      k::Break(value) => {
        let span = node.span;
        if let Some(expr) = value {
          block = self.analyze_node(expr, block)?;
          if self.blocks[block].is_terminal() {
            return Ok(block);
          }
        }
        self.push(block, ir(i::Const(ConstValue::Never)));
        self.push(block, ir(i::EndScope));
        block = if let Some(target) = self.break_stack.last() {
          self.blocks[block].set_next(*target);
          *target
        } else {
          return Err(lint(NameLint::NoBreakTarget, span, &[]));
        };
      }
    };
    Ok(block)
  }
}
