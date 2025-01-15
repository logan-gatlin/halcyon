use crate::Immediate;

macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! primitives {
  ( $($i:ident),* ) => {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, dead_code)]
    pub enum Primitive {
      integer_literal,
      real_literal,
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

      pub fn mangle(&self) -> crate::semantic::Mangle {
        match self {
          Primitive::integer_literal => crate::semantic::mangle_builtin("integer_literal"),
          Primitive::real_literal => crate::semantic::mangle_builtin("real_literal"),
          $(
          Primitive::$i => crate::semantic::mangle_builtin(stringify!{$i}),
          )*
        }
      }
    }
    impl std::fmt::Display for Primitive {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          Primitive::integer_literal => write!(f, "ambiguous integer"),
          Primitive::real_literal => write!(f, "ambiguous real"),
          $(Primitive::$i => write!(f, stringify!{$i}),)*
        }
      }
    }
  };
}

primitives! {
  nothing, never,
  integer,
  real,
  boolean,
  string, glyph
}

impl Immediate {
  pub fn type_of(&self) -> Primitive {
    use Immediate as i;
    use Primitive as p;
    match self {
      i::Unit => p::nothing,
      i::Integer(..) => p::integer_literal,
      i::Real(..) => p::real_literal,
      i::String(..) => p::string,
      i::Glyph(..) => p::glyph,
      i::Boolean(..) => p::boolean,
    }
  }
}
