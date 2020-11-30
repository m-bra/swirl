use crate::*;
use std::fmt;

impl fmt::Display for Invocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Invocation::RuleInvocation(var, rule, invoc_str) if *invoc_str == InvocationString::empty() =>
                write!(f, ":{}:{}", var, rule),
            Invocation::RuleInvocation(var, rule, invoc_str) =>
                write!(f, ":{}:{}({})", var, rule, invoc_str),
            Invocation::VarInvocation(var) =>
                write!(f, ":{}", var),
        }
    }
}

impl fmt::Display for InvocationString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (part, invocations) in unsafe {self.iter()} {
            if !part.is_empty() {
                write!(f, "'{}`", part)?;
            }
            for invocation in invocations {
                if let Invocation::RuleInvocation(_, ident, _) = invocation {
                    if ident == SWIRL_INSERTED_WHITESPACE_IDENT {
                        write!(f, " ")?;
                    } else {
                        write!(f, "{}", invocation)?;
                    }
                } else {
                    write!(f, "{}", invocation)?;
                }
            }
        }
        Ok(())
    }
}
