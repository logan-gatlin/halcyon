macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! primitives {
  ( $($i:ident),* ) => {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, dead_code)]
    pub enum Primitive {
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

      pub fn mangle(&self) -> crate::naming::Mangle {
        match self {
          $(
          Primitive::$i => crate::naming::mangle_builtin(stringify!{$i}),
          )*
        }
      }
    }
    impl std::fmt::Display for Primitive {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Primitive::$i => write!(f, stringify!{$i}),)*
        }
      }
    }
  };
}

primitives! {
  nothing, unreachable,
  integer,
  real,
  boolean,
  string, glyph
}

impl Primitive {
  pub fn promote(self) -> Type {
    Type::Primitive(self)
  }
}

#[derive(Debug, Clone)]
pub enum Type {
  /// Indeterminate type
  Ambiguous,
  /// A primitive type
  Primitive(Primitive),
  /// User defined type
  Struct {
    member_names: Vec<String>,
    member_types: Vec<Type>,
  },
  /// Function type
  Function {
    param_types: Vec<Type>,
    return_type: Box<Type>,
  },
  /// Alias type
  Reference(Box<Type>),
  /// Higher level type
  Type,
}

impl Type {
  pub fn ambiguous(&self) -> bool {
    if let Self::Ambiguous = self {
      true
    } else {
      false
    }
  }

  pub fn unwrap_reference(mut self) -> Self {
    while let Self::Reference(t) = self {
      self = *t;
    }
    self
  }
}

impl Default for Type {
  fn default() -> Self {
    Self::Ambiguous
  }
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type as t;
    match (self, other) {
      (t::Ambiguous, t::Ambiguous) => {
        panic!("Tried to compare ambiguous types")
      },
      (t::Type, t::Type) => true,
      (t::Primitive(p1), t::Primitive(p2)) => p1 == p2,
      (
        t::Struct {
          member_names: names1,
          member_types: types1,
        },
        t::Struct {
          member_names: names2,
          member_types: types2,
        },
      ) => names1 == names2 && types1 == types2,
      (
        t::Function {
          param_types: p1,
          return_type: r1,
        },
        t::Function {
          param_types: p2,
          return_type: r2,
        },
      ) => p1.len() == p2.len() && p1 == p2 && r1 == r2,
      _ => false,
    }
  }
}

impl Eq for Type {
}

impl std::hash::Hash for Type {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Type::Primitive(primitive) => primitive.hash(state),
      Type::Struct {
        member_names,
        member_types,
      } => {
        member_names.hash(state);
        member_types.hash(state);
      },
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types.hash(state);
        return_type.hash(state);
      },
      Type::Type => "type".hash(state),
      Type::Ambiguous => panic!("Tried to hash ambiguous type"),
      Type::Reference(t) => {
        "ref".hash(state);
        t.hash(state);
      },
    }
  }
}

fn indent(s: String) -> String {
  s.lines()
    .map(|l| format!("    {l}"))
    .collect::<Vec<_>>()
    .join("\n")
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Ambiguous => write!(f, "?"),
      Type::Primitive(primitive) => write!(f, "{primitive}"),
      Type::Struct {
        member_names,
        member_types,
      } => {
        let fields = member_names
          .into_iter()
          .zip(member_types.into_iter())
          .map(|(name, type_)| format!("{name}: {type_}"))
          .collect::<Vec<_>>()
          .join(",\n");
        let fields = indent(fields);
        write!(f, "struct {{\n{fields}\n}}")
      },
      Type::Type => write!(f, "type"),
      Type::Function {
        param_types,
        return_type,
      } => write!(
        f,
        "({}) -> {}",
        param_types
          .iter()
          .map(|t| format!("{t}"))
          .collect::<Vec<_>>()
          .join(", "),
        return_type
      ),
      Type::Reference(t) => write!(f, "{t}&"),
    }
  }
}
