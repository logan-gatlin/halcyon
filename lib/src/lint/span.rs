use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, sx::SXRepr)]
pub struct Span {
    pub start: usize,
    pub width: usize,
}

impl Add<Span> for Span {
    type Output = Span;

    fn add(self, rhs: Span) -> Self::Output {
        let (min, max) = if self.start < rhs.start {
            (self, rhs)
        } else {
            (rhs, self)
        };
        Span {
            start: min.start,
            width: max.width + (max.start - min.start),
        }
    }
}

impl AddAssign for Span {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait WithSpan: Sized {
    fn with_span(self, span: Span) -> Spanned<Self>;
    fn with_default_span(self) -> Spanned<Self> {
        self.with_span(Span { start: 0, width: 0 })
    }
}

impl<T> WithSpan for T {
    fn with_span(self, span: Span) -> Spanned<Self> {
        Spanned { inner: self, span }
    }
}

impl<T> sx::SXRepr for Spanned<T>
where
    T: sx::SXRepr,
{
    fn sx(self) -> sx::SX {
        self.inner.sx()
    }
}

pub fn map_span<T>(span: Span) -> Box<dyn Fn(T) -> Spanned<T>> {
    Box::new(move |i: T| i.with_span(span))
}

pub fn without_span<T>(inner: Spanned<T>) -> T {
    inner.inner
}
