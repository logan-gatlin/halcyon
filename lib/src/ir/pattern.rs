use super::*;

pub type Pattern = Typed<Spanned<PatternKind>>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum PatternKind {
    Hole,
    Name(Path),
    Tuple(Vec<Pattern>),
    Array(ArrayPattern),
    Constructor(Constructor, Box<Pattern>),
    Literal(ConstValue),
    TypeHint(Box<Pattern>, Type),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ArrayPattern {
    Exact(Vec<Pattern>),
    Leading {
        head: Vec<Pattern>,
        tail: Option<Path>,
    },
    Trailing {
        head: Option<Path>,
        tail: Vec<Pattern>,
    },
    LeadingAndTrailing {
        head: Vec<Pattern>,
        tail: Vec<Pattern>,
    },
}

impl Pattern {
    pub fn introduced_names(&self) -> usize {
        let mut count = 0;
        self.clone().visit(|p: &mut Pattern| {
            if let PatternKind::Name(_) = *p.inner {
                count += 1
            } else if let PatternKind::Array(ap) = &*p.inner {
                match ap {
                    ArrayPattern::Leading { tail, .. } => count += tail.is_some() as usize,
                    ArrayPattern::Trailing { head, .. } => count += head.is_some() as usize,
                    _ => {}
                }
            }
        });
        count
    }

    pub fn is_irrefutable(&self) -> bool {
        match &self.inner.inner {
            PatternKind::Hole | PatternKind::Name(_) => true,
            PatternKind::Tuple(pats) => pats.iter().all(|p| p.is_irrefutable()),
            PatternKind::Array(..) => false,
            PatternKind::Constructor(..) => false,
            PatternKind::Literal(const_value) => const_value == &ConstValue::Unit,
            PatternKind::TypeHint(pat, _) => pat.is_irrefutable(),
        }
    }
}

impl Visit<Pattern> for Pattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Pattern)) {
        match &mut *self.inner {
            PatternKind::Hole | PatternKind::Name(_) | PatternKind::Literal(_) => {}
            PatternKind::Array(pat) => pat._visit(f),
            PatternKind::Tuple(items) => items._visit(f),
            PatternKind::Constructor(_, items) => items._visit(f),
            PatternKind::TypeHint(pat, _) => {
                pat._visit(f);
            }
        }
        f(self);
    }
}

impl Visit<Pattern> for ArrayPattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Pattern)) {
        match self {
            ArrayPattern::Exact(array_patterns) => array_patterns._visit(f),
            ArrayPattern::Leading { head, .. } => head._visit(f),
            ArrayPattern::Trailing { tail, .. } => tail._visit(f),
            ArrayPattern::LeadingAndTrailing { head, tail } => {
                head._visit(f);
                tail._visit(f);
            }
        }
    }
}

impl Visit<Type> for Pattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
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

impl Visit<(Path, Type)> for Pattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut (Path, Type))) {
        self.visit(|p: &mut Pattern| {
            if let PatternKind::Name(path) = &mut *p.inner {
                let mut tup = (path.clone(), p.type_.clone());
                f(&mut tup);
                *path = tup.0;
                p.type_ = tup.1;
            } else if let PatternKind::Array(ap) = &mut *p.inner {
                let Type::Array(inner_type) = p.type_.clone() else {
                    panic!()
                };
                match ap {
                    ArrayPattern::Leading {
                        tail: Some(tail), ..
                    } => {
                        let mut tup = (tail.clone(), *inner_type);
                        f(&mut tup);
                        *tail = tup.0;
                        p.type_ = Type::Array(tup.1.into());
                    }
                    ArrayPattern::Trailing {
                        head: Some(head), ..
                    } => {
                        let mut tup = (head.clone(), *inner_type);
                        f(&mut tup);
                        *head = tup.0;
                        p.type_ = Type::Array(tup.1.into());
                    }
                    _ => {}
                }
            }
        })
    }
}
