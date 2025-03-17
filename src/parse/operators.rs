use super::*;
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
        write!(f, "{:?}", self)
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
pub const FIELD_PREC: Precedence = 13;
pub const CALL_PREC: Precedence = 12;

// Name, precedence, associativity;
op! {
  BinaryOp;
  Star, 10, LEFT_ASSOC;
  Slash, 10, LEFT_ASSOC;
  Percent, 10, LEFT_ASSOC;
  Plus, 9, LEFT_ASSOC;
  Minus, 9, LEFT_ASSOC;
  Nand, 8, LEFT_ASSOC;
  Xor, 7, LEFT_ASSOC;
  Xnor, 7, LEFT_ASSOC;
  Or, 6, LEFT_ASSOC;
  Nor, 6, LEFT_ASSOC;
  DoubleEqual, 5, LEFT_ASSOC;
  BangEqual, 5, LEFT_ASSOC;
  Less, 5, LEFT_ASSOC;
  LessEqual, 5, LEFT_ASSOC;
  Greater, 5, LEFT_ASSOC;
  GreaterEqual, 5, LEFT_ASSOC;
  And, 4, LEFT_ASSOC;
}

op! {
  UnaryOp;
  Ampersand, 11, RIGHT_ASSOC;
  Tilda, 11, RIGHT_ASSOC;
  Minus, 11, LEFT_ASSOC;
  Not, 11, LEFT_ASSOC;
}

pub fn is_mixed_op(t: &TokenKind) -> bool {
  BinaryOp::try_from(t).is_ok() && UnaryOp::try_from(t).is_ok()
}
