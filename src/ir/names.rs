use super::*;
use std::collections::{
    HashMap,
    HashSet,
};

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
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
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

#[derive(Debug, Clone)]
pub struct CanonicalMap {
    pub(super) logger: FileLogger,
    module_name: String,
    map: HashMap<(String, NameSpace), Path>,
    globals: HashSet<(NameSpace, Path)>,
    history: Vec<((String, NameSpace), Option<Path>)>,
    salt: usize,
}

impl Path {
    pub const DELIMETER: &str = "::";

    pub fn new(
        major: impl Into<String>,
        minor: impl Into<String>,
    ) -> Self {
        Self {
            major: major.into(),
            minor: minor.into(),
        }
    }

    pub fn is_in_module(
        &self,
        module: &str,
    ) -> bool {
        self.major == module
    }
}

impl CanonicalMap {
    pub fn new(
        module_name: String,
        logger: FileLogger,
    ) -> Self {
        Self {
            module_name,
            logger,
            map: HashMap::new(),
            globals: HashSet::new(),
            history: vec![],
            salt: 0,
        }
    }
    pub fn unknown_name(
        &'_ mut self,
        namespace: NameSpace,
        Spanned { inner: name, span }: Spanned<impl std::fmt::Display>,
    ) -> LogBuilder<'_> {
        self.logger
            .error(format!("Unknown {namespace} {}", name))
            .primary("Used here", span)
            .note(format!("A {namespace} must be defined before it is used"))
    }
    pub fn get_name(
        &'_ mut self,
        Spanned { inner: name, span }: Spanned<String>,
        namespace: NameSpace,
    ) -> Result<'_, Path> {
        self.map
            .get(&(name.clone(), namespace))
            .cloned()
            .ok_or_else(|| self.unknown_name(namespace, Spanned { inner: name, span }))
    }
    pub fn get_global_name(
        &'_ mut self,
        Spanned { inner: name, span }: &Spanned<String>,
        namespace: NameSpace,
    ) -> Result<'_, Path> {
        let path = Path::new(self.module_name.clone(), name);
        if self.globals.contains(&(namespace, path.clone())) {
            Ok(path)
        } else {
            Err(self
                .logger
                .error(format!("Unknown {namespace} {path}"))
                .primary("Used here", *span)
                .note(format!("A {namespace} must be defined before it is used")))
        }
    }
    pub fn define_global(
        &'_ mut self,
        Spanned { inner: name, span }: Spanned<String>,
        namespace: NameSpace,
    ) -> Result<'_, Path> {
        let path = Path::new(self.module_name.clone(), name.clone());
        if !self.globals.insert((namespace, path.clone())) {
            Err(self
                .logger
                .error(format!("Multiple definitions of {namespace} {name}"))
                .primary("Definition here", span)
                .note("Global definitions must be unique"))
        } else {
            self.map.insert((name, namespace), path.clone());
            Ok(path)
        }
    }
    pub fn define_local(
        &mut self,
        Spanned { inner: name, .. }: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        let salt = self.salt;
        self.salt += 1;
        let path = Path::new(self.module_name.clone(), format!("{name}#{salt}"));
        let old = self.map.insert((name.clone(), namespace), path.clone());
        self.history.push(((name, namespace), old));
        path
    }
    pub fn define(
        &'_ mut self,
        name: Spanned<String>,
        namespace: NameSpace,
        is_global: bool,
    ) -> Result<'_, Path> {
        if is_global {
            self.define_global(name, namespace)
        } else {
            Ok(self.define_local(name, namespace))
        }
    }
    pub fn end_local_scopes(
        &mut self,
        n: usize,
    ) {
        for _ in 0..n {
            let (key, val) = self
                .history
                .pop()
                .unwrap_or_else(|| unreachable!("Variable scope failed to end"));
            match val {
                Some(val) => self.map.insert(key, val),
                None => self.map.remove(&key),
            };
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}::{}", self.major, self.minor)
    }
}

impl From<Path> for String {
    fn from(val: Path) -> Self {
        format!("{val}")
    }
}
