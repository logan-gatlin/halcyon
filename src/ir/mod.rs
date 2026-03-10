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

fn resolve_constructor_reference(
    scope: &mut ModuleScope,
    target: ast::PathOrIdent,
) -> Option<Path> {
    Some(match target {
        ast::PathOrIdent::Ident(ident) => {
            scope.query_string(ident.name_text_spanned()?, NameSpace::Constructor)
        }
        ast::PathOrIdent::Path(path) => {
            let resolved = scope
                .resolve_path(&path, NameSpace::Constructor, path.span())?
                .with_span(path.span());
            scope.query_path(resolved, NameSpace::Constructor)
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
                    assert_eq!(term_path, constructor_path);
                    let target =
                        resolve_constructor_reference(module_scope, let_statement.alias_target()?)?;
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
                    let target = match trait_statement.alias_target()? {
                        ast::PathOrIdent::Ident(ident) => {
                            module_scope.query_string(ident.name_text_spanned()?, NameSpace::Trait)
                        }
                        ast::PathOrIdent::Path(path) => {
                            let resolved = module_scope
                                .resolve_path(&path, NameSpace::Trait, path.span())?
                                .with_span(path.span());
                            module_scope.query_path(resolved, NameSpace::Trait)
                        }
                    };
                    Statement::TraitAlias {
                        comments,
                        path,
                        target,
                    }
                } else {
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
                        methods,
                    }
                }
            }
            ast::Statement::Impl(impl_statement) => {
                let trait_path = match impl_statement.trait_name()? {
                    ast::PathOrIdent::Ident(ident) => {
                        module_scope.query_string(ident.name_text_spanned()?, NameSpace::Trait)
                    }
                    ast::PathOrIdent::Path(path) => {
                        let resolved = module_scope
                            .resolve_path(&path, NameSpace::Trait, path.span())?
                            .with_span(path.span());
                        module_scope.query_path(resolved, NameSpace::Trait)
                    }
                };

                let arguments = impl_statement
                    .type_args()
                    .into_iter()
                    .map(|arg| type_expr(module_scope, logger, arg))
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
                lint_kebab_case_name(
                    logger,
                    "Module",
                    &nested_module_name.inner,
                    nested_module_name.span,
                );
                module_scope.enter_module(nested_module_name);
                module_scope.register_implicit_open_use(&["core", "prelude"]);
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

pub fn module_with_prelude(
    module_node: ast::Module,
    logger: &mut FileLogger,
    prelude: &[(Path, NameSpace)],
) -> Option<Module<()>> {
    let name = module_node.name_text_spanned()?;
    lint_kebab_case_name(logger, "Module", &name.inner, name.span);
    let name = name.inner;
    let module_name = name.clone();
    let mut module_scope = ModuleScope::new(name.clone());
    for (path, namespace) in prelude {
        module_scope.predefine(path.clone(), *namespace);
    }
    module_scope.register_implicit_open_use(&["core", "prelude"]);
    let mut wasm_type_defs: IndexMap<String, WasmType> = IndexMap::new();
    let mut statements = Vec::new();
    lower_module_statements(
        &mut module_scope,
        &module_name,
        &mut wasm_type_defs,
        &mut statements,
        logger,
        &module_node.statements(),
    )?;
    module_scope.report_name_resolution_errors(logger);
    Some(Module {
        name,
        statements: statements.into_boxed_slice(),
    })
}
