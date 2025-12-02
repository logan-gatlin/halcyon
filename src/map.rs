use std::collections::HashMap;

pub trait Visit<T> {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut T),
    );
    fn visit(
        &mut self,
        mut f: impl FnMut(&mut T),
    ) {
        self._visit(&mut f);
    }
}

impl<T, U> Visit<U> for Vec<T>
where
    T: Visit<U>,
{
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.iter_mut().for_each(|t| t._visit(f))
    }
}

impl<T, U> Visit<U> for (T, T)
where
    T: Visit<U>,
{
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.0._visit(f);
        self.1._visit(f);
    }
}

impl<T, U> Visit<U> for Option<T>
where
    T: Visit<U>,
{
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.iter_mut().for_each(|t| t._visit(f))
    }
}

impl<T, U> Visit<U> for Box<T>
where
    T: Visit<U>,
{
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.as_mut()._visit(f)
    }
}

impl<K, T, U> Visit<U> for HashMap<K, T>
where
    T: Visit<U>,
{
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.values_mut().for_each(|v| v._visit(f));
    }
}

impl<K, T, U> Visit<U> for indexmap::IndexMap<K, T>
where
    T: Visit<U>,
{
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.values_mut().for_each(|v| v._visit(f));
    }
}
