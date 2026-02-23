mod names;
mod patterns;
mod terms;
mod types;

pub use names::*;
pub use patterns::*;
pub use terms::*;
pub use types::*;

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
    },
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

pub fn module(
    module_node: ast::Module,
    logger: &mut FileLogger,
) -> Option<Module<()>> {
    let name = module_node.name_text()?;
    let mut module_scope = ModuleScope::new(name.clone());
    Some(Module {
        name,
        statements: module_node
            .statements()
            .into_iter()
            .flat_map(|s| {
                let comments = s.leading_comment_text();
                match s {
                    ast::Statement::Let(let_statement) => {
                        Some(Statement::Term(Term {
                            comments,
                            kind: TermKind::Let {
                                assignee: pattern(
                                    &mut module_scope,
                                    logger,
                                    let_statement.pattern()?,
                                )?,
                                value: term(&mut module_scope, logger, let_statement.value()?)?
                                    .into(),
                                scope: ScopeKind::Global,
                                then: Term::unit().into(),
                                else_: Term::unreachable().into(),
                            },
                            span: let_statement.span(),
                            type_: (),
                        }))
                    }
                    ast::Statement::Type(type_statement) => {
                        let path = module_scope
                            .define(type_statement.name_text_spanned()?, NameSpace::Type);
                        let mut parameter_scope = module_scope.nest_scope();
                        Some(Statement::Type {
                            path,
                            parameters: type_statement
                                .type_params()
                                .into_iter()
                                .map(|param| parameter_scope.define(param, NameSpace::Type))
                                .collect(),
                            def: typedef(&mut parameter_scope, type_statement.type_def()?)?,
                        })
                    }
                }
            })
            .collect(),
    })
}
