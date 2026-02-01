use crate::parse::*;
use crate::{
    Span,
    WithSpan,
};

use super::*;

pub type Pattern = Typed<Spanned<PatternKind>>;

#[derive(Debug, Clone)]
pub enum PatternKind {
    Hole,
    Identifier(Path),
    Tuple(Vec<Pattern>),
    Array {
        starting: Vec<Pattern>,
        glob: Glob,
        ending: Vec<Pattern>,
    },
    Struct(IndexMap<Spanned<String>, Pattern>),
    Constructor(Constructor, Box<Pattern>),
    Immediate(ConstValue),
    TypeHint(Box<Pattern>, Type),
}

/// Glob pattern in array destructuring.
#[derive(Debug, Clone)]
pub enum Glob {
    /// No glob present - exact length match required: `[a, b, c]`
    None,
    /// Unnamed glob - matches any remaining elements: `[a, .., b]`
    Unnamed,
    /// Named glob - captures remaining elements: `[a, ..rest, b]`
    Named(Path),
}

#[derive(Debug, Clone)]
pub enum Constructor {
    SumConstant(usize, Type),
    SumFunction(usize, Type, Type),
    Structure(Type),
}

impl Visit<Type> for Constructor {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        match self {
            Constructor::SumConstant(_, t) => t._visit(f),
            Constructor::SumFunction(_, t1, t2) => {
                t1._visit(f);
                t2._visit(f);
            }
            Constructor::Structure(t) => t._visit(f),
        }
    }
}

impl Glob {
    pub fn is_exact(&self) -> bool {
        matches!(self, Glob::None)
    }

    pub fn name(&self) -> Option<&Path> {
        match self {
            Glob::Named(path) => Some(path),
            _ => None,
        }
    }
}

impl Pattern {
    pub fn introduced_names(&self) -> usize {
        let mut count = 0;
        self.clone().visit(|p: &mut Pattern| {
            if let PatternKind::Identifier(_) = *p.inner {
                count += 1
            } else if let PatternKind::Array { glob, .. } = &*p.inner {
                count += matches!(glob, Glob::Named(_)) as usize
            }
        });
        count
    }

    pub fn find_refutable_pattern(&self) -> Option<Span> {
        match &self.inner.inner {
            PatternKind::Hole | PatternKind::Identifier(_) => None,
            PatternKind::Tuple(pats) => pats.iter().find_map(Pattern::find_refutable_pattern),
            PatternKind::Struct(map) => map.values().find_map(Pattern::find_refutable_pattern),
            PatternKind::Constructor(Constructor::Structure(_), pat) => {
                pat.find_refutable_pattern()
            }
            PatternKind::Array { .. } | PatternKind::Constructor(..) => Some(self.span),
            PatternKind::Immediate(const_value) => {
                if const_value == &ConstValue::Unit {
                    None
                } else {
                    Some(self.span)
                }
            }
            PatternKind::TypeHint(pat, _) => pat.find_refutable_pattern(),
        }
    }

    pub fn is_refutable(&self) -> bool {
        match &self.inner.inner {
            PatternKind::Hole | PatternKind::Identifier(_) => false,
            PatternKind::Tuple(pats) => pats.iter().any(|p| p.is_refutable()),
            PatternKind::Struct(map) => map.values().any(|p| p.is_refutable()),
            PatternKind::Array { .. } => true,
            PatternKind::Constructor(Constructor::Structure(_), pat) => pat.is_refutable(),
            PatternKind::Constructor(..) => true,
            PatternKind::Immediate(const_value) => const_value != &ConstValue::Unit,
            PatternKind::TypeHint(pat, _) => pat.is_refutable(),
        }
    }
}

impl Visit<Pattern> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Pattern),
    ) {
        match &mut *self.inner {
            PatternKind::Hole | PatternKind::Identifier(_) | PatternKind::Immediate(_) => {}
            PatternKind::Array {
                starting, ending, ..
            } => {
                starting._visit(f);
                ending._visit(f);
            }
            PatternKind::Tuple(items) => items._visit(f),
            PatternKind::Struct(map) => map._visit(f),
            PatternKind::Constructor(_, items) => items._visit(f),
            PatternKind::TypeHint(pat, _) => {
                pat._visit(f);
            }
        }
        f(self);
    }
}

impl Visit<Type> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        self.visit(|p: &mut Pattern| {
            match &mut p.inner.inner {
                PatternKind::Constructor(c, _) => c._visit(f),
                PatternKind::TypeHint(p, t) => {
                    p._visit(f);
                    t._visit(f);
                }
                _ => {}
            }
            p.type_._visit(f);
        })
    }
}

