mod build_hlir;
pub mod builtins;
pub mod constant;
pub mod hlir_module;
pub mod printing;
pub mod types;

use std::collections::HashMap;

use crate::{lint::*, memory::*, operator::*, parse::*};

pub use build_hlir::*;
pub use builtins::*;
pub use constant::*;
pub use hlir_module::*;
pub use types::*;

pub type IrPtr = usize;
pub type Mangle = String;

pub fn build_hlir(expr: Expression) -> Result<HlIrModule> {
  let canon = Canonizer::new();
  canon.canonize_expr(expr)
}

/// Name mangle syntax:
/// mangle ::= "$" path salt
/// path ::= {path-element}*
/// path-element ::= length ident
/// ident ::= _a-zA-Z {_a-zA-Z0-9}*
/// length ::= {0-9}+
/// salt ::= {a-zA-Z}*
pub fn mangle_name(path: Vec<String>, salt: &str) -> Mangle {
  let mut buf: Vec<u8> = vec![];
  for p in path {
    let puny = punycode::encode(&p).unwrap();
    let bytes = format!("{}{puny}", puny.len());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}

/// Builtin mangle syntax:
/// "$" {ident}
pub fn mangle_builtin(name: impl std::fmt::Display) -> Mangle {
  format!("_{name}")
}
