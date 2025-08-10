use super::*;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatternKind,
    pub type_: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    Name(Path),
    Tuple(Vec<Pattern>),
    Constructor(Constructor, Box<Pattern>),
    Literal(ConstValue),
}

impl Pattern {
    pub fn introduced_names(&self) -> usize {
        match &self.kind {
            PatternKind::Name(_) => 1,
            PatternKind::Tuple(patterns) => patterns
                .into_iter()
                .fold(0, |v, p| v + p.introduced_names()),
            PatternKind::Literal(_) => 0,
            PatternKind::Constructor(_, pattern) => pattern.introduced_names(),
        }
    }

    pub fn iter_names(&self, f: &mut impl FnMut(&Path, &TypeRef)) {
        match &self.kind {
            PatternKind::Name(path) => f(path, &self.type_),
            PatternKind::Tuple(patterns) => patterns.iter().for_each(|p| p.iter_names(f)),
            PatternKind::Literal(_) => {}
            PatternKind::Constructor(_, pattern) => pattern.iter_names(f),
        }
    }
}

impl Unify for Pattern {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.type_.unify(tv, type_);
        match &mut self.kind {
            PatternKind::Name(_) => {}
            PatternKind::Tuple(patterns) => patterns.iter_mut().for_each(|p| p.unify(tv, type_)),
            PatternKind::Constructor(_, pattern) => {
                pattern.unify(tv, type_);
            }
            PatternKind::Literal(_) => {}
        }
    }
}
