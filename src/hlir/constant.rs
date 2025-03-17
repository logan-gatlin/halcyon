use super::*;

#[derive(Clone, Debug)]
pub enum ConstValue {
  Nothing,
  Never,
  Integer(i64),
  Real(f64),
  Boolean(bool),
  String {
    virtual_address: usize,
    length: usize,
  },
  Glyph(char),
  Function(Mangle),
  StructLiteral {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
  },
  Type(Type),
}

impl std::fmt::Display for ConstValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConstValue::Nothing => write!(f, "()"),
      ConstValue::Never => write!(f, "!"),
      ConstValue::String { .. } => write!(f, "<string>"),
      ConstValue::Function(_) => write!(f, "<function>"),
      ConstValue::StructLiteral { .. } => write!(f, "<struct>"),
      ConstValue::Type(val) => write!(f, "{val}"),
      ConstValue::Integer(val) => write!(f, "{val}"),
      ConstValue::Real(val) => write!(f, "{val}"),
      ConstValue::Glyph(val) => write!(f, "{val}"),
      ConstValue::Boolean(val) => write!(f, "{val}"),
    }
  }
}
