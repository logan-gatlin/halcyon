use super::{primitives::Primitive, SID};

pub fn mangle(input: &str) -> SID {
  format!("$B${input}")
}

// Nothing ever happens
pub fn nothing() -> SID {
  mangle("nothing")
}

pub fn integer() -> SID {
  Primitive::integer_ambiguous.mangle()
}

pub fn real() -> SID {
  Primitive::real_ambiguous.mangle()
}

pub fn all() -> Vec<SID> {
  let mut uids = Primitive::ALL.map(|p| p.mangle()).to_vec();
  uids.push(nothing());
  uids
}
