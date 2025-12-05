use std::ops::{
    Add,
    AddAssign,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub file_id: usize,
    pub start: usize,
    pub width: usize,
}

impl Add<Span> for Span {
    type Output = Span;

    fn add(
        self,
        rhs: Span,
    ) -> Self::Output {
        assert_eq!(
            self.file_id, rhs.file_id,
            "Attempted to add spans from different files"
        );
        let (min, max) = if self.start < rhs.start {
            (self, rhs)
        } else {
            (rhs, self)
        };
        Span {
            file_id: self.file_id,
            start: min.start,
            width: max.width + (max.start - min.start),
        }
    }
}

impl AddAssign for Span {
    fn add_assign(
        &mut self,
        rhs: Self,
    ) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> PartialEq for Spanned<T>
where
    T: PartialEq,
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.inner == other.inner
    }
}

impl<T> std::hash::Hash for Spanned<T>
where
    T: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.inner.hash(state);
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::fmt::Display for Spanned<T>
where
    T: std::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait WithSpan: Sized {
    fn with_span(
        self,
        span: Span,
    ) -> Spanned<Self>;
}

impl<T> WithSpan for T {
    fn with_span(
        self,
        span: Span,
    ) -> Spanned<Self> {
        Spanned { inner: self, span }
    }
}

pub fn map_span<T>(span: Span) -> Box<dyn Fn(T) -> Spanned<T>> {
    Box::new(move |i: T| i.with_span(span))
}

pub fn without_span<T>(inner: Spanned<T>) -> T {
    inner.inner
}

impl std::fmt::Display for Span {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "[{}+{}]", self.start, self.width)
    }
}
