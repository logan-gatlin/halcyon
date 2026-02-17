mod names;
mod patterns;
mod terms;
mod types;

pub use names::*;
use patterns::*;
use terms::*;
use types::*;

use crate::new_ir::names::ModuleScope;
use crate::parse_lossless::SyntaxKind;
use crate::parse_lossless::ast::{
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
pub enum Statement {
    Term(UntypedTerm),
    Type {
        path: Path,
        parameters: Box<[Path]>,
        def: TypeDef,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Module {
    name: String,
    statements: Box<[Statement]>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            statements: Default::default(),
        }
    }
}

pub fn lower_module(
    module_node: ast::Module,
    logger: &mut FileLogger,
) -> Option<Module> {
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
                                assignee: pattern(&mut module_scope, let_statement.pattern()?)?,
                                value: term(&mut module_scope, let_statement.value()?)?.into(),
                                scope: ScopeKind::Global,
                                then: Term::unit().into(),
                                else_: Term::unreachable().into(),
                            },
                            span: let_statement.span(),
                            type_: (),
                        }))
                    }
                    ast::Statement::Type(type_statement) => {
                        module_scope.define(type_statement.name_text_spanned()?, NameSpace::Type);
                        let mut parameter_scope = module_scope.nest_scope();
                        todo!()
                    }
                }
            })
            .collect(),
    })
}
