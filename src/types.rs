use super::err::*;

macro_rules! primitives {
  ( $($i:ident),* ) => {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types, dead_code)]
    pub enum Primitive {
      whole_ambiguous,
      integer_ambiguous,
      real_ambiguous,
      $($i,)*
    }

    impl Primitive {
      pub fn from_string(string: &'static str) -> Option<Self> {
        match string {
          $(stringify!{$i} => Some(Self::$i),)*
          _ => None,
        }
      }
    }
    impl std::fmt::Display for Primitive {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          Primitive::whole_ambiguous => write!(f, "<ambiguous whole>"),
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

impl Primitive {
  pub fn coerce(a: Self, b: Self) -> Result<Self> {
    use Primitive::*;
    match (a, b) {
      // Whole? coerces to any whole or integer
      (whole_ambiguous, w @ (w8 | w16 | w32 | w64 | i8 | i16 | i32 | i64))
      | (w @ (w8 | w16 | w32 | w64 | i8 | i16 | i32 | i64), whole_ambiguous) => {
        w
      },
      // Integer? coerces to any integer
      (integer_ambiguous, i @ (i8 | i16 | i32 | i64))
      | (i @ (i8 | i16 | i32 | i64), integer_ambiguous) => i,
      _ => whole_ambiguous,
    };
    todo!()
  }
}

use Primitive as p;

// Implement math operations for regular types
macro_rules! selfsame_op {
  ($trait:ident, $fn:ident, $($i:ident),* ) => {
    impl std::ops::$trait for Primitive {
      type Output = Result<Primitive>;
      fn $fn(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
          $((p::$i, p::$i) => Ok(p::$i),)*
          _ => error()
          .reason(format!("Operation not defined for primitives {} and {}", self, rhs))
        }
      }
    }
  };
}

// Implement all regular math
macro_rules! all_selfsame {
  ($($i:ident),*) => {
    selfsame_op!(Add, add, $($i),*);
    selfsame_op!(Sub, sub, $($i),*);
    selfsame_op!(Mul, mul, $($i),*);
    selfsame_op!(Div, div, $($i),*);
  };
}

all_selfsame!(
  w8, w16, w32, w64, whole, i8, i16, i32, i64, integer, r32, r64, real, string
);
