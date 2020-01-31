use std::collections::{BTreeMap, HashMap, HashSet};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleVariant {
    pub header: Header,
    pub body: Option<Body>,
    pub flags: HashSet<String>,
    pub catch_unknown_rule: Option<Body>,
}

impl RuleVariant {
    pub fn new(header: Header, body: Option<Body>) -> RuleVariant {
        RuleVariant {
            header,
            body,
            flags: HashSet::new(),
            catch_unknown_rule: None,
        }
    }

    pub fn header_negated(&self) -> bool {
        self.flags.contains("not")
    }
}
