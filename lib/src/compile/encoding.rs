pub trait Encode<T> {
    fn encode(&mut self, obj: T) -> &mut Self;
}

impl<T, U, const N: usize> Encode<[T; N]> for U
where
    U: Encode<T>,
{
    fn encode(&mut self, objs: [T; N]) -> &mut Self {
        for obj in objs {
            self.encode(obj);
        }
        self
    }
}

impl<T, U> Encode<&[T]> for U
where
    T: Clone,
    U: Encode<T>,
{
    fn encode(&mut self, objs: &[T]) -> &mut Self {
        for obj in objs {
            self.encode(obj.clone());
        }
        self
    }
}

impl<T, U> Encode<Option<T>> for U
where
    U: Encode<T>,
{
    fn encode(&mut self, obj: Option<T>) -> &mut Self {
        if let Some(obj) = obj {
            self.encode(obj)
        } else {
            self
        }
    }
}

impl<T, U> Encode<Box<T>> for U
where
    U: Encode<T>,
{
    fn encode(&mut self, obj: Box<T>) -> &mut Self {
        self.encode(*obj)
    }
}
