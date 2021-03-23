pub trait OptionExtDefault<T> {
    fn or_default(self) -> T;
}

impl<T> OptionExtDefault<T> for Option<T> where
    T: Default,
{
    #[inline]
    fn or_default(self) -> T {
        self.unwrap_or_else(Default::default)
    }
}

fn flip_tuple<A, B>(tup: (A, B)) -> (B, A) {
    let (a, b) = tup;
    (b, a)
}

pub trait OptionExtTup<T>: Sized {
    fn and_then_tup<F, Ret>(self, f: F) -> Option<(T, Ret)> where
        Ret: Sized,
        F: FnOnce(&T) -> Option<Ret>;
    fn and_then_tup_flipped<F, Ret>(self, f: F) -> Option<(Ret, T)> where
        Ret: Sized,
        F: FnOnce(&T) -> Option<Ret>
    {
        self.and_then_tup(f).map(flip_tuple)
    }
    fn and_tup<Ret>(self, v: Option<Ret>) -> Option<(T, Ret)> {
        self.and_then_tup(move |_| v)
    }
    fn and_tup_flipped<Ret>(self, v: Option<Ret>) -> Option<(Ret, T)> {
        self.and_tup(v).map(flip_tuple)
    }
}

impl<T: Sized> OptionExtTup<T> for Option<T> {
    fn and_then_tup<F, Ret>(self, f: F) -> Option<(T, Ret)> where
        Ret: Sized,
        F: FnOnce(&T) -> Option<Ret>,
    {
        self.and_then(move |v| f(&v).map(move |vv| (v, vv)))
    }
}
