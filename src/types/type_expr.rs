use crate::Span;
use crate::ir::{
    Path,
    TypeExpr,
    TypeExprKind,
};

use super::Type;

pub(crate) fn lower_type_expr<E, F>(
    expr: &TypeExpr,
    lower_instantiation: &mut F,
) -> Result<Type, E>
where
    F: FnMut(&Path, Vec<Type>, Span) -> Result<Type, E>,
{
    match &expr.kind {
        TypeExprKind::Tuple(items) => {
            items
                .iter()
                .map(|item| lower_type_expr(item, lower_instantiation))
                .collect::<Result<Vec<_>, _>>()
                .map(Type::Tuple)
        }
        TypeExprKind::Instantiation(path, args) => {
            let arguments = args
                .iter()
                .map(|arg| lower_type_expr(arg, lower_instantiation))
                .collect::<Result<Vec<_>, _>>()?;
            lower_instantiation(path, arguments, expr.span)
        }
    }
}
