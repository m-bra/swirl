
use regex::Regex;
use std::collections::HashMap;

mod tap;
use tap::*;

mod util;
#[allow(unused_imports)]
use util::*;

#[macro_use]
extern crate lazy_static;

mod error;
use error::*;

pub const ESCAPE_CHAR: char = '.';
pub const RULE_INVOCATION_CHAR: char = ':';

// todo: idea:
// program watches input file
// and then whenever it changes it updates input.txt.1 with the first stage
// input.txt.2 with the second stage
// and input.txt.n with the last stage (both literally 'input.txt.n' and 'input.txt.5' if n=5)
// enables interactive usage and viewing stages directly with shortcuts

// todo: remove unescaped whitespace

// todo: %: rule {:x:rule1 :y:rule2} should be {:x:y}

#[derive(PartialEq, Eq, Debug)]
struct RuleVariant {
    match_: Option<String>,
    replace: Option<String>,
    append: String,
}

#[derive(PartialEq, Eq, Debug)]
struct Rule {
    name: String,
    variants: Vec<RuleVariant>,
}

impl Rule {
    pub fn new_empty(name: String) -> Rule {
        Rule {
            name: name,
            variants: Vec::new(),
        }
    }
}

type Rules = HashMap<String, Rule>;


#[test]
fn test_process_escapes() {
    // ESCAPE_CHAR == .
    assert_eq!(_process_escapes(".{abc.d.e.}.."), Some("{abcde}.".to_string()));
    assert_eq!(_process_escapes("aaaa..."), None);
}

/// none if string ends in non-escaped "."
fn _process_escapes(text: &str) -> Option<String> {
    let mut result = String::new();
    let mut escape = false;
    for c in text.chars() {
        if escape {
            result.push(c);
            escape = false;
        } else {
            if c == ESCAPE_CHAR {
                escape = true;
            } else {
                result.push(c);
            }
        }
    }
    if escape {
        None
    } else {
        Some(result)
    }
}

#[test]
fn test_find_first_definition() {
    assert_eq!(
        find_first_definition("0123%:  name1{...}}19"),

        Some(((4, 19), "name1".to_string(), RuleVariant {
            match_: Some("...}".to_string()),
            replace: None,
            append: "".to_string(),
        }
    )));
}

fn find_first_definition(grammar: &str) -> Option<((usize, usize), String, RuleVariant)> {
    lazy_static! {
    static ref RULE_RE: Regex = {
        assert_eq!(ESCAPE_CHAR, '.'); // all regexes in lazy_static! use . as escape char
        assert_eq!(RULE_INVOCATION_CHAR, ':'); // and : as rule invocation
        Regex::new(r"%:(?:\s*([a-zA-Z0-9_]+))?(?:\s*\{((?:[^\.\}]|(?:\..))*)\}(?:\s*\{((?:[^\.\}]|(?:\..))*)\}(?:\s*\{((?:[^\.\}]|(?:\..))*)\})?)?)?").unwrap()
        // https://regex101.com/r/Jlvyng/2
    };
}
    RULE_RE.captures_iter(&grammar).next().map(|capture| {
        let name = capture.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        ((capture.get(0).unwrap().start(), capture.get(0).unwrap().end()), name, RuleVariant {
            match_:  capture.get(2).map(|x| (x.as_str().to_string())),
            replace: capture.get(3).map(|x| (x.as_str().to_string())),
            append:  capture.get(4).map(|x| (x.as_str().to_string())).unwrap_or("".to_string())
        })
    })
}


#[test]
fn test() {
    // in my mind it works like that
    assert_eq!(&"01234"[..2], "01");
}

#[test]
fn test_match_rule_head() {
    RuleVariant::_test_match_rule_head();
}

#[test]
fn test_replace_vars() {
    RuleVariant::_test_replace_vars();
}

mod mmatch;
use mmatch::*;

impl RuleVariant {
    fn _test_match_rule_head() {
        let mut rules = HashMap::new();
        rules.insert("number".to_string(), Rule {
            name: "number".to_string(),
            variants: vec![
                RuleVariant {
                    match_: Some("0".to_string()),
                    replace: None,
                    append: "".to_string()
                },
                RuleVariant {
                    match_: Some("1".to_string()),
                    replace: None,
                    append: "".to_string(),
                }
            ]
        });
        let mut results = HashMap::new();
        results.insert("n1".to_string(), "1".to_string());
        results.insert("n2".to_string(), "0".to_string());
        let anon_results = vec!["1".to_string()];
        assert_eq!(
            Self::match_rule_head(
                "I have:10:apples...1 and 2 bananas", "I have.::n1:number:n2:number.:apples......::number", &rules
            ),
            Ok((" and 2 bananas", (results, anon_results)))
        );
    }

