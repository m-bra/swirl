use std::collections::{BTreeMap, HashMap};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
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
