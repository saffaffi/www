pub trait SwapResult<T1, E1> {
    type Swapped<T2, E2>;
    fn swap(self) -> Self::Swapped<E1, T1>;
}

impl<T, E> SwapResult<T, E> for Result<T, E> {
    type Swapped<T2, E2> = Result<T2, E2>;

    fn swap(self) -> Self::Swapped<E, T> {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e),
        }
    }
}
