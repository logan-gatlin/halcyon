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
    Type {
        path: Path,
        parameters: Box<[Path]>,
        def: TypeDef,
        kind: TypeDeclKind,
    },
    Trait {
        path: Path,
        parameters: Box<[Path]>,
        methods: Box<[TraitMethodDecl]>,
    },
    Impl {
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

fn define_sum_constructors(
    scope: &mut ModuleScope,
    type_def: &ast::TypeDef,
) -> Option<()> {
    let ast::TypeDef::Sum(sum_def) = type_def else {
        return Some(());
    };
    for variant in sum_def.variants() {
        scope.define(variant.name_text_spanned()?, NameSpace::Constructor);
    }
    Some(())
}

pub fn module(
    module_node: ast::Module,
    logger: &mut FileLogger,
) -> Option<Module<()>> {
    let name = module_node.name_text()?;
    let module_name = name.clone();
    let mut module_scope = ModuleScope::new(name.clone());
    let mut wasm_type_defs: IndexMap<String, WasmType> = IndexMap::new();
    let mut statements = Vec::new();
    for statement in module_node.statements() {
        let comments = statement.leading_comment_text();
        let statement = match statement {
            ast::Statement::Let(let_statement) => {
                Statement::Term(Term {
                    comments,
                    kind: TermKind::Let {
                        assignee: pattern(&mut module_scope, logger, let_statement.pattern()?)?,
                        value: term(
                            &mut module_scope,
                            &wasm_type_defs,
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
            ast::Statement::Type(type_statement) => {
                let path =
                    module_scope.define(type_statement.name_text_spanned()?, NameSpace::Type);
                let type_def = type_statement.type_def()?;
                let kind = if type_statement.is_alias() {
                    TypeDeclKind::Alias
                } else {
                    TypeDeclKind::Named
                };
                if kind == TypeDeclKind::Named {
                    define_sum_constructors(&mut module_scope, &type_def)?;
                }
                let mut parameter_scope = module_scope.nest_scope();
                Statement::Type {
                    path,
                    parameters: type_statement
                        .type_params()
                        .into_iter()
                        .map(|param| parameter_scope.define(param, NameSpace::Type))
                        .collect(),
                    def: typedef(&mut parameter_scope, type_def)?,
                    kind,
                }
            }
            ast::Statement::Trait(trait_statement) => {
                let path =
                    module_scope.define(trait_statement.name_text_spanned()?, NameSpace::Type);
                let method_nodes = trait_statement.methods();
                let method_paths = method_nodes
                    .into_iter()
                    .map(|method| {
                        Some((
                            module_scope.define(method.name_text_spanned()?, NameSpace::Term),
                            method,
                        ))
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
                            type_expr: type_expr(&mut parameter_scope, method.ty()?)?,
                            span: method.span(),
                        })
                    })
                    .collect::<Option<Box<[_]>>>()?;
                Statement::Trait {
                    path,
                    parameters,
                    methods,
                }
            }
            ast::Statement::Impl(impl_statement) => {
                let trait_path = match impl_statement.trait_name()? {
                    ast::PathOrIdent::Ident(ident) => {
                        module_scope.query_string(ident.name_text_spanned()?, NameSpace::Type)
                    }
                    ast::PathOrIdent::Path(path) => {
                        module_scope.query_path(
                            Path::new(path.qualifier()?, path.name_text()?).with_span(path.span()),
                            NameSpace::Type,
                        )
                    }
                };

                let arguments = impl_statement
                    .type_args()
                    .into_iter()
                    .map(|arg| type_expr(&mut module_scope, arg))
                    .collect::<Option<Box<[_]>>>()?;

                let mut impl_scope = module_scope.nest_scope();
                let methods = impl_statement
                    .methods()
                    .into_iter()
                    .map(|method| {
                        let method_name = method.name_text_spanned()?;
                        let trait_method =
                            Path::new(module_name.clone(), method_name.inner.clone());
                        impl_scope.query_path(
                            trait_method.clone().with_span(method_name.span),
                            NameSpace::Term,
                        );
                        let impl_path = impl_scope.define(method_name, NameSpace::Term);
                        Some(ImplMethod {
                            trait_method,
                            impl_path,
                            value: term(&mut impl_scope, &wasm_type_defs, logger, method.value()?)?,
                            span: method.span(),
                        })
                    })
                    .collect::<Option<Box<[_]>>>()?;

                Statement::Impl {
                    trait_path,
                    arguments,
                    methods,
                }
            }
            ast::Statement::Wasm(wasm_statement) => {
                Statement::Wasm(wasm::build_toplevel(
                    &wasm_statement.sexpr()?,
                    &module_name,
                    &mut wasm_type_defs,
                    logger,
                    &mut module_scope,
                ))
            }
        };
        statements.push(statement);
    }
    module_scope.report_name_resolution_errors(logger);
    Some(Module {
        name,
        statements: statements.into_boxed_slice(),
    })
}
