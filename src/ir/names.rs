use std::collections::{
    HashMap,
    HashSet,
};

use crate::parse::ast;
use crate::{
    CORE_MODULE_NAME,
    FileLogger,
    Span,
    Spanned,
    WithContext,
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
            value.qualifier().ok_or(())?,
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
    Wasm,
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

#[derive(Debug, Clone)]
struct LocalBinding {
    path: Path,
    function_depth: usize,
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
                NameSpace::Wasm => "register",
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
    locals: HashMap<ScopedString, LocalBinding>,
    /// Symbols that could not be found in the current scope
    #[allow(dead_code)]
    external: HashSet<ScopedPath>,
    /// Local variable introductions, used to rewind scope
    history_buffer: Vec<(ScopedString, Option<LocalBinding>)>,
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

    pub fn report_name_resolution_errors(
        &self,
        logger: &mut FileLogger,
    ) {
        self.report_multiple_definitions(logger);
        self.report_undefined_usages(logger);
    }

    pub fn predefine(
        &mut self,
        path: Path,
        namespace: NameSpace,
    ) {
        let scoped_path = ScopedPath {
            path,
            namespace,
        };
        self.globals.insert(scoped_path.clone());
        self.definitions
            .entry(scoped_path)
            .or_default()
            .push(Span::Generated);
    }

    fn report_multiple_definitions(
        &self,
        logger: &mut FileLogger,
    ) {
        let mut duplicates = self
            .multiple_definitions
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        duplicates
            .sort_by_key(|scoped_path| format!("{}:{}", scoped_path.namespace, scoped_path.path));
        for scoped_path in duplicates {
            let Some(definitions) = self.definitions.get(&scoped_path) else {
                continue;
            };
            let Some(first_span) = definitions.first().copied() else {
                continue;
            };
            let mut log = logger
                .error(format!("Duplicate {} definition", scoped_path.namespace))
                .primary(
                    format!("`{}` is already defined.", scoped_path.path),
                    first_span,
                );
            for span in definitions.iter().skip(1) {
                log = log.secondary("Redefined here.", *span);
            }
            log.done();
        }
    }

    fn report_undefined_usages(
        &self,
        logger: &mut FileLogger,
    ) {
        let mut undefined = self
            .undefined_usages
            .iter()
            .map(|(scoped_path, spans)| (scoped_path.clone(), spans.as_slice()))
            .collect::<Vec<_>>();
        undefined.sort_by_key(|(scoped_path, _)| {
            format!("{}:{}", scoped_path.namespace, scoped_path.path)
        });
        for (scoped_path, spans) in undefined {
            let Some(first_span) = spans.first().copied() else {
                continue;
            };
            let mut log = logger
                .error(format!("Undefined {}", scoped_path.namespace))
                .primary(
                    format!("`{}` is not defined.", scoped_path.path),
                    first_span,
                );
            for span in spans.iter().skip(1) {
                log = log.secondary("Also used here.", *span);
            }
            log.done();
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
            Some(binding) => {
                if namespace == NameSpace::Term {
                    let current_depth = self.capture_buffer.len();
                    if current_depth > 0
                        && binding.function_depth < current_depth
                        && let Some(buffer) = self.capture_buffer.last_mut()
                    {
                        buffer.insert(binding.path.clone());
                    }
                }
                self.usages
                    .entry(ScopedPath {
                        path: binding.path.clone(),
                        namespace,
                    })
                    .or_default()
                    .push(string.span);
                binding.path
            }
            None => {
                let path = Path::new(&self.module_name, &string.inner);
                let scoped_path = ScopedPath {
                    path: path.clone(),
                    namespace,
                };
                if self.definitions.contains_key(&scoped_path) {
                    self.usages
                        .entry(scoped_path)
                        .or_default()
                        .push(string.span);
                } else {
                    self.undefined_usages
                        .entry(scoped_path)
                        .or_default()
                        .push(string.span);
                }
                path
            }
        }
    }
    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path {
        let scoped_path = ScopedPath {
            path: path.inner.clone(),
            namespace,
        };
        let is_local_module_path = path.inner.major == self.module_name;
        if is_local_module_path && !self.definitions.contains_key(&scoped_path) {
            self.undefined_usages
                .entry(scoped_path)
                .or_default()
                .push(path.span);
        } else {
            self.usages.entry(scoped_path).or_default().push(path.span);
        }
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
        let binding = LocalBinding {
            path: path.clone(),
            function_depth: self.module.capture_buffer.len(),
        };
        let old_value = self.module.locals.insert(scoped_string.clone(), binding);
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
                Some(binding) => {
                    self.module.locals.insert(key, binding);
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