impl Visit<Path> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Path),
    ) {
        self.visit(|p: &mut Pattern| {
            if let PatternKind::Identifier(id) = &mut p.inner.inner {
                f(id)
            } else if let PatternKind::Array {
                glob: Glob::Named(glob),
                ..
            } = &mut p.inner.inner
            {
                f(glob)
            }
        });
    }
}

impl Visit<(Path, Type)> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut (Path, Type)),
    ) {
        self.visit(|p: &mut Pattern| {
            match &mut p.inner.inner {
                PatternKind::Identifier(path) => {
                    let mut tup = (path.clone(), p.type_.clone());
                    f(&mut tup);
                    *path = tup.0;
                    p.type_ = tup.1;
                }
                PatternKind::Array {
                    glob: Glob::Named(glob),
                    ..
                } => {
                    let mut tup = (glob.clone(), p.type_.clone());
                    f(&mut tup);
                    *glob = tup.0;
                    p.type_ = tup.1;
                }
                _ => {}
            }
        })
    }
}

impl<'a> super::build_ir::Builder<'a> {
    pub fn pattern(
        &mut self,
        pat: PatternExpression,
        is_global: bool,
    ) -> Option<Pattern> {
        use PatternExpressionKind::*;
        let span = pat.span;
        Some(
            match pat.inner {
                Literal(literal) => PatternKind::Immediate(self.literal(literal.with_span(span))?),
                Identifier(name) if name == "_" => PatternKind::Hole,
                Identifier(name) => {
                    if let Ok(path) =
                        self.query_name(name.clone().with_span(span), NameSpace::Constructor)
                    {
                        let cons = self.symbols.get_constructor(&path).clone();
                        PatternKind::Constructor(
                            cons,
                            PatternKind::Immediate(ConstValue::Unit)
                                .with_span(span)
                                .with_type(Type::Any)
                                .into(),
                        )
                    } else {
                        PatternKind::Identifier(self.define_name(
                            name.with_span(span),
                            NameSpace::Term,
                            is_global,
                        )?)
                    }
                }
                Tuple(pats) => {
                    PatternKind::Tuple(
                        pats.into_iter()
                            .map(|p| self.pattern(p, is_global))
                            .collect::<Option<_>>()?,
                    )
                }
                Structure(map) => {
                    PatternKind::Struct(
                        map.into_iter()
                            .map(|(k, v)| self.pattern(v, is_global).map(|v| (k, v)))
                            .collect::<Option<_>>()?,
                    )
                }
                Array(pats) => {
                    let mut starting = vec![];
                    let mut glob = Glob::None;
                    let mut ending = vec![];
                    let glob_err = |this: &mut Self, span| {
                        this.logger
                            .error("Multiple glob patterns in an array are ambiguous")
                            .primary("This glob is not allowed", span)
                            .done();
                    };
                    for p in pats {
                        match p {
                            ParsedArrayPattern::Pattern(pat) => {
                                let pat = self.pattern(pat, is_global)?;
                                if glob.is_exact() {
                                    starting.push(pat);
                                } else {
                                    ending.push(pat)
                                }
                            }
                            ParsedArrayPattern::ExpansionAssign(id) => {
                                if !glob.is_exact() {
                                    glob_err(self, id.span);
                                } else {
                                    glob = Glob::Named(self.define_name(
                                        id,
                                        NameSpace::Term,
                                        is_global,
                                    )?);
                                }
                            }
                            ParsedArrayPattern::Expansion(span) => {
                                if !glob.is_exact() {
                                    glob_err(self, span);
                                } else {
                                    glob = Glob::Unnamed;
                                }
                            }
                        }
                    }
                    PatternKind::Array {
                        starting,
                        glob,
                        ending,
                    }
                }
                Constructor((a, b), pat) => {
                    let path = if let Some(b) = b {
                        let path = Path::new(a, b);
                        self.query_path(&path.clone().with_span(span), NameSpace::Constructor)
                            .done()?;
                        path
                    } else {
                        self.query_name(a.with_span(span), NameSpace::Constructor)
                            .done()?
                    };
                    let cons = self.symbols.get_constructor(&path).clone();
                    let pat = self.pattern(*pat, is_global)?;
                    PatternKind::Constructor(cons, pat.into())
                }
                ModulePath(a, b) => {
                    let path = Path::new(a, b);
                    self.query_path(&path.clone().with_span(span), NameSpace::Constructor)
                        .done()?;
                    let cons = self.symbols.get_constructor(&path).clone();
                    PatternKind::Constructor(
                        cons,
                        PatternKind::Immediate(ConstValue::Unit)
                            .with_span(span)
                            .with_type(Type::Any)
                            .into(),
                    )
                }
                TypeHint(pat, type_) => {
                    PatternKind::TypeHint(
                        self.pattern(*pat, is_global)?.into(),
                        self.type_expr(*type_)?,
                    )
                }
            }
            .with_span(span)
            .with_type(Type::Any),
        )
    }
}
