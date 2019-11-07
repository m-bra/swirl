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

pub trait AssertEq: Sized {
    fn assert_eq(self, other: &Self) -> Self;
}

impl<T: PartialEq> AssertEq for T where T: std::fmt::Debug {
    fn assert_eq(self, other: &Self) -> Self {
        assert_eq!(&self, other);
        self
    }
}

pub trait Assert: Sized {
    fn assert(self) -> Self;
}

impl Assert for bool {
    fn assert(self) -> Self {
        assert!(self);
        self
    }
}
