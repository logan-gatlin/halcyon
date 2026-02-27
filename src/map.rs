use std::collections::HashMap;

pub trait Visit<T> {
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut T),
    );
    fn visit(
        &mut self,
        mut f: impl FnMut(&mut T),
    ) {
        self.visit_with(&mut f);
    }
}

impl<T, U> Visit<U> for Vec<T>
where
    T: Visit<U>,
{
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.iter_mut().for_each(|t| t.visit_with(f))
    }
}

impl<T, U> Visit<U> for (T, T)
where
    T: Visit<U>,
{
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.0.visit_with(f);
        self.1.visit_with(f);
    }
}

impl<T, U> Visit<U> for Option<T>
where
    T: Visit<U>,
{
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.iter_mut().for_each(|t| t.visit_with(f))
    }
}

impl<T, U> Visit<U> for Box<T>
where
    T: Visit<U>,
{
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.as_mut().visit_with(f)
    }
}

impl<K, T, U> Visit<U> for HashMap<K, T>
where
    T: Visit<U>,
{
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.values_mut().for_each(|v| v.visit_with(f));
    }
}

impl<K, T, U> Visit<U> for indexmap::IndexMap<K, T>
where
    T: Visit<U>,
{
    fn visit_with(
        &mut self,
        f: &mut impl FnMut(&mut U),
    ) {
        self.values_mut().for_each(|v| v.visit_with(f));
    }
}
