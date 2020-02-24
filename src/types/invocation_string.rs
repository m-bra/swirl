#![allow(mutable_transmutes)]

use std::collections::{BTreeMap, HashMap};
use std::cell::UnsafeCell;
use crate::*;

#[derive(Debug, Clone)]
pub struct InvocationString {
    text: String,
    /// each usize, an index in .text, is associated with all the rule invocations that appear (in the given order) right before the index
    /// it is assumed that there are no keys > text.len()
    /// a key of text.len() means that the invocations are at the end of the rule part, without any text afterwards
    invocations: BTreeMap<usize, CloneUnsafeCell<Vec<Invocation>>>,
}

#[derive(Clone)]
pub struct InvocationStringBuilder(InvocationString);

use std::mem::transmute;

impl InvocationString {
    pub fn debug_print(&self) {
        println!("invocations with text '{}': ", self.text);
        for (i, cell) in self.invocations.iter() {
            println!("{}, {:?}", i, unsafe {
                &*cell.get()
            })
        }
    }
}

impl InvocationString {
    pub fn empty() -> InvocationString {
        InvocationString {
            text: String::new(),
            invocations: BTreeMap::new(),
        }
    }

    pub fn new() -> InvocationStringBuilder {
        InvocationStringBuilder(
            InvocationString::empty()
        )
    }

    pub fn literally(text: impl Into<String>) -> InvocationString {
        InvocationString {
            text: text.into(),
            invocations: BTreeMap::new(),
        }
    }

    pub fn has_invocations(&self) -> bool {
        self.invocations.len() != 0
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && !self.has_invocations()
    }

    /// iterate text and invocation segments
    /// might include empty items ("", &[])
    pub fn iter(&self) -> impl Iterator<Item=(&str, &[Invocation])> {
        use std::rc::Rc;
        use std::cell::RefCell;

        let last_index = Rc::new(RefCell::new(0usize));
        let last_index2 = last_index.clone();
        self.invocations.iter()
            // assuming .map() is called exactly once, in the correct order.
            .map(move |(&index, invocs)| {
                let invocs = unsafe {&*invocs.get()};
                let from: usize = *last_index2.borrow();
                let to: usize = index;
                *last_index2.borrow_mut() = index;
                (&self.text[from..to], invocs.as_slice())
            })
            .chain(Some(1).iter().map(move |_| {
                (&self.text[*last_index.borrow()..], &[][..])
            }))
    }

    /// return last invocation at the end of rule if the rule does not end in text
    pub fn end_invocation(&self) -> Option<&Invocation> {
        self.invocations.get(&self.text.len()).and_then(|invocs| unsafe{&*invocs.get()}.iter().last())
    }

    unsafe fn pop_end_invoc(&self) -> Option<Invocation> {
        self.invocations.get(&self.text.len()).and_then(|invocs| {
            let invocs: &mut Vec<_> = transmute(invocs);
            invocs.pop()
        })
    }

    unsafe fn push_invoc(&self, invoc: Invocation) {
        if let Some(invocs) = self.invocations.get(&self.text.len()) {
            let invocs: &mut Vec<_> = transmute(invocs);
            invocs.push(invoc);
        }
    }
}

#[test]
fn test_iter_last() {
    let ab = Invocation::new_rule_invocation("a", "b");
    let cd = Invocation::new_rule_invocation("c", "d");
    let ef = Invocation::new_rule_invocation("e", "f");

    // filter out ("", &[]) from InvocationString::iter()
    let nonempty = |(string, slice): &(&str, &[_])| !string.is_empty() || slice.len() != 0;

    let mut part = InvocationString::new();
    part.add_invoc(ab.clone());
    assert_eq!(part.clone().seal().iter().filter(nonempty).last(), Some(("", &[ab][..])));
    part.add_str("01");
    part.add_invoc(cd.clone());
    part.add_invoc(ef.clone());
    assert_eq!(part.clone().seal().iter().filter(nonempty).last(), Some(("01", &[cd, ef][..])));
    part.add_str("23");
    assert_eq!(part.clone().seal().iter().filter(nonempty).last(), Some(("23", &[][..])))
}

 pub fn parse_header(s: impl AsRef<str>) -> MatchResult<InvocationString> {
    let added_braces = format!("{{{}}}", s.as_ref());
    let (rest, header) = match_invocation_string_def(&added_braces, '{', '}')?;
    if rest.is_empty() {
        Ok(header.unwrap())
    } else {
        Err(MatchError::expected("end of string", rest))
    }
}

pub fn parse_body(s: impl AsRef<str>) -> MatchResult<InvocationString> {
    let added_braces = format!("{{{}}}", s.as_ref());
    let (rest, header) = match_invocation_string_def(&added_braces, '{', '}')?;
    if rest.is_empty() {
        Ok(header.unwrap())
    } else {
        Err(MatchError::expected("end of string", rest))
    }
}

