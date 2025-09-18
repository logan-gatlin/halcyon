use super::*;
#[derive(Debug, Clone, sx::SXRepr)]
pub enum PatternExpressionKind {
    Literal(super::Literal),
    Identifier(String),
    ModulePath(Vec<String>),
    Tuple(Vec<PatternExpression>),
    Array(Box<ParsedArrayPattern>),
    Constructor(Vec<String>, Box<PatternExpression>),
    TypeHint(Box<PatternExpression>, Box<TypeExpression>),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ParsedArrayPattern {
    Exact(Vec<PatternExpression>),
    Leading {
        head: Vec<PatternExpression>,
        tail: Option<String>,
    },
    Trailing {
        head: Option<String>,
        tail: Vec<PatternExpression>,
    },
    LeadingAndTrailing {
        head: Vec<PatternExpression>,
        tail: Vec<PatternExpression>,
    },
}

pub type PatternExpression = Expression<PatternExpressionKind>;
use PatternExpressionKind as e;
