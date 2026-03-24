use std::collections::{
    HashMap,
    HashSet,
};

use crate::parse::ast::{
    self,
    AstNode,
    HasName,
};
use crate::{
    CORE_BUNDLE_NAME,
    FileLogger,
    Span,
    Spanned,
    WithContext,
    WithSpan,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct Path {
    pub major: String,
    pub minor: String,
}

impl TryFrom<ast::Path> for Path {
    type Error = ();

    fn try_from(value: ast::Path) -> Result<Self, Self::Error> {
        Self::from_segments(&value.segments()).ok_or(())
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
        Self::new(CORE_BUNDLE_NAME, minor)
    }

    pub fn from_segments(segments: &[String]) -> Option<Self> {
        let (major, minor_segments) = segments.split_first()?;
        if minor_segments.is_empty() {
            return None;
        }
        Some(Self::new(
            major.clone(),
            minor_segments.join(Self::DELIMETER),
        ))
    }

    pub fn child(
        &self,
        segment: impl AsRef<str>,
    ) -> Self {
        let mut minor = self.minor.clone();
        minor.push_str(Self::DELIMETER);
        minor.push_str(segment.as_ref());
        Self::new(self.major.clone(), minor)
    }

    pub fn sibling(
        &self,
        segment: impl AsRef<str>,
    ) -> Self {
        let segment = segment.as_ref();
        match self.minor.rsplit_once(Self::DELIMETER) {
            Some((prefix, _)) => {
                Self::new(
                    self.major.clone(),
                    format!("{prefix}{}{}", Self::DELIMETER, segment),
                )
            }
            None => Self::new(self.major.clone(), segment),
        }
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
    Module,
    Constructor,
    Trait,
    Type,
    Term,
    Wasm,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopedPath {
    pub path: Path,
    pub namespace: NameSpace,
}

#[derive(Debug, Clone, Default)]
pub struct NameIndex {
    pub definitions: HashMap<ScopedPath, Box<[Span]>>,
    pub usages: HashMap<ScopedPath, Box<[Span]>>,
}

impl NameIndex {
    pub fn references(
        &self,
        symbol: &ScopedPath,
    ) -> Box<[Span]> {
        self.definitions
            .get(symbol)
            .into_iter()
            .flatten()
            .copied()
            .chain(self.usages.get(symbol).into_iter().flatten().copied())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub fn symbol_at(
        &self,
        file_id: usize,
        byte_offset: usize,
    ) -> Option<ScopedPath> {
        fn contains_offset(
            span: Span,
            file_id: usize,
            byte_offset: usize,
        ) -> Option<usize> {
            match span {
                Span::Source {
                    start,
                    width,
                    file_id: Some(span_file_id),
                } if span_file_id == file_id
                    && byte_offset >= start
                    && byte_offset < start + width =>
                {
                    Some(width)
                }
                _ => None,
            }
        }

        self.definitions
            .iter()
            .chain(self.usages.iter())
            .flat_map(|(symbol, spans)| {
                spans
                    .iter()
                    .filter_map(|span| contains_offset(*span, file_id, byte_offset))
                    .map(|width| (symbol, width))
                    .collect::<Vec<_>>()
            })
            .min_by_key(|(_, width)| *width)
            .map(|(symbol, _)| symbol.clone())
    }
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

#[derive(Debug, Clone)]
struct UseImport {
    module_segments: Box<[String]>,
}

#[derive(Debug, Clone)]
struct UseAlias {
    module_segments: Box<[String]>,
    span: Span,
}

#[derive(Debug, Clone, Default)]
struct UseScopeFrame {
    imports: Vec<UseImport>,
    aliases: HashMap<String, UseAlias>,
}

#[derive(Debug, Clone)]
struct AliasCollision {
    alias: String,
    first_span: Span,
    second_span: Span,
}

#[derive(Debug, Clone)]
struct AmbiguousUsage {
    namespace: NameSpace,
    reference: String,
    span: Span,
    candidates: Box<[Path]>,
}

#[derive(Debug, Clone)]
struct ImplicitBundleRelativeUsage {
    namespace: NameSpace,
    reference: String,
    explicit_reference: String,
    resolved: Path,
    span: Span,
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
                NameSpace::Module => "module",
                NameSpace::Constructor => "constructor",
                NameSpace::Trait => "trait",
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
    /// Nested module path currently being lowered.
    module_path: Vec<String>,
    /// Unique number added to local names
    salt: usize,
    /// Symbols defined globally in this module
    globals: HashSet<ScopedPath>,
    /// In-scope local names
    locals: HashMap<ScopedString, LocalBinding>,
    /// Symbols that could not be found in the current scope
    #[allow(dead_code)]
    external: HashSet<ScopedPath>,
    /// Active `use` scopes in each nested module frame.
    use_scopes: Vec<Vec<UseScopeFrame>>,
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
    ambiguous_usages: Vec<AmbiguousUsage>,
    implicit_bundle_relative_usages: Vec<ImplicitBundleRelativeUsage>,
    alias_collisions: Vec<AliasCollision>,
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
    fn query_string_if_defined(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Option<Path>;
    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path;
    fn resolve_path(
        &mut self,
        path: &ast::Path,
        namespace: NameSpace,
        usage_span: Span,
    ) -> Option<Path>;
    fn register_use(
        &mut self,
        target: ast::PathOrIdent,
        alias: Option<Spanned<String>>,
        span: Span,
    ) -> Option<()>;
    fn push_use_scope(&mut self);
    fn pop_use_scope(&mut self);
    fn nest_scope(&mut self) -> impl Scope;
    fn nest_function_scope(&mut self) -> LocalFunctionScope<impl Scope>;
    fn _end_capture(&mut self) -> Box<[Path]>;
}

impl ModuleScope {
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            module_path: Default::default(),
            salt: Default::default(),
            globals: Default::default(),
            locals: Default::default(),
            external: Default::default(),
            use_scopes: vec![vec![UseScopeFrame::default()]],
            history_buffer: Default::default(),
            capture_buffer: Default::default(),
            definitions: Default::default(),
            usages: Default::default(),
            undefined_usages: Default::default(),
            multiple_definitions: Default::default(),
            ambiguous_usages: Default::default(),
            implicit_bundle_relative_usages: Default::default(),
            alias_collisions: Default::default(),
        }
    }

    pub fn with_salt(
        module_name: String,
        salt: usize,
    ) -> Self {
        let mut scope = Self::new(module_name);
        scope.salt = salt;
        scope
    }

    pub fn salt(&self) -> usize {
        self.salt
    }

    pub fn report_name_resolution_errors(
        &self,
        logger: &mut FileLogger,
    ) {
        self.report_multiple_definitions(logger);
        self.report_alias_collisions(logger);
        self.report_ambiguous_usages(logger);
        self.report_implicit_bundle_relative_usages(logger);
        self.report_undefined_usages(logger);
    }

    pub fn name_index(&self) -> NameIndex {
        NameIndex {
            definitions: self
                .definitions
                .iter()
                .map(|(symbol, spans)| (symbol.clone(), spans.clone().into_boxed_slice()))
                .collect(),
            usages: self
                .usages
                .iter()
                .map(|(symbol, spans)| (symbol.clone(), spans.clone().into_boxed_slice()))
                .collect(),
        }
    }

    pub fn predefine(
        &mut self,
        path: Path,
        namespace: NameSpace,
    ) {
        let scoped_path = ScopedPath { path, namespace };
        self.globals.insert(scoped_path.clone());
        self.definitions
            .entry(scoped_path)
            .or_default()
            .push(Span::Generated);
    }

    pub fn define_path(
        &mut self,
        path: Path,
        namespace: NameSpace,
        span: Span,
    ) {
        let scoped_path = ScopedPath { path, namespace };
        self.globals.insert(scoped_path.clone());
        let defs = self.definitions.entry(scoped_path.clone()).or_default();
        if !defs.is_empty() {
            self.multiple_definitions.insert(scoped_path);
        }
        defs.push(span);
    }

    pub fn enter_module(
        &mut self,
        name: Spanned<String>,
    ) {
        self.module_path.push(name.inner);
        self.use_scopes.push(vec![UseScopeFrame::default()]);
    }

    pub fn leave_module(&mut self) {
        let _ = self
            .module_path
            .pop()
            .unwrap_or_else(|| unreachable!("left nested module scope without entering one"));
        let _ = self
            .use_scopes
            .pop()
            .unwrap_or_else(|| unreachable!("left nested module scope without entering one"));
    }

    pub fn push_use_scope(&mut self) {
        let Some(scopes) = self.use_scopes.last_mut() else {
            unreachable!("missing module use scope stack")
        };
        scopes.push(UseScopeFrame::default());
    }

    pub fn pop_use_scope(&mut self) {
        let Some(scopes) = self.use_scopes.last_mut() else {
            unreachable!("missing module use scope stack")
        };
        assert!(
            scopes.len() > 1,
            "Attempted to pop module base `use` scope frame"
        );
        let _ = scopes.pop();
    }

    pub fn register_use(
        &mut self,
        target: ast::PathOrIdent,
        alias: Option<Spanned<String>>,
        span: Span,
    ) -> Option<()> {
        #[derive(Clone, Copy)]
        enum PathPrefix {
            None,
            Root,
            Bundle,
        }

        let (source_segments, path_prefix) = match target {
            ast::PathOrIdent::Ident(ident) => (vec![ident.name_text_spanned()?], PathPrefix::None),
            ast::PathOrIdent::Path(path) => {
                let path_prefix = if path.is_rooted() {
                    PathPrefix::Root
                } else if path.is_bundle_rooted() {
                    PathPrefix::Bundle
                } else {
                    PathPrefix::None
                };
                (path.segments_spanned(), path_prefix)
            }
        };
        let segments = source_segments
            .iter()
            .map(|segment| segment.inner.clone())
            .collect::<Vec<_>>();
        let reference_segments = segments.clone();

        let mut used_absolute_fallback = false;
        let module_segments = match path_prefix {
            PathPrefix::Root => {
                if segments.is_empty() {
                    return None;
                }
                segments
            }
            PathPrefix::Bundle => {
                let mut bundle_segments = Vec::with_capacity(1 + segments.len());
                bundle_segments.push(self.module_name.clone());
                bundle_segments.extend(segments);
                bundle_segments
            }
            PathPrefix::None => {
                let mut relative_segments =
                    Vec::with_capacity(1 + self.module_path.len() + segments.len());
                relative_segments.push(self.module_name.clone());
                relative_segments.extend(self.module_path.iter().cloned());
                relative_segments.extend(segments.iter().cloned());
                if self.has_definition_with_prefix(&relative_segments) {
                    relative_segments
                } else {
                    used_absolute_fallback = true;
                    segments
                }
            }
        };

        if used_absolute_fallback {
            let resolved_module = if module_segments.len() == 1 {
                Path::new(module_segments[0].clone(), String::new())
            } else if let Some(path) = Path::from_segments(&module_segments) {
                path
            } else {
                return None;
            };
            self.maybe_record_implicit_bundle_relative_usage(
                NameSpace::Module,
                &reference_segments,
                &resolved_module,
                span,
            );
        }

        self.record_module_usages_for_use_target(
            &source_segments,
            matches!(path_prefix, PathPrefix::Root),
            &module_segments,
        );

        if let Some(alias) = alias {
            if let Some(previous) = self.lookup_alias(&alias.inner) {
                self.alias_collisions.push(AliasCollision {
                    alias: alias.inner,
                    first_span: previous.span,
                    second_span: alias.span,
                });
                return Some(());
            }
            let frame = self.current_use_frame_mut()?;
            frame.aliases.insert(
                alias.inner,
                UseAlias {
                    module_segments: module_segments.into_boxed_slice(),
                    span: alias.span,
                },
            );
            return Some(());
        }

        let frame = self.current_use_frame_mut()?;
        frame.imports.push(UseImport {
            module_segments: module_segments.into_boxed_slice(),
        });
        Some(())
    }

    pub fn register_implicit_open_use(
        &mut self,
        module_segments: &[&str],
    ) {
        if module_segments.is_empty() {
            return;
        }
        let Some(frame) = self.current_use_frame_mut() else {
            return;
        };
        frame.imports.push(UseImport {
            module_segments: module_segments
                .iter()
                .map(|segment| segment.to_string())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        });
    }

    fn current_use_frame_mut(&mut self) -> Option<&mut UseScopeFrame> {
        let scopes = self.use_scopes.last_mut()?;
        scopes.last_mut()
    }

    fn active_use_frames(&self) -> &[UseScopeFrame] {
        self.use_scopes
            .last()
            .map(Vec::as_slice)
            .unwrap_or_else(|| unreachable!("missing use scope stack"))
    }

    fn lookup_alias(
        &self,
        alias: &str,
    ) -> Option<&UseAlias> {
        self.active_use_frames()
            .iter()
            .rev()
            .find_map(|frame| frame.aliases.get(alias))
    }

    fn scoped_minor(
        &self,
        leaf_name: &str,
    ) -> String {
        if self.module_path.is_empty() {
            leaf_name.to_string()
        } else {
            format!(
                "{}{}{}",
                self.module_path.join(Path::DELIMETER),
                Path::DELIMETER,
                leaf_name
            )
        }
    }

    fn global_path_for_name(
        &self,
        leaf_name: &str,
    ) -> Path {
        Path::new(self.module_name.clone(), self.scoped_minor(leaf_name))
    }

    fn has_definition(
        &self,
        path: &Path,
        namespace: NameSpace,
    ) -> bool {
        self.definitions.contains_key(&ScopedPath {
            path: path.clone(),
            namespace,
        })
    }

    fn path_has_prefix(
        path: &Path,
        prefix_segments: &[String],
    ) -> bool {
        let Some((major, minor_prefix)) = prefix_segments.split_first() else {
            return false;
        };
        if path.major != *major {
            return false;
        }
        if minor_prefix.is_empty() {
            return true;
        }
        let minor_segments = path.minor.split(Path::DELIMETER).collect::<Vec<_>>();
        if minor_segments.len() < minor_prefix.len() {
            return false;
        }
        minor_segments
            .iter()
            .zip(minor_prefix.iter())
            .all(|(left, right)| *left == right)
    }

    fn has_definition_with_prefix(
        &self,
        prefix_segments: &[String],
    ) -> bool {
        self.definitions
            .keys()
            .any(|scoped_path| Self::path_has_prefix(&scoped_path.path, prefix_segments))
    }

    pub fn direct_children(
        &self,
        parent: &Path,
        namespace: NameSpace,
    ) -> Vec<Path> {
        let prefix = format!("{}{}", parent.minor, Path::DELIMETER);
        self.definitions
            .keys()
            .filter(|scoped_path| {
                scoped_path.namespace == namespace
                    && scoped_path.path.major == parent.major
                    && scoped_path.path.minor.starts_with(&prefix)
                    && !scoped_path.path.minor[prefix.len()..].contains(Path::DELIMETER)
            })
            .map(|scoped_path| scoped_path.path.clone())
            .collect()
    }

    fn concat_segments(
        left: &[String],
        right: &[String],
    ) -> Option<Path> {
        let mut segments = Vec::with_capacity(left.len() + right.len());
        segments.extend(left.iter().cloned());
        segments.extend(right.iter().cloned());
        Path::from_segments(&segments)
    }

    fn resolve_from_alias(
        &self,
        segments: &[String],
    ) -> Option<Path> {
        let (alias_name, tail_segments) = segments.split_first()?;
        if tail_segments.is_empty() {
            return None;
        }
        let alias = self.lookup_alias(alias_name)?;
        Self::concat_segments(&alias.module_segments, tail_segments)
    }

    fn resolve_from_use_imports(
        &mut self,
        tail_segments: &[String],
        namespace: NameSpace,
        reference: String,
        usage_span: Span,
    ) -> Option<Path> {
        let mut seen_paths = HashSet::new();
        let mut known_candidates = Vec::new();
        for frame in self.active_use_frames().iter().rev() {
            for use_import in frame.imports.iter().rev() {
                let Some(path) = Self::concat_segments(&use_import.module_segments, tail_segments)
                else {
                    continue;
                };
                if !seen_paths.insert(path.clone()) {
                    continue;
                }
                if self.has_definition(&path, namespace) {
                    known_candidates.push(path.clone());
                }
            }
        }

        if known_candidates.len() > 1 {
            self.ambiguous_usages.push(AmbiguousUsage {
                namespace,
                reference,
                span: usage_span,
                candidates: known_candidates.clone().into_boxed_slice(),
            });
            return known_candidates.into_iter().next();
        }

        known_candidates.into_iter().next()
    }

    fn maybe_record_implicit_bundle_relative_usage(
        &mut self,
        namespace: NameSpace,
        reference_segments: &[String],
        resolved: &Path,
        span: Span,
    ) {
        if span == Span::Generated || matches!(span.file_id(), None | Some(0)) {
            return;
        }
        if reference_segments.is_empty() {
            return;
        }
        if reference_segments.first() != Some(&self.module_name) {
            return;
        }
        let resolved_exists = if namespace == NameSpace::Module && resolved.minor.is_empty() {
            self.has_definition_with_prefix(reference_segments)
        } else {
            self.has_definition(resolved, namespace)
        };
        if !resolved_exists {
            return;
        }
        let explicit_reference = if reference_segments.len() == 1 {
            "bundle".to_string()
        } else {
            format!("bundle::{}", reference_segments[1..].join(Path::DELIMETER))
        };
        self.implicit_bundle_relative_usages
            .push(ImplicitBundleRelativeUsage {
                namespace,
                reference: reference_segments.join(Path::DELIMETER),
                explicit_reference,
                resolved: resolved.clone(),
                span,
            });
    }

    fn module_prefixes_for_symbol_path(path: &Path) -> Vec<Path> {
        let minor_segments = path
            .minor
            .split(Path::DELIMETER)
            .map(str::to_string)
            .collect::<Vec<_>>();
        if minor_segments.len() < 2 {
            return Vec::new();
        }
        (1..minor_segments.len())
            .map(|prefix_len| {
                Path::new(
                    path.major.clone(),
                    minor_segments[..prefix_len].join(Path::DELIMETER),
                )
            })
            .collect()
    }

    fn module_prefixes_for_module_path_segments(segments: &[String]) -> Vec<Path> {
        if segments.len() < 2 {
            return Vec::new();
        }
        (2..=segments.len())
            .filter_map(|prefix_len| Path::from_segments(&segments[..prefix_len]))
            .collect()
    }

    fn record_module_usages_from_alignment(
        &mut self,
        source_module_segments: &[Spanned<String>],
        resolved_module_paths: &[Path],
    ) {
        let paired_count = source_module_segments
            .len()
            .min(resolved_module_paths.len());
        if paired_count == 0 {
            return;
        }

        let source_start = source_module_segments.len() - paired_count;
        let path_start = resolved_module_paths.len() - paired_count;
        for (segment, module_path) in source_module_segments[source_start..]
            .iter()
            .zip(resolved_module_paths[path_start..].iter())
        {
            if !self.has_definition(module_path, NameSpace::Module) {
                continue;
            }

            self.usages
                .entry(ScopedPath {
                    path: module_path.clone(),
                    namespace: NameSpace::Module,
                })
                .or_default()
                .push(segment.span);
        }
    }

    fn record_module_usages_for_value_path(
        &mut self,
        path: &ast::Path,
        source_segments: &[Spanned<String>],
        resolved_path: &Path,
        used_alias_for_first_segment: bool,
    ) {
        if source_segments.len() < 2 {
            return;
        }

        let mut source_module_segments = source_segments[..source_segments.len() - 1].to_vec();
        if path.is_rooted() && !source_module_segments.is_empty() {
            source_module_segments.remove(0);
        }
        if used_alias_for_first_segment && !source_module_segments.is_empty() {
            source_module_segments.remove(0);
        }
        if source_module_segments.is_empty() {
            return;
        }

        let resolved_module_paths = Self::module_prefixes_for_symbol_path(resolved_path);
        self.record_module_usages_from_alignment(&source_module_segments, &resolved_module_paths);
    }

    fn record_module_usages_for_use_target(
        &mut self,
        source_segments: &[Spanned<String>],
        source_is_rooted: bool,
        resolved_module_segments: &[String],
    ) {
        if source_segments.is_empty() {
            return;
        }

        let mut source_module_segments = source_segments.to_vec();
        if source_is_rooted && !source_module_segments.is_empty() {
            source_module_segments.remove(0);
        }
        if source_module_segments.is_empty() {
            return;
        }

        let resolved_module_paths =
            Self::module_prefixes_for_module_path_segments(resolved_module_segments);
        self.record_module_usages_from_alignment(&source_module_segments, &resolved_module_paths);
    }

    fn resolve_ast_path(
        &mut self,
        path: &ast::Path,
        namespace: NameSpace,
        usage_span: Span,
    ) -> Option<Path> {
        let source_segments = path.segments_spanned();
        let segments = source_segments
            .iter()
            .map(|segment| segment.inner.clone())
            .collect::<Vec<_>>();

        if path.is_rooted() {
            let resolved = Path::from_segments(&segments)?;
            self.record_module_usages_for_value_path(path, &source_segments, &resolved, false);
            return Some(resolved);
        }
        if path.is_bundle_rooted() {
            if segments.is_empty() {
                return None;
            }
            let resolved = Path::new(self.module_name.clone(), segments.join(Path::DELIMETER));
            self.record_module_usages_for_value_path(path, &source_segments, &resolved, false);
            return Some(resolved);
        }

        let mut scoped_segments = Vec::with_capacity(1 + self.module_path.len() + segments.len());
        scoped_segments.push(self.module_name.clone());
        scoped_segments.extend(self.module_path.iter().cloned());
        scoped_segments.extend(segments.iter().cloned());
        let scoped_path = Path::from_segments(&scoped_segments)?;

        let mut used_alias_for_first_segment = false;
        let mut used_absolute_fallback = false;
        let resolved = if self.has_definition(&scoped_path, namespace) {
            scoped_path
        } else if let Some(alias_path) = self.resolve_from_alias(&segments) {
            used_alias_for_first_segment = true;
            alias_path
        } else if let Some(use_path) = self.resolve_from_use_imports(
            &segments,
            namespace,
            segments.join(Path::DELIMETER),
            usage_span,
        ) {
            use_path
        } else {
            used_absolute_fallback = true;
            Path::from_segments(&segments)?
        };

        if used_absolute_fallback {
            self.maybe_record_implicit_bundle_relative_usage(
                namespace,
                &segments,
                &resolved,
                path.span(),
            );
        }

        self.record_module_usages_for_value_path(
            path,
            &source_segments,
            &resolved,
            used_alias_for_first_segment,
        );
        Some(resolved)
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

    fn report_alias_collisions(
        &self,
        logger: &mut FileLogger,
    ) {
        let mut collisions = self.alias_collisions.clone();
        collisions.sort_by_key(|collision| collision.alias.clone());
        for collision in collisions {
            logger
                .error("Duplicate module alias")
                .primary(
                    format!(
                        "`{}` is already used as a module alias in this scope.",
                        collision.alias
                    ),
                    collision.second_span,
                )
                .secondary("First aliased here.", collision.first_span)
                .done();
        }
    }

    fn report_ambiguous_usages(
        &self,
        logger: &mut FileLogger,
    ) {
        let mut usages = self.ambiguous_usages.clone();
        usages.sort_by_key(|usage| {
            format!(
                "{}:{}:{}",
                usage.namespace,
                usage.reference,
                usage
                    .candidates
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("|")
            )
        });
        for usage in usages {
            let candidates = usage
                .candidates
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            logger
                .error(format!("Ambiguous {}", usage.namespace))
                .primary(
                    format!(
                        "`{}` is provided by multiple `use` imports: {candidates}",
                        usage.reference
                    ),
                    usage.span,
                )
                .done();
        }
    }

    fn report_implicit_bundle_relative_usages(
        &self,
        logger: &mut FileLogger,
    ) {
        let mut usages = self.implicit_bundle_relative_usages.clone();
        usages.sort_by_key(|usage| {
            format!(
                "{}:{}:{}",
                usage.namespace, usage.reference, usage.explicit_reference
            )
        });
        for usage in usages {
            logger
                .warning("Implicit bundle-relative path")
                .primary(
                    format!(
                        "`{}` resolves to `root::{}`; use `{}` to refer to the current bundle in a less ambiguous way.",
                        usage.reference, usage.resolved, usage.explicit_reference
                    ),
                    usage.span,
                )
                .done();
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
        let path = self.global_path_for_name(&name.inner);
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
                    if current_depth > 0 && binding.function_depth < current_depth {
                        for buffer in self.capture_buffer.iter_mut().skip(binding.function_depth) {
                            buffer.insert(binding.path.clone());
                        }
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
                let path = self.global_path_for_name(&string.inner);
                let scoped_path = ScopedPath {
                    path: path.clone(),
                    namespace,
                };
                if self.definitions.contains_key(&scoped_path) {
                    self.usages
                        .entry(scoped_path)
                        .or_default()
                        .push(string.span);
                    path
                } else {
                    let tail_segments = vec![string.inner.clone()];
                    if let Some(use_path) = self.resolve_from_use_imports(
                        &tail_segments,
                        namespace,
                        string.inner.clone(),
                        string.span,
                    ) {
                        self.query_path(use_path.with_span(string.span), namespace)
                    } else {
                        self.undefined_usages
                            .entry(scoped_path)
                            .or_default()
                            .push(string.span);
                        path
                    }
                }
            }
        }
    }
    fn query_string_if_defined(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Option<Path> {
        match self
            .locals
            .get(&ScopedString {
                string: string.inner.clone(),
                namespace,
            })
            .cloned()
        {
            Some(binding) => {
                self.usages
                    .entry(ScopedPath {
                        path: binding.path.clone(),
                        namespace,
                    })
                    .or_default()
                    .push(string.span);
                Some(binding.path)
            }
            None => {
                let path = self.global_path_for_name(&string.inner);
                let scoped_path = ScopedPath {
                    path: path.clone(),
                    namespace,
                };
                if self.definitions.contains_key(&scoped_path) {
                    self.usages
                        .entry(scoped_path)
                        .or_default()
                        .push(string.span);
                    Some(path)
                } else {
                    let tail_segments = vec![string.inner.clone()];
                    if let Some(use_path) = self.resolve_from_use_imports(
                        &tail_segments,
                        namespace,
                        string.inner,
                        string.span,
                    ) {
                        Some(self.query_path(use_path.with_span(string.span), namespace))
                    } else {
                        None
                    }
                }
            }
        }
    }
    fn resolve_path(
        &mut self,
        path: &ast::Path,
        namespace: NameSpace,
        usage_span: Span,
    ) -> Option<Path> {
        self.resolve_ast_path(path, namespace, usage_span)
    }
    fn register_use(
        &mut self,
        target: ast::PathOrIdent,
        alias: Option<Spanned<String>>,
        span: Span,
    ) -> Option<()> {
        ModuleScope::register_use(self, target, alias, span)
    }
    fn push_use_scope(&mut self) {
        ModuleScope::push_use_scope(self);
    }
    fn pop_use_scope(&mut self) {
        ModuleScope::pop_use_scope(self);
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
    fn query_string_if_defined(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Option<Path> {
        self.module.query_string_if_defined(string, namespace)
    }
    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path {
        self.module.query_path(path, namespace)
    }
    fn resolve_path(
        &mut self,
        path: &ast::Path,
        namespace: NameSpace,
        usage_span: Span,
    ) -> Option<Path> {
        self.module.resolve_path(path, namespace, usage_span)
    }
    fn register_use(
        &mut self,
        target: ast::PathOrIdent,
        alias: Option<Spanned<String>>,
        span: Span,
    ) -> Option<()> {
        self.module.register_use(target, alias, span)
    }
    fn push_use_scope(&mut self) {
        self.module.push_use_scope()
    }
    fn pop_use_scope(&mut self) {
        self.module.pop_use_scope()
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
    fn query_string_if_defined(
        &mut self,
        string: Spanned<String>,
        namespace: NameSpace,
    ) -> Option<Path> {
        self.0.query_string_if_defined(string, namespace)
    }

    fn query_path(
        &mut self,
        path: Spanned<Path>,
        namespace: NameSpace,
    ) -> Path {
        self.0.query_path(path, namespace)
    }
    fn resolve_path(
        &mut self,
        path: &ast::Path,
        namespace: NameSpace,
        usage_span: Span,
    ) -> Option<Path> {
        self.0.resolve_path(path, namespace, usage_span)
    }
    fn register_use(
        &mut self,
        target: ast::PathOrIdent,
        alias: Option<Spanned<String>>,
        span: Span,
    ) -> Option<()> {
        self.0.register_use(target, alias, span)
    }
    fn push_use_scope(&mut self) {
        self.0.push_use_scope()
    }
    fn pop_use_scope(&mut self) {
        self.0.pop_use_scope()
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
        assert!(
            self.module_path.is_empty(),
            "Nested module scopes failed to end"
        );
        assert!(
            self.use_scopes.len() == 1,
            "Nested module `use` scope stacks failed to end"
        );
        assert!(
            self.use_scopes
                .first()
                .is_some_and(|frames| frames.len() == 1),
            "Expression-level `use` scopes failed to end"
        );
    }
}
