mod collections;
mod primitives;
mod printing;
mod tuples;

pub use sx_derive::SXRepr;

#[derive(Debug, Clone, PartialEq)]
pub enum SX {
    Nil,
    Atom(String),
    Expr(Vec<SX>),
    Field(String, Box<SX>),
}

pub trait SXRepr {
    fn sx(self) -> SX;
}

impl<T> From<T> for SX
where
    T: SXRepr,
{
    fn from(value: T) -> Self {
        value.sx()
    }
}

impl SX {
    pub fn push(self, item: impl Into<SX>) -> Self {
        let item = item.into();
        if item == SX::Nil {
            return self;
        }
        match self {
            SX::Nil => item,
            SX::Atom(..) | SX::Field(..) => SX::Expr(vec![self, item]),
            SX::Expr(mut items) => {
                items.push(item);
                SX::Expr(items)
            }
        }
    }

    pub fn field(self, name: impl std::fmt::Display, item: impl SXRepr) -> Self {
        self.push(SX::Field(format!("{name}"), Box::new(item.sx())))
    }
}
