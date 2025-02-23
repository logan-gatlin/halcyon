array :: (T: type, size: integer) -> type {
    if size > 0 {
        struct {
            inner: T,
            next: array(T, size - 1)
        }
    } else {
        nothing
    }
};

IntArray4 :: array(integer, 4);
