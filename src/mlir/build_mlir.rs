use super::*;

pub fn build_mlir(hlir: &HlIrModule) -> MlIrModule {
  let mut this = Analyzer::new(hlir);
  this.new_block("global".into(), BlockKind::GlobalScope);
  todo!()
}

struct Analyzer<'a> {
  blocks: HashMap<Mangle, Block>,
  hl: &'a HlIrModule,
  label_count: usize,
  break_labels: Vec<usize>,
}

impl<'a> Analyzer<'a> {
  pub fn new(hl: &'a HlIrModule) -> Self {
    Self {
      blocks: HashMap::new(),
      hl,
      label_count: 0,
      break_labels: vec![],
    }
  }

  fn push(&mut self, mangle: &Mangle, ir: MlIrNode) -> IrPtr {
    let block = self.blocks.get_mut(mangle).unwrap();
    block.push(ir);
    block.body.len() - 1
  }

  fn new_block(&mut self, name: Mangle, kind: BlockKind) {
    self.blocks.insert(name.clone(), Block::new(kind));
  }

  fn new_constant(&mut self, name: &Mangle, start: IrPtr) {
    self.new_block(name.clone(), BlockKind::Constant { evaluation: None });
    todo!()
  }

  fn new_parameter(&mut self, name: &Mangle, start: IrPtr) {
    self.new_block(name.clone(), BlockKind::Parameter);
    todo!()
  }

  fn new_function(
    &mut self,
    name: &Mangle,
    parameters: Vec<Mangle>,
    returns: Option<Mangle>,
    start: IrPtr,
  ) {
    self.new_block(name.clone(), BlockKind::Function { parameters });
    todo!()
  }

  fn lower(&mut self, block: &Mangle, node_ptr: IrPtr) {
    let node = &self.hl.nodes[node_ptr];
    let new = |kind: MlIrKind| MlIrNode {
      span: node.span,
      kind: kind.clone(),
    };
    use HlIrKind::*;
    use MlIrKind as ml;
    match node.kind.clone() {
      Declaration {
        assignee,
        is_constant,
        value,
        ..
      } => {
        if is_constant {
          self.new_constant(&assignee, value);
        } else {
          self.lower(block, value);
          self.push(block, new(ml::Set(assignee)));
        }
      },
      Immediate(const_value) => {
        self.push(block, new(ml::Const(const_value)));
      },
      Block(items) => {
        for item in items {
          self.lower(block, item);
        }
      },
      Identifier(mangle) => {
        self.push(block, new(ml::Get(mangle)));
      },
      StructDef { fields, types } => {
        for type_ in types {
          self.lower(block, type_);
        }
        self.push(block, new(ml::StructDef { fields }));
      },
      StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => {
        for value in field_values {
          self.lower(block, value);
        }
        if let Some((struct_t, mangle)) = &struct_t {
          self.lower(block, *struct_t);
          self.push(block, new(ml::Set(mangle.clone())));
        }
        self.push(
          block,
          new(ml::StructLiteral {
            param_names: field_names,
          }),
        );
        if let Some((_, mangle)) = struct_t {
          self.push(block, new(ml::Get(mangle)));
          self.push(block, new(ml::TypeAssert(None)));
        }
      },
      Field { of, index } => {
        self.lower(block, of);
        self.push(block, new(ml::Field(index)));
      },
      Binary {
        op, left, right, ..
      } => {
        self.lower(block, left);
        self.lower(block, right);
        self.push(block, new(ml::BinaryOp { kind: op }));
      },
      Unary { op, child, .. } => {
        self.lower(block, child);
        self.push(block, new(ml::UnaryOp { kind: op }));
      },
      FunctionDef {
        name,
        parameter_names,
        parameter_types,
        returns,
        body,
      } => {
        for (name, type_) in
          parameter_names.iter().zip(parameter_types.into_iter())
        {
          self.new_parameter(name, type_);
        }
        if let Some((type_, name)) = &returns {
          self.new_parameter(name, *type_);
        }
        self.new_function(
          &name,
          parameter_names.clone(),
          returns.map(|r| r.1),
          body,
        );
        self.push(block, new(ml::Const(ConstValue::Function(name))));
      },
      FunctionCall {
        callee,
        callee_name,
        arguments,
      } => {
        let arity = arguments.len();
        self.lower(block, callee);
        self.push(block, new(ml::Set(callee_name.clone())));
        for argument in arguments {
          self.lower(block, argument);
        }
        self.push(block, new(ml::Get(callee_name)));
        self.push(block, new(ml::Call { arity }));
      },
      If {
        predicate,
        then,
        else_,
      } => {
        self.lower(block, predicate);
        let branch_pos = self.push(block, new(ml::Noop));
        self.lower(block, then);
        let end_then_pos = self.new_label(block);
        if let Some(else_) = else_ {
          self.lower(block, else_);
          let end_else_pos = self.new_label(block);
          self.set_instr(block, branch_pos, new(ml::Branch(end_then_pos)));
          self.set_instr(block, end_then_pos, new(ml::Jump(end_else_pos)));
        } else {
          self.set_instr(block, branch_pos, new(ml::Branch(end_then_pos)));
        }
      },
      Loop {
        parameter_names,
        parameter_values,
        body,
      } => {
        let break_label = self.label_count;
        self.label_count += 1;
        for value in parameter_values {
          self.lower(block, value);
        }
        for name in parameter_names.into_iter().rev() {
          self.push(block, new(ml::Set(name)));
        }
        let loop_start_pos = self.new_label(block);
        self.break_labels.push(break_label);
        self.lower(block, body);
        self.break_labels.pop();
        self.push(block, new(ml::Jump(loop_start_pos)));
        self.push(block, new(ml::Label(break_label)));
      },
      Break(e) => {
        if let Some(e) = e {
          self.lower(block, e);
        }
        self.push(block, new(ml::Jump(*self.break_labels.last().unwrap())));
      },
    }
  }

  fn new_label(&mut self, block: &Mangle) -> usize {
    let label = self.label_count;
    self.label_count += 1;
    self.push(
      block,
      MlIrNode {
        span: Default::default(),
        kind: MlIrKind::Label(label),
      },
    );
    label
  }

  fn next_instr(&self, block: &Mangle) -> IrPtr {
    self.blocks.get(block).unwrap().body.len()
  }

  fn set_instr(&mut self, block: &Mangle, position: IrPtr, ir: MlIrNode) {
    self.blocks.get_mut(block).unwrap().body[position] = ir;
  }
}
