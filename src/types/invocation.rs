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

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarInvocation(pub String);

impl VarInvocation {
    pub fn new(name: impl Into<String>) -> VarInvocation {
        VarInvocation(name.into())
    }

    pub fn var_name(&self) -> &str {&self.0}
}
