use super::*;
use crate::parse::*;
use crate::{
    IntoLog,
    WithSpan,
};

impl<'a> super::build_ir::Builder<'a> {
    pub fn type_definition(
        &mut self,
        Spanned { inner: path, span }: Spanned<Path>,
        Spanned {
            inner: def,
            span: def_span,
        }: TypeDefinition,
    ) {
        match def {
            TypeDefinitionKind::TypeFunction { arguments, body } => todo!(),
            TypeDefinitionKind::Structure { lhs, rhs } => todo!(),
            TypeDefinitionKind::Sum {
                variant_names,
                variant_types,
            } => todo!(),
            TypeDefinitionKind::Expression(spanned) => todo!(),
        }
    }
    pub fn type_expr(
        &mut self,
        expr: TypeExpression,
    ) -> Option<Type> {
        use TypeExpressionKind::*;
        let span = expr.span;
        Some(match expr.inner {
            Call(a, b) => todo!(),
            Identifier(id) => {
                let path = self
                    .query_name(id.with_span(span), NameSpace::Type)
                    .done()?;
                self.symbols
                    .get_type(&path)
                    .clone()
                    .instantiate(&[], self.symbols.fresh_tv_source())
                    .into_log(self.logger, span)?
            }
            ModulePath(a, b) => {
                let path = Path::new(a, b);
                self.query_path(&path.clone().with_span(span), NameSpace::Type)
                    .done()?;
                self.symbols
                    .get_type(&path)
                    .clone()
                    .instantiate(&[], self.symbols.fresh_tv_source())
                    .into_log(self.logger, span)?
            }
            Product(ts) => {
                Type::Product(
                    ts.into_iter()
                        .map(|t| self.type_expr(t))
                        .collect::<Option<_>>()?,
                )
            }
            Function(a, b) => {
                Type::Function(self.type_expr(*a)?.into(), self.type_expr(*b)?.into())
            }
            Array(t) => Type::Array(self.type_expr(*t)?.into()),
            Unit => Type::Unit,
        })
    }
}
