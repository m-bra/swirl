use std::collections::{BTreeMap, HashMap};
use std::cell::UnsafeCell;
use crate::*;

#[derive(Debug)]
pub struct RulePart<Invocation: Clone> {
    text: String,
    /// each usize, an index in .text, is associated with all the rule invocations that appear (in the given order) right before the index
    /// it is assumed that there are no keys > text.len()
    /// a key of text.len() means that the invocations are at the end of the rule part, without any text afterwards
    invocations: BTreeMap<usize, UnsafeCell<Vec<Invocation>>>,
}

#[derive(Clone)]
pub struct RulePartBuilder<Invocation: Clone>(RulePart<Invocation>);

use std::mem::transmute;
#[allow(mutable_transmutes)]

impl<Invocation: Clone> RulePart<Invocation> {
    pub fn new() -> RulePartBuilder<Invocation> {
        RulePartBuilder(
            RulePart {
                text: String::new(),
                invocations: BTreeMap::new(),
            }
        )
    }

    pub fn literally(text: impl Into<String>) -> RulePart<Invocation> {
        RulePart {
            text: text.into(),
            invocations: BTreeMap::new(),
        }
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
    let ab = RuleInvocation::new("a", "b");
    let cd = RuleInvocation::new("c", "d");
    let ef = RuleInvocation::new("e", "f");

    // filter out ("", &[]) from RulePart::iter()
    let nonempty = |(string, slice): &(&str, &[_])| !string.is_empty() || slice.len() != 0;

    let mut part = Header::new();
    part.add_invoc(ab.clone());
    assert_eq!(part.clone().seal().iter().filter(nonempty).last(), Some(("", &[ab][..])));
    part.add_str("01");
    part.add_invoc(cd.clone());
    part.add_invoc(ef.clone());
    assert_eq!(part.clone().seal().iter().filter(nonempty).last(), Some(("01", &[cd, ef][..])));
    part.add_str("23");
    assert_eq!(part.clone().seal().iter().filter(nonempty).last(), Some(("23", &[][..])))
}

 pub fn parse_header(s: impl AsRef<str>) -> MatchResult<Header> {
    let added_braces = format!("{{{}}}", s.as_ref());
    let (rest, header) = match_rule_part(&added_braces, match_invocation)?;
    if rest.is_empty() {
        Ok(header.unwrap())
    } else {
        Err(MatchError::expected("end of string", rest))
    }
}

pub fn parse_body(s: impl AsRef<str>) -> MatchResult<Body> {
    let added_braces = format!("{{{}}}", s.as_ref());
    let (rest, header) = match_rule_part(&added_braces, match_var)?;
    if rest.is_empty() {
        Ok(header.unwrap())
    } else {
        Err(MatchError::expected("end of string", rest))
    }
}

#[test]
fn test_rule_part_iter() {
    let ab = RuleInvocation::new("a", "b");
    let cd = RuleInvocation::new("c", "d");
    let ef = RuleInvocation::new("e", "f");

    let mut header = Header::new();
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
}

impl<Invocation: Clone> RulePartBuilder<Invocation> {
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
                .or_insert(UnsafeCell::new(Vec::new()))
                .get().as_mut().unwrap()
                .push(invoc);
        }
    }

    pub fn seal(self) -> RulePart<Invocation> {self.0}
}

pub type Header = RulePart<RuleInvocation>;

pub type Body = RulePart<VarInvocation>;

impl Header {
    /// accessing the header memory from any other than the reference given to the closure will result in undefined behaviour
    /// because the header is modified without having a mutable reference to it.
    pub fn without_tail_recursion<R>(&self, tail_name: impl AsRef<str>, inner: impl FnOnce(&RulePart<RuleInvocation>) -> MatchResult<R>) -> MatchResult<R> {
        let mut tail = None;

        unsafe {
            if let Some(invoc) = self.pop_end_invoc() {
                if invoc.rule() == tail_name.as_ref() {
                    tail = Some(invoc);
                } else {
                    self.push_invoc(invoc);
                }
            }

            let result = inner(&self);

            if let Some(invoc) = tail {
                self.push_invoc(invoc);
            }
            result
        }
    }

    pub fn as_body(&self) -> Body {
        let invocations = self.invocations.iter()
            .map(|(key, invocations)| {
                let invocations = unsafe { &*invocations.get() };
                let invocations = invocations.iter().map(|invoc| VarInvocation(invoc.result_var().into())).collect::<Vec<_>>();
                (*key, UnsafeCell::new(invocations))
            })
            .collect();

        RulePart {
            text: self.text.clone(),
            invocations: invocations,
        }
    }
}

impl Body {
    pub fn bind_vars(&self, named_binds: &HashMap<String, String>, unnamed_binds: &Vec<String>) -> MatchResult<String> {
        let mut anon_i = 0;
        let mut buf = String::new();
        for (part, invocations) in self.iter() {
            buf.push_str(part);
            for VarInvocation(var) in invocations {
                if !var.is_empty() {
                    if named_binds.contains_key(var) {
                        buf.push_str(&named_binds[var]);
                    } else {
                        return MatchError::unknown_variable(var, "<>").tap(Err)
                    }
                } else {
                    if anon_i < unnamed_binds.len() {
                        buf.push_str(&unnamed_binds[anon_i]);
                    } else {
                        return MatchError::new("Too many anonymous variables").tap(Err);
                    }
                    anon_i += 1;
                }
            }
        }
        Ok(buf)
    }
}

// boring stuff ugh

impl<I: Clone + PartialEq> PartialEq for RulePart<I> {
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
impl<I: Clone + PartialEq + Eq> Eq for RulePart<I> {}

impl<I: Clone> Clone for RulePart<I> {
    fn clone(&self) -> Self {
        RulePart {
            text: self.text.clone(),
            invocations: self.invocations.iter().map(|(key, invocs)| {
                (*key, unsafe {&*invocs.get()}.clone().tap(|invocs| UnsafeCell::new(invocs)))
            }).collect::<BTreeMap<_, _>>()
        }
    }
}
