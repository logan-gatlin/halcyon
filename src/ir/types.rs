use std::{
  cell::RefCell,
  rc::{Rc, Weak},
};

pub type TypeVariable = usize;
pub type TypeRef = Rc<RefCell<Type>>;
pub type WeakTypeRef = Weak<RefCell<Type>>;

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
    member_types: Vec<TypeRef>,
  },
  /// Tuple
  Product(Vec<TypeRef>),
  /// Function type
  Function(TypeRef, TypeRef),
  /// Placeholder until arrays are implemented, so I can
  /// generate ANYREF array in type section
  _ClosureCapture,
  Weak(WeakTypeRef),
}

/*
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
*/

impl Type {
  pub fn to_ref(self) -> TypeRef {
    Rc::new(RefCell::new(self))
  }

  pub fn primitives() -> Vec<(TypeRef, &'static str)> {
    vec![
      (Self::Unit.to_ref(), "unit"),
      (Self::Integer.to_ref(), "integer"),
      (Self::Real.to_ref(), "real"),
      (Self::Boolean.to_ref(), "boolean"),
      (Self::String.to_ref(), "string"),
      (Self::Glyph.to_ref(), "glyph"),
    ]
  }

  pub fn func(
    parameter: impl Into<TypeRef>,
    returns: impl Into<TypeRef>,
  ) -> TypeRef {
    Type::Function(parameter.into(), returns.into()).to_ref()
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
        .fold(false, |accum, x| accum || x.borrow().contains_type_var(tv)),
      Type::Product(items) => items
        .into_iter()
        .fold(false, |accum, x| accum || x.borrow().contains_type_var(tv)),
      /*
      Type::Sum(hash_set) => hash_set
        .into_iter()
        .fold(false, |accum, x| accum || x.contains_type_var(tv)),
      */
      Type::Function(a, b) => {
        a.borrow().contains_type_var(tv) || b.borrow().contains_type_var(tv)
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
        member_types.iter_mut().for_each(|t| {
          t.borrow_mut().substitute(tv, type_);
        });
      },
      Type::Product(items) => items.iter_mut().for_each(|i| {
        i.borrow_mut().substitute(tv, type_);
      }),
      /*
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
      */
      Type::Function(a, b) => {
        a.borrow_mut().substitute(tv, type_);
        b.borrow_mut().substitute(tv, type_);
      },
      Type::Weak(_)
      | Type::Any
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

  pub fn product(a: TypeRef, b: TypeRef) -> TypeRef {
    let a_ = &*a.borrow();
    let b_ = &*b.borrow();
    if let Type::Product(v) = a_ {
      let mut new = v.clone();
      if let Type::Product(v2) = b_ {
        new.append(&mut v2.clone());
        Type::Product(new).to_ref()
      } else {
        new.push(b.clone());
        Type::Product(new).to_ref()
      }
    } else if let Type::Product(v) = b_ {
      let mut new = v.clone();
      if let Type::Product(v2) = a_ {
        new.append(&mut v2.clone());
        Type::Product(new).to_ref()
      } else {
        new.push(a.clone());
        Type::Product(new).to_ref()
      }
    } else {
      Type::Product(vec![a.clone(), b.clone()]).to_ref()
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
    use crate::ir::Type::*;
    use std::cmp::Ordering::*;
    Some(match (self, other) {
      (Any, _) | (_, Any) => return None,
      (TypeVariable(_), TypeVariable(_)) => Equal,
      (TypeVariable(_), _) => Greater,
      (_, TypeVariable(_)) => Less,
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
      //(t::Sum(v1), t::Sum(v2)) => v1 == v2,
      (t::TypeVariable(_), t::TypeVariable(_)) => true,
      (t::Weak(_), t::Weak(_)) => true,
      _ => false,
    }
  }
}

impl Eq for Type {
}

impl Into<TypeRef> for Type {
  fn into(self) -> TypeRef {
    self.to_ref()
  }
}

impl std::hash::Hash for Type {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Type::Struct {
        member_names,
        member_types,
      } => {
        member_names.hash(state);
        member_types.iter().for_each(|t| {
          t.borrow().hash(state);
        })
      },
      Type::Function(a, b) => {
        a.borrow().hash(state);
        b.borrow().hash(state);
      },
      Type::Type => "type".hash(state),
      Type::Any => {
        "any".hash(state);
      },
      Type::TypeVariable(id) => {
        "poly".hash(state);
        id.hash(state);
      },
      Type::Product(items) => {
        "tuple".hash(state);
        for item in items {
          item.borrow().hash(state);
        }
      },
      Type::Weak(_)
      | Type::Unit
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

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Any => write!(f, "any"),
      Type::_ClosureCapture => write!(f, "capture"),
      Type::Unit => write!(f, "unit"),
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
          .map(|(name, type_)| format!("{name}: {}", type_.borrow()))
          .collect::<Vec<_>>()
          .join(", ");
        write!(f, "{{ {fields} }}")
      },
      Type::Type => write!(f, "type"),
      Type::Function(a, b) => write!(f, "{} -> {}", a.borrow(), b.borrow()),
      Type::TypeVariable(id) => write!(f, "'{id}"),
      Type::Product(items) => write!(
        f,
        "({})",
        items
          .into_iter()
          .map(|i| format!("{}", i.borrow()))
          .collect::<Vec<_>>()
          .join(" * ")
      ),
      /*
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
      */
      Type::Weak(w) => {
        w.upgrade().unwrap();
        write!(f, "(cycle)")
      },
    }
  }
}
