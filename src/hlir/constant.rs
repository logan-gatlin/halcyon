use super::*;

#[derive(Clone, Debug)]
pub enum ConstValue {
  Unit,
  Integer(i64),
  Real(f64),
  Boolean(bool),
  String(String),
  Glyph(char),
  Function {
    func_index: u32,
    type_index: u32,
  },
  StructLiteral {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
    type_id: u32,
  },
  Tuple {
    members: Vec<ConstValue>,
    type_id: u32,
  },
  Type(Type),
}

impl From<Type> for ConstValue {
  fn from(value: Type) -> Self {
    Self::Type(value)
  }
}

impl std::fmt::Display for ConstValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConstValue::Unit => write!(f, "()"),
      ConstValue::String { .. } => write!(f, "<string>"),
      ConstValue::Function { func_index, .. } => write!(f, "{func_index}"),
      ConstValue::StructLiteral { .. } => write!(f, "<struct>"),
      ConstValue::Type(val) => write!(f, "{val}"),
      ConstValue::Tuple { members, .. } => write!(
        f,
        "({})",
        members
          .iter()
          .map(|i| format!("{i}"))
          .collect::<Vec<_>>()
          .join(", ")
      ),
      ConstValue::Integer(val) => write!(f, "{val}"),
      ConstValue::Real(val) => write!(f, "{val}"),
      ConstValue::Glyph(val) => write!(f, "{val}"),
      ConstValue::Boolean(val) => write!(f, "{val}"),
    }
  }
}
