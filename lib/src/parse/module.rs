use super::*;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ModuleExpressionKind {
    Error,
    Let {
        assignee: PatternExpression,
        value: Box<ValueExpression>,
    },
    Do(Box<ValueExpression>),
    Type {
        assignee: String,
        assignee_span: Span,
        value: Box<TypeDefinition>,
    },
    Import {
        name: String,
        type_: Box<TypeExpression>,
        major: String,
        minor: String,
    },
}

pub type ModuleExpression = Expression<ModuleExpressionKind>;

#[allow(dead_code)]
#[derive(Debug, Clone, sx::SXRepr)]
pub struct _ParsedModule {
    pub name: Spanned<String>,
    pub contents: Vec<ModuleExpression>,
}

pub type ParsedModule = Spanned<_ParsedModule>;
