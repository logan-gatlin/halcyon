use super::{UID, primitives::Primitive};

pub fn mangle(input: &str) -> UID {
  format!("$${input}")
}

// Nothing ever happens
pub fn nothing() -> UID {
  mangle("nothing")
}

pub fn integer() -> UID {
  Primitive::integer_ambiguous.mangle()
}

pub fn real() -> UID {
  Primitive::real_ambiguous.mangle()
}

pub fn all() -> Vec<UID> {
  let mut uids = Primitive::ALL.map(|p| p.mangle()).to_vec();
  uids.push(nothing());
  uids
}
