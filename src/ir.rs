use crate::{semantic::Type, BinaryOp, Immediate, UnaryOp};

#[derive(Debug, Clone)]
pub enum IR {
  BinOp { op: BinaryOp, type_: Type },
  UnOp { op: UnaryOp, type_: Type },
  Imm(Immediate),
  NewLocal { uid: usize, type_: Type },
  AssignLocal { uid: usize },
  AccessLocal { uid: usize },
  NewGlobal { uid: usize, type_: Type },
  AssignGlobal { uid: usize },
  AccessGlobal { uid: usize },
  StartFunc { uid: usize },
  NewParam { uid: usize, type_: Type },
  EndFunc,
}
