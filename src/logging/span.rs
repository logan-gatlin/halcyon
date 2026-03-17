use std::ops::{
    Add,
    AddAssign,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Span {
    Source {
        start: usize,
        width: usize,
        file_id: Option<usize>,
    },
    #[default]
    Generated,
}

impl Span {
    pub fn new(
        start: usize,
        width: usize,
    ) -> Self {
        Self::Source {
            start,
            width,
            file_id: None,
        }
    }

    pub fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Source { start, width, .. } => {
                Self::Source {
                    start,
                    width,
                    file_id: Some(file_id),
                }
            }
            Self::Generated => Self::Generated,
        }
    }

    pub fn file_id(self) -> Option<usize> {
        match self {
            Self::Source { file_id, .. } => file_id,
            Self::Generated => None,
        }
    }

    pub fn range(self) -> std::ops::Range<usize> {
        match self {
            Self::Source { start, width, .. } => start..(start + width),
            Self::Generated => 0..0,
        }
    }

    pub fn then<T, F>(
        &self,
        f: F,
    ) -> Option<T>
    where
        F: FnOnce(Span) -> T,
    {
        match self {
            Span::Source { .. } => Some(f(*self)),
            Span::Generated => None,
        }
    }
}

impl From<rowan::TextRange> for Span {
    fn from(value: rowan::TextRange) -> Self {
        let start: usize = value.start().into();
        let end: usize = value.end().into();
        Span::new(start, end - start)
    }
}

impl Add<Span> for Span {
    type Output = Span;

    fn add(
        self,
        rhs: Span,
    ) -> Self::Output {
        match (self, rhs) {
            (s @ Self::Source { .. }, Self::Generated)
            | (Self::Generated, s @ Self::Source { .. }) => s,
            (Self::Generated, Self::Generated) => self,
            (
                Self::Source {
                    start: start1,
                    width: width1,
                    file_id: file_id1,
                },
                Self::Source {
                    start: start2,
                    width: width2,
                    file_id: file_id2,
                },
            ) => {
                let (min, max) = if start1 < start2 {
                    ((start1, width1), (start2, width2))
                } else {
                    ((start2, width2), (start1, width1))
                };
                let file_id = if file_id1 == file_id2 { file_id1 } else { None };
                Self::Source {
                    start: min.0,
                    width: max.1 + (max.0 - min.0),
                    file_id,
                }
            }
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
        match self {
            Span::Source { start, width, .. } => write!(f, "[{}+{}]", start, width),
            Span::Generated => Ok(()),
        }
    }
}
