use std::collections::{BTreeMap, HashMap, HashSet};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleVariant {
    pub parameter_header: Option<InvocationString>,
    pub header: InvocationString,
    pub body: Option<InvocationString>,
    pub flags: HashSet<String>,
    pub catch_unknown_rule: Option<InvocationString>,
}

impl RuleVariant {
    pub fn new(header: InvocationString, body: Option<InvocationString>) -> RuleVariant {
        RuleVariant {
            parameter_header: None,
            header,
            body,
            flags: HashSet::new(),
            catch_unknown_rule: None,
        }
    }

    pub fn header_negated(&self) -> bool {
        self.flags.contains("not")
    }

    pub fn shallow_call(&self) -> bool {
        self.flags.contains("call")
    }

    pub fn deep_call(&self) -> bool {
        !self.shallow_call()
    }

    // since this is called often, it might be worth it to optimize this 
    pub fn is_any(&self) -> bool {
        self.flags.contains("any")
    }
}
