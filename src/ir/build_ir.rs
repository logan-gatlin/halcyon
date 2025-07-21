use std::collections::HashMap;

use super::*;
use crate::{builtin::Builtin, lint::*, parse::*};

#[derive(Debug, Clone)]
struct Scope {
  clean: String,
  old: Option<(Mangle, FunctionDepth)>,
}

type FunctionDepth = usize;

#[derive(Debug, Clone)]
pub struct NameSpace {
  name_table: HashMap<String, (Mangle, FunctionDepth)>,
  builtins: HashMap<String, Mangle>,
  salt: usize,
  scopes: Vec<Scope>,
  captures: Vec<Vec<Mangle>>,
}

impl NameSpace {
  pub fn new() -> Self {
    let mut builtins = HashMap::new();
    Type::primitives().into_iter().for_each(|(_, name)| {
      builtins.insert(name.to_string(), mangle_builtin(name));
    });
    Builtin::ALL.into_iter().for_each(|bt| {
      builtins.insert(bt.to_string(), bt.get_mangle());
    });
    Self {
      name_table: HashMap::new(),
      builtins,
      salt: 0,
      scopes: vec![],
      captures: vec![],
    }
  }

  pub fn push(&mut self, name: String) -> Mangle {
    let mangle = mangle_name(vec![name.clone()], &format!("{}", self.salt));
    self.salt += 1;
    self.scopes.push(Scope {
      clean: name.clone(),
      old: self
        .name_table
        .insert(name.clone(), (mangle.clone(), self.captures.len())),
    });
    mangle
  }

  pub fn pop(&mut self) {
    let Scope { clean, old, .. } = self.scopes.pop().unwrap();
    match old {
      Some(old) => self.name_table.insert(clean, old),
      None => self.name_table.remove(&clean),
    };
  }

  pub fn get(&mut self, name: &String) -> Option<Mangle> {
    match self.name_table.get(name) {
      Some((mangle, depth)) => {
        for capture in (*depth)..(self.captures.len()) {
          self.captures[capture].push(mangle.clone());
        }
        Some(mangle.clone())
      },
      None => self.builtins.get(name).cloned(),
    }
  }

  pub fn new_func(&mut self) {
    self.captures.push(vec![]);
  }

  pub fn end_func(&mut self) -> Vec<Mangle> {
    self.captures.pop().unwrap()
  }
}
