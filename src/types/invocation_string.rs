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
    /// todo: idea: replace CloneUnsafeCell<_> with Rc<_>
    invocations: BTreeMap<usize, CloneUnsafeCell<Vec<Invocation>>>,
}

#[derive(Clone)]
pub struct InvocationStringBuilder(InvocationString);

use std::mem::transmute;

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
    /// might include empty items ("", _) or (_, &[])
    /// while returned references live, do not call pop_end_invoc() or push_invoc() 
    pub unsafe fn iter(&self) -> impl Iterator<Item=(&str, &[Invocation])> {
        use std::rc::Rc;
        use std::cell::RefCell;

        let last_index = Rc::new(RefCell::new(0usize));
        let last_index2 = last_index.clone();
        self.invocations.iter()
            // assuming .map() is called exactly once, in the correct order.
            .map(move |(&index, invocs)| {
                let invocs = &*invocs.get();
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
    /// while returned references live, do not call pop_end_invoc() or push_invoc() 
    pub unsafe fn end_invocation(&self) -> Option<&Invocation> {
        self.invocations.get(&self.text.len()).and_then(|invocs| (&*invocs.get()).iter().last())
    }

    // no reference given by self.end_invocation or self.iter may be alive when calling this function.
    unsafe fn pop_end_invoc(&self) -> Option<Invocation> {
        self.invocations.get(&self.text.len()).and_then(|invocs| {
            let invocs: *mut Vec<_> = invocs.get();
            (&mut *invocs).pop()
        })
    }

    // no reference given by self.end_invocation or self.iter may be alive when calling this function.
    unsafe fn push_invoc(&self, invoc: Invocation) {
        if let Some(invocs) = self.invocations.get(&self.text.len()) {
            let invocs: *mut Vec<_> = invocs.get();
            (&mut *invocs).push(invoc)
        }
    }
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

    pub fn add_invoc_str(&mut self, invoc_str: &InvocationString) {
        unsafe {
            for (stri, invocs) in invoc_str.iter() {
                self.add_str(stri);
                for invoc in invocs {
                    self.add_invoc(invoc.clone());
                }
            }
        }
    }

    pub fn seal(self) -> InvocationString {self.0}
}

impl InvocationString {
    /// In order to ensure requirement [B], there must be not a single living reference on self.
    pub unsafe fn without_tail_recursion_unsafe<R>(&self, tail_name: impl AsRef<str>, inner: impl FnOnce(&InvocationString) -> MatchResult<R>) -> MatchResult<R> {
        let mut tail = None;

        // requirement [B]: we have to ensure aliasing rules in [1] and [2]
        if let Some(invoc) = self.pop_end_invoc() { // [1]
            let rulename = match &invoc {
                Invocation::RuleInvocation(_, rulename, _) => rulename,
                _ => panic!("ouf")
            };

            if rulename == tail_name.as_ref() {
                tail = Some(invoc);
            } else {
                self.push_invoc(invoc); // [2]
            }
        }

        // see [A]
        let result = inner(&self);

        if let Some(invoc) = tail {
            self.push_invoc(invoc); // [3]
        }
        result
    }

    pub fn without_tail_recursion<R>(&self, tail_name: impl AsRef<str>, inner: impl FnOnce(&InvocationString) -> MatchResult<R>) -> MatchResult<R> {
        let mut tail = None;

        unsafe {
            // by cloning, we ensure that there are no references pointing to this specific memory region.
            // thus ensuring aliasing rules in [1] and [2]
            let this = self.clone();
            if let Some(invoc) = this.pop_end_invoc() { // [1]
                let rulename = match &invoc {
                    Invocation::RuleInvocation(_, rulename, _) => rulename,
                    _ => panic!("ouf")
                };

                if rulename == tail_name.as_ref() {
                    tail = Some(invoc);
                } else {
                    this.push_invoc(invoc); //[2]
                }
            }

            // [A]:
            // after the inner function has been called, there must be no references remaining that point to `this`.
            // this is true since we only provide the inner function only has an immutable reference to `this`
            // thus ensuring aliasing rules in [3]
            let result = inner(&this);

            if let Some(invoc) = tail {
                this.push_invoc(invoc); // [3]
            }
            result
        }
    }

    pub fn ensure_only_var_invocs(self) -> Result<VarInvocationString, InvocationString> {
        if unsafe {self.iter()}.any(|(_, invocs)| invocs.iter().any(|invoc| match invoc {
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
        for (part, invocations) in unsafe {this.iter()} {
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
