use super::*;

pub type TypeDefinition = Expression<TypeDefinitionKind>;
pub type TypeExpression = Expression<TypeExpressionKind>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum TypeDefinitionKind {
    TypeFunction {
        arguments: Vec<String>,
        body: Box<TypeDefinition>,
    },
    Structure {
        lhs: Vec<String>,
        rhs: Vec<TypeExpression>,
    },
    Sum {
        variant_names: Vec<String>,
        variant_types: Vec<Option<TypeExpression>>,
    },
    Expression(TypeExpression),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum TypeExpressionKind {
    Function(Box<TypeExpression>, Box<TypeExpression>),
    Call(Box<TypeExpression>, Box<TypeExpression>),
    Identifier(String),
    Product(Vec<TypeExpression>),
    ModulePath(Vec<String>),
    Array(Box<TypeExpression>),
    Unit,
}
