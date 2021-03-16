pub trait OptionExt<T: Sized> {
    fn or_default(self) -> T;
}

impl<T> OptionExt<T> for Option<T> where
    T: Default,
{
    fn or_default(self) -> T {
        self.unwrap_or_else(Default::default)
    }
}
