//! Shared recovery helpers for resolve failures.

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
    remap_term_types(&term, &mut |type_| table.normalize(type_))
}

pub(super) fn fallback_term(term: &Term<()>) -> Term<Type> {
    remap_term_types(term, &mut |_| Type::Unit)
}

// Shared recursive rebuild for both normalization and fallback recovery so the
// two paths stay structurally aligned.
fn remap_term_types<T, U>(
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
                assignee: remap_pattern_types(assignee, map_type),
                scope: *scope,
                value: Box::new(remap_term_types(value, map_type)),
                then: Box::new(remap_term_types(then, map_type)),
                else_: Box::new(remap_term_types(else_, map_type)),
            }
        }
        TermKind::Immediate(value) => TermKind::Immediate(value.clone()),
        TermKind::Identifier(path) => TermKind::Identifier(path.clone()),
        TermKind::Tuple(items) => {
            TermKind::Tuple(
                items
                    .iter()
                    .map(|item| remap_term_types(item, map_type))
                    .collect(),
            )
        }
        TermKind::Struct(fields) => {
            TermKind::Struct(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), remap_term_types(value, map_type)))
                    .collect(),
            )
        }
        TermKind::Field { of, index } => {
            TermKind::Field {
                of: Box::new(remap_term_types(of, map_type)),
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
                body: Box::new(remap_term_types(body, map_type)),
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
                callee: Box::new(remap_term_types(callee, map_type)),
                argument: Box::new(remap_term_types(argument, map_type)),
            }
        }
        TermKind::Semicolon(left, right) => {
            TermKind::Semicolon(
                Box::new(remap_term_types(left, map_type)),
                Box::new(remap_term_types(right, map_type)),
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

fn remap_pattern_types<T, U>(
    pattern: &Pattern<T>,
    map_type: &mut impl FnMut(&T) -> U,
) -> Pattern<U> {
    let kind = match &pattern.kind {
        PatternKind::Hole => PatternKind::Hole,
        PatternKind::Identifier(path) => PatternKind::Identifier(path.clone()),
        PatternKind::ConstConstructor(path) => PatternKind::ConstConstructor(path.clone()),
        PatternKind::Constructor(path, payload) => {
            PatternKind::Constructor(
                path.clone(),
                Box::new(remap_pattern_types(payload, map_type)),
            )
        }
        PatternKind::Tuple(items) => {
            PatternKind::Tuple(
                items
                    .iter()
                    .map(|item| remap_pattern_types(item, map_type))
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
                    .map(|item| remap_pattern_types(item, map_type))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                glob: glob.clone(),
                ending: ending
                    .iter()
                    .map(|item| remap_pattern_types(item, map_type))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            }
        }
        PatternKind::Struct(fields) => {
            PatternKind::Struct(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), remap_pattern_types(value, map_type)))
                    .collect(),
            )
        }
        PatternKind::Immediate(value) => PatternKind::Immediate(value.clone()),
        PatternKind::TypeHint(inner, type_expr) => {
            PatternKind::TypeHint(
                Box::new(remap_pattern_types(inner, map_type)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        Glob,
        ImmediateValue,
        Path,
        ScopeKind,
    };
    use crate::{
        Span,
        WithSpan,
    };

    fn untyped_pattern(kind: PatternKind<()>) -> Pattern<()> {
        Pattern {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    fn untyped_term(kind: TermKind<()>) -> Term<()> {
        Term {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }

    #[test]
    fn fallback_term_maps_all_types_to_unit_and_preserves_shape() {
        let input = untyped_term(TermKind::Let {
            assignee: untyped_pattern(PatternKind::Array {
                starting: [untyped_pattern(PatternKind::Identifier(Path::new(
                    "demo", "head",
                )))]
                .into(),
                glob: Glob::Named(Path::new("demo", "rest")),
                ending: [untyped_pattern(PatternKind::Identifier(Path::new(
                    "demo", "tail",
                )))]
                .into(),
            }),
            scope: ScopeKind::Local,
            value: Box::new(untyped_term(TermKind::Tuple(vec![
                untyped_term(TermKind::Immediate(ImmediateValue::Integer(1))),
                untyped_term(TermKind::Immediate(ImmediateValue::Integer(2))),
            ]))),
            then: Box::new(untyped_term(TermKind::Function {
                parameter_name: Path::new("demo", "x").with_span(Span::Generated),
                parameter_type: None,
                captures: [(Path::new("demo", "captured"), ())].into(),
                body: Box::new(untyped_term(TermKind::Identifier(Path::new("demo", "x")))),
            })),
            else_: Box::new(untyped_term(TermKind::Unreachable)),
        });

        let recovered = fallback_term(&input);
        assert_eq!(recovered.type_, Type::Unit);

        let TermKind::Let {
            assignee,
            value,
            then,
            else_,
            ..
        } = recovered.kind
        else {
            panic!("expected let term");
        };
        assert_eq!(assignee.type_, Type::Unit);
        assert_eq!(value.type_, Type::Unit);
        assert_eq!(then.type_, Type::Unit);
        assert_eq!(else_.type_, Type::Unit);

        let TermKind::Function { captures, .. } = then.kind else {
            panic!("expected function term");
        };
        assert_eq!(
            captures,
            [(Path::new("demo", "captured"), Type::Unit)].into()
        );
    }

    #[test]
    fn normalize_term_types_prunes_meta_variables_everywhere() {
        let mut table = UnificationTable::default();
        let meta = table.new_meta(0);
        let typed_term = Term {
            comments: String::new(),
            kind: TermKind::Function {
                parameter_name: Path::new("demo", "x").with_span(Span::Generated),
                parameter_type: None,
                captures: [(Path::new("demo", "captured"), meta.clone())].into(),
                body: Box::new(Term {
                    comments: String::new(),
                    kind: TermKind::Identifier(Path::new("demo", "x")),
                    span: Span::Generated,
                    type_: meta.clone(),
                }),
            },
            span: Span::Generated,
            type_: meta.clone(),
        };

        table
            .unify(&meta, &Type::Integer)
            .expect("meta binding should succeed");

        let normalized = normalize_term_types(typed_term, &mut table);
        assert_eq!(normalized.type_, Type::Integer);

        let TermKind::Function { captures, body, .. } = normalized.kind else {
            panic!("expected function term");
        };
        assert_eq!(
            captures,
            [(Path::new("demo", "captured"), Type::Integer)].into()
        );
        assert_eq!(body.type_, Type::Integer);
    }
}
