#![allow(non_snake_case)]

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MatchError {
    fatal: bool,
    error_type: ErrorType,
    backtrace: Vec<String>
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ErrorType {
    Generic {msg: String, subErrors: Vec<MatchError>},
}

impl MatchError {
    pub fn is_fatal(&self) -> bool {self.fatal}

    pub fn new(msg: impl AsRef<str>) -> MatchError {
        MatchError { 
            error_type: ErrorType::Generic {
                msg: msg.as_ref().to_string(),
                subErrors: vec![]
            },
            fatal: false,
            backtrace: vec![]
        }
    }
    
    pub fn compose(msg: impl AsRef<str>, subErrors: Vec<MatchError>) -> MatchError {
        MatchError { 
            error_type: ErrorType::Generic {
                msg: msg.as_ref().to_string(),
                subErrors: subErrors,
            },
            fatal: false,
            backtrace: vec![]
        }
    }

    // \n included
    fn display_without_backtrace(&self, indent: impl AsRef<str>) -> String {
        let indent = indent.as_ref();

        match &self.error_type {
            ErrorType::Generic {msg, subErrors} => {
                let subs = subErrors.iter().map(|err| {
                    err.display_without_backtrace(indent.to_string() + "  ")
                }).collect::<Vec<_>>().join("");
                format!("{}{}\n{}candidates: {}\n{}\n", indent, msg, indent, subErrors.len(), subs)
            }
        }
        
    }

    pub fn expected(expected: &str, input: &str) -> MatchError {
        MatchError {
            error_type: ErrorType::Generic {
                msg: format!("Expected {}, got {}", expected, error_region(input)),
                subErrors: vec![],
            },
            fatal: false,
            backtrace: vec![],
        }
    }

    pub fn unknown_variable(var_ident: &str, input: &str) -> MatchError {
        MatchError {
            error_type: ErrorType::Generic {
                msg: format!("Unknown variable '{}': {}", var_ident, error_region(input)),
                subErrors: vec![],
            },
            fatal: true,
            backtrace: vec![],
        }
    }

    pub fn unknown_rule(rule_ident: &str, input: &str) -> MatchError {
        MatchError {
            error_type: ErrorType::Generic {
                msg: format!("Unknown rule: '{}': {}", rule_ident, error_region(input)),
                subErrors: vec![],
            },
            fatal: true,
            backtrace: vec![],
        }
    }
}

use std::fmt;
impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bt = self.backtrace.join("\n");
        write!(f, "{} at {}", self.display_without_backtrace(""), bt)
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

// return until end of line (in simple quotes), or (if input is at end of line), return "end of line" without quotes
pub fn error_region(input: &str) -> String {
    let line = input.lines().next().unwrap_or("");
    if line.is_empty() {
        "end of line".to_string()
    } else {
        line.to_string()
    }
}
