#![allow(non_snake_case)]

use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MatchError {
    _fatal: bool,
    error_type: ErrorType,
    backtrace: Vec<String>
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ErrorType {
    Generic {msg: String, subErrors: Vec<MatchError>},
    UnknownRule {msg: String}
}

impl MatchError {
    pub fn is_fatal(&self) -> bool {self._fatal}

    pub fn is_unknown_rule(&self) -> bool {
        match self.error_type {
            ErrorType::UnknownRule {msg: _} => true,
            _ => false
        }
    }

    pub fn new(msg: impl AsRef<str>) -> MatchError {
        MatchError { 
            error_type: ErrorType::Generic {
                msg: msg.as_ref().to_string(),
                subErrors: vec![]
            },
            _fatal: false,
            backtrace: vec![]
        }
    }
    
    pub fn compose(msg: impl AsRef<str>, subErrors: Vec<MatchError>) -> MatchError {
        MatchError { 
            error_type: ErrorType::Generic {
                msg: msg.as_ref().to_string(),
                subErrors: subErrors,
            },
            _fatal: false,
            backtrace: vec![]
        }
    }

    pub fn expected(expected: &str, input: &str) -> MatchError {
        MatchError::new(format!("Expected '{}', got '{}'", expected, error_region(input)))
    }

    pub fn unknown_variable(var_ident: &str, input: &str) -> MatchError {
        MatchError::new(format!("Unknown variable '{}': '{}'", var_ident, error_region(input)))
    }

    pub fn unknown_rule(rule_ident: &str, input: &str) -> MatchError {
        MatchError {
            _fatal: false,
            error_type: ErrorType::UnknownRule {
                msg: format!("Unknown rule: '{}': '{}'", rule_ident, error_region(input))
            },
            backtrace: Vec::new()
        }
    }

    pub fn rule_variant_verification_failure(rule_name: &str, rule_variant: &UntrustedRuleVariant, verifier_message: String) -> MatchError {
        MatchError::new(format!("\
            {} --- Failed to verify rule variant %: {} {{{:?}}}\n\
            {} --- {}\
        ", get_indent(), rule_name, rule_variant,
        get_indent(), verifier_message))
    }

    pub fn rejected_tail_optimization(rule_name: &str, reason: String) -> MatchError {
        MatchError::new(format!("{} --- Rejected tail optimization of rule '{}': {}", get_indent(), rule_name, reason))
    }

    pub fn fatal(mut self) -> MatchError {
        self._fatal = true;
        self
    }
}


impl MatchError {
    // \n included
    fn display_without_backtrace(&self, indent: impl AsRef<str>, with_candidates: bool) -> String {
        let indent = indent.as_ref();

        match &self.error_type {
            ErrorType::Generic {msg, subErrors} => {
                if with_candidates && !subErrors.is_empty() {
                    let subs = subErrors.iter().map(|err| {
                        err.display_without_backtrace(indent.to_string() + "  ", true)
                    }).collect::<Vec<_>>().join("");
                    format!("{}{}\n{}candidates: {}\n{}\n", indent, msg, indent, subErrors.len(), subs)
                } else {
                    format!("{}{}\n", indent, msg)
                }
            }
            ErrorType::UnknownRule {msg} => {
                format!("{}{}\n", indent, msg)
            }
        }
    }

    pub fn short_display(&self) -> String {
        self.display_without_backtrace("", false)
    }
}

use std::fmt;
impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bt = self.backtrace.join("\n");
        write!(f, "{} at {}", self.display_without_backtrace("", true), bt)
    }
}

use std::error::Error;

impl Error for MatchError {}

pub type MatchResult<T> = Result<T, MatchError>;

pub trait SetBacktrace: Sized {
    fn set_backtrace(self, bt: Vec<String>) -> Self;
    fn trace(self, tr: String) -> Self;
}

impl SetBacktrace for MatchError {
    fn set_backtrace(mut self, bt: Vec<String>) -> MatchError {
        self.backtrace = bt;
        self
    }

    fn trace(mut self, tr: String) -> MatchError {
        self.backtrace.push(tr);
        self
    }
}

impl<T> SetBacktrace for MatchResult<T> {
    fn set_backtrace(self, bt: Vec<String>) -> MatchResult<T> {
        self.map_err(|err| err.set_backtrace(bt))
    }

    fn trace(self, tr: String) -> MatchResult<T> {
        self.map_err(|err| err.trace(tr))
    }
}

pub fn trace<T>(msg: String, f: impl FnOnce() -> MatchResult<T>) -> MatchResult<T> {
    f().trace(msg)
}

// return until end of line (in simple quotes), or (if input is at end of line), return "end of line" without quotes
pub fn error_region(input: &str) -> String {
    let line = input.lines().next().unwrap_or("");
    if line.is_empty() {
        "end of line".to_string()
    } else {
        line.to_string()
    }
}
