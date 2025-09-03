use std::collections::HashSet;

use super::*;

#[derive(Debug, Clone, Default)]
pub struct ModuleNameSpace {
    module_name: Path,
    salt: usize,
    // Values
    value_lookup: HashMap<String, Path>,
    value_history: Vec<NameEvent>,
    capture_list: Vec<Vec<Path>>,
    values_available: HashSet<Path>,
    imported_value_types: HashMap<Path, Type>,
    // Constructors
    constructor_lookup: HashMap<String, Path>,
    constructors_available: HashMap<Path, Constructor>,
    // Types
    type_lookup: HashMap<String, Path>,
    type_history: Vec<NameEvent>,
    types_available: HashSet<Path>,
    local_types: HashMap<Path, Type>,
}

impl ModuleNameSpace {
    pub fn new(module_name: impl Into<Path>) -> Self {
        Self {
            module_name: module_name.into(),
            ..Default::default()
        }
    }

    pub fn import_interface(&mut self, interface: &ModuleInterface) -> Result<()> {
        todo!()
    }
    // Values
    pub fn new_global_value(&mut self, name: &str) -> Result<Path> {
        todo!()
    }

    pub fn new_local_value(&mut self, name: &str, is_parameter: bool) -> Path {
        todo!()
    }

    pub fn get_value(&mut self, name: &str) -> Result<Path> {
        todo!()
    }

    pub fn get_imported_value_type(&self, path: &Path) -> Result<Type> {
        todo!()
    }

    pub fn begin_capture(&mut self) {
        self.capture_list.push(vec![]);
    }

    pub fn end_capture(&mut self) -> Vec<Path> {
        self.capture_list
            .pop()
            .expect("Popped an empty capture list")
    }

    pub fn end_value_scopes(&mut self, num: usize) {
        for _ in 0..num {
            let NameEvent {
                name,
                previous_value,
            } = self.value_history.pop().unwrap();
            if let Some(p) = previous_value {
                self.value_lookup.insert(name, p);
            } else {
                self.value_lookup.remove(&name);
            }
        }
    }

    // Constructors
    pub fn new_constructor(&mut self, name: &str, constructor: Constructor) -> Result<Path> {
        todo!()
    }

    pub fn get_constructor(&self, name: &str) -> Result<Constructor> {
        todo!()
    }

    pub fn get_constructor_exact(&self, path: &Path) -> Result<Constructor> {
        todo!()
    }

    // Types
    pub fn new_global_type(&mut self, name: &str) -> Result<Path> {
        todo!()
    }

    pub fn new_local_type(&mut self, name: &str, type_: Type) {
        todo!()
    }

    pub fn get_type(&self, name: &str) -> Result<Type> {
        todo!()
    }

    pub fn get_type_exact(&self, path: &Path) -> Result<Type> {
        todo!()
    }

    pub fn get_type_path(&self, name: &str) -> Result<Path> {
        todo!()
    }

    pub fn get_type_path_exact(&self, path: &Path) -> Result<Path> {
        todo!()
    }

