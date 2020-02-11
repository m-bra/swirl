use crate::*;
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

impl<Invocation: Clone> fmt::Display for RulePart<Invocation> where Invocation: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (part, invocations) in self.iter() {
            if part.contains(char::is_whitespace) {
                write!(f, "{{'{}'}} ", part)?;
            } else {
                write!(f, "{} ", part)?;
            }
            for invocation in invocations {
                write!(f, "{} ", invocation)?;
            }
        }
        Ok(())
    }
}
