use super::*;

pub fn build_mlir(hlir: &mut HlIrModule) -> Result<MlIrModule> {
  let mut this = Analyzer::new(hlir);
  this.new_block(GLOBAL_SCOPE_MANGLE.into(), BlockKind::GlobalScope(None));
  this.lower(&GLOBAL_SCOPE_MANGLE.to_string(), 0, &mut 0)?;
  let mut module = MlIrModule {
    blocks: this.blocks,
    dependencies: HashMap::new(),
  };
  module.build_dependency_graph();
  let mut ordered_dependencies = module.dependencies.clone().into_iter().collect::<Vec<_>>();
  ordered_dependencies.sort_unstable_by(|(_, a), (_, b)| a.len().cmp(&b.len()));
  for (name, deps) in ordered_dependencies {
    module.evaluate(&name, &mut hlir.heap)?;
  }
  let global = module.get_const(&GLOBAL_SCOPE_MANGLE.to_string()).unwrap();
  //println!("---\nEvaluated to\n---\n{global}");
  Ok(module)
}

impl MlIrModule {
  pub fn evaluates_to(&self) -> ConstValue {
    self.get_const(&GLOBAL_SCOPE_MANGLE.to_string()).unwrap()
  }
}

struct Analyzer<'a> {
  blocks: HashMap<Mangle, Block>,
  hl: &'a HlIrModule,
}

impl<'a> Analyzer<'a> {
  fn new(hl: &'a HlIrModule) -> Self {
    Self {
      blocks: HashMap::new(),
      hl,
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

  fn lower(&mut self, block: &Mangle, node_ptr: IrPtr, break_depth: &mut usize) -> Result<()> {
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
      } => {
        if is_constant {
          self.new_block(assignee.clone(), BlockKind::Constant(None));
          self.lower(&assignee, value, &mut 0)?;
        } else {
          self.lower(block, value, break_depth)?;
          self.push(block, new(ml::Set(assignee)));
        }
        self.push(block, new(ml::Const(ConstValue::Nothing)));
      }
      Immediate(const_value) => {
        self.push(block, new(ml::Const(const_value)));
      }
      Block(items) => {
        let length = items.len();
        for (id, item) in items.into_iter().enumerate() {
          self.lower(block, item, break_depth)?;
          if id != length - 1 {
            self.push(block, new(ml::Drop));
          }
        }
        if length == 0 {
          self.push(block, new(ml::Const(ConstValue::Nothing)));
        }
      }
      Identifier(mangle) => {
        self.push(block, new(ml::Get(mangle)));
      }
      StructDef {
        field_names: fields,
        field_types: types,
      } => {
        for type_ in types {
          self.lower(block, type_, &mut 0)?;
        }
        self.push(block, new(ml::StructDef { fields }));
      }
      StructLiteral {
        struct_t,
        field_names,
        field_values,
      } => {
        for value in field_values {
          self.lower(block, value, break_depth)?;
        }
        if let Some((struct_t, mangle)) = &struct_t {
          self.lower(block, *struct_t, break_depth)?;
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
          self.push(block, new(ml::TypeAssert));
        }
      }
      Field { of, index } => {
        self.lower(block, of, break_depth)?;
        self.push(block, new(ml::Field(index)));
      }
      Binary {
        op, left, right, ..
      } => {
        self.lower(block, left, break_depth)?;
        self.lower(block, right, break_depth)?;
        self.push(block, new(ml::BinaryOp { kind: op }));
      }
      Unary { op, child, .. } => {
        self.lower(block, child, break_depth)?;
        self.push(block, new(ml::UnaryOp { kind: op }));
      }
      FunctionDef {
        name,
        parameter_names,
        parameter_spans,
        body,
      } => {
        self.new_block(name.clone(), BlockKind::Function {
          parameters: parameter_names,
          parameter_spans,
          value: None,
        });
        self.lower(&name, body, &mut 0)?;
        self.push(block, new(ml::Get(name)));
      }
      FunctionCall {
        callee, arguments, ..
      } => {
        let arity = arguments.len();
        let mut spans = vec![];
        self.lower(block, callee, break_depth)?;
        for argument in arguments {
          spans.push(self.hl.value_span(argument));
          self.lower(block, argument, break_depth)?;
        }
        self.push(block, new(ml::Call { arity, spans }));
      }
      If {
        predicate,
        then,
        else_,
      } => {
        self.lower(block, predicate, break_depth)?;
        self.push(block, new(ml::If));
        self.lower(block, then, break_depth)?;
        if let Some(else_) = else_ {
          self.push(block, new(ml::Else));
          self.lower(block, else_, break_depth)?;
        }
        self.push(block, new(ml::End));
      }
      Loop {
        parameter_names,
        parameter_values,
        parameter_spans,
        body,
      } => {
        for value in parameter_values {
          self.lower(block, value, break_depth)?;
        }
        for name in parameter_names.iter().rev() {
          self.push(block, new(ml::Set(name.clone())));
        }
        self.push(block, new(ml::Loop));
        *break_depth += 1;
        self.lower(block, body, break_depth)?;
        *break_depth -= 1;
        for name in parameter_names.iter().rev() {
          self.push(block, new(ml::Set(name.clone())));
        }
        self.push(block, new(ml::Repeat));
      }
      Break(e) => {
        if *break_depth == 0 {
          return Err(lint(NameLint::NoBreakTarget, node.span, &[]));
        }
        if let Some(e) = e {
          self.lower(block, e, break_depth)?;
        } else {
          self.push(block, new(ml::Const(ConstValue::Nothing)));
        }
        self.push(block, new(ml::Break));
      }
      Tuple(items) => todo!(),
      Match {
        on,
        patterns,
        branches,
      } => todo!(),
    }
    Ok(())
  }
}
