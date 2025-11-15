use std::collections::HashSet;

use crate::{LResult, err};

use super::*;

fn undefined_name(name: impl std::fmt::Display) -> Log {
    err(format!("The symbol {name} is not defined"))
}

fn redefined_name(name: impl std::fmt::Display) -> Log {
    err(format!("The symbol {name} is defined more than once"))
}

#[derive(Debug, Clone, Copy)]
pub struct NameInfo {
    depth: usize,
    is_finalized: bool,
    is_global: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleNameSpace {
    module_name: Path,
    salt: usize,
    // Values
    value_lookup: HashMap<String, Path>,
    value_history: Vec<NameEvent>,
    local_value_info: HashMap<Path, NameInfo>,
    capture_list: Vec<Vec<Path>>,
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

    pub fn import_interface(&mut self, interface: &ModuleInterface) -> LResult<()> {
        for (path, type_) in interface.values.clone() {
            self.imported_value_types.insert(path, type_);
        }
        for (path, constructor) in interface.constructors.clone() {
            self.constructors_available.insert(path, constructor);
        }
        for path in interface.types.clone() {
            self.types_available.insert(path);
        }
        Ok(())
    }
    // Values
    pub fn new_global_value(&mut self, name: &str) -> LResult<Path> {
        let name = name.to_string();
        let path = self.module_name.child(&name);
        if self
            .value_lookup
            .insert(name.clone(), path.clone())
            .is_some()
            || self.imported_value_types.contains_key(&path)
        {
            return Err(err(format!("The name {name} is defined multiple times")));
        }
        self.local_value_info.insert(
            path.clone(),
            NameInfo {
                depth: 0,
                is_finalized: false,
                is_global: true,
            },
        );
        Ok(path)
    }

    pub fn new_local_value(&mut self, name: &str, is_parameter: bool) -> Path {
        let salt = self.salt;
        self.salt += 1;
        let path = Path::from(format!("{name}#{salt}"));
        self.value_history.push(NameEvent {
            name: name.to_string(),
            previous_value: self.value_lookup.insert(name.to_string(), path.clone()),
        });
        self.local_value_info.insert(
            path.clone(),
            NameInfo {
                depth: self.capture_list.len(),
                is_finalized: is_parameter,
                is_global: false,
            },
        );
        path
    }

    pub fn get_value(&mut self, name: &str) -> LResult<Path> {
        let path = self
            .value_lookup
            .get(name)
            .ok_or(undefined_name(name))?
            .clone();
        if let Some(NameInfo {
            depth,
            is_finalized,
            is_global,
        }) = self.local_value_info.get(&path).copied()
            && !is_global
        {
            let current_depth = self.capture_list.len();
            if !is_finalized && current_depth <= depth {
                return Err(err(format!("The definition of {name} is cyclical")));
            }

            for capture in depth..current_depth {
                if !self.capture_list[capture].contains(&path) {
                    self.capture_list[capture].push(path.clone());
                }
            }
        }
        Ok(path)
    }

    pub fn finalize_value(&mut self, path: &Path) {
        self.local_value_info.get_mut(path).unwrap().is_finalized = true;
    }

    pub fn get_imported_value_type(&self, path: &Path) -> LResult<Type> {
        self.imported_value_types
            .get(path)
            .ok_or_else(|| undefined_name(path))
            .cloned()
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
    pub fn new_constructor(&mut self, name: &str, constructor: Constructor) -> LResult<Path> {
        let path = self.module_name.child(name);
        if self
            .constructor_lookup
            .insert(name.to_string(), path.clone())
            .is_some()
        {
            return Err(redefined_name(name));
        }
        self.constructors_available
            .insert(path.clone(), constructor);
        Ok(path)
    }

    pub fn get_constructor(&self, name: &str) -> LResult<Constructor> {
        let path = self
            .constructor_lookup
            .get(name)
            .ok_or(undefined_name(name))?;
        Ok(self.constructors_available.get(path).unwrap().clone())
    }

    pub fn get_constructor_exact(&self, path: &Path) -> LResult<Constructor> {
        self.constructors_available
            .get(path)
            .ok_or(undefined_name(path))
            .cloned()
    }

    // Types
    pub fn new_global_type(&mut self, name: &str) -> LResult<Path> {
        let path = self.module_name.child(name);
        if self
            .type_lookup
            .insert(name.to_string(), path.clone())
            .is_some()
        {
            return Err(redefined_name(name));
        }
        self.types_available.insert(path.clone());
        Ok(path)
    }

    pub fn new_local_type(&mut self, name: &str, type_: Type) {
        let salt = self.salt;
        self.salt += 1;
        let path = Path::from(format!("{name}#{salt}"));
        self.type_history.push(NameEvent {
            name: name.to_string(),
            previous_value: self.type_lookup.insert(name.to_string(), path.clone()),
        });
        self.local_types.insert(path, type_);
    }

    pub fn get_type(&self, name: &str) -> LResult<Type> {
        let path = self.type_lookup.get(name).ok_or(undefined_name(name))?;
        match self.local_types.get(path).cloned() {
            Some(t) => Ok(t),
            None => {
                let at = Universe::get().get_named_type(path);
                at.clone().instantiate(&[])?;
                Ok(Type::Instantiation(path.clone(), vec![]))
            }
        }
    }

    pub fn get_type_exact(&self, path: &Path) -> LResult<Type> {
        self.types_available.get(path).ok_or(undefined_name(path))?;
        match self.local_types.get(path).cloned() {
            Some(t) => Ok(t),
            None => Universe::get().get_named_type(path).instantiate(&[]),
        }
    }

    pub fn get_type_path(&self, name: &str) -> LResult<Path> {
        self.type_lookup
            .get(name)
            .ok_or(undefined_name(name))
            .cloned()
    }

    pub fn get_type_path_exact(&self, path: &Path) -> LResult<Path> {
        self.types_available
            .get(path)
            .ok_or(undefined_name(path))
            .cloned()
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
