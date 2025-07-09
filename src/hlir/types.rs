use std::collections::HashSet;

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

      pub fn mangle(&self) -> crate::hlir::Mangle {
        match self {
          $(
          Primitive::$i => crate::hlir::mangle_builtin(stringify!{$i}),
          )*
        }
      }
    }
    impl std::fmt::Display for Primitive {
      #[allow(unreachable_patterns)]
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          Primitive::nothing => write!(f, "()"),
          $(Primitive::$i => write!(f, stringify!{$i}),)*
        }
      }
    }
  };
}

primitives! {
  nothing,
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

pub type TypeVariable = usize;

#[derive(Debug, Clone)]
pub enum Type {
  /// Indeterminate type
  Ambiguous,
  // Type variable
  TypeVariable(TypeVariable),
  /// A primitive type
  Primitive(Primitive),
  /// Record type
  Struct {
    member_names: Vec<String>,
    member_types: Vec<Type>,
  },
  /// Tuple
  Product(Vec<Type>),
  /// Variant or enum
  Sum(HashSet<Type>),
  /// Function type
  Function {
    param_types: Vec<Type>,
    return_type: Box<Type>,
  },
  /// Higher order type
  Type,
}

impl From<Primitive> for Type {
  fn from(value: Primitive) -> Self {
    Type::Primitive(value)
  }
}

impl std::ops::Add for Type {
  type Output = Type;

  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (t1, t2) if t1 == t2 => t1,
      (Type::Sum(s1), Type::Sum(s2)) => Type::Sum(s1.union(&s2).cloned().collect::<HashSet<_>>()),
      (Type::Sum(mut s), t) | (t, Type::Sum(mut s)) => {
        s.insert(t);
        Type::Sum(s)
      }
      (t1, t2) => {
        let mut hs = HashSet::new();
        hs.insert(t1);
        hs.insert(t2);
        Type::Sum(hs)
      }
    }
  }
}

impl std::ops::Mul for Type {
  type Output = Type;

  fn mul(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Type::Product(mut v1), Type::Product(mut v2)) => {
        v1.append(&mut v2);
        Type::Product(v1)
      }
      (Type::Product(mut s), t) => {
        s.push(t);
        Type::Product(s)
      }
      (t, Type::Product(mut s)) => {
        let mut v = vec![t];
        v.append(&mut s);
        Type::Product(v)
      }
      (t1, t2) => Type::Product(vec![t1, t2]),
    }
  }
}

impl Type {
  pub fn main_fn() -> Type {
    Type::Function {
      param_types: vec![],
      return_type: Type::Primitive(Primitive::nothing).into(),
    }
  }

  pub fn field_index(&self, name: &str) -> Option<u32> {
    if let Type::Struct { member_names, .. } = self {
      let mut index = 0;
      let mut found = false;
      for n in member_names.iter() {
        if n == name {
          found = true;
          break;
        }
        index += 1;
      }
      if found { Some(index) } else { None }
    } else {
      None
    }
  }

  pub fn ambiguous(&self) -> bool {
    if let Self::Ambiguous = self {
      true
    } else {
      false
    }
  }

  pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
    match self {
      Type::TypeVariable(t) => tv == *t,
      Type::Struct { member_types, .. } => member_types
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Product(items) => items
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Sum(hash_set) => hash_set
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types
          .into_iter()
          .fold(false, |accum, x| accum || x.contains_type_var(tv))
          || return_type.contains_type_var(tv)
      }
      Type::Type => false,
      Type::Ambiguous => false,
      Type::Primitive(_) => false,
    }
  }

  pub fn substitute(&mut self, tv: TypeVariable, type_: &Type) {
    match self {
      Type::Ambiguous => {}
      Type::TypeVariable(t) => {
        if *t == tv {
          *self = type_.clone();
        }
      }
      Type::Primitive(_) => {}
      Type::Struct { member_types, .. } => {
        member_types
          .iter_mut()
          .for_each(|t| t.substitute(tv, type_));
      }
      Type::Product(items) => items.iter_mut().for_each(|i| i.substitute(tv, type_)),
      Type::Sum(hash_set) => {
        *self = Type::Sum(
          hash_set
            .clone()
            .into_iter()
            .map(|mut t| {
              t.substitute(tv, type_);
              t
            })
            .collect::<HashSet<_>>(),
        );
      }
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types.iter_mut().for_each(|t| t.substitute(tv, type_));
        return_type.substitute(tv, type_);
      }
      Type::Type => {}
    }
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
      }
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
      (t::Product(t1), t::Product(t2)) => t1 == t2,
      (t::Sum(v1), t::Sum(v2)) => v1 == v2,
      (t::TypeVariable(p1), t::TypeVariable(p2)) => p1 == p2,
      _ => false,
    }
  }
}

impl Eq for Type {}

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
      }
      Type::Function {
        param_types,
        return_type,
      } => {
        param_types.hash(state);
        return_type.hash(state);
      }
      Type::Type => "type".hash(state),
      Type::Ambiguous => {
        panic!("Tried to hash ambiguous type")
      }
      Type::Sum(_) => todo!(),
      Type::TypeVariable(id) => {
        "poly".hash(state);
        id.hash(state);
      }
      Type::Product(items) => {
        "tuple".hash(state);
        for item in items {
          item.hash(state);
        }
      }
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
      }
      Type::Type => write!(f, "type"),
      Type::Function {
        param_types,
        return_type,
      } => match param_types.as_slice() {
        [] => write!(
          f,
          "{} -> {return_type}",
          Type::Primitive(Primitive::nothing),
        ),
        [t] => write!(f, "{t} -> {return_type}"),
        _ => {
          write!(f, "{} -> {return_type}", Type::Product(param_types.clone()))
        }
      },
      Type::TypeVariable(id) => write!(f, "'{id}"),
      Type::Product(items) => write!(
        f,
        "({})",
        items
          .into_iter()
          .map(|i| format!("{i}"))
          .collect::<Vec<_>>()
          .join(" * ")
      ),
      Type::Sum(items) => {
        write!(
          f,
          "({})",
          items
            .into_iter()
            .map(|i| format!("{i}"))
            .collect::<Vec<_>>()
            .join(" + ")
        )
      }
    }
  }
}
