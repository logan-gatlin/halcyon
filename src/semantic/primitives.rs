use crate::{BinaryOp, UnaryOp};

use crate::err::*;

macro_rules! primitives {
  ( $($i:ident),* ) => {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types, dead_code)]
    pub enum Primitive {
      integer_ambiguous,
      real_ambiguous,
      $($i,)*
    }

    impl Primitive {
      pub fn from_string(string: &str) -> Option<Self> {
        match string {
          $(stringify!{$i} => Some(Self::$i),)*
          _ => None,
        }
      }
    }
    impl std::fmt::Display for Primitive {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          Primitive::integer_ambiguous => write!(f, "<ambiguous integer>"),
          Primitive::real_ambiguous => write!(f, "<ambiguous real>"),
          $(Primitive::$i => write!(f, stringify!{$i}),)*
        }
      }
    }
  };
}

primitives! {
  w8, w16, w32, w64, whole,
  i8, i16, i32, i64, integer,
  r32, r64, real,
  boolean,
  string, glyph
}

macro_rules! selfsame_basic {
  ( $lhs:ident, $op:ident, $rhs:ident, $binop:ident, $i:ident ) => {
    if ($i == $rhs && $rhs == $lhs) && $op == BinaryOp::$binop {
      return Ok($i);
    }
  };

  ( $lhs:ident, $op:ident, $rhs:ident; $($i:ident),* ) => {
    $(
      selfsame_basic!($lhs, $op, $rhs, Plus, $i);
      selfsame_basic!($lhs, $op, $rhs, Minus, $i);
      selfsame_basic!($lhs, $op, $rhs, Star, $i);
      selfsame_basic!($lhs, $op, $rhs, Slash, $i);
    )*
    logical!($lhs, $op, $rhs; $($i),*);
  };
}

macro_rules! logical {
  ( $lhs:ident, $op:ident, $rhs:ident; $($i:ident),* ) => {
    $(
      selfsame_basic!($lhs, $op, $rhs, And, $i);
      selfsame_basic!($lhs, $op, $rhs, Nand, $i);
      selfsame_basic!($lhs, $op, $rhs, Xor, $i);
      selfsame_basic!($lhs, $op, $rhs, Xnor, $i);
      selfsame_basic!($lhs, $op, $rhs, Or, $i);
      selfsame_basic!($lhs, $op, $rhs, Nor, $i);
    )*
  };
}

macro_rules! comparison {
  ( $lhs:ident, $op:ident, $rhs:ident, $binop:ident, $i:ident ) => {
    if ($i == $rhs && $rhs == $lhs) && $op == BinaryOp::$binop {
      return Ok(boolean);
    }
  };

  ( $lhs:ident, $op:ident, $rhs:ident; $($i:ident),* ) => {
    $(
    comparison!($lhs, $op, $rhs, DoubleEqual, $i);
    comparison!($lhs, $op, $rhs, BangEqual, $i);
    comparison!($lhs, $op, $rhs, Less, $i);
    comparison!($lhs, $op, $rhs, LessEqual, $i);
    comparison!($lhs, $op, $rhs, Greater, $i);
    comparison!($lhs, $op, $rhs, GreaterEqual, $i);
    )*
  };
}

impl Primitive {
  pub fn coerce_ambiguous(
    lhs: Primitive,
    rhs: Primitive,
  ) -> (Primitive, Primitive) {
    use Primitive::*;
    let is_int = |i| {
      if let i8 | i16 | i32 | i64 | integer | integer_ambiguous = i {
        true
      } else {
        false
      }
    };
    let is_real = |r| {
      if let r32 | r64 | real | real_ambiguous = r {
        true
      } else {
        false
      }
    };
    // Ambiguous integer coercion
    if lhs == integer_ambiguous && is_int(rhs) {
      return (rhs, rhs);
    } else if rhs == integer_ambiguous && is_int(lhs) {
      return (lhs, lhs);
    }
    // Ambiguous real coercion
    if lhs == real_ambiguous && is_real(rhs) {
      return (rhs, rhs);
    } else if rhs == real_ambiguous && is_real(lhs) {
      return (lhs, lhs);
    }
    return (lhs, rhs);
  }

  pub fn binary_op(
    lhs: Primitive,
    op: BinaryOp,
    rhs: Primitive,
  ) -> Result<Primitive> {
    use Primitive::*;
    let (lhs, rhs) = Primitive::coerce_ambiguous(lhs, rhs);
    selfsame_basic! {
      lhs, op, rhs; w8, w16, w32, w64, i8, i16, i32, i64,
      integer, integer_ambiguous, real, real_ambiguous
    }
    logical! { lhs, op, rhs; boolean }
    comparison! {
      lhs, op, rhs; w8, w16, w32, w64, i8, i16, i32, i64,
      integer, integer_ambiguous, real, real_ambiguous,
      boolean, string
    }
    error().reason(format!(
      "Binary {} is not defined for {} and {}",
      op, lhs, rhs
    ))
  }

  pub fn unary_op(op: UnaryOp, child: Primitive) -> Result<Primitive> {
    use Primitive::*;
    use UnaryOp::*;
    let e =
      error().reason(format!("Unary {} is not defined for {}", op, child));
    match op {
      Minus => match child {
        boolean | string | glyph | w8 | w16 | w32 | w64 => e,
        _ => Ok(child),
      },
      Plus => match child {
        boolean | string | glyph => e,
        _ => Ok(child),
      },
      Not => match child {
        string | glyph => e,
        _ => Ok(child),
      },
      _ => error().reason(format!("Unary {} is not implemented (yet)", op)),
    }
  }
}
