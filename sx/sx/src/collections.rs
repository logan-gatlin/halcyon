use std::collections::HashSet;

use super::*;

impl<T> SXRepr for Option<T>
where
    T: SXRepr,
{
    fn sx(self) -> SX {
        if let Some(s) = self { s.sx() } else { SX::Nil }
    }
}

impl<T, E> SXRepr for Result<T, E>
where
    T: SXRepr,
{
    fn sx(self) -> SX {
        if let Ok(s) = self { s.sx() } else { SX::Nil }
    }
}

impl<T> SXRepr for std::boxed::Box<T>
where
    T: SXRepr + Clone,
{
    fn sx(self) -> SX {
        self.as_ref().clone().sx()
    }
}

impl<T> SXRepr for std::rc::Rc<T>
where
    T: SXRepr + Clone,
{
    fn sx(self) -> SX {
        self.as_ref().clone().sx()
    }
}

impl<T> SXRepr for std::sync::Arc<T>
where
    T: SXRepr + Clone,
{
    fn sx(self) -> SX {
        self.as_ref().clone().sx()
    }
}

impl<T> SXRepr for Vec<T>
where
    T: SXRepr,
{
    fn sx(self) -> SX {
        SX::Expr(self.into_iter().map(|e| e.sx()).collect())
    }
}

impl<T> SXRepr for HashSet<T>
where
    T: SXRepr,
{
    fn sx(self) -> SX {
        SX::Expr(self.into_iter().map(|e| e.sx()).collect())
    }
}

impl<T, const N: usize> SXRepr for [T; N]
where
    T: SXRepr,
{
    fn sx(self) -> SX {
        SX::Expr(self.into_iter().map(|e| e.sx()).collect())
    }
}

impl<K, V> SXRepr for std::collections::HashMap<K, V>
where
    K: std::fmt::Display,
    V: SXRepr,
{
    fn sx(self) -> SX {
        SX::Expr(
            self.into_iter()
                .map(|(k, v)| SX::Field(format!("{k}"), Box::new(v.sx())))
                .collect(),
        )
    }
}
