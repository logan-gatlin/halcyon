use super::*;

pub fn build_mlir(hlir: &HlIrModule) -> MlIrModule {
  let mut this = Analyzer::new(hlir);
  this.new_block("global".into(), BlockKind::GlobalScope);
  this.lower(&"global".to_string(), 0);
  MlIrModule {
    blocks: this.blocks,
  }
}

struct Analyzer<'a> {
  blocks: HashMap<Mangle, Block>,
  hl: &'a HlIrModule,
}

impl<'a> Analyzer<'a> {
  pub fn new(hl: &'a HlIrModule) -> Self {
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
        type_assert,
        value,
      } => {
        if is_constant {
          self.new_block(assignee.clone(), BlockKind::Constant { evaluation: None });
          self.lower(&assignee, value);
          if let Some(type_) = type_assert {
            self.lower(&assignee, type_);
            self.push(&assignee, new(ml::TypeAssert));
          }
        } else {
          self.lower(block, value);
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
          self.lower(block, item);
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
      StructDef { fields, types } => {
        for type_ in types {
          self.lower(block, type_);
        }
        self.push(block, new(ml::StructDef { fields }));
      }
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
          self.push(block, new(ml::TypeAssert));
        }
      }
      Field { of, index } => {
        self.lower(block, of);
        self.push(block, new(ml::Field(index)));
      }
      Binary {
        op, left, right, ..
      } => {
        self.lower(block, left);
        self.lower(block, right);
        self.push(block, new(ml::BinaryOp { kind: op }));
      }
      Unary { op, child, .. } => {
        self.lower(block, child);
        self.push(block, new(ml::UnaryOp { kind: op }));
      }
      FunctionDef {
        name,
        parameter_names,
        parameter_types,
        returns,
        body,
      } => {
        for (name, type_) in parameter_names.iter().zip(parameter_types.into_iter()) {
          self.new_block(name.clone(), BlockKind::Parameter);
          self.lower(name, type_);
        }
        if let Some((type_, name)) = &returns {
          self.new_block(name.clone(), BlockKind::Parameter);
          self.lower(name, *type_);
        }
        self.new_block(name.clone(), BlockKind::Function {
          parameters: parameter_names,
        });
        self.lower(&name, body);
        self.push(block, new(ml::Get(name)));
      }
      FunctionCall {
        callee, arguments, ..
      } => {
        let arity = arguments.len();
        self.lower(block, callee);
        for argument in arguments {
          self.lower(block, argument);
        }
        self.push(block, new(ml::Call { arity }));
      }
      If {
        predicate,
        then,
        else_,
      } => {
        self.lower(block, predicate);
        self.push(block, new(ml::If));
        self.lower(block, then);
        if let Some(else_) = else_ {
          self.push(block, new(ml::Else));
          self.lower(block, else_);
        }
        self.push(block, new(ml::End));
      }
      Loop {
        parameter_names,
        parameter_values,
        body,
      } => {
        for value in parameter_values {
          self.lower(block, value);
        }
        for name in parameter_names.iter().rev() {
          self.push(block, new(ml::Set(name.clone())));
        }
        self.push(block, new(ml::Loop));
        self.lower(block, body);
        for name in parameter_names.iter().rev() {
          self.push(block, new(ml::Set(name.clone())));
        }
        self.push(block, new(ml::Repeat));
      }
      Break(e) => {
        if let Some(e) = e {
          self.lower(block, e);
        } else {
          self.push(block, new(ml::Const(ConstValue::Nothing)));
        }
        self.push(block, new(ml::Break));
      }
    }
  }
}
