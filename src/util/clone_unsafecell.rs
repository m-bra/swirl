
use std::cell::UnsafeCell;

#[repr(transparent)]
#[derive(Debug)]
pub struct CloneUnsafeCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

impl<T> CloneUnsafeCell<T> {

    #[inline]
    pub const fn new(value: T) -> CloneUnsafeCell<T> {
        CloneUnsafeCell {
            value: UnsafeCell::new(value)
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: ?Sized> CloneUnsafeCell<T> {
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }
}

impl<T: Clone> Clone for CloneUnsafeCell<T> {
    fn clone(&self) -> Self { unsafe {
        CloneUnsafeCell::new((*self.get()).clone())
    }}
}

impl <T: PartialEq> PartialEq for CloneUnsafeCell<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            (&*self.get()).eq(&*other.get())
        }
    }
}

impl <T: Eq> Eq for CloneUnsafeCell<T> {}