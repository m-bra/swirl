use std::collections::{BTreeMap, HashMap};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub variants: Vec<RuleVariant>,
    _is_macro: bool,
}

impl Rule {
    pub fn new(name: String) -> Rule {
        Rule {
            variants: Vec::new(),
            _is_macro: name == "swirlcl",
            name: name,
        }
    }

    pub fn variant(mut self, v: RuleVariant) -> Rule {
        self.variants.push(v);
        self
    }

    pub fn is_macro(&self) -> bool {
        self._is_macro
    }
}
