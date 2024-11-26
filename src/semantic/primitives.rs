use crate::{BinaryOp, UnaryOp};

use crate::err::*;
use crate::semantic::{UID, builtin};

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

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
      pub const ALL: [Primitive; count!($($i)*,) - 1] = [$(Primitive::$i),*];

      pub fn from_string(string: &str) -> Option<Self> {
        match string {
          $(stringify!{$i} => Some(Self::$i),)*
          _ => None,
        }
      }

      pub fn mangle(&self) -> UID {
        match self {
          Primitive::integer_ambiguous => builtin::mangle("integer_ambiguous"),
          Primitive::real_ambiguous => builtin::mangle("real_ambiguous"),
          $(
          Primitive::$i => builtin::mangle(stringify!{$i}),
          )*
        }
      }
    }
    impl std::fmt::Display for Primitive {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          Primitive::integer_ambiguous => write!(f, "ambiguous integer"),
          Primitive::real_ambiguous => write!(f, "ambiguous real"),
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

impl Primitive {
  pub fn as_wat(&self) -> &'static str {
    use Primitive::*;
    match self {
      boolean | glyph | w8 | w16 | w32 | whole | i8 | i16 | i32 | integer => {
        "i32"
      },
      w64 | i64 => "i64",
      r32 | real => "f32",
      r64 => "f64",
      string => todo!(),
      _ => panic!(),
    }
  }
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
  pub fn coerce(self, expect: Primitive) -> Result<Self> {
    use Primitive::*;
    match (self, expect) {
      (
        integer_ambiguous,
        a @ (i8 | i16 | i32 | i64 | integer | w8 | w16 | w32 | w64 | whole),
      )
      | (
        a @ (i8 | i16 | i32 | i64 | integer | w8 | w16 | w32 | w64 | whole),
        integer_ambiguous,
      ) => Ok(a),
      (real_ambiguous, a @ (r32 | r64 | real))
      | (a @ (r32 | r64 | real), real_ambiguous) => Ok(a),
      (t1, t2) if t1 == t2 => Ok(t1),
      _ => error().reason(format!("Cannot coerce '{self}' into '{expect}'")),
    }
  }

  pub fn is_ambiguous(&self) -> bool {
    match self {
      Primitive::integer_ambiguous | Primitive::real_ambiguous => true,
      _ => false,
    }
  }

  pub fn promote(&mut self) {
    *self = match *self {
      Primitive::integer_ambiguous => Primitive::integer,
      Primitive::real_ambiguous => Primitive::real,
      _ => *self,
    }
  }

  pub fn binary_op(
    mut lhs: Primitive,
    op: BinaryOp,
    mut rhs: Primitive,
  ) -> Result<Primitive> {
    use Primitive::*;
    if lhs.is_ambiguous() && !rhs.is_ambiguous() {
      lhs = lhs.coerce(rhs)?;
    } else if rhs.is_ambiguous() && !lhs.is_ambiguous() {
      rhs = rhs.coerce(lhs)?;
    }
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
        boolean | string | glyph | whole | w8 | w16 | w32 | w64 => e,
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
