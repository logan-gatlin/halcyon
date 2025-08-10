use super::*;

#[derive(Debug, Clone)]
struct NameEvent {
    name: String,
    previous_value: Option<Path>,
}

#[derive(Debug, Clone, Default)]
struct NameSpace {
    module_name: Path,
    salt: usize,
    lookup_table: HashMap<String, Path>,
    state: Vec<NameEvent>,
}

impl NameSpace {
    pub fn new(module_name: Path) -> Self {
        Self {
            module_name,
            ..Default::default()
        }
    }
}

impl NameSpace {
    fn define_global(&mut self, name: &str) -> Result<Path> {
        assert!(self.state.len() == 0);
        let name = if name == "_" {
            let salt = self.salt;
            self.salt += 1;
            format!("_#{salt}")
        } else {
            name.to_string()
        };
        let path = self.module_name.child(&name);
        if self
            .lookup_table
            .insert(name.clone(), path.clone())
            .is_some()
        {
            return Err(lint_nospan(NameLint::NameRedefinition)).context(name);
        }
        Ok(path)
    }

    fn get(&self, name: &str) -> Result<Path> {
        self.lookup_table
            .get(name)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(name)
            .cloned()
    }

    fn define_local(&mut self, name: &str) -> Path {
        let salt = self.salt;
        self.salt += 1;
        let path = Path::from(format!("{name}#{salt}"));
        let ev = NameEvent {
            name: name.to_string(),
            previous_value: self.lookup_table.insert(name.to_string(), path.clone()),
        };
        self.state.push(ev);
        path
    }

    fn end_local_scope(&mut self) {
        let NameEvent {
            name,
            previous_value,
        } = self.state.pop().unwrap();
        if let Some(p) = previous_value {
            self.lookup_table.insert(name, p);
        } else {
            self.lookup_table.remove(&name);
        }
    }
}

macro_rules! inherit {
  ($fname:ident (&self, $($param:ident : $type:ty),*) -> $returns:ty) => {
    pub fn $fname(&self, $($param : $type,)*) -> $returns {
      self.ns.$fname($($param,)*)
    }
  };

  ($fname:ident (&mut self, $($param:ident : $type:ty),*) -> $returns:ty) => {
    pub fn $fname(&mut self, $($param : $type,)*) -> $returns {
      self.ns.$fname($($param,)*)
    }
  };

  ($first:ident $($i:ident)+) => {
    inherit!($first);
    $(inherit!($i);)*
  };

  (get) => {
    inherit!(get(&self, name: &str) -> Result<Path>);
  };

  (define_global) => {
    inherit!(define_global (&mut self, name: &str) -> Result<Path>);
  };

  (define_local) => {
    inherit!(define_local (&mut self, name: &str) -> Path);
  };

  (end_local_scope) => {
    inherit!(end_local_scope (&mut self,) -> ());
  };
}

#[derive(Debug, Clone, Default)]
pub struct ValueNameSpace {
    ns: NameSpace,
    capture_list: Vec<Vec<Path>>,
    depth_table: HashMap<Path, usize>,
    import_types: HashMap<Path, TypeRef>,
}

impl ValueNameSpace {
    inherit!(define_global);

    pub fn new(module_name: Path) -> Self {
        Self {
            ns: NameSpace::new(module_name),
            ..Default::default()
        }
    }

    pub fn begin_capture(&mut self) {
        self.capture_list.push(vec![]);
    }

    pub fn define_local(&mut self, name: &str) -> Path {
        let mangle = self.ns.define_local(name);
        self.depth_table
            .insert(mangle.clone(), self.capture_list.len());
        mangle
    }

    pub fn end_local_scope(&mut self) {
        self.ns.end_local_scope();
    }

    pub fn end_capture(&mut self) -> Vec<Path> {
        self.capture_list.pop().unwrap()
    }

    pub fn get(&mut self, name: &str) -> Result<Path> {
        let mangle = self.ns.get(name)?;
        match self.depth_table.get(&mangle) {
            Some(depth) => {
                for capture in (*depth)..(self.capture_list.len()) {
                    self.capture_list[capture].push(mangle.clone());
                }
            }
            None => {}
        }
        Ok(mangle)
    }

    pub fn import_module(&mut self, items: impl IntoIterator<Item = (Path, TypeRef)>) {
        for (name, type_) in items {
            self.import_types.insert(name, type_);
        }
    }

    pub fn get_import_type(&self, path: &Path) -> Result<TypeRef> {
        self.import_types
            .get(path)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(path)
            .cloned()
    }
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub variant: usize,
    pub in_type: TypeRef,
    pub out_type: TypeRef,
}

#[derive(Debug, Clone, Default)]
pub struct ConstructorNameSpace {
    ns: NameSpace,
    constructor_map: HashMap<Path, Constructor>,
}

impl ConstructorNameSpace {
    pub fn new(module_name: Path) -> Self {
        Self {
            ns: NameSpace::new(module_name),
            ..Default::default()
        }
    }

    pub fn get(&self, name: &str) -> Result<Constructor> {
        let path = self.ns.get(name)?;
        let cons = self.constructor_map.get(&path).unwrap();
        Ok(cons.clone())
    }

    pub fn define(&mut self, path: &str, cons: Constructor) -> Result<()> {
        let path = self.ns.define_global(path)?;
        self.constructor_map.insert(path, cons);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeNameSpace {
    ns: NameSpace,
    type_map: HashMap<Path, TypeRef>,
}

#[allow(dead_code)]
impl TypeNameSpace {
    inherit!(get end_local_scope);

    pub fn new(module_name: Path) -> Self {
        Self {
            ns: NameSpace::new(module_name),
            ..Default::default()
        }
    }

    pub fn get_type(&self, name: &String) -> Result<TypeRef> {
        let mangle = self.get(name)?;
        Ok(self.type_map.get(&mangle).unwrap().clone())
    }

    pub fn update_type(&mut self, mangle: &Path, new_t: TypeRef) {
        *self.type_map.get_mut(mangle).unwrap() = new_t;
    }

    pub fn define_local(&mut self, name: &str, type_: TypeRef) -> Path {
        let mangle = self.ns.define_local(&name);
        self.type_map.insert(mangle.clone(), type_);
        mangle
    }

    pub fn define_global(&mut self, name: &str, type_: TypeRef) -> Result<Path> {
        let mangle = self.ns.define_global(name)?;
        self.type_map.insert(mangle.clone(), type_);
        Ok(mangle)
    }

    pub fn import_module(&mut self, items: impl IntoIterator<Item = (Path, TypeRef)>) {
        for (name, type_) in items {
            self.type_map.insert(name, type_);
        }
    }

    pub fn get_import_type(&self, path: &Path) -> Result<TypeRef> {
        self.type_map
            .get(path)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(path)
            .cloned()
    }

    pub fn to_universe(self) -> HashMap<Path, TypeRef> {
        self.type_map
    }
}
