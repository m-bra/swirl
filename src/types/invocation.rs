use std::collections::{BTreeMap, HashMap};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Invocation {
    RuleInvocation(String, String, InvocationString),
    VarInvocation(String)
}

impl Invocation {
    pub fn new_rule_invocation(result_var: impl Into<String>, rule: impl Into<String>) -> Invocation {
        Invocation::RuleInvocation(result_var.into(), rule.into(), InvocationString::empty())
    }

    pub fn new_rule_invoc_with_param(result_var: impl Into<String>, rule: impl Into<String>, parameter: InvocationString) -> Invocation {
        Invocation::RuleInvocation(result_var.into(), rule.into(), parameter)
    }

    pub fn new_var_invocation(name: impl Into<String>) -> Invocation {
        Invocation::VarInvocation(name.into())
    }
}
