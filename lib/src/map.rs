use crate::{
    Span, Spanned,
    semantic::{Type, Typed},
};

pub trait Visit<T> {
    fn _visit(&mut self, f: &mut impl FnMut(&mut T));
    fn visit(&mut self, mut f: impl FnMut(&mut T)) {
        self._visit(&mut f);
    }
}

impl<T, U> Visit<U> for Vec<T>
where
    T: Visit<U>,
{
    fn _visit(&mut self, f: &mut impl FnMut(&mut U)) {
        self.iter_mut().for_each(|t| t._visit(f))
    }
}

impl<T, U> Visit<U> for Option<T>
where
    T: Visit<U>,
{
    fn _visit(&mut self, f: &mut impl FnMut(&mut U)) {
        self.iter_mut().for_each(|t| t._visit(f))
    }
}

impl<T, U> Visit<U> for Box<T>
where
    T: Visit<U>,
{
    fn _visit(&mut self, f: &mut impl FnMut(&mut U)) {
        self.as_mut()._visit(f)
    }
}

/*
pub trait IMap<Inner>: Sized {
    /// In place map from T -> T
    fn _imap(self, f: &mut impl FnMut(Inner) -> Inner) -> Self;
    fn imap(self, mut f: impl FnMut(Inner) -> Inner) -> Self {
        self._imap(&mut f)
    }
}

impl<T, I> IMap<I> for Box<T>
where
    T: IMap<I>,
{
    fn _imap(self, f: &mut impl FnMut(I) -> I) -> Self {
        (*self)._imap(f).into()
    }
}

impl<T, I> IMap<I> for Option<T>
where
    T: IMap<I>,
{
    fn _imap(self, f: &mut impl FnMut(I) -> I) -> Self {
        self.map(|t| t._imap(f))
    }
}

impl<T, I> IMap<I> for Vec<T>
where
    T: IMap<I>,
{
    fn _imap(self, f: &mut impl FnMut(I) -> I) -> Self {
        let mut newvec = Vec::with_capacity(self.capacity());
        for i in self {
            newvec.push(i._imap(f));
        }
        newvec
    }
}
*/
