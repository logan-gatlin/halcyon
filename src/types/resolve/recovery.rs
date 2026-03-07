use super::super::unify::UnificationTable;
use super::{
    Pattern,
    PatternKind,
    Term,
    TermKind,
    Type,
};

pub(super) fn normalize_term_types(
    term: Term<Type>,
    table: &mut UnificationTable,
) -> Term<Type> {
    map_term_types(&term, &mut |type_| table.normalize(type_))
}

pub(super) fn fallback_term(term: &Term<()>) -> Term<Type> {
    map_term_types(term, &mut |_| Type::Unit)
}

// Shared recursive rebuild for both normalization and fallback recovery so the
// two paths stay structurally aligned.
fn map_term_types<T, U>(
    term: &Term<T>,
    map_type: &mut impl FnMut(&T) -> U,
) -> Term<U> {
    let kind = match &term.kind {
        TermKind::Let {
            assignee,
            scope,
            value,
            then,
            else_,
        } => {
            TermKind::Let {
                assignee: map_pattern_types(assignee, map_type),
                scope: *scope,
                value: Box::new(map_term_types(value, map_type)),
                then: Box::new(map_term_types(then, map_type)),
                else_: Box::new(map_term_types(else_, map_type)),
            }
        }
        TermKind::Immediate(value) => TermKind::Immediate(value.clone()),
        TermKind::Identifier(path) => TermKind::Identifier(path.clone()),
        TermKind::Tuple(items) => {
            TermKind::Tuple(
                items
                    .iter()
                    .map(|item| map_term_types(item, map_type))
                    .collect(),
            )
        }
        TermKind::Struct(fields) => {
            TermKind::Struct(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), map_term_types(value, map_type)))
                    .collect(),
            )
        }
        TermKind::Field { of, index } => {
            TermKind::Field {
                of: Box::new(map_term_types(of, map_type)),
                index: index.clone(),
            }
        }
        TermKind::Function {
            parameter_name,
            parameter_type,
            captures,
            body,
        } => {
            TermKind::Function {
                parameter_name: parameter_name.clone(),
                parameter_type: parameter_type.clone(),
                captures: captures
                    .iter()
                    .map(|(path, type_)| (path.clone(), map_type(type_)))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                body: Box::new(map_term_types(body, map_type)),
            }
        }
        TermKind::InlineWasm {
            asserted_type,
            definitions,
            instructions,
        } => {
            TermKind::InlineWasm {
                asserted_type: asserted_type.clone(),
                definitions: definitions.clone(),
                instructions: instructions.clone(),
            }
        }
        TermKind::Call { callee, argument } => {
            TermKind::Call {
                callee: Box::new(map_term_types(callee, map_type)),
                argument: Box::new(map_term_types(argument, map_type)),
            }
        }
        TermKind::Semicolon(left, right) => {
            TermKind::Semicolon(
                Box::new(map_term_types(left, map_type)),
                Box::new(map_term_types(right, map_type)),
            )
        }
        TermKind::Unreachable => TermKind::Unreachable,
    };
    Term {
        comments: term.comments.clone(),
        kind,
        span: term.span,
        type_: map_type(&term.type_),
    }
}

fn map_pattern_types<T, U>(
    pattern: &Pattern<T>,
    map_type: &mut impl FnMut(&T) -> U,
) -> Pattern<U> {
    let kind = match &pattern.kind {
        PatternKind::Hole => PatternKind::Hole,
        PatternKind::Identifier(path) => PatternKind::Identifier(path.clone()),
        PatternKind::ConstConstructor(path) => PatternKind::ConstConstructor(path.clone()),
        PatternKind::Constructor(path, payload) => {
            PatternKind::Constructor(path.clone(), Box::new(map_pattern_types(payload, map_type)))
        }
        PatternKind::Tuple(items) => {
            PatternKind::Tuple(
                items
                    .iter()
                    .map(|item| map_pattern_types(item, map_type))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            PatternKind::Array {
                starting: starting
                    .iter()
                    .map(|item| map_pattern_types(item, map_type))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                glob: glob.clone(),
                ending: ending
                    .iter()
                    .map(|item| map_pattern_types(item, map_type))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            }
        }
        PatternKind::Struct(fields) => {
            PatternKind::Struct(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), map_pattern_types(value, map_type)))
                    .collect(),
            )
        }
        PatternKind::Immediate(value) => PatternKind::Immediate(value.clone()),
        PatternKind::TypeHint(inner, type_expr) => {
            PatternKind::TypeHint(
                Box::new(map_pattern_types(inner, map_type)),
                type_expr.clone(),
            )
        }
    };
    Pattern {
        comments: pattern.comments.clone(),
        kind,
        span: pattern.span,
        type_: map_type(&pattern.type_),
    }
}
