use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(Default)]
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