    // match rule header with the start of "input", possibly invoking other rules, escaping the rule header
    // bind results of invocations to the specified variables (:var:rule)
    // bind results of anonymous invocations to vector in correct order (::rule)
    // return advanced input pointer or MatchError and bind results
    fn match_rule_head<'a>(input: &'a Input, rule_head: &str, rules: &Rules)
                -> MatchResult<(&'a Input, (HashMap<String, String>, Vec<String>))> {
        let mut results = HashMap::new();
        let mut anon_results = vec![];
        let mut input: &'a Input = input;
        let mut rule_head = rule_head;
        while !rule_head.is_empty() {
            match match_invocation(rule_head, &()) {
                Ok((rule_head, (var, rule))) => {
                    let (input, result) = apply_named_rule(input, &rule, rules)?;

                    if !var.is_empty() {
                        // TODO: could panic here
                        assert!(!results.contains_key(&var.to_string()));
                        results.insert(var.to_string(), result);
                    } else {
                        anon_results.push(result);
                    }

                    (input, rule_head)
                }
                Err(_) => {
                    let (rule_head, c) = match_escapable_char(rule_head, ESCAPE_CHAR)?;
                    let input = match_char(input, c)?;
                    (input, rule_head)
                }
            }
            .tap(|(new_input, new_rule_head)| {input = new_input; rule_head = new_rule_head;})
        }

        Ok((input, (results, anon_results)))
    }

    fn _test_replace_vars() {
        let mut vars = HashMap::new();
        vars.insert("v1".to_string(), ".!".to_string());
        vars.insert("v2".to_string(), "not used".to_string());
        assert_eq!(
            RuleVariant::replace_vars(":v1.:..:v1", &vars), Ok(".!:..!".to_string()),
        );
        assert!(
            RuleVariant::replace_vars(":nonexistingvariable", &vars).is_err()
        )
    }

    /// processes and escapes `escape_text`, replacing occurences of `match_fn` with its output piped into the `replace` function
    /// yes we are processing escapes here, because that needs to be coupled with searching matches
    /// so that occurences can always be escaped and 'hidden' from this function
    /// errors in `match_fn` will be treated as "cant match; ignore", while errors in `replace` will be returned by this function
    fn replace_matches<'a, 'b, In, Out, AR>(escape_text: &'a str, mut match_fn: impl MatchFn<'a, &'b In, Out>, param: &'b In, 
            mut replace: impl FnMut(&str, Out) -> MatchResult<AR>) -> MatchResult<String> where AR: AsRef<str> {
        let mut text = escape_text;
        let mut result = String::with_capacity(text.len() * 2);
        while !text.is_empty() {
            text = match match_fn(text, param) {
                Err(_) => {
                    let (text, c) = match_escapable_char(text, '.')?;
                    result.push(c);
                    text
                }
                Ok((text, out)) => {
                    result.push_str(replace(text, out)?.as_ref());
                    text
                }
            }
        }
        Ok(result)
    }

    /// replaces occurences of :var with the associated string
    /// taking into account escaped ".:", then processing escapes
    fn replace_vars(text: &str, variables: &HashMap<String, String>) -> MatchResult<String> {
        Self::replace_matches(text, match_var, &(), |input, ident| variables.get(ident).ok_or(MatchError::unknown_variable(ident, input)))
    }

    /// try matching one rule variant and resolve the result text
    /// return the remaining unconsumed input and the replacement string
    fn try_match<'a>(&self, input: &'a str, rules: &Rules, _name: impl AsRef<str>) -> MatchResult<(&'a str, String)> {
        //let btmatch = self.match_.as_ref().map(|s| format!(" {{{}}}", s)).unwrap_or("".to_string());    
        //backtrace.push(format!("%: {} {} on '{}'", name.as_ref(), btmatch, firstline(input)));

        // TODO: view compiled assembly
        (|| {
            let match_ = self.match_.as_ref()
                    .ok_or(MatchError::new(format!("Variant has no header, can't match."), &mut vec![]))?;

            let (input_after_match, (results, anon_results)) = RuleVariant::match_rule_head(input, match_, rules)?;
                
            match &self.replace {
                None => {
                    let mut anon_i = 0;
                    // result is rule header with invocations replaced by their results
                    (input_after_match, Self::replace_matches(match_, match_invocation, &(), |_, (var, rule)| {
                        if !var.is_empty() {
                            &results[var]
                        } else {
                            anon_i+= 1;
                            &anon_results[anon_i - 1]
                        }.tap(Ok)
                    }).expect("MatchErrors should already have been thrown on earlier call on match_rule_head!??"))
                }
                Some(replace) => {
                    (input_after_match, RuleVariant::replace_vars(&replace, &results)?)
                }
            }.tap(Ok)
        })().tap(|result| {
            //backtrace.pop();
            result
        })
        
    }
}

