use std::collections::HashSet;

use super::*;

#[derive(Debug, Clone)]
struct NameEvent {
  name: String,
  previous_value: Option<Mangle>,
}

#[derive(Debug, Clone, Default)]
pub struct Ns {
  module_name: String,
  globals: HashSet<Mangle>,
  lookup_table: HashMap<String, Mangle>,
  state: Vec<NameEvent>,
}

impl Ns {
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

#[derive(Debug, Clone, Default)]
pub struct NameSpace {
  module_name: String,
  pub module_table: HashMap<String, ModuleInterface>,
  globals: HashMap<String, Mangle>,
  value_name_table: HashMap<String, (Mangle, FunctionDepth)>,
  type_name_table: HashMap<String, Mangle>,
  pub type_table: HashMap<Mangle, TypeRef>,
  pub salt: usize,
  scopes: Vec<Scope>,
  captures: Vec<Vec<Mangle>>,
}

impl NameSpace {
  pub fn new(module_name: String) -> Self {
    let mut this = Self::default();
    this.module_name = module_name;
    Type::primitives().into_iter().for_each(|(prim, name)| {
      let mangle = mangle_builtin(prim.borrow());
      this
        .type_name_table
        .insert(name.to_string(), mangle.clone());
      this.type_table.insert(mangle, prim);
    });
    this
  }

  pub fn resolve_module_value_path(
    &self,
    path: &[String],
  ) -> Result<(Mangle, TypeRef)> {
    self.resolve_module_path(path, false)
  }

  pub fn resolve_module_type_path(&self, path: &[String]) -> Result<TypeRef> {
    self.resolve_module_path(path, true).map(|(_, t)| t)
  }

  fn resolve_module_path(
    &self,
    path: &[String],
    is_type: bool,
  ) -> Result<(Mangle, TypeRef)> {
    let mangle = path.join(":");
    match path {
      [] | [_] => unreachable!(),
      [a, b] => {
        let module = self
          .module_table
          .get(a)
          .ok_or(lint_nospan(NameLint::NotImported))
          .context(a)?;
        if is_type {
          &module.types
        } else {
          &module.values
        }
        .get(&mangle)
        .ok_or(lint_nospan(NameLint::UndefinedName))
        .context(b)
        .cloned()
        .map(|t| (mangle, t))
      },
      [.., a] => Err(lint_nospan(NameLint::UndefinedName)).context(a),
    }
  }

  pub fn new_global(&mut self, name: String) -> Mangle {
    let mangle = mangle_global(&[&self.module_name], name.clone());
    self
      .value_name_table
      .insert(name.clone(), (mangle.clone(), 0));
    mangle
  }

  pub fn push_type(&mut self, name: String, type_: TypeRef) -> Mangle {
    let mangle = mangle_global(&[&self.module_name], name.clone());
    self.type_name_table.insert(name, mangle.clone());
    self.type_table.insert(mangle.clone(), type_);
    mangle
  }

  pub fn push_value(&mut self, name: String) -> Mangle {
    let mangle = mangle_name(
      vec![self.module_name.clone(), name.clone()],
      &format!("{}", self.salt),
    );
    self.salt += 1;
    self.scopes.push(Scope::Value {
      clean: name.clone(),
      old: self
        .value_name_table
        .insert(name.clone(), (mangle.clone(), self.captures.len() + 1)),
    });
    mangle
  }

  pub fn update_type(&mut self, mangle: Mangle, type_: TypeRef) {
    self.type_table.insert(mangle, type_);
  }

  pub fn pop(&mut self) {
    match self.scopes.pop().unwrap() {
      Scope::Value { clean, old } => match old {
        Some(old) => {
          self.value_name_table.insert(clean, old);
        },
        None => {
          self.value_name_table.remove(&clean);
        },
      },
      Scope::Type { clean, old } => match old {
        Some(old) => {
          self.type_name_table.insert(clean, old);
        },
        None => {
          self.type_name_table.remove(&clean);
        },
      },
    };
  }

  pub fn get_value(&mut self, name: &String) -> Option<Mangle> {
    match self.value_name_table.get(name) {
      Some((mangle, depth)) if *depth != 0 => {
        for capture in (*depth - 1)..(self.captures.len()) {
          self.captures[capture].push(mangle.clone());
        }
        Some(mangle.clone())
      },
      Some((mangle, _)) => Some(mangle.clone()),
      None => self.globals.get(name).cloned(),
    }
  }

  pub fn get_type(&mut self, name: &String) -> Option<TypeRef> {
    let mangle = self.type_name_table.get(name)?;
    Some(self.type_table.get(mangle).unwrap().clone())
  }

  pub fn new_func(&mut self) {
    self.captures.push(vec![]);
  }

  pub fn end_func(&mut self) -> Vec<Mangle> {
    self.captures.pop().unwrap()
  }
}
