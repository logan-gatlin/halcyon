use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use crate::{LResult, Log, err};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Path {
    pub major: String,
    pub minor: String,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NameSpace {
    Constructor,
    Type,
    Term,
}

impl std::fmt::Display for NameSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NameSpace::Constructor => "constructor",
                NameSpace::Type => "type",
                NameSpace::Term => "term",
            }
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CanonicalMap {
    module_name: String,
    map: HashMap<(String, NameSpace), Path>,
    globals: HashSet<(NameSpace, Path)>,
    history: Vec<((String, NameSpace), Option<Path>)>,
    salt: usize,
}

impl Path {
    const DELIMETER: &str = "::";

    pub fn new(major: impl Into<String>, minor: impl Into<String>) -> Self {
        Self {
            major: major.into(),
            minor: minor.into(),
        }
    }

    pub fn is_in_module(&self, module: &str) -> bool {
        self.major == module
    }
}

impl CanonicalMap {
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            ..Default::default()
        }
    }

    pub fn get(&self, name: String, namespace: NameSpace) -> LResult<&Path> {
        self.map
            .get(&(name.clone(), namespace))
            .ok_or_else(|| err(format!("There is no {namespace} {name} in this module. A {namespace} must be defined before it is used.")))
    }

    pub fn define_global(&mut self, name: String, namespace: NameSpace) -> LResult<Path> {
        let path = Path::new(self.module_name.clone(), name.clone());
        if !self.globals.insert((namespace, path.clone())) {
            return Err(err(format!(
                "There is already a {namespace} called {name}. Once a {namespace} is defined, it cannot be changed."
            )));
        }
        self.map.insert((name, namespace), path.clone());
        Ok(path)
    }

    pub fn define_local(&mut self, name: String, namespace: NameSpace) -> Path {
        let salt = self.salt;
        self.salt += 1;
        let path = Path::new(self.module_name.clone(), format!("{name}#{salt}"));
        let old = self.map.insert((name.clone(), namespace), path.clone());
        self.history.push(((name, namespace), old));
        path
    }

    pub fn define(&mut self, name: String, namespace: NameSpace, is_global: bool) -> LResult<Path> {
        if is_global {
            self.define_global(name, namespace)
        } else {
            Ok(self.define_local(name, namespace))
        }
    }

    pub fn end_local_scopes(&mut self, n: usize) {
        for _ in 0..n {
            let (key, val) = self.history.pop().unwrap();
            match val {
                Some(val) => self.map.insert(key, val),
                None => self.map.remove(&key),
            };
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.major, self.minor)
    }
}

impl FromStr for Path {
    type Err = Log;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.split(Self::DELIMETER);
        let (Some(first), Some(last), None) = (it.next(), it.next(), it.next()) else {
            return Err(err(format!("The string `{s}` is not a valid path")));
        };
        Ok(Self {
            major: first.into(),
            minor: last.into(),
        })
    }
}

impl Into<String> for Path {
    fn into(self) -> String {
        format!("{self}")
    }
}
