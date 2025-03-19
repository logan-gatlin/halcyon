use crate::compile::VirtualMachine;

use super::*;

#[derive(Clone)]
enum ControlStack {
  Function {
    previous_values: HashMap<Mangle, ConstValue>,
    return_block: Mangle,
    return_ip: usize,
  },
}

impl MlIrModule {
  pub fn evaluate_block(&self, mangle: &Mangle) -> Result<ConstValue> {
    let mut block = match self.blocks.get(mangle) {
      // Already evaluated
      Some(Block {
        kind: BlockKind::Constant {
          evaluation: Some(value),
        },
        ..
      }) => return Ok(value.clone()),
      Some(Block {
        kind:
          BlockKind::Constant { evaluation: None } | BlockKind::Parameter | BlockKind::GlobalScope,
        body,
      }) => body,
      _ => panic!(),
    };
    let optable = OpTable::new();
    let mut ip = 0;
    let stack = &mut vec![];
    let pop = |stack: &mut Vec<ConstValue>| stack.pop().unwrap_or(ConstValue::Nothing);
    let mut name_map = HashMap::new();
    loop {
      if ip == block.len() {
        return Ok(pop(stack));
      }
      let instr = block.get(ip).unwrap();
      use MlIrKind::*;
      match instr.kind.clone() {
        Const(const_value) => {
          stack.push(const_value);
        }
        Set(mangle) => {
          let value = pop(stack);
          name_map.insert(mangle, value);
        }
        Get(mangle) => {
          let value = name_map.get(&mangle).unwrap();
          stack.push(value.clone());
        }
        BinaryOp { kind } => {
          let first = pop(stack);
          let second = pop(stack);
          let opkind = optable
            .try_binary(kind, &first.type_of(), &second.type_of())
            .unwrap();
          let result =
            VirtualMachine::run(vec![second, first], opkind.asm, opkind.produces).unwrap();
          stack.push(result);
        }
        UnaryOp { kind } => {
          let value = pop(stack);
          let opkind = optable.try_unary(kind, &value.type_of()).unwrap();
          let result = VirtualMachine::run(vec![value], opkind.asm, opkind.produces).unwrap();
          stack.push(result);
        }
        Field(field) => {
          let ConstValue::StructLiteral {
            member_names,
            member_values,
          } = pop(stack)
          else {
            panic!()
          };
          let position = member_names.iter().position(|s| s == &field).unwrap();
          stack.push(member_values[position].clone());
        }
        StructLiteral { param_names } => {
          let mut param_values = vec![];
          for _ in 0..param_names.len() {
            param_values.push(pop(stack));
          }
          param_values.reverse();
          stack.push(ConstValue::StructLiteral {
            member_names: param_names,
            member_values: param_values,
          });
        }
        StructDef { fields } => {
          let mut types = vec![];
          for _ in 0..fields.len() {
            let ConstValue::Type(t) = pop(stack) else {
              panic!()
            };
            types.push(t);
          }
          stack.push(ConstValue::Type(Type::Struct {
            member_names: fields,
            member_types: types,
          }));
        }
        TypeAssert => {
          let ConstValue::Type(t) = pop(stack) else {
            panic!()
          };
          let value = pop(stack);
          if t != value.type_of() {
            panic!();
          }
          stack.push(value);
        }
        Call { arity } => {
          todo!()
        }
        Drop => {
          pop(stack);
        }
        If => {
          let ConstValue::Boolean(predicate) = pop(stack) else {
            panic!()
          };
          if !predicate {
            let mut nesting = 0;
            loop {
              ip += 1;
              if ip >= block.len() {
                panic!()
              }
              match &block.get(ip).unwrap().kind {
                If => nesting += 1,
                End => {
                  if nesting == 0 {
                    break;
                  }
                  nesting -= 1;
                }
                Else if nesting == 0 => break,
                _ => {}
              }
            }
          }
        }
        Else => {
          let mut nesting = 0;
          loop {
            ip += 1;
            if ip >= block.len() {
              panic!()
            }
            match &block.get(ip).unwrap().kind {
              If => nesting += 1,
              End => {
                if nesting == 0 {
                  break;
                }
                nesting -= 1;
              }
              _ => {}
            }
          }
        }
        End => {}
        Loop => {}
        Repeat => {
          let mut nesting = 0;
          loop {
            if ip == 0 {
              panic!();
            }
            ip -= 1;
            match &block.get(ip).unwrap().kind {
              Repeat => nesting += 1,
              Loop => {
                if nesting == 0 {
                  break;
                }
                nesting -= 1;
              }
              _ => {}
            }
          }
        }
        Break => {
          let mut nesting = 0;
          loop {
            ip += 1;
            if ip >= block.len() {
              panic!()
            }
            match &block.get(ip).unwrap().kind {
              Loop => nesting += 1,
              Repeat => {
                if nesting == 0 {
                  break;
                }
                nesting -= 1;
              }
              _ => {}
            }
          }
        }
      };
      ip += 1;
    }
  }
}
