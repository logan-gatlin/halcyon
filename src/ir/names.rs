use std::collections::{
    HashMap,
    HashSet,
};

use crate::parse::ast;
use crate::{
    Span,
    Spanned,
    CORE_MODULE_NAME,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Path {
    pub major: String,
    pub minor: String,
}

impl TryFrom<ast::Path> for Path {
    type Error = ();

    fn try_from(value: ast::Path) -> Result<Self, Self::Error> {
        Ok(Path::new(
            value.qualifier().ok_or(())?.text(),
            value.name_text().ok_or(())?,
        ))
    }
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
    pub fn core(minor: impl Into<String>) -> Self {
        Self::new(CORE_MODULE_NAME, minor)
    }
}

impl std::fmt::Display for Path {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}{}{}", self.major, Self::DELIMETER, self.minor)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NameSpace {
    Constructor,
    Type,
    Term,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopedPath {
    pub path: Path,
    pub namespace: NameSpace,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopedString {
    pub string: String,
    pub namespace: NameSpace,
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

#[derive(Debug, Default)]
pub struct ModuleScope {
    /// This module's name
    module_name: String,
    /// Unique number added to local names
    salt: usize,
    /// Symbols defined globally in this module
    globals: HashSet<ScopedPath>,
    /// In-scope local names
    locals: HashMap<ScopedString, Path>,
    /// Symbols that could not be found in the current scope
    external: HashSet<ScopedPath>,
    /// Local variable introductions, used to rewind scope
    history_buffer: Vec<(ScopedString, Option<Path>)>,
    /// Function capture scopes
    capture_buffer: Vec<HashSet<Path>>,
    /// Definition(s) of items
    definitions: HashMap<ScopedPath, Vec<Span>>,
    /// Usages of items
    usages: HashMap<ScopedPath, Vec<Span>>,

    // State used for error reporting:
    undefined_usages: HashMap<ScopedPath, Vec<Span>>,
    multiple_definitions: HashSet<ScopedPath>,
}

pub struct LocalScope<'a> {
    module: &'a mut ModuleScope,
    history_size: usize,
}

pub struct LocalFunctionScope<S: Scope>(S);

pub trait Scope {
    fn define(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
    ) -> Path;
    fn query_string(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Path;
    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path;
    fn nest_scope(&mut self) -> impl Scope;
    fn nest_function_scope(&mut self) -> LocalFunctionScope<impl Scope>;
    fn _end_capture(&mut self) -> Box<[Path]>;
}

impl ModuleScope {
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            salt: Default::default(),
            globals: Default::default(),
            locals: Default::default(),
            external: Default::default(),
            history_buffer: Default::default(),
            capture_buffer: Default::default(),
            definitions: Default::default(),
            usages: Default::default(),
            undefined_usages: Default::default(),
            multiple_definitions: Default::default(),
        }
    }
}

impl Scope for ModuleScope {
    fn define(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        let path = Path::new(self.module_name.clone(), name.inner.clone());
        let scoped_path = ScopedPath {
            path: path.clone(),
            namespace,
        };
        self.globals.insert(scoped_path.clone());
        let defs = self.definitions.entry(scoped_path.clone()).or_default();
        if !defs.is_empty() {
            self.multiple_definitions.insert(scoped_path);
        }
        defs.push(name.span);
        path
    }
    fn nest_scope(&mut self) -> impl Scope {
        LocalScope {
            history_size: self.history_buffer.len(),
            module: self,
        }
    }
    fn query_string(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        match self
            .locals
            .get(&ScopedString {
                string: string.inner.clone(),
                namespace,
            })
            .cloned()
        {
            Some(path) => {
                self.capture_buffer.iter_mut().for_each(|b| {
                    b.insert(path.clone());
                });
                self.usages
                    .entry(ScopedPath {
                        path: path.clone(),
                        namespace,
                    })
                    .or_default()
                    .push(string.span);
                path
            }
            None => {
                let path = Path::new(&self.module_name, &string.inner);
                self.undefined_usages
                    .entry(ScopedPath {
                        path: path.clone(),
                        namespace,
                    })
                    .or_default()
                    .push(string.span);
                path
            }
        }
    }
    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path {
        self.usages
            .entry(ScopedPath {
                path: path.inner.clone(),
                namespace,
            })
            .or_default()
            .push(path.span);
        path.inner
    }
    fn nest_function_scope(&mut self) -> LocalFunctionScope<impl Scope> {
        self.capture_buffer.push(Default::default());
        LocalFunctionScope(self.nest_scope())
    }
    fn _end_capture(&mut self) -> Box<[Path]> {
        self.capture_buffer
            .pop()
            .unwrap_or_else(|| unreachable!())
            .into_iter()
            .collect()
    }
}

impl<'a> Scope for LocalScope<'a> {
    fn define(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        let path = Path::new(
            self.module.module_name.clone(),
            format!("{}#{}", name.inner, self.module.salt),
        );
        self.module.salt += 1;
        let scoped_string = ScopedString {
            string: name.inner,
            namespace,
        };
        let old_value = self
            .module
            .locals
            .insert(scoped_string.clone(), path.clone());
        self.module.history_buffer.push((scoped_string, old_value));
        self.module
            .definitions
            .entry(ScopedPath {
                path: path.clone(),
                namespace,
            })
            .or_default()
            .push(name.span);
        path
    }

    fn nest_scope(&mut self) -> impl Scope {
        LocalScope {
            history_size: self.module.history_buffer.len(),
            module: self.module,
        }
    }
    fn nest_function_scope(&mut self) -> LocalFunctionScope<impl Scope> {
        self.module.nest_function_scope()
    }
    fn query_string(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        self.module.query_string(string, namespace)
    }
    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path {
        self.module.query_path(path, namespace)
    }
    fn _end_capture(&mut self) -> Box<[Path]> {
        self.module._end_capture()
    }
}

impl<S: Scope> LocalFunctionScope<S> {
    pub fn into_captures(mut self) -> Box<[Path]> {
        self._end_capture()
    }
}

impl<S: Scope> Scope for LocalFunctionScope<S> {
    fn define(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        self.0.define(name, namespace)
    }

    fn nest_scope(&mut self) -> impl Scope {
        self.0.nest_scope()
    }

    fn query_string(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Path {
        self.0.query_string(string, namespace)
    }

    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path {
        self.0.query_path(path, namespace)
    }
    fn nest_function_scope(&mut self) -> LocalFunctionScope<impl Scope> {
        self.0.nest_function_scope()
    }
    fn _end_capture(&mut self) -> Box<[Path]> {
        self.0._end_capture()
    }
}

impl<'a> Drop for LocalScope<'a> {
    fn drop(&mut self) {
        while self.module.history_buffer.len() > self.history_size {
            let Some((key, old_value)) = self.module.history_buffer.pop() else {
                break;
            };
            match old_value {
                Some(path) => {
                    self.module.locals.insert(key, path);
                }
                None => {
                    self.module.locals.remove(&key);
                }
            }
        }
    }
}

impl Drop for ModuleScope {
    fn drop(&mut self) {
        assert!(self.history_buffer.is_empty(), "Local scopes failed to end");
    }
}
