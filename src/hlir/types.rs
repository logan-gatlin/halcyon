use std::collections::HashSet;

pub type TypeVariable = usize;

#[derive(Debug, Clone)]
pub enum Type {
  /// Indeterminate type
  Any,
  /// The empty type ()
  Unit,
  /// Signed 64 bit integer
  Integer,
  /// IEEE 64 bit floating point
  Real,
  /// true or false
  Boolean,
  /// Fat pointer to byte array of UTF-8
  String,
  /// UTF-8 codepoint 32 bit
  Glyph,
  /// Higher order type
  Type,
  // Type variable
  TypeVariable(TypeVariable),
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
  Function(Box<Type>, Box<Type>),
  /// Placeholder until arrays are implemented, so I can
  /// generate ANYREF array in type section
  _ClosureCapture,
}

impl std::ops::Add for Type {
  type Output = Type;

  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (t1, t2) if t1 == t2 => t1,
      (Type::Sum(s1), Type::Sum(s2)) => {
        Type::Sum(s1.union(&s2).cloned().collect::<HashSet<_>>())
      },
      (Type::Sum(mut s), t) | (t, Type::Sum(mut s)) => {
        s.insert(t);
        Type::Sum(s)
      },
      (t1, t2) => {
        let mut hs = HashSet::new();
        hs.insert(t1);
        hs.insert(t2);
        Type::Sum(hs)
      },
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
      },
      (Type::Product(mut s), t) => {
        s.push(t);
        Type::Product(s)
      },
      (t, Type::Product(mut s)) => {
        let mut v = vec![t];
        v.append(&mut s);
        Type::Product(v)
      },
      (t1, t2) => Type::Product(vec![t1, t2]),
    }
  }
}

impl Type {
  pub fn primitives() -> Vec<(Type, &'static str)> {
    vec![
      (Self::Unit, "unit"),
      (Self::Integer, "integer"),
      (Self::Real, "real"),
      (Self::Boolean, "boolean"),
      (Self::String, "string"),
      (Self::Glyph, "glyph"),
    ]
  }

  pub fn func(parameter: Type, returns: Type) -> Type {
    Type::Function(parameter.into(), returns.into())
  }

  pub fn is_subtype(&self, other: &Type) -> bool {
    match self.partial_cmp(other) {
      Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {
        true
      },
      _ => false,
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
    if let Self::Any = self { true } else { false }
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
      Type::Function(a, b) => {
        a.contains_type_var(tv) || b.contains_type_var(tv)
      },
      _ => false,
    }
  }

  pub fn substitute(&mut self, tv: TypeVariable, type_: &Type) {
    match self {
      Type::TypeVariable(t) => {
        if *t == tv {
          *self = type_.clone();
        }
      },
      Type::Struct { member_types, .. } => {
        member_types
          .iter_mut()
          .for_each(|t| t.substitute(tv, type_));
      },
      Type::Product(items) => {
        items.iter_mut().for_each(|i| i.substitute(tv, type_))
      },
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
      },
      Type::Function(a, b) => {
        a.substitute(tv, type_);
        b.substitute(tv, type_);
      },
      Type::Any
      | Type::_ClosureCapture
      | Type::Unit
      | Type::Integer
      | Type::Real
      | Type::Boolean
      | Type::String
      | Type::Glyph
      | Type::Type => {},
    }
  }
}

impl Default for Type {
  fn default() -> Self {
    Self::Any
  }
}

impl PartialOrd for Type {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    use crate::hlir::Type::*;
    use std::cmp::Ordering::*;
    Some(match (self, other) {
      (Any, Any) => Equal,
      (Any, _) => Greater,
      (t1, t2) if t1 == t2 => Equal,
      _ => return None,
    })
  }
}

impl PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    use Type as t;
    match (self, other) {
      (t::Any, t::Any) => {
        panic!("Tried to compare ambiguous types")
      },
      (t::_ClosureCapture, t::_ClosureCapture)
      | (t::Unit, t::Unit)
      | (t::Integer, t::Integer)
      | (t::Real, t::Real)
      | (t::Boolean, t::Boolean)
      | (t::Glyph, t::Glyph)
      | (t::String, t::String)
      | (t::Type, t::Type) => true,
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
      (t::Function(p1, r1), t::Function(p2, r2)) => p1 == p2 && r1 == r2,
      (t::Product(t1), t::Product(t2)) => t1 == t2,
      (t::Sum(v1), t::Sum(v2)) => v1 == v2,
      (t::TypeVariable(p1), t::TypeVariable(p2)) => p1 == p2,
      _ => false,
    }
  }
}

impl Eq for Type {
}

impl std::hash::Hash for Type {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Type::Struct {
        member_names,
        member_types,
      } => {
        member_names.hash(state);
        member_types.hash(state);
      },
      Type::Function(a, b) => {
        a.hash(state);
        b.hash(state);
      },
      Type::Type => "type".hash(state),
      Type::Any => {
        "any".hash(state);
      },
      Type::Sum(_) => todo!(),
      Type::TypeVariable(id) => {
        "poly".hash(state);
        id.hash(state);
      },
      Type::Product(items) => {
        "tuple".hash(state);
        for item in items {
          item.hash(state);
        }
      },
      Type::Unit
      | Type::_ClosureCapture
      | Type::Integer
      | Type::Real
      | Type::Boolean
      | Type::String
      | Type::Glyph => {
        format!("{self}").hash(state);
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
      Type::Any => write!(f, "?"),
      Type::_ClosureCapture => write!(f, "_ClosureCapture"),
      Type::Unit => write!(f, "()"),
      Type::Integer => write!(f, "integer"),
      Type::Real => write!(f, "real"),
      Type::Boolean => write!(f, "boolean"),
      Type::String => write!(f, "string"),
      Type::Glyph => write!(f, "glyph"),
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
      Type::Function(a, b) => write!(f, "{} -> {}", a, b),
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
      },
    }
  }
}
