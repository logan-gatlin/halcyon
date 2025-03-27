pub mod assembly;
use crate::token::*;

pub use assembly::*;

pub type Precedence = usize;

macro_rules! op {
  ($name:ident; $($op:ident, $prec:expr, $assoc:expr);*;) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($op,)*
    }

    impl $name {
      pub fn precedence(&self) -> Precedence {
        match self {
          $(Self::$op => $prec),*
        }
      }

      pub fn assoc(&self) -> bool {
        match self {
          $(Self::$op => $assoc),*
        }
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Self::$op => write!(f, "{}", TokenKind::$op)),*
        }
      }
    }

    impl TryFrom<&TokenKind> for $name {
      type Error = ();
      fn try_from(value: &TokenKind) -> std::result::Result<Self, ()> {
        match value {
          $(TokenKind::$op => Ok(Self::$op),)*
          _ => Err(()),
        }
      }
    }
  }
}

pub const RIGHT_ASSOC: bool = true;
pub const LEFT_ASSOC: bool = false;
pub const CALL_PREC: Precedence = 12;

// Name, precedence, associativity;
op! {
  BinaryOp;
  Dot, 14, LEFT_ASSOC;
  Star, 12, LEFT_ASSOC;
  Slash, 12, LEFT_ASSOC;
  Percent, 12, LEFT_ASSOC;
  Plus, 11, LEFT_ASSOC;
  Minus, 11, LEFT_ASSOC;
  Nand, 10, LEFT_ASSOC;
  Xor, 9, LEFT_ASSOC;
  Xnor, 9, LEFT_ASSOC;
  Or, 8, LEFT_ASSOC;
  Nor, 8, LEFT_ASSOC;
  DoubleEqual, 7, LEFT_ASSOC;
  BangEqual, 7, LEFT_ASSOC;
  Less, 7, LEFT_ASSOC;
  LessEqual, 7, LEFT_ASSOC;
  Greater, 7, LEFT_ASSOC;
  GreaterEqual, 7, LEFT_ASSOC;
  And, 6, LEFT_ASSOC;
  FatArrow, 6, RIGHT_ASSOC;
  Colon, 5, RIGHT_ASSOC;
  Arrow, 5, RIGHT_ASSOC;
  Equal, 4, RIGHT_ASSOC;
  DoubleColon, 4, RIGHT_ASSOC;
  Comma, 3, LEFT_ASSOC;
}

op! {
  UnaryOp;
  Ampersand, 13, RIGHT_ASSOC;
  Tilda, 13, RIGHT_ASSOC;
  Minus, 13, LEFT_ASSOC;
  Not, 13, LEFT_ASSOC;
  Break, 0, LEFT_ASSOC;
}
