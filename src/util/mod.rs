
mod string_utils;
pub use string_utils::*;

pub fn _firstline<'a>(string: &str) -> &str {
    string.split("\n").next().unwrap_or("")
}

pub fn firstline<'a>(string: &str) -> &str {
    _firstline(string.trim())
}

pub fn skip_str(s: &str, start: usize) -> &str {
    if start == s.len() {
        ""
    } else {
        let start = s.char_indices().nth(start).map(|(n, _)| n)
            .unwrap_or_else(|| panic!("index {} out of bounds in '{}'", start, s));
        &s[start..]   
    }
}

/// is it so hard to have this in std:: ?
pub fn substr(s: &str, start_char: usize, end_char: usize) -> &str {
   s.substring(start_char, end_char)
}

pub fn input_view(input: &str) -> &str {
    let mut input_view = input;
    while input_view.chars().next().map(|c| c == '\n').unwrap_or(false) {
        input_view = skip_str(input_view, 1);
    }
    input_view = firstline(input);
    if input_view.len() > 32 {
        input_view = substr(input_view, 0, 20);
    }
    input_view
}

mod dump_file;
pub use dump_file::*;

mod tap;
pub use tap::*;

mod clone_unsafecell;
pub use clone_unsafecell::*;

use std::ops::*;
use std::cmp::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaybeInf<T> where T: Eq + Clone + Copy {
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

impl<T: SubAssign> SubAssign<T> for MaybeInf<T> where T: Eq + Clone + Copy {
    fn sub_assign(&mut self, rhs: T) {
        match self {
            MaybeInf::Finite(x) => {
                *x -= rhs;
            }
            MaybeInf::Infinite => {}
        }
    }
}

#[cfg(debug_assertions)]
pub fn breakpoint() {
    unsafe {
        ::std::intrinsics::breakpoint();
    }
}

#[cfg(not(debug_assertions))]
pub fn breakpoint() {
    
}