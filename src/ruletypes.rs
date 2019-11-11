use std::collections::{BTreeMap, HashMap};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleInvocation(pub String, pub String);

impl RuleInvocation {
    pub fn new(result_var: impl Into<String>, rule: impl Into<String>) -> RuleInvocation {
        RuleInvocation(result_var.into(), rule.into())
    }
    pub fn result_var(&self) -> &str {&self.0}
    pub fn rule(&self) -> &str {&self.1}
}

#[derive(PartialEq, Eq, Debug)]
pub struct VarInvocation(pub String);

impl VarInvocation {
    pub fn new(name: impl Into<String>) -> VarInvocation {
        VarInvocation(name.into())
    }

    pub fn var_name(&self) -> &str {&self.0}
}

#[derive(PartialEq, Eq, Debug)]
pub struct RulePart<Invocation> {
    text: String,
    /// each usize, an index in .text, is associated with all the rule invocations that appear (in the given order) right before the index
    invocations: BTreeMap<usize, Vec<Invocation>>,
}

pub struct RulePartBuilder<Invocation>(RulePart<Invocation>);

impl<Invocation> RulePart<Invocation> {
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
    pub fn iter(&self) -> impl Iterator<Item=(&str, &[Invocation])> {
        use std::rc::Rc;
        use std::cell::RefCell;

        let last_index = Rc::new(RefCell::new(0usize));
        let last_index2 = last_index.clone();
        self.invocations.iter()
            // assuming .map() is called exactly once, in the correct order.
            .map(move |(&index, invocs)| {
                let from: usize = *last_index2.borrow();
                let to: usize = index;
                *last_index2.borrow_mut() = index;
                (&self.text[from..to], invocs.as_slice())
            })
            .chain(Some(1).iter().map(move |_| {
                (&self.text[*last_index.borrow()..], &[][..])
            }))
    }
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

impl<Invocation> RulePartBuilder<Invocation> {
    pub fn add_char(&mut self, c: char) {
        self.0.text.push(c);
    }

    pub fn add_str(&mut self, s: impl AsRef<str>) {
        self.0.text.push_str(s.as_ref());
    }

    /// adds invocation to the end of the header/body definition
    pub fn add_invoc(&mut self, invoc: Invocation) {
        self.0.invocations.entry(self.0.text.len())
            .or_insert(Vec::new()).push(invoc);
    }

    pub fn seal(self) -> RulePart<Invocation> {self.0}
}

pub type Header = RulePart<RuleInvocation>;
pub type Body = RulePart<VarInvocation>;

#[derive(PartialEq, Eq, Debug)]
pub struct RuleVariant {
    pub match_: Header,
    pub replace: Option<Body>,
    pub append: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Rule {
    pub name: String,
    pub variants: Vec<RuleVariant>,
}

impl Rule {
    pub fn new(name: String) -> Rule {
        Rule {
            name: name,
            variants: Vec::new(),
        }
    }

    pub fn variant(mut self, v: RuleVariant) -> Rule {
        self.variants.push(v);
        self
    }
}

pub type Rules = HashMap<String, Rule>;

use std::fmt;

impl fmt::Display for RuleInvocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}:{}", self.result_var(), self.rule())
    }
}

impl fmt::Display for VarInvocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.var_name())
    }
}

impl<Invocation> fmt::Display for RulePart<Invocation> where Invocation: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (part, invocations) in self.iter() {
            write!(f, "{}", part)?;
            for invocation in invocations {
                write!(f, "{}", invocation)?;
            }
        }
        Ok(())
    }
}