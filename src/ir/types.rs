use crate::WithSpan;
use crate::hc_core::CoreType;
use crate::types::symbol_table::Symbol;

use super::*;

#[derive(Debug, Clone)]
pub enum TypeExprKind {
    Tuple(Box<[TypeExpr]>),
    Instantiation(Path, Box<[TypeExpr]>),
}

impl TypeExprKind {
    /// Instantiation of a type without parameters
    pub fn alias(path: Path) -> Self {
        Self::Instantiation(path, [].into())
    }
}

#[derive(Debug, Clone)]
pub struct TypeExpr {
    pub comments: String,
    pub kind: TypeExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeDefKind {
    Struct(IndexMap<String, TypeExpr>),
    Sum(IndexMap<String, TypeExpr>),
    Expr(TypeExpr),
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    comments: String,
    kind: TypeDefKind,
    span: Span,
}

impl TypeDef {
    pub fn kind(&self) -> &TypeDefKind {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

pub fn typedef(
    scope: &mut impl Scope,
    typedef: ast::TypeDef,
) -> Option<TypeDef> {
    Some(TypeDef {
        comments: String::new(),
        span: typedef.span(),
        kind: match typedef {
            ast::TypeDef::Struct(struct_def) => {
                TypeDefKind::Struct(
                    struct_def
                        .fields()
                        .into_iter()
                        .flat_map(|f| Some((f.name_text()?, type_expr(scope, f.ty()?)?)))
                        .collect(),
                )
            }
            ast::TypeDef::Sum(sum_def) => {
                TypeDefKind::Sum(
                    sum_def
                        .variants()
                        .into_iter()
                        .flat_map(|v| {
                            Some((
                                v.name_text()?,
                                match v.payload_type() {
                                    Some(t) => type_expr(scope, t)?,
                                    None => {
                                        TypeExpr {
                                            kind: TypeExprKind::alias(CoreType::Unit.path()),
                                            comments: Default::default(),
                                            span: Default::default(),
                                        }
                                    }
                                },
                            ))
                        })
                        .collect(),
                )
            }
            ast::TypeDef::Alias(type_alias) => {
                TypeDefKind::Expr(type_expr(scope, type_alias.type_expr()?)?)
            }
        },
    })
}

pub fn type_expr(
    scope: &mut impl Scope,
    expr: ast::TypeExpr,
) -> Option<TypeExpr> {
    Some(TypeExpr {
        comments: String::new(),
        span: expr.span(),
        kind: match expr {
            ast::TypeExpr::Unit(_) => TypeExprKind::alias(Path::core("unit")),
            ast::TypeExpr::Array(_) => {
                TypeExprKind::Instantiation(CoreType::Array.path(), [].into())
            }
            ast::TypeExpr::Path(path_expr) => {
                let span = path_expr.span();
                let path = Path::try_from(path_expr).ok()?;
                scope.query_path(path.clone().with_span(span), NameSpace::Type);
                TypeExprKind::alias(path)
            }
            ast::TypeExpr::Ident(ident) => {
                TypeExprKind::alias(scope.query_string(ident.name_text_spanned()?, NameSpace::Type))
            }
            ast::TypeExpr::Function(function_type) => {
                TypeExprKind::Instantiation(
                    CoreType::Function.path(),
                    [
                        type_expr(scope, function_type.param_type()?)?,
                        type_expr(scope, function_type.return_type()?)?,
                    ]
                    .into(),
                )
            }
            ast::TypeExpr::Tuple(tuple_type) => {
                TypeExprKind::Tuple(
                    tuple_type
                        .fields()
                        .into_iter()
                        .flat_map(|f| type_expr(scope, f))
                        .collect(),
                )
            }
            ast::TypeExpr::Application(type_application) => {
                TypeExprKind::Instantiation(
                    match type_application.base()? {
                        ast::TypeExpr::Array(_) => CoreType::Array.path(),
                        ast::TypeExpr::Unit(_) => CoreType::Unit.path(),
                        ast::TypeExpr::Path(path_expr) => path_expr.try_into().ok()?,
                        ast::TypeExpr::Ident(_) => todo!(),
                        ast::TypeExpr::Function(..)
                        | ast::TypeExpr::Application(..)
                        | ast::TypeExpr::Tuple(..) => {
                            // TODO report error
                            return None;
                        }
                    },
                    [].into(),
                )
            }
        },
    })
}