#[test]
fn test_apply_named_rule() {
    let mut rules = HashMap::new();
    rules.insert("digit".to_string(), Rule {
        name: "digit".to_string(),
        variants: vec![
            RuleVariant {
                match_: Some(".0".to_string()), replace: None, append: "".to_string(),
            },
            RuleVariant {
                match_: Some("1".to_string()), replace: None, append: "".to_string(),
            }
    ]});
    rules.insert("digits".to_string(), Rule {
        name: "digits".to_string(),
        variants: vec![
            RuleVariant {
                match_: Some("::digit".to_string()), replace: None, append: "".to_string(),
            },
            RuleVariant {
                match_: Some(":d:digit::digits".to_string()), replace: Some("Two .times: :d:d".to_string()), append: "".to_string(),
            },
    ]});

    assert_eq!(apply_named_rule("01110d01", "digits", &rules), Ok(("d01", "Two times: 00".to_string())));
    assert!(apply_named_rule("abcde", "digits", &rules).is_err());
}

fn apply_named_rule<'a>(input: &'a str, name: &str, rules: &Rules) -> Result<(&'a str, String), MatchError> {
    let variants = &rules.get(name).ok_or(MatchError::new(format!("Rule '{}' does not exist.", name), &mut vec![]))?.variants;
    let mut candidate_errors = Vec::new();
    for v in variants.iter().rev() {
        match v.try_match(input, rules, name) {
            Ok((input, result)) => return Ok((input, result)),
            Err(err) => candidate_errors.push(err),
        }
    }
    return Err(MatchError::compose(format!("No variant of '{}' matched.", name), candidate_errors, &mut vec![]));
}

fn apply_unnamed_rules(input: &str, rules: &Rules) -> Result<String, MatchError> {
    let mut input = input.to_string();
    for variant in rules.get("").map(|r| &r.variants).unwrap_or(&Vec::new()).iter().rev() {
        //backtrace.push(format!("%: {{{}}}", variant.match_.as_ref().unwrap_or(&"".to_string())));
        //let _f =  finally(|| {backtrace.pop();});

        let (unconsumed, replace) = variant.try_match(&input, rules, "")?;
        input = replace + unconsumed;
    }
    Ok(input)
}

fn process(input: &str, rules: &mut Rules) -> Result<String, MatchError> {
    if let Some(((start, end), name, variant)) = find_first_definition(input) {
        let name = || name.clone();
        rules.entry(name()).or_insert(Rule::new_empty(name())).variants.push(variant);
        let name = name();
        // whether or not to remove rule definition from processed output
        input[..end].to_string() + &{
            if !name.is_empty() {
                process(&input[end..], rules)?
            } else {
                process(apply_unnamed_rules(&input[end..], rules)?.as_ref(), rules)?
            }
        }
    } else {
        input.to_string()
    }.tap(Ok)
}

use std::io::{self, Read};
use std::fs::File;

fn main() -> io::Result<()> {

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut _handle = stdin.lock();

    File::open("input.txt")?.read_to_string(&mut buffer)?;

    //buffer.push_str(r"");

    match process(&buffer, &mut HashMap::new()) {
        Ok(result) => println!("{}", result),
        Err(err) => eprint!("Error: {}", err)
    }

    Ok(())
}

#[test]
fn test_input() {
    (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = String::new();
        File::open("testinput.txt")?.read_to_string(&mut buffer)?;

        let result = process(&buffer, &mut HashMap::new())?;
        let last_line = result.lines().last().unwrap();
        assert_eq!(last_line, "success: testescape.)");

        Ok(())
    })().unwrap();
}
