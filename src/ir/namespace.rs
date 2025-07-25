use std::collections::HashSet;

use super::*;

#[derive(Debug, Clone)]
struct NameEvent {
  name: String,
  previous_value: Option<Mangle>,
}

#[derive(Debug, Clone, Default)]
pub struct NameSpace {
  module_name: String,
  globals: HashSet<Mangle>,
  lookup_table: HashMap<String, Mangle>,
  state: Vec<NameEvent>,
}

impl NameSpace {
  pub fn new(module_name: String) -> Self {
    let mut this = Self::default();
    this.module_name = module_name;
    this
  }

  pub fn define_global(&mut self, name: String) -> Result<Mangle> {
    assert!(self.state.len() == 0);
    let mangle = mangle_global(&[&self.module_name], &name);
    if !self.globals.insert(mangle.clone()) {
      return Err(lint_nospan(NameLint::NameRedefinition)).context(&name);
    }
    self.lookup_table.insert(name, mangle.clone());
    Ok(mangle)
  }

  pub fn get(&self, name: &String) -> Result<Mangle> {
    self
      .lookup_table
      .get(name)
      .ok_or(lint_nospan(NameLint::UndefinedName))
      .context(name)
      .cloned()
  }

  pub fn begin_local_scope(&mut self, name: String, salt: usize) -> Mangle {
    let mangle =
      mangle_name(vec![self.module_name.clone()], &format!("{salt}"));
    let ev = NameEvent {
      name: name.clone(),
      previous_value: self.lookup_table.insert(name, mangle.clone()),
    };
    self.state.push(ev);
    mangle
  }

  pub fn end_local_scope(&mut self) {
    let NameEvent {
      name,
      previous_value,
    } = self.state.pop().unwrap();
    if let Some(p) = previous_value {
      self.lookup_table.insert(name, p);
    }
  }
}
