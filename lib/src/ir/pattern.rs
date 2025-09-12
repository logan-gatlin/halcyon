use super::*;

pub type Pattern = Typed<Spanned<PatternKind>>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum PatternKind {
    Hole,
    Name(Path),
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Constructor(Constructor, Box<Pattern>),
    Literal(ConstValue),
    TypeHint(Box<Pattern>, Type),
}

impl Pattern {
    pub fn introduced_names(&self) -> usize {
        let mut count = 0;
        self.clone().visit(|p: &mut Pattern| {
            if let PatternKind::Name(_) = *p.inner {
                count += 1
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
            PatternKind::Array(items) | PatternKind::Tuple(items) => items._visit(f),
            PatternKind::Constructor(_, items) => items._visit(f),
            PatternKind::TypeHint(pat, _) => {
                pat._visit(f);
            }
        }
        f(self);
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
            }
        })
    }
}
