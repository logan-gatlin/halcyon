use super::*;

macro_rules! write_repr {
    ($t:ty) => {
        impl SXRepr for $t {
            fn sx(self) -> SX {
                SX::Atom(format!("{}", self))
            }
        }
    };
    ($($t:ty,)+) => {
        $(write_repr!($t);)*
    }
}

write_repr! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64, bool, char,
    String,
}

impl SXRepr for &str {
    fn sx(self) -> SX {
        SX::Atom(self.to_string())
    }
}

impl SXRepr for () {
    fn sx(self) -> SX {
        SX::Atom("()".to_string())
    }
}
