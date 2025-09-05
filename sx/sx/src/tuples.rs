use super::*;

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        #[allow(non_snake_case)]
        impl<$($name: SXRepr),+> SXRepr for ($($name,)+)
        {
            fn sx(self) -> SX {
                let ($($name,)+) = self;
                SX::Expr(vec!{
                    $($name.sx(),)+
                })
            }
        }
    };
}

tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
tuple_impls! { A B C D E F G H I J }
tuple_impls! { A B C D E F G H I J K }
tuple_impls! { A B C D E F G H I J K L }
