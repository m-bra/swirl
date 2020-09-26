use std::collections::{BTreeMap, HashMap, HashSet};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UntrustedRuleVariant {
    _parameter_header: Option<InvocationString>,
    _header: InvocationString,
    _body: Option<InvocationString>,
    _flags: HashSet<String>,
    _catch_unknown_rule: Option<InvocationString>,
}

impl UntrustedRuleVariant {
    pub fn body(mut self, invocation_string: InvocationString) -> UntrustedRuleVariant {
        self._body = Some(invocation_string);
        self
    }

    pub fn body_option(mut self, invocation_string_option: Option<InvocationString>) -> UntrustedRuleVariant {
        self._body = invocation_string_option;
        self
    }

    pub fn parameter_header(mut self, invocation_string: InvocationString) -> UntrustedRuleVariant {
        self._parameter_header = Some(invocation_string);
        self
    }

    pub fn parameter_header_option(mut self, invocation_string_option: Option<InvocationString>) -> UntrustedRuleVariant {
        self._parameter_header = invocation_string_option;
        self
    }

    pub fn flag(mut self, string: String) -> UntrustedRuleVariant {
        self._flags.insert(string);
        self
    }

    pub fn flags(mut self, strings: HashSet<String>) -> UntrustedRuleVariant {
        for s in strings {
            self._flags.insert(s);
        }
        self
    }

    pub fn catch_unknown_rule(mut self, invocation_string: InvocationString) -> UntrustedRuleVariant {
        self._catch_unknown_rule = Some(invocation_string);
        self
    }

    pub fn catch_unknown_rule_option(mut self, invocation_string_option: Option<InvocationString>) -> UntrustedRuleVariant {
        self._catch_unknown_rule = invocation_string_option;
        self
    }

    pub fn verify(self, rule_name: &str) -> MatchResult<RuleVariant> {
        if self._flags.contains("debug") {
            MatchError::rule_variant_verification_failure(rule_name, &self, "Error: Flag 'debug' is deprecated.".to_string())
                .tap(Err)
        } else {
            self
                .tap(|untrusted_rule_variant| RuleVariant(untrusted_rule_variant))
                .tap(Ok)
        }
    }
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleVariant(UntrustedRuleVariant);

impl RuleVariant {
    pub fn new(header: InvocationString) -> UntrustedRuleVariant {
        UntrustedRuleVariant {
            _parameter_header: None,
            _header: header,
            _body: None,
            _flags: HashSet::new(),
            _catch_unknown_rule: None,
        }
    }

    pub fn empty() -> RuleVariant {
        Self::new(InvocationString::empty()).verify("<INVALID>").expect("Investigate the source code of this panic.")
    }

    pub fn header_negated(&self) -> bool {
        self.0._flags.contains("not")
    }

    pub fn shallow_call(&self) -> bool {
        !self.deep_call()
    }

    pub fn deep_call(&self) -> bool {
        self.0._flags.contains("syntax")
    }

    pub fn is_undefine(&self) -> bool {
        self.0._flags.contains("clear") || self.0._flags.contains("undefine")
    }


    pub fn on_enter(&self, rule_name: &str, input: &str) {
        if self.0._flags.contains("print") || self.0._flags.contains("print_enter") {
            println!("{} --- Try %: {} {{{}}} on '{}'", get_indent(), rule_name, self.0._header, firstline(input));
            push_indent();
        }

        if self.0._flags.contains("break") || self.0._flags.contains("break_enter") {
            breakpoint();
        }
    }

    pub fn on_success(&self, rule_name: &str, input: &str) {
        if self.0._flags.contains("print") || self.0._flags.contains("print_success") {
            pop_indent();
            println!("{} >>> Success! ::{} on '{}'", get_indent(), rule_name, firstline(input));
        }

        if self.0._flags.contains("break_success") {
            breakpoint();
        }
    }


    pub fn on_failure(&self, rule_name: &str, input: &str) {
        if self.0._flags.contains("print") || self.0._flags.contains("print_failure") {
            pop_indent();
            println!("{} >>> Failure! ::{} on '{}'", get_indent(), rule_name, firstline(input));
        }

        if self.0._flags.contains("break_failure") {
            breakpoint();
        }
    }

    /*

    pub fn is_print(&self) -> bool {
        self.flags.contains("print") || self.flags.contains("debug")
    }

    pub fn break_enter(&self) -> bool {
        self.flags.contains("break_enter") || self.flags.contains("break")
    }

    pub fn break_exit(&self) -> bool {
        self.flags.contains("break_exit")
    }

    */

    // since this is called often, it might be worth it to optimize this 
    pub fn is_any(&self) -> bool {
        self.flags().contains("any")
    }

    pub fn body(&self) -> Option<&InvocationString> {
        self.0._body.as_ref()
    }

    pub fn header(&self) -> &InvocationString {
        &self.0._header
    }

    pub fn parameter_header(&self) -> Option<&InvocationString> {
        self.0._parameter_header.as_ref()
    }

    pub fn flags(&self) -> &HashSet<String> {
        &self.0._flags
    }

    pub fn unknown_rule_catch_body(&self) -> Option<&InvocationString> {
        self.0._catch_unknown_rule.as_ref()
    }
}