    pub fn end_type_scopes(&mut self, num: usize) {
        for _ in 0..num {
            let NameEvent {
                name,
                previous_value,
            } = self.type_history.pop().unwrap();
            if let Some(p) = previous_value {
                self.type_lookup.insert(name, p);
            } else {
                self.type_lookup.remove(&name);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct NameEvent {
    name: String,
    previous_value: Option<Path>,
}

/*
#[derive(Debug, Clone, Default)]
pub struct NameSpace<T: Clone> {
    module_name: Path,
    salt: usize,
    lookup_table: HashMap<String, Path>,
    value_table: HashMap<Path, T>,
    state: Vec<NameEvent>,
}

impl<T: Clone> NameSpace<T> {
    pub fn new(module_name: Path) -> Self {
        Self {
            module_name,
            salt: 0,
            lookup_table: HashMap::new(),
            value_table: HashMap::new(),
            state: vec![],
        }
    }

    fn define_import(&mut self, name: Path, value: T) -> Result<()> {
        if self.value_table.insert(name.clone(), value).is_some() {
            Err(lint_nospan(NameLint::NameRedefinition)).context(name)
        } else {
            Ok(())
        }
    }

    pub fn define_global(&mut self, name: &str, value: T) -> Result<Path> {
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
        self.value_table.insert(path.clone(), value);
        Ok(path)
    }

    pub fn get(&self, name: &str) -> Result<T> {
        let path = self
            .lookup_table
            .get(name)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(name)?;
        Ok(self.value_table.get(path).unwrap().clone())
    }

    pub fn get_path(&self, name: &str) -> Result<Path> {
        self.lookup_table
            .get(name)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(name)
            .cloned()
    }

    pub fn get_exact(&self, path: &Path) -> Result<T> {
        self.value_table
            .get(path)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(path)
            .cloned()
    }

    pub fn update(&mut self, path: &Path, new_value: T) {
        *self.value_table.get_mut(path).unwrap() = new_value;
    }

    pub fn define_local(&mut self, name: &str, value: T) -> Path {
        let salt = self.salt;
        self.salt += 1;
        let path = Path::from(format!("{name}#{salt}"));
        self.value_table.insert(path.clone(), value);
        let ev = NameEvent {
            name: name.to_string(),
            previous_value: self.lookup_table.insert(name.to_string(), path.clone()),
        };
        self.state.push(ev);
        path
    }

    pub fn end_local_scopes(&mut self, num: usize) {
        for _ in 0..num {
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
}

#[derive(Debug, Clone, Default)]
pub struct NameInfo {
    /// How many `fn`s deep
    depth: usize,
    is_parameter: bool,
}

#[derive(Debug, Clone)]
pub struct ModuleNameSpace {
    // Type ns
    pub types: NameSpace<Type>,
    // Value ns
    pub values: NameSpace<NameInfo>,
    value_capture_list: Vec<Vec<Path>>,
    value_import_types: HashMap<Path, Type>,
    // Constructor ns
    pub constructors: NameSpace<Constructor>,
}

impl ModuleNameSpace {
    pub fn new(module_name: Path) -> Self {
        Self {
            types: NameSpace::new(module_name.clone()),
            values: NameSpace::new(module_name.clone()),
            value_capture_list: Default::default(),
            value_import_types: Default::default(),
            constructors: NameSpace::new(module_name),
        }
    }

    pub fn define_local_value(&mut self, name: &str, is_parameter: bool) -> Path {
        self.values.define_local(
            name,
            NameInfo {
                depth: self.value_capture_list.len(),
                is_parameter,
            },
        )
    }

    pub fn define_global_value(&mut self, name: &str) -> Result<Path> {
        self.values.define_global(name, NameInfo::default())
    }

    pub fn define_type(&mut self, name: &str, type_: Type) -> Result<Path> {
        let path = self.types.define_global(name, Type::Any)?;
        Universe::get().new_named_type(path.clone(), type_);
        Ok(path)
    }

    pub fn define_temporary_type(&mut self, name: &str, parameters: usize) -> Result<Path> {
        let path = self.types.define_global(name, Type::Any)?;
        // TODO hack, but probably ok
        Universe::get().new_named_type(
            path.clone(),
            Type::Product((0..parameters).map(Type::Variable).collect()),
        );
        Ok(path)
    }

    pub fn update_type(&mut self, name: Path, type_: Type) {
        Universe::get().modify_named_type(name, type_);
    }

    pub fn get_value(&mut self, name: &str) -> Result<Path> {
        let mangle = self.values.get_path(name)?;
        let name_info = self.values.get(name)?;
        let current_depth = self.value_capture_list.len();
        for capture in name_info.depth..current_depth {
            self.value_capture_list[capture].push(mangle.clone());
        }
        /*
        if !name_info.is_parameter && current_depth <= name_info.depth {
            return Err(lint_nospan(NameLint::CyclicalDefinition)).context(name);
        }
        */
        Ok(mangle)
    }

    pub fn begin_capture(&mut self) {
        self.value_capture_list.push(vec![]);
    }

    pub fn end_capture(&mut self) -> Vec<Path> {
        self.value_capture_list.pop().unwrap()
    }

    pub fn get_import_type(&self, path: &Path) -> Result<Type> {
        self.value_import_types
            .get(path)
            .ok_or(lint_nospan(NameLint::UndefinedName))
            .context(path)
            .cloned()
    }

    pub fn import_module(&mut self, interface: &ModuleInterface) -> Result<()> {
        for (name, type_) in interface.values.clone() {
            self.values
                .define_import(name.clone(), NameInfo::default())?;
            self.value_import_types.insert(name, type_);
        }
        for (name, cons) in interface.constructors.clone() {
            self.constructors.define_import(name, cons)?;
        }
        for name in interface.types.clone() {
            self.types.define_import(name, Type::Any)?;
        }
        Ok(())
    }
}
*/

#[derive(Debug, Clone, sx::SXRepr)]
pub struct Constructor {
    pub variant: usize,
    pub in_type: Type,
    pub out_type: Type,
}

impl Constructor {
    pub fn function_type(&self) -> Type {
        Type::func(self.in_type.clone(), self.out_type.clone())
    }
}
