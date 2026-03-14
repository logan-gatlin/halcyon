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
            ast::Pattern::Ident(pat_ident) if pat_ident.name_text().is_some_and(|n| n == "_") => {
                PatternKind::Hole
            }
            ast::Pattern::Ident(pat_ident) => {
                let name = pat_ident.name_text_spanned()?;
                if let Some(path) =
                    scope.query_string_if_defined(name.clone(), NameSpace::Constructor)
                {
                    PatternKind::ConstConstructor(path)
                } else {
                    super::lint_snake_case_name(logger, "Let binding", &name.inner, name.span);
                    PatternKind::Identifier(scope.define(name, NameSpace::Term))
                }
            }
            ast::Pattern::Literal(pat_literal) => {
                PatternKind::Immediate(immediate(logger, pat_literal)?)
            }
            ast::Pattern::Unit(_) => PatternKind::Immediate(ImmediateValue::Unit),
            ast::Pattern::Tuple(pat_tuple) => {
                let mut patterns = pat_tuple
                    .patterns()
                    .into_iter()
                    .map(|pattern_node| pattern(scope, logger, pattern_node))
                    .collect::<Option<Vec<_>>>()?;
                if !pat_tuple.is_tuple() {
                    let inner = patterns.pop()?;
                    return Some(inner);
                }
                PatternKind::Tuple(patterns.into())
            }
            ast::Pattern::Array(pat_array) => {
                let mut starting = Vec::new();
                let mut glob = Glob::None;
                let mut ending = Vec::new();
                for child in pat_array.syntax().children() {
                    if let Some(rest) = ast::PatRest::cast(child.clone()) {
                        if !matches!(glob, Glob::None) {
                            logger
                                .error("Invalid array pattern")
                                .primary(
                                    "Array patterns may contain at most one `..` rest pattern.",
                                    rest.span(),
                                )
                                .done();
                            return None;
                        }
                        glob = match rest.binding_name_spanned() {
                            Some(name) => {
                                super::lint_snake_case_name(
                                    logger,
                                    "Let binding",
                                    &name.inner,
                                    name.span,
                                );
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
                            let field_name = f.name_text_spanned()?;
                            let field_pattern = match f.pattern() {
                                Some(inner) => pattern(scope, logger, inner)?,
                                None if !pat_field_has_equals(&f) => {
                                    super::lint_snake_case_name(
                                        logger,
                                        "Let binding",
                                        &field_name.inner,
                                        field_name.span,
                                    );
                                    Pattern {
                                        comments: String::new(),
                                        kind: PatternKind::Identifier(
                                            scope.define(field_name.clone(), NameSpace::Term),
                                        ),
                                        span: field_name.span,
                                        type_: (),
                                    }
                                }
                                None => return None,
                            };
                            Some((field_name, field_pattern))
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
                        ast::PathOrIdent::Path(pat_path) => {
                            let resolved = scope
                                .resolve_path(&pat_path, NameSpace::Constructor, pat_path.span())?
                                .with_span(pat_path.span());
                            scope.query_path(resolved, NameSpace::Constructor)
                        }
                    },
                    pattern(scope, logger, constructor.payload()?)?.into(),
                )
            }
            ast::Pattern::TypeHint(pat_type_hint) => {
                PatternKind::TypeHint(
                    pattern(scope, logger, pat_type_hint.pattern()?)?.into(),
                    type_expr(scope, logger, pat_type_hint.ty()?)?,
                )
            }
            ast::Pattern::Path(pat_path) => {
                let resolved = scope
                    .resolve_path(&pat_path, NameSpace::Constructor, pat_path.span())?
                    .with_span(pat_path.span());
                PatternKind::ConstConstructor(scope.query_path(resolved, NameSpace::Constructor))
            }
        },
    })
}

fn pat_field_has_equals(field: &ast::PatField) -> bool {
    field
        .syntax()
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| token.kind() == SyntaxKind::EQUAL)
}
