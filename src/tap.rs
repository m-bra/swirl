/// Tap operations for all types.
pub trait TapOps: Sized {
    fn tap<R, F>(self, f: F) -> R
        where F: FnOnce(Self) -> R;
}

impl<T> TapOps for T where T: Sized {
    fn tap<R, F>(self, f: F) -> R
        where F: FnOnce(Self) -> R
    {
        f(self)
    }
}
