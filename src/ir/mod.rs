mod elaborate;
mod names;
mod patterns;
mod pretty_print;
mod terms;
mod types;
pub mod wasm;

pub use elaborate::*;
pub use names::*;
pub use patterns::*;
pub use pretty_print::*;
pub use terms::*;
pub use types::*;

use crate::asm::Type as WasmType;
use crate::logging::WithContext;
use crate::parse::SyntaxKind;
use crate::parse::ast::{
    self,
    AstNode,
    HasLeadingComments,
    HasName,
};
use crate::{
    FileLogger,
    Span,
    Spanned,
    WithSpan,
};
use indexmap::IndexMap;
use inflections::Inflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeKind {
    Local,
    Global,
}

impl std::fmt::Display for ImmediateValue {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ImmediateValue::Unit => write!(f, "()"),
            ImmediateValue::String(s) => write!(f, "\"{s}\""),
            ImmediateValue::Integer(val) => write!(f, "{val}"),
            ImmediateValue::Natural(val) => write!(f, "{val}n"),
            ImmediateValue::Real(val) => write!(f, "{val}"),
            ImmediateValue::Glyph(val) => write!(f, "'{val}'"),
            ImmediateValue::Boolean(val) => write!(f, "{val}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement<T> {
    Term(Term<T>),
    ConstructorAlias {
        comments: String,
        path: Path,
        target: Path,
        span: Span,
    },
    Type {
        comments: String,
        path: Path,
        parameters: Box<[Path]>,
        def: TypeDef,
        kind: TypeDeclKind,
    },
    Trait {
        comments: String,
        path: Path,
        parameters: Box<[Path]>,
        associated_types: Box<[TraitTypeDecl]>,
        methods: Box<[TraitMethodDecl]>,
    },
    TraitAlias {
        comments: String,
        path: Path,
        target: Path,
    },
    Impl {
        comments: String,
        trait_path: Path,
        arguments: Box<[TypeExpr]>,
        associated_types: Box<[ImplTypeDef]>,
        methods: Box<[ImplMethod<T>]>,
    },
    Wasm(Box<[wasm::Declaration]>),
}

#[derive(Debug, Clone)]
pub struct TraitMethodDecl {
    pub path: Path,
    pub type_expr: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitTypeDecl {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplTypeDef {
    pub name: Spanned<String>,
    pub type_expr: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplMethod<T> {
    pub trait_method: Path,
    pub impl_path: Path,
    pub value: Term<T>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct Module<T> {
    pub name: String,
    pub statements: Box<[Statement<T>]>,
}

impl<T> Module<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            statements: Default::default(),
        }
    }
}

fn define_named_type_constructors(
    scope: &mut ModuleScope,
    logger: &mut FileLogger,
    type_def: &ast::TypeDef,
    type_name: Spanned<String>,
) -> Option<()> {
    match type_def {
        ast::TypeDef::Sum(sum_def) => {
            for variant in sum_def.variants() {
                let name = variant.name_text_spanned()?;
                lint_pascal_case_name(logger, "Constructor", &name.inner, name.span);
                scope.define(name.clone(), NameSpace::Term);
                scope.define(name, NameSpace::Constructor);
            }
        }
        ast::TypeDef::Struct(_) | ast::TypeDef::Alias(_) => {
            scope.define(type_name.clone(), NameSpace::Term);
            scope.define(type_name, NameSpace::Constructor);
        }
    }
    Some(())
}

pub(super) fn resolve_path_or_ident(
    scope: &mut impl Scope,
    target: ast::PathOrIdent,
    namespace: NameSpace,
) -> Option<Path> {
    Some(match target {
        ast::PathOrIdent::Ident(ident) => scope.query_string(ident.name_text_spanned()?, namespace),
        ast::PathOrIdent::Path(path) => {
            let span = path
                .name_text_spanned()
                .map_or_else(|| path.span(), |segment| segment.span);
            let resolved = scope.resolve_path(&path, namespace, span)?.with_span(span);
            scope.query_path(resolved, namespace)
        }
    })
}

fn is_bracketed_operator_name(name: &str) -> bool {
    name.starts_with('[') && name.ends_with(']')
}

pub(super) fn lint_case_name(
    logger: &mut FileLogger,
    subject: &str,
    name: &str,
    span: Span,
    case_name: &str,
    matches_case: impl FnOnce(&str) -> bool,
) {
    if is_bracketed_operator_name(name) || matches_case(name) {
        return;
    }
    logger
        .warning("Naming style")
        .primary(format!("{subject} `{name}` should use {case_name}."), span)
        .done();
}

pub(super) fn lint_snake_case_name(
    logger: &mut FileLogger,
    subject: &str,
    name: &str,
    span: Span,
) {
    lint_case_name(logger, subject, name, span, "snake_case", |value| {
        value == value.to_snake_case()
    });
}

pub(super) fn lint_kebab_case_name(
    logger: &mut FileLogger,
    subject: &str,
    name: &str,
    span: Span,
) {
    lint_case_name(logger, subject, name, span, "kebab-case", |value| {
        value == value.to_kebab_case()
    });
}

pub(super) fn lint_pascal_case_name(
    logger: &mut FileLogger,
    subject: &str,
    name: &str,
    span: Span,
) {
    lint_case_name(logger, subject, name, span, "PascalCase", |value| {
        value == value.to_pascal_case()
    });
}

fn enter_nested_module_scope(
    module_scope: &mut ModuleScope,
    logger: &mut FileLogger,
    nested_module_name: Spanned<String>,
) {
    lint_kebab_case_name(
        logger,
        "Module",
        &nested_module_name.inner,
        nested_module_name.span,
    );
    module_scope.enter_module(nested_module_name);
    module_scope.register_implicit_open_use(&[crate::CORE_BUNDLE_NAME, "prelude"]);
}

fn lower_module_statements(
    module_scope: &mut ModuleScope,
    module_name: &str,
    wasm_type_defs: &mut IndexMap<String, WasmType>,
    statements: &mut Vec<Statement<()>>,
    logger: &mut FileLogger,
    ast_statements: &[ast::Statement],
) -> Option<()> {
    for statement in ast_statements {
        let comments = statement.leading_comment_text();
        let statement = match statement {
            ast::Statement::Bundle(_) | ast::Statement::Import(_) => continue,
            ast::Statement::Let(let_statement) => {
                if let_statement.is_pattern_alias() {
                    let alias_name = let_statement.alias_name_spanned()?;
                    lint_pascal_case_name(
                        logger,
                        "Constructor alias",
                        &alias_name.inner,
                        alias_name.span,
                    );
                    let term_path = module_scope.define(alias_name.clone(), NameSpace::Term);
                    let constructor_path = module_scope.define(alias_name, NameSpace::Constructor);
                    assert_eq!(
                        term_path, constructor_path,
                        "constructor alias paths must match across term and constructor namespaces"
                    );
                    let target = resolve_path_or_ident(
                        module_scope,
                        let_statement.alias_target()?,
                        NameSpace::Constructor,
                    )?;
                    Statement::ConstructorAlias {
                        comments,
                        path: constructor_path,
                        target,
                        span: let_statement.span(),
                    }
                } else {
                    Statement::Term(Term {
                        comments,
                        kind: TermKind::Let {
                            assignee: pattern(module_scope, logger, let_statement.pattern()?)?,
                            value: term(
                                module_scope,
                                wasm_type_defs,
                                logger,
                                let_statement.value()?,
                            )?
                            .into(),
                            scope: ScopeKind::Global,
                            then: Term::unit().into(),
                            else_: Term::unreachable().into(),
                        },
                        span: let_statement.span(),
                        type_: (),
                    })
                }
            }
            ast::Statement::Do(do_statement) => {
                Statement::Term(Term {
                    comments,
                    kind: TermKind::Let {
                        assignee: Pattern {
                            comments: String::new(),
                            kind: PatternKind::Hole,
                            span: do_statement.span(),
                            type_: (),
                        },
                        value: term(module_scope, wasm_type_defs, logger, do_statement.value()?)?
                            .into(),
                        scope: ScopeKind::Global,
                        then: Term::unit().into(),
                        else_: Term::unreachable().into(),
                    },
                    span: do_statement.span(),
                    type_: (),
                })
            }
            ast::Statement::Type(type_statement) => {
                let type_name = type_statement.name_text_spanned()?;
                lint_pascal_case_name(logger, "Type", &type_name.inner, type_name.span);
                let path = module_scope.define(type_name.clone(), NameSpace::Type);
                let type_def = type_statement.type_def()?;
                let kind = if type_statement.is_alias() {
                    TypeDeclKind::Alias
                } else {
                    TypeDeclKind::Named
                };
                if kind == TypeDeclKind::Named {
                    define_named_type_constructors(
                        module_scope,
                        logger,
                        &type_def,
                        type_name.clone(),
                    )?;
                }
                let mut parameter_scope = module_scope.nest_scope();
                Statement::Type {
                    comments,
                    path,
                    parameters: type_statement
                        .type_params()
                        .into_iter()
                        .map(|param| parameter_scope.define(param, NameSpace::Type))
                        .collect(),
                    def: typedef(&mut parameter_scope, logger, type_def)?,
                    kind,
                }
            }
            ast::Statement::Trait(trait_statement) => {
                let trait_name = trait_statement.name_text_spanned()?;
                lint_pascal_case_name(logger, "Trait", &trait_name.inner, trait_name.span);
                let path = module_scope.define(trait_name, NameSpace::Trait);
                if trait_statement.is_alias() {
                    let target = resolve_path_or_ident(
                        module_scope,
                        trait_statement.alias_target()?,
                        NameSpace::Trait,
                    )?;
                    for target_associated_type in
                        module_scope.direct_children(&target, NameSpace::Type)
                    {
                        let Some((_, associated_type_name)) =
                            target_associated_type.minor.rsplit_once(Path::DELIMETER)
                        else {
                            continue;
                        };
                        module_scope.define_path(
                            path.child(associated_type_name),
                            NameSpace::Type,
                            trait_statement.span(),
                        );
                    }
                    Statement::TraitAlias {
                        comments,
                        path,
                        target,
                    }
                } else {
                    let associated_types = trait_statement
                        .associated_types()
                        .into_iter()
                        .map(|associated_type| {
                            let associated_type_name = associated_type.name_text_spanned()?;
                            lint_pascal_case_name(
                                logger,
                                "Associated type",
                                &associated_type_name.inner,
                                associated_type_name.span,
                            );
                            let associated_type_path = path.child(&associated_type_name.inner);
                            module_scope.define_path(
                                associated_type_path.clone(),
                                NameSpace::Type,
                                associated_type_name.span,
                            );
                            Some(TraitTypeDecl {
                                path: associated_type_path,
                                span: associated_type.span(),
                            })
                        })
                        .collect::<Option<Box<[_]>>>()?;
                    let method_nodes = trait_statement.methods();
                    let method_paths = method_nodes
                        .into_iter()
                        .map(|method| {
                            let method_name = method.name_text_spanned()?;
                            lint_snake_case_name(
                                logger,
                                "Trait item",
                                &method_name.inner,
                                method_name.span,
                            );
                            Some((module_scope.define(method_name, NameSpace::Term), method))
                        })
                        .collect::<Option<Vec<_>>>()?;

                    let mut parameter_scope = module_scope.nest_scope();
                    let parameters = trait_statement
                        .trait_params()
                        .into_iter()
                        .map(|param| parameter_scope.define(param, NameSpace::Type))
                        .collect::<Box<[_]>>();
                    let methods = method_paths
                        .into_iter()
                        .map(|(method_path, method)| {
                            Some(TraitMethodDecl {
                                path: method_path,
                                type_expr: type_expr(&mut parameter_scope, logger, method.ty()?)?,
                                span: method.span(),
                            })
                        })
                        .collect::<Option<Box<[_]>>>()?;
                    Statement::Trait {
                        comments,
                        path,
                        parameters,
                        associated_types,
                        methods,
                    }
                }
            }
            ast::Statement::Impl(impl_statement) => {
                let trait_path = resolve_path_or_ident(
                    module_scope,
                    impl_statement.trait_name()?,
                    NameSpace::Trait,
                )?;

                let arguments = impl_statement
                    .type_args()
                    .into_iter()
                    .map(|arg| type_expr(module_scope, logger, arg))
                    .collect::<Option<Box<[_]>>>()?;

                let associated_types = impl_statement
                    .associated_types()
                    .into_iter()
                    .map(|associated_type| {
                        let associated_type_name = associated_type.name_text_spanned()?;
                        lint_pascal_case_name(
                            logger,
                            "Associated type",
                            &associated_type_name.inner,
                            associated_type_name.span,
                        );
                        Some(ImplTypeDef {
                            name: associated_type_name,
                            type_expr: type_expr(module_scope, logger, associated_type.ty()?)?,
                            span: associated_type.span(),
                        })
                    })
                    .collect::<Option<Box<[_]>>>()?;

                let mut impl_scope = module_scope.nest_scope();
                let methods = impl_statement
                    .methods()
                    .into_iter()
                    .map(|method| {
                        let method_name = method.name_text_spanned()?;
                        lint_snake_case_name(
                            logger,
                            "Trait item",
                            &method_name.inner,
                            method_name.span,
                        );
                        let trait_method = trait_path.sibling(&method_name.inner);
                        let impl_path = impl_scope.define(method_name, NameSpace::Term);
                        Some(ImplMethod {
                            trait_method,
                            impl_path,
                            value: term(&mut impl_scope, wasm_type_defs, logger, method.value()?)?,
                            span: method.span(),
                        })
                    })
                    .collect::<Option<Box<[_]>>>()?;

                Statement::Impl {
                    comments,
                    trait_path,
                    arguments,
                    associated_types,
                    methods,
                }
            }
            ast::Statement::Wasm(wasm_statement) => {
                Statement::Wasm(wasm::build_toplevel(
                    &wasm_statement.sexpr()?,
                    module_name,
                    wasm_type_defs,
                    logger,
                    module_scope,
                ))
            }
            ast::Statement::Use(use_statement) => {
                module_scope.register_use(
                    use_statement.target()?,
                    use_statement.alias_name_spanned(),
                    use_statement.span(),
                )?;
                continue;
            }
            ast::Statement::Module(nested_module) => {
                let nested_module_name = nested_module.name_text_spanned()?;
                module_scope.define(nested_module_name.clone(), NameSpace::Module);
                enter_nested_module_scope(module_scope, logger, nested_module_name);
                let lowered = lower_module_statements(
                    module_scope,
                    module_name,
                    wasm_type_defs,
                    statements,
                    logger,
                    &nested_module.statements(),
                );
                module_scope.leave_module();
                lowered?;
                continue;
            }
        };
        statements.push(statement);
    }
    Some(())
}

pub fn module(
    module_node: ast::Module,
    logger: &mut FileLogger,
) -> Option<Module<()>> {
    module_with_prelude(module_node, logger, &[])
}

pub fn bundle_statements(
    bundle_name: String,
    statements: &[ast::Statement],
    logger: &mut FileLogger,
) -> Option<Module<()>> {
    bundle_statements_with_prelude(bundle_name, statements, logger, &[])
}

pub fn module_with_prelude(
    module_node: ast::Module,
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
) -> Option<Module<()>> {
    let name = module_node.name_text_spanned()?;
    lint_kebab_case_name(logger, "Module", &name.inner, name.span);
    bundle_statements_with_prelude(name.inner, &module_node.statements(), logger, prelude)
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_statements_with_prelude(
    bundle_name: String,
    ast_statements: &[ast::Statement],
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
) -> Option<Module<()>> {
    let mut wasm_type_defs: IndexMap<String, WasmType> = IndexMap::new();
    bundle_statements_with_prelude_and_wasm_types(
        bundle_name,
        ast_statements,
        logger,
        prelude,
        &mut wasm_type_defs,
    )
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_statements_with_prelude_and_wasm_types(
    bundle_name: String,
    ast_statements: &[ast::Statement],
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
    wasm_type_defs: &mut IndexMap<String, WasmType>,
) -> Option<Module<()>> {
    let mut salt = 0;
    bundle_statements_with_prelude_and_wasm_types_and_salt(
        bundle_name,
        ast_statements,
        logger,
        prelude,
        wasm_type_defs,
        &mut salt,
    )
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_statements_with_prelude_indexed(
    bundle_name: String,
    ast_statements: &[ast::Statement],
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
) -> Option<(Module<()>, NameIndex)> {
    let mut wasm_type_defs: IndexMap<String, WasmType> = IndexMap::new();
    bundle_statements_with_prelude_and_wasm_types_indexed(
        bundle_name,
        ast_statements,
        logger,
        prelude,
        &mut wasm_type_defs,
    )
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_statements_with_prelude_and_wasm_types_indexed(
    bundle_name: String,
    ast_statements: &[ast::Statement],
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
    wasm_type_defs: &mut IndexMap<String, WasmType>,
) -> Option<(Module<()>, NameIndex)> {
    let mut salt = 0;
    bundle_statements_with_prelude_and_wasm_types_and_salt_indexed(
        bundle_name,
        ast_statements,
        logger,
        prelude,
        wasm_type_defs,
        &mut salt,
    )
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_statements_with_prelude_and_wasm_types_and_salt(
    bundle_name: String,
    ast_statements: &[ast::Statement],
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
    wasm_type_defs: &mut IndexMap<String, WasmType>,
    salt: &mut usize,
) -> Option<Module<()>> {
    bundle_statements_with_prelude_and_wasm_types_and_salt_indexed(
        bundle_name,
        ast_statements,
        logger,
        prelude,
        wasm_type_defs,
        salt,
    )
    .map(|(module, _)| module)
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_statements_with_prelude_and_wasm_types_and_salt_indexed(
    bundle_name: String,
    ast_statements: &[ast::Statement],
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
    wasm_type_defs: &mut IndexMap<String, WasmType>,
    salt: &mut usize,
) -> Option<(Module<()>, NameIndex)> {
    let _profile_total = crate::profiling::scope("ir.bundle_statements.total");
    let mut module_scope = ModuleScope::with_salt(bundle_name.clone(), *salt);
    for (path, namespace) in prelude {
        module_scope.predefine(path.clone(), *namespace);
    }
    module_scope.register_implicit_open_use(&[crate::CORE_BUNDLE_NAME, "prelude"]);
    let mut lowered_statements = Vec::new();
    {
        let _profile = crate::profiling::scope("ir.bundle_statements.lower_statements");
        lower_module_statements(
            &mut module_scope,
            &bundle_name,
            wasm_type_defs,
            &mut lowered_statements,
            logger,
            ast_statements,
        )?;
    }
    *salt = module_scope.salt();
    {
        let _profile = crate::profiling::scope("ir.bundle_statements.report_name_errors");
        module_scope.report_name_resolution_errors(logger);
    }
    let name_index = module_scope.name_index();
    Some((
        Module {
            name: bundle_name,
            statements: lowered_statements.into_boxed_slice(),
        },
        name_index,
    ))
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_source_file_with_imports_and_prelude<R>(
    bundle_name: String,
    source_file: ast::SourceFile,
    root_file_logger: FileLogger,
    logger: &mut crate::Logger,
    prelude: &[(Path, NameSpace)],
    resolve_import_source: &mut R,
) -> Option<Module<()>>
where
    R: FnMut(String) -> Option<String>,
{
    bundle_source_file_with_imports_and_prelude_indexed(
        bundle_name,
        source_file,
        root_file_logger,
        logger,
        prelude,
        resolve_import_source,
    )
    .map(|(module, _)| module)
}

#[tracing::instrument(skip_all, fields(bundle = %bundle_name))]
pub fn bundle_source_file_with_imports_and_prelude_indexed<R>(
    bundle_name: String,
    source_file: ast::SourceFile,
    root_file_logger: FileLogger,
    logger: &mut crate::Logger,
    prelude: &[(Path, NameSpace)],
    resolve_import_source: &mut R,
) -> Option<(Module<()>, NameIndex)>
where
    R: FnMut(String) -> Option<String>,
{
    let _profile_total = crate::profiling::scope("ir.bundle_with_imports.total");

    fn resolve_import_lookup_path(
        import_path: &str,
        current_file_name: &str,
    ) -> String {
        let import_path = std::path::Path::new(import_path);
        let resolved_path = if import_path.is_absolute() {
            import_path.to_path_buf()
        } else {
            std::path::Path::new(current_file_name)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(import_path)
        };
        resolved_path.to_string_lossy().replace('\\', "/")
    }

    struct ImportLoweringContext<'a, R>
    where
        R: FnMut(String, FileLogger) -> Option<(ast::SourceFile, FileLogger)>,
    {
        deferred_loggers: &'a std::cell::RefCell<Vec<FileLogger>>,
        module_scope: &'a mut ModuleScope,
        module_name: &'a str,
        wasm_type_defs: &'a mut IndexMap<String, WasmType>,
        lowered_statements: &'a mut Vec<Statement<()>>,
        resolve_import: &'a mut R,
        first_bundle_declaration_file: &'a mut Option<String>,
    }

    impl<R> ImportLoweringContext<'_, R>
    where
        R: FnMut(String, FileLogger) -> Option<(ast::SourceFile, FileLogger)>,
    {
        fn lower_source_file(
            &mut self,
            source_file: ast::SourceFile,
            mut file_logger: FileLogger,
        ) -> Option<()> {
            let result = self.lower_statements(&source_file.statements(), &mut file_logger);
            self.deferred_loggers.borrow_mut().push(file_logger);
            result
        }

        fn lower_statements(
            &mut self,
            ast_statements: &[ast::Statement],
            file_logger: &mut FileLogger,
        ) -> Option<()> {
            for statement in ast_statements {
                match statement {
                    ast::Statement::Bundle(bundle_declaration) => {
                        if let Some(first_file) = self.first_bundle_declaration_file {
                            file_logger
                                .error("Duplicate bundle declaration")
                                .primary(
                                    "A bundle may only be declared once across all imported files.",
                                    bundle_declaration.span(),
                                )
                                .note(format!(
                                    "First bundle declaration appears in `{first_file}`."
                                ))
                                .done();
                        } else {
                            *self.first_bundle_declaration_file =
                                Some(file_logger.file_name().to_string());
                        }
                    }
                    ast::Statement::Import(import_statement) => {
                        for path_literal in import_statement.path_literals() {
                            let Some(import_path) =
                                crate::parse::lexer::decode_quoted_string_literal(
                                    &path_literal.inner,
                                )
                            else {
                                file_logger
                                    .error("Invalid import path")
                                    .primary(
                                        "Expected a valid string literal path.",
                                        path_literal.span,
                                    )
                                    .done();
                                continue;
                            };

                            let request_logger = file_logger.spawn_new();
                            let Some((import_source_file, import_file_logger)) =
                                (self.resolve_import)(import_path, request_logger)
                            else {
                                continue;
                            };

                            self.lower_source_file(import_source_file, import_file_logger)?;
                        }
                    }
                    ast::Statement::Module(nested_module) => {
                        let nested_module_name = nested_module.name_text_spanned()?;
                        self.module_scope
                            .define(nested_module_name.clone(), NameSpace::Module);
                        enter_nested_module_scope(
                            self.module_scope,
                            file_logger,
                            nested_module_name,
                        );
                        let lowered =
                            self.lower_statements(&nested_module.statements(), file_logger);
                        self.module_scope.leave_module();
                        lowered?;
                    }
                    _ => {
                        lower_module_statements(
                            self.module_scope,
                            self.module_name,
                            self.wasm_type_defs,
                            self.lowered_statements,
                            file_logger,
                            std::slice::from_ref(statement),
                        )?;
                    }
                }
            }

            Some(())
        }
    }

    let deferred_loggers = std::cell::RefCell::new(Vec::new());
    let mut imported_paths = std::collections::HashSet::new();
    let mut first_bundle_declaration_file = None;
    let mut module_scope = ModuleScope::new(bundle_name.clone());
    for (path, namespace) in prelude {
        module_scope.predefine(path.clone(), *namespace);
    }
    module_scope.register_implicit_open_use(&[crate::CORE_BUNDLE_NAME, "prelude"]);

    let mut name_resolution_logger = root_file_logger.spawn_new();
    let mut wasm_type_defs: IndexMap<String, WasmType> = IndexMap::new();
    let mut lowered_statements = Vec::new();

    {
        let _profile = crate::profiling::scope("ir.bundle_with_imports.lower_all");
        let mut resolve_import = |path: String,
                                  mut current_logger: FileLogger|
         -> Option<(ast::SourceFile, FileLogger)> {
            let lookup_path = resolve_import_lookup_path(&path, current_logger.file_name());

            if !imported_paths.insert(lookup_path.clone()) {
                current_logger
                    .error("Duplicate import")
                    .primary(
                        format!("Import `{lookup_path}` has already been loaded."),
                        Span::Generated,
                    )
                    .done();
                deferred_loggers.borrow_mut().push(current_logger);
                return None;
            }

            let Some(source) = resolve_import_source(lookup_path.clone()) else {
                current_logger
                    .error("Unable to resolve import")
                    .primary(
                        format!("Could not load import `{lookup_path}`."),
                        Span::Generated,
                    )
                    .done();
                deferred_loggers.borrow_mut().push(current_logger);
                return None;
            };

            let mut import_file_logger = logger.new_file(lookup_path, source.clone());
            let Some(import_source_file) = crate::parse::parse(&source, &mut import_file_logger)
            else {
                deferred_loggers.borrow_mut().push(import_file_logger);
                return None;
            };

            Some((import_source_file, import_file_logger))
        };

        let mut lowering_context = ImportLoweringContext {
            deferred_loggers: &deferred_loggers,
            module_scope: &mut module_scope,
            module_name: &bundle_name,
            wasm_type_defs: &mut wasm_type_defs,
            lowered_statements: &mut lowered_statements,
            resolve_import: &mut resolve_import,
            first_bundle_declaration_file: &mut first_bundle_declaration_file,
        };

        lowering_context.lower_source_file(source_file, root_file_logger)?;
    }

    {
        let _profile = crate::profiling::scope("ir.bundle_with_imports.report_name_errors");
        module_scope.report_name_resolution_errors(&mut name_resolution_logger);
    }
    deferred_loggers.borrow_mut().push(name_resolution_logger);

    for file_logger in deferred_loggers.into_inner() {
        logger.consume_file(file_logger);
    }

    let name_index = module_scope.name_index();
    Some((
        Module {
            name: bundle_name,
            statements: lowered_statements.into_boxed_slice(),
        },
        name_index,
    ))
}
