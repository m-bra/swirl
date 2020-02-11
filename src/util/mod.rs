
pub fn _firstline<'a>(string: &str) -> &str {
    string.split("\n").next().unwrap_or("")
}

pub fn firstline<'a>(string: &str) -> &str {
    _firstline(string)
}

mod dump_file;
pub use dump_file::*;

mod tap;
pub use tap::*;

mod clone_unsafecell;
pub use clone_unsafecell::*;

use std::ops::*;
use std::cmp::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeInf<T> where T: Eq {
    Finite(T),
    Infinite
}

#[test]
fn test_ord() {
    assert!(MaybeInf::Finite(2) < MaybeInf::Finite(3));
    assert!(MaybeInf::Finite(3) == MaybeInf::Finite(3));
    assert!(MaybeInf::Infinite > MaybeInf::Finite(100));
    assert!(MaybeInf::<i32>::Infinite == MaybeInf::Infinite);
}

impl<T: SubAssign> SubAssign<T> for MaybeInf<T> where T: Eq {
    fn sub_assign(&mut self, rhs: T) {
        match self {
            MaybeInf::Finite(x) => {
                *x -= rhs;
            }
            MaybeInf::Infinite => {}
        }
    }
}
