pub trait OptionExt<T> {
    fn or_default(self) -> T;
}

impl<T> OptionExt<T> for Option<T> where
    T: Default,
{
    #[inline]
    fn or_default(self) -> T {
        self.unwrap_or_else(Default::default)
    }
}
