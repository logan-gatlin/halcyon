use crate::{Expression, ExpressionKind, Statement, StatementKind};

use super::*;

impl Compiler {
  fn statement(&mut self, stmt: Statement) {
    use StatementKind as s;
    match stmt.kind {
      s::Declaration {
        name,
        type_str,
        type_actual,
        value,
        mutable,
        uid,
      } => self.push(IR::New {
        uid,
        type_: type_actual,
      }),
      s::Assignment { name, value, uid } => todo!(),
      s::If {
        predicate,
        block,
        else_,
      } => todo!(),
      s::While { predicate, block } => todo!(),
      s::Print(expression) => todo!(),
      s::Expression(expression) => todo!(),
      s::Block(vec) => todo!(),
      s::Return(expression) => todo!(),
      s::Error(diagnostic) => todo!(),
    }
  }

  fn expression(&mut self, expr: Expression) {
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
      e::Identifier(_, uid) => {
        self.push(IR::Get { uid });
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
        returns_str,
        returns_actual,
        body,
        id,
      } => {
        self.push(IR::StartFunc {
          fid: id,
          params: params.into_iter().map(|p| p.type_actual).collect(),
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
        for arg in args {
          self.expression(arg);
        }
        let fid = if let Type::Function(fid) = expr.type_ {
          fid
        } else {
          unreachable!()
        };
        self.push(IR::Call { fid })
      },
      e::StructDef(..) => {},
      e::StructLiteral { name, args, id } => {},
      e::Field {
        namespace,
        field,
        uid,
      } => {
        self.expression(*namespace);
      },
    }
  }
}
