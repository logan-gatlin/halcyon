use crate::Span;

use super::*;

pub type Pattern = Typed<Spanned<PatternKind>>;

#[derive(Debug, Clone)]
pub enum PatternKind {
    Hole,
    Identifier(Path),
    Tuple(Vec<Pattern>),
    Array(ArrayPattern),
    Constructor(Constructor, Box<Pattern>),
    Immediate(ConstValue),
    TypeHint(Box<Pattern>, Type),
}

#[derive(Debug, Clone)]
pub enum ConstructorKind {
    Unitary(Type),
    Function(Type, Type),
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub variant_id: usize,
    pub kind: ConstructorKind,
}

impl Visit<Type> for Constructor {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        match &mut self.kind {
            ConstructorKind::Unitary(t) => {
                t._visit(f);
            }
            ConstructorKind::Function(a, b) => {
                a._visit(f);
                b._visit(f);
            }
        }
    }
}

#[derive(Debug, Clone)]
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
        middle: Option<Path>,
        tail: Vec<Pattern>,
    },
}

impl Pattern {
    pub fn introduced_names(&self) -> usize {
        let mut count = 0;
        self.clone().visit(|p: &mut Pattern| {
            if let PatternKind::Identifier(_) = *p.inner {
                count += 1
            } else if let PatternKind::Array(ap) = &*p.inner {
                match ap {
                    ArrayPattern::Leading { tail, .. } => count += tail.is_some() as usize,
                    ArrayPattern::Trailing { head, .. } => count += head.is_some() as usize,
                    ArrayPattern::LeadingAndTrailing { middle, .. } => {
                        count += middle.is_some() as usize
                    }
                    _ => {}
                }
            }
        });
        count
    }

    pub fn find_refutable_pattern(&self) -> Option<Span> {
        match &self.inner.inner {
            PatternKind::Hole | PatternKind::Identifier(_) => None,
            PatternKind::Tuple(pats) => pats.iter().find_map(Pattern::find_refutable_pattern),
            PatternKind::Array(..) | PatternKind::Constructor(..) => Some(self.span),
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
            PatternKind::Array(..) => true,
            PatternKind::Constructor(..) => true,
            PatternKind::Immediate(const_value) => const_value != &ConstValue::Unit,
            PatternKind::TypeHint(pat, _) => pat.is_refutable(),
        }
    }
}

impl Visit<Pattern> for Pattern {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Pattern)) {
        match &mut *self.inner {
            PatternKind::Hole | PatternKind::Identifier(_) | PatternKind::Immediate(_) => {}
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
            ArrayPattern::LeadingAndTrailing { head, tail, .. } => {
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
        self.visit(|p: &mut Pattern| match &mut p.inner.inner {
            PatternKind::Identifier(path) => {
                let mut tup = (path.clone(), p.type_.clone());
                f(&mut tup);
                *path = tup.0;
                p.type_ = tup.1;
            }
            PatternKind::Array(ArrayPattern::Leading {
                tail: Some(tail), ..
            }) => {
                let mut tup = (tail.clone(), p.type_.clone());
                f(&mut tup);
                *tail = tup.0;
            }
            PatternKind::Array(ArrayPattern::Trailing {
                head: Some(head), ..
            }) => {
                let mut tup = (head.clone(), p.type_.clone());
                f(&mut tup);
                *head = tup.0;
            }
            PatternKind::Array(ArrayPattern::LeadingAndTrailing {
                middle: Some(middle),
                ..
            }) => {
                let mut tup = (middle.clone(), p.type_.clone());
                f(&mut tup);
                *middle = tup.0;
            }
            _ => {}
        })
    }
}
