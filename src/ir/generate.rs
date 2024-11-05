/*
use crate::{Expression, ExpressionKind, Statement, StatementKind};

use super::*;

impl Compiler {
  pub fn generate(&mut self, statements: Vec<Statement>) {
    for s in statements {
      self.statement(s);
    }
  }

  pub fn new_temporary(&mut self, type_: Type) -> UID {
    let uid = format!("$$tmp{}", self.tmp_num);
    self.push(IR::New {
      uid: uid.clone(),
      type_,
      mutable: true,
      global: false,
    });
    self.tmp_num += 1;
    uid
  }

  pub fn statement(&mut self, stmt: Statement) {
    use StatementKind as s;
    match stmt.kind {
      s::Declaration {
        type_actual,
        value,
        mutable,
        uid,
        ..
      } => {
        let global = self.table.table.get(&uid).unwrap().global;
        match type_actual {
          Type::Prim(_) | Type::Struct(_) => {
            self.push(IR::New {
              uid: uid.clone(),
              type_: type_actual.clone(),
              mutable,
              global,
            });
            self.expression(value);
            self.push(IR::Set {
              uid,
              type_: type_actual,
              global,
            });
          },
          Type::Nothing | Type::Function(_) | Type::Alias(_) => {
            self.expression(value);
          },
          Type::Ambiguous => unreachable!(),
        };
      },
      s::Assignment { name, value, uid } => {
        let global = self.table.table.get(&uid).unwrap().global;
        let type_ = value.type_.clone();
        self.expression(value);
        self.push(IR::Set { uid, type_, global });
      },
      s::If {
        predicate,
        block,
        else_,
      } => todo!(),
      s::While { predicate, block } => todo!(),
      s::Print(expression) => {
        let type_ = expression.type_.clone();
        self.expression(expression);
        self.push(IR::Print { type_ });
      },
      s::Expression(expression) => {
        let type_ = expression.type_.clone();
        self.expression(expression);
        self.push(IR::Drop { type_ });
      },
      s::Block(block) => {
        for s in block {
          self.statement(s);
        }
      },
      s::Return(expression) => {
        let type_ = if let Some(expression) = expression {
          let type_ = expression.type_.clone();
          self.expression(expression);
          type_
        } else {
          Type::Nothing
        };
        self.push(IR::Return { type_ });
      },
      s::Error(diagnostic) => {
        panic!("{diagnostic}")
      },
    }
  }

  pub fn expression(&mut self, expr: Expression) {
    use ExpressionKind as e;
    match expr.kind {
      e::Immediate(immediate) => {
        let Type::Prim(p) = expr.type_ else {
          unreachable!();
        };
        self.push(IR::Push {
          value: immediate,
          prim: p,
        })
      },
      e::Identifier(_, uid) => match expr.type_ {
        Type::Prim(_) | Type::Struct(_) => self.push(IR::Get {
          global: self.table.table.get(&uid).unwrap().global,
          uid,
          type_: expr.type_,
        }),
        Type::Ambiguous => unreachable!(),
        _ => {},
      },
      e::Binary { op, left, right } => {
        self.expression(*left);
        self.expression(*right);
        self.push(IR::BinOp {
          op,
          type_: expr.type_,
        })
      },
      e::Unary { op, child } => {
        self.expression(*child);
        self.push(IR::UnOp {
          op,
          type_: expr.type_,
        })
      },
      e::Parenthesis(inner) => self.expression(*inner),
      e::FunctionDef {
        params,
        returns_actual,
        body,
        id,
        ..
      } => {
        self.push(IR::StartFunc {
          uid: id,
          params: params
            .into_iter()
            .map(|p| (p.name, p.type_actual))
            .collect(),
          returns: returns_actual,
        });
        for s in body {
          self.statement(s);
        }
        self.push(IR::EndFunc);
      },
      e::FunctionCall {
        callee, args, id, ..
      } => {
        self.expression(*callee);
        for arg in args.into_iter().rev() {
          self.expression(arg);
        }
        self.push(IR::Call { uid: id })
      },
      e::StructDef(..) => {},
      e::StructLiteral { args, .. } => {
        let struct_id = if let Type::Struct(s) = expr.type_ {
          s
        } else {
          unreachable!()
        };
        let length = args.len();
        let mut temp_buffer = vec![None; length];
        let mut iter = args.into_iter();
        let mut index = length - 1;
        loop {
          // If struct param has already been saved
          if let Some((uid, type_)) = temp_buffer[index].take() {
            self.ir.push(IR::Get {
              uid,
              type_,
              global: false,
            });
            if index == 0 {
              break;
            }
            index -= 1;
          }
          // If struct parameter has not been saved
          else {
            let (name, arg) = iter.next().unwrap();
            let type_ = arg.type_.clone();
            self.expression(arg);
            let argno = self.table.get_field_no(struct_id, &name);
            if argno != index {
              let temp = self.new_temporary(type_.clone());
              temp_buffer[argno] = Some((temp.clone(), type_.clone()));
              self.push(IR::Set {
                uid: temp,
                type_: type_.clone(),
                global: false,
              });
            } else {
              if index == 0 {
                break;
              }
              index -= 1;
            }
          }
        }
      },
      e::Field {
        namespace, field, ..
      } => {
        if let Type::Struct(sid) = namespace.type_ {
          self.expression(*namespace);
          let name = if let e::Identifier(name, _) = field.kind {
            name
          } else {
            unreachable!()
          };
          // TODO extract field
          let field_type = self.table.get_field(sid, &name).unwrap();
          let temp = self.new_temporary(field_type.clone());
          for (field_name, uid) in self.table.structs[sid].0.clone() {
            let type_ =
              self.table.resolve_type(&uid).unwrap().is_alias().unwrap();
            if field_name != name {
              self.push(IR::Drop { type_ });
            } else {
              self.push(IR::Set {
                uid: temp.clone(),
                type_: field_type.clone(),
                global: false,
              })
            }
          }
          self.push(IR::Get {
            uid: temp,
            type_: field_type.clone(),
            global: false,
          });
        } else {
          self.expression(*namespace);
        }
      },
    }
  }
}
*/
