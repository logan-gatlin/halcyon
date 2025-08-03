#[derive(Debug, Clone)]
pub enum PatternExpressionKind {
    Literal(super::Literal),
    Identifier(String),
    Tuple(Vec<PatternExpression>),
}

pub type PatternExpression = Expression<PatternExpressionKind>;
