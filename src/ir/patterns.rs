use crate::WithSpan;

use super::*;

/// Glob pattern in array destructuring.
#[derive(Debug, Clone)]
pub enum Glob {
    /// No glob present - exact length match required: `[a, b, c]`
    None,
    /// Unnamed glob - matches any remaining elements: `[a, .., b]`
    Anonymous,
    /// Named glob - captures remaining elements: `[a, ..rest, b]`
    Named(Path),
}

#[derive(Debug, Clone, Default)]
pub enum PatternKind<T> {
    #[default]
    Hole,
    Identifier(Path),
    ConstConstructor(Path),
    Constructor(Path, Box<Pattern<T>>),
    Tuple(Box<[Pattern<T>]>),
    Array {
        starting: Box<[Pattern<T>]>,
        glob: Glob,
        ending: Box<[Pattern<T>]>,
    },
    Struct(IndexMap<Spanned<String>, Pattern<T>>),
    Immediate(ImmediateValue),
    TypeHint(Box<Pattern<T>>, TypeExpr),
}

#[derive(Debug, Clone, Default)]
pub struct Pattern<T> {
    pub comments: String,
    pub kind: PatternKind<T>,
    pub span: Span,
    pub type_: T,
}

pub type UntypedPattern = Pattern<()>;

pub fn pattern(
    scope: &mut impl Scope,
    logger: &mut FileLogger,
    pat: ast::Pattern,
) -> Option<UntypedPattern> {
    Some(Pattern {
        comments: String::new(),
        span: pat.span(),
        type_: (),
        kind: match pat {
            ast::Pattern::Ident(pat_ident) => {
                PatternKind::Identifier(
                    scope.define(pat_ident.name_text_spanned()?, NameSpace::Term),
                )
            }
            ast::Pattern::Literal(pat_literal) => {
                PatternKind::Immediate(immediate(logger, pat_literal)?)
            }
            ast::Pattern::Unit(_) => PatternKind::Immediate(ImmediateValue::Unit),
            ast::Pattern::Tuple(pat_tuple) => {
                PatternKind::Tuple(
                    pat_tuple
                        .patterns()
                        .into_iter()
                        .map(|p| pattern(scope, logger, p))
                        .collect::<Option<_>>()?,
                )
            }
            ast::Pattern::Array(pat_array) => {
                let mut starting = Vec::new();
                let mut glob = Glob::None;
                let mut ending = Vec::new();
                for child in pat_array.syntax().children() {
                    if let Some(rest) = ast::PatRest::cast(child.clone()) {
                        if !matches!(glob, Glob::None) {
                            return None;
                        }
                        glob = match rest.binding_token() {
                            Some(token) => {
                                let name = token
                                    .text()
                                    .to_string()
                                    .with_span(token.text_range().into());
                                Glob::Named(scope.define(name, NameSpace::Term))
                            }
                            None => Glob::Anonymous,
                        };
                        continue;
                    }
                    if let Some(pattern_node) = ast::Pattern::cast(child) {
                        let pat = pattern(scope, logger, pattern_node)?;
                        if matches!(glob, Glob::None) {
                            starting.push(pat);
                        } else {
                            ending.push(pat);
                        }
                    }
                }
                PatternKind::Array {
                    starting: starting.into_boxed_slice(),
                    glob,
                    ending: ending.into_boxed_slice(),
                }
            }
            ast::Pattern::Struct(pat_struct) => {
                PatternKind::Struct(
                    pat_struct
                        .fields()
                        .into_iter()
                        .map(|f| {
                            Some((
                                f.name_text_spanned()?,
                                pattern(scope, logger, f.pattern()?)?,
                            ))
                        })
                        .collect::<Option<_>>()?,
                )
            }
            ast::Pattern::Constructor(constructor) => {
                PatternKind::Constructor(
                    match constructor.head()? {
                        ast::PathOrIdent::Ident(pat_ident) => {
                            scope.query_string(
                                pat_ident.name_text_spanned()?,
                                NameSpace::Constructor,
                            )
                        }
                        ast::PathOrIdent::Path(pat_path) => pat_path.try_into().ok()?,
                    },
                    pattern(scope, logger, constructor.payload()?)?.into(),
                )
            }
            ast::Pattern::TypeHint(pat_type_hint) => {
                PatternKind::TypeHint(
                    pattern(scope, logger, pat_type_hint.pattern()?)?.into(),
                    type_expr(scope, pat_type_hint.ty()?)?,
                )
            }
            ast::Pattern::Path(pat_path) => {
                PatternKind::ConstConstructor(
                    scope.query_path(
                        Path::new(pat_path.qualifier()?.text(), pat_path.name_text()?)
                            .clone()
                            .with_span(pat_path.span()),
                        NameSpace::Constructor,
                    ),
                )
            }
        },
    })
}