#[test]
fn test_invocation_string_iter() {
    let ab = Invocation::new_rule_invocation("a", "b");
    let cd = Invocation::new_rule_invocation("c", "d");
    let ef = Invocation::new_rule_invocation("e", "f");

    let mut header = InvocationString::new();
    header.add_invoc(ab.clone());
    header.add_str("12");
    header.add_invoc(cd.clone());
    header.add_invoc(ef.clone());
    header.add_str("34");
    let header = header.seal();

    assert_eq!(
        header.iter().collect::<Vec::<_>>(),
        vec![("", &[ab][..]), ("12", &[cd, ef][..]), ("34", &[][..])]
    );

    let header = match_invocation_string_def("{::expr {' or '} ::or}", '{', '}')  .unwrap().1.unwrap();

    let expr = Invocation::new_rule_invocation("", "expr");
    let or = Invocation::new_rule_invocation("", "or");

    assert_eq!(
        header.iter().collect::<Vec::<_>>(),
        vec![("", &[expr][..]), (" or ", &[or][..]), ("", &[])]
    );
}

impl InvocationStringBuilder {
    pub fn add_char(&mut self, c: char) {
        self.0.text.push(c);
    }

    pub fn add_str(&mut self, s: impl AsRef<str>) {
        self.0.text.push_str(s.as_ref());
    }

    /// adds invocation to the end of the header/body definition
    pub fn add_invoc(&mut self, invoc: Invocation) {
        unsafe {
            self.0.invocations.entry(self.0.text.len())
                .or_insert(CloneUnsafeCell::new(Vec::new()))
                .get().as_mut().unwrap()
                .push(invoc);
        }
    }

    pub fn seal(self) -> InvocationString {self.0}
}

impl InvocationString {
    /// accessing the header memory from any other than the reference given to the closure will result in undefined behaviour
    /// because the header is modified without having a mutable reference to it.
    pub fn without_tail_recursion<R>(&self, tail_name: impl AsRef<str>, inner: impl FnOnce(&InvocationString) -> MatchResult<R>) -> MatchResult<R> {
        let mut tail = None;

        unsafe {
            let this = self.clone();
            if let Some(invoc) = this.pop_end_invoc() {
                let rulename = match &invoc {
                    Invocation::RuleInvocation(_, rulename, _) => rulename,
                    _ => panic!("ouf")
                };

                if rulename == tail_name.as_ref() {
                    tail = Some(invoc);
                } else {
                    this.push_invoc(invoc);
                }
            }

            let result = inner(&this);

            if let Some(invoc) = tail {
                this.push_invoc(invoc);
            }
            result
        }
    }

    pub fn ensure_only_var_invocs(self) -> Result<VarInvocationString, InvocationString> {
        if self.iter().any(|(_, invocs)| invocs.iter().any(|invoc| match invoc {
            Invocation::VarInvocation(_) => false,
            _ => true,
        })) {
            Err(self)
        } else {
            Ok(VarInvocationString(self))
        }
    }

    pub fn assume_only_var_invocs(self) -> VarInvocationString {
        VarInvocationString(self)
    }
}

// an invocation string with only variable invocations :var
#[derive(PartialEq, Eq)]
pub struct VarInvocationString(InvocationString);

impl VarInvocationString {
    pub fn bind_vars(&self, named_binds: &HashMap<String, String>, unnamed_binds: &Vec<String>) -> MatchResult<String> {
        let VarInvocationString(this) = self;
        let mut anon_i = 0;
        let mut buf = String::new();
        for (part, invocations) in this.iter() {
            buf.push_str(part);
            for invoc in invocations {
                match invoc {
                    Invocation::VarInvocation(var) if !var.is_empty() => {
                        if named_binds.contains_key(var) {
                            buf.push_str(&named_binds[var]);
                        } else {
                            return MatchError::unknown_variable(var, "<>").tap(Err)
                        }
                    }
                    Invocation::VarInvocation(_/*empty*/) => {
                        if anon_i < unnamed_binds.len() {
                            buf.push_str(&unnamed_binds[anon_i]);
                        } else {
                            return MatchError::new("Too many anonymous variables").tap(Err);
                        }
                        anon_i += 1;
                    }
                    _ => unreachable!()
                }
            }
        }
        Ok(buf)
    }

    pub fn unwrap(self) -> InvocationString {
        self.0
    }
}

// boring stuff ugh

impl PartialEq for InvocationString {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && {
            self.invocations.iter().all(|(key, value)| {
                other.invocations.get(key).map(|other_value| unsafe {
                    *value.get() == *other_value.get()
                })
                    .unwrap_or(false)
            })
        }
    }
}
impl Eq for InvocationString {}

/*impl<I: Clone> Clone for InvocationString {
    fn clone(&self) -> Self {
        InvocationString {
            text: self.text.clone(),
            invocations: self.invocations.iter().map(|(key, invocs)| {
                (*key, unsafe {&*invocs.get()}.clone().tap(|invocs| UnsafeCell::new(invocs)))
            }).collect::<BTreeMap<_, _>>()
        }
    }
}*/
