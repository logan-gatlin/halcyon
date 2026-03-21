use crate::hc_core::CoreType;
use crate::types::symbol_table::Symbol;
use crate::{
    Spanned,
    WithSpan,
};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDeclKind {
    Named,
    Alias,
}

#[derive(Debug, Clone)]
pub enum TypeExprKind {
    Tuple(Box<[TypeExpr]>),
    Instantiation(Path, Box<[TypeExpr]>),
    ForAll(Box<[Path]>, Box<[TypeExprConstraint]>, Box<TypeExpr>),
    Placeholder,
}

#[derive(Debug, Clone)]
pub struct TypeExprConstraint {
    pub trait_name: Path,
    pub arguments: Box<[TypeExpr]>,
    pub span: Span,
}

impl TypeExprKind {
    /// Instantiation of a type without parameters
    pub fn alias(path: Path) -> Self {
        Self::Instantiation(path, [].into())
    }
}

#[derive(Debug, Clone)]
pub struct TypeExpr {
    #[allow(dead_code)]
    pub comments: String,
    pub kind: TypeExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StructTypeMember {
    Field {
        name: Spanned<String>,
        type_expr: TypeExpr,
    },
    Spread {
        type_expr: TypeExpr,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum TypeDefKind {
    Struct(Box<[StructTypeMember]>),
    Sum(IndexMap<String, TypeExpr>),
    Expr(TypeExpr),
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    #[allow(dead_code)]
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
    logger: &mut FileLogger,
    typedef: ast::TypeDef,
) -> Option<TypeDef> {
    Some(TypeDef {
        comments: String::new(),
        span: typedef.span(),
        kind: match typedef {
            ast::TypeDef::Struct(struct_def) => {
                TypeDefKind::Struct(
                    struct_def
                        .members()
                        .into_iter()
                        .flat_map(|member| {
                            match member {
                                ast::StructTypeMemberDecl::Field(field) => {
                                    let field_name = field.name_text_spanned()?;
                                    super::lint_snake_case_name(
                                        logger,
                                        "Struct field",
                                        &field_name.inner,
                                        field_name.span,
                                    );
                                    Some(StructTypeMember::Field {
                                        name: field_name,
                                        type_expr: type_expr(scope, logger, field.ty()?)?,
                                    })
                                }
                                ast::StructTypeMemberDecl::Spread(spread) => {
                                    Some(StructTypeMember::Spread {
                                        type_expr: type_expr(scope, logger, spread.ty()?)?,
                                        span: spread.span(),
                                    })
                                }
                            }
                        })
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
                                    Some(t) => type_expr(scope, logger, t)?,
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
                TypeDefKind::Expr(type_expr(scope, logger, type_alias.type_expr()?)?)
            }
        },
    })
}

pub fn type_expr(
    scope: &mut impl Scope,
    logger: &mut FileLogger,
    expr: ast::TypeExpr,
) -> Option<TypeExpr> {
    Some(TypeExpr {
        comments: String::new(),
        span: expr.span(),
        kind: match expr {
            ast::TypeExpr::Unit(_) => TypeExprKind::alias(CoreType::Unit.path()),
            ast::TypeExpr::Array(_) => {
                TypeExprKind::Instantiation(CoreType::Array.path(), [].into())
            }
            ast::TypeExpr::Path(path_expr) => {
                let usage_span = path_expr
                    .name_text_spanned()
                    .map_or_else(|| path_expr.span(), |segment| segment.span);
                let path = scope.resolve_path(&path_expr, NameSpace::Type, usage_span)?;
                scope.query_path(path.clone().with_span(usage_span), NameSpace::Type);
                TypeExprKind::alias(path)
            }
            ast::TypeExpr::Ident(ident) => {
                let name = ident.name_text_spanned()?;
                if name.inner == "_" {
                    TypeExprKind::Placeholder
                } else {
                    TypeExprKind::alias(scope.query_string(name, NameSpace::Type))
                }
            }
            ast::TypeExpr::Function(function_type) => {
                TypeExprKind::Instantiation(
                    CoreType::Function.path(),
                    [
                        type_expr(scope, logger, function_type.param_type()?)?,
                        type_expr(scope, logger, function_type.return_type()?)?,
                    ]
                    .into(),
                )
            }
            ast::TypeExpr::Tuple(tuple_type) => {
                let mut fields = tuple_type
                    .fields()
                    .into_iter()
                    .map(|field| type_expr(scope, logger, field))
                    .collect::<Option<Vec<_>>>()?;
                if !tuple_type.is_tuple() {
                    return fields.pop();
                }
                TypeExprKind::Tuple(fields.into())
            }
            ast::TypeExpr::ForAll(forall_type) => {
                let mut inner_scope = scope.nest_scope();
                let params = forall_type
                    .params()
                    .into_iter()
                    .flat_map(|ident| {
                        let name = ident.name_text_spanned()?;
                        Some(inner_scope.define(name, NameSpace::Type))
                    })
                    .collect::<Box<[_]>>();
                let constraints = forall_type
                    .constraints()
                    .into_iter()
                    .map(|constraint| {
                        let trait_name = super::resolve_path_or_ident(
                            &mut inner_scope,
                            constraint.trait_name()?,
                            NameSpace::Trait,
                        )?;
                        let arguments = constraint
                            .args()
                            .into_iter()
                            .map(|arg| type_expr(&mut inner_scope, logger, arg))
                            .collect::<Option<Box<[_]>>>()?;
                        Some(TypeExprConstraint {
                            trait_name,
                            arguments,
                            span: constraint.span(),
                        })
                    })
                    .collect::<Option<Box<[_]>>>()?;
                let body = type_expr(&mut inner_scope, logger, forall_type.body()?)?;
                TypeExprKind::ForAll(params, constraints, body.into())
            }
            ast::TypeExpr::Application(type_application) => {
                let arguments = type_application
                    .args()
                    .into_iter()
                    .map(|arg| type_expr(scope, logger, arg))
                    .collect::<Option<Box<[_]>>>()?;
                let base = type_application.base()?;
                let base_span = base.span();
                TypeExprKind::Instantiation(
                    match base {
                        ast::TypeExpr::Array(_) => CoreType::Array.path(),
                        ast::TypeExpr::Unit(_) => CoreType::Unit.path(),
                        ast::TypeExpr::Path(path_expr) => {
                            let span = path_expr
                                .name_text_spanned()
                                .map_or_else(|| path_expr.span(), |segment| segment.span);
                            let resolved = scope
                                .resolve_path(&path_expr, NameSpace::Type, span)?
                                .with_span(span);
                            scope.query_path(resolved, NameSpace::Type)
                        }
                        ast::TypeExpr::Ident(ident) => {
                            scope.query_string(ident.name_text_spanned()?, NameSpace::Type)
                        }
                        ast::TypeExpr::Function(..)
                        | ast::TypeExpr::Application(..)
                        | ast::TypeExpr::ForAll(..)
                        | ast::TypeExpr::Tuple(..) => {
                            logger
                                .error("Invalid type application")
                                .primary(
                                    "Type applications must start with a type constructor path (for example `Option a`).",
                                    base_span,
                                )
                                .done();
                            return None;
                        }
                    },
                    arguments,
                )
            }
        },
    })
}
