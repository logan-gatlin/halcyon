use super::*;

pub type Pattern = Typed<Spanned<PatternKind>>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum PatternKind {
    Name(Path),
    Tuple(Vec<Pattern>),
    Constructor(Constructor, Box<Pattern>),
    Literal(ConstValue),
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
}

impl Visit<Pattern> for Pattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Pattern)) {
        match &mut *self.inner {
            PatternKind::Name(_) | PatternKind::Literal(_) => {}
            PatternKind::Tuple(items) => items._visit(f),
            PatternKind::Constructor(_, items) => items._visit(f),
        }
        f(self);
    }
}

impl Visit<Type> for Pattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        self.visit(|p: &mut Pattern| {
            if let PatternKind::Constructor(c, _) = &mut ***p {
                c.in_type._visit(f);
                c.out_type._visit(f);
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
