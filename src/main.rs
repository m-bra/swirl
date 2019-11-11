#![allow(dead_code)]
#![allow(unused_imports)]

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

mod ruletypes;
use ruletypes::*;

mod applicationsleft;
use applicationsleft::*;


#[test]
fn test_match_rule_def() {
    let header = || {
        let mut header = Header::new();
        header.add_char('.');
        header.add_invoc(RuleInvocation("".into(), "rule".into()));
        header.add_char('}');
        header.seal()
    };

    let body = {
        let mut body = Body::new();
        body.add_invoc(VarInvocation ("var".into()));
        body.add_char(':');
        body.add_invoc(VarInvocation ("othervar".into()));
        body.seal()
    };

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}19"),
        Ok(("19", ("name1".into(), RuleVariant {
            match_: header(),
            replace: None,
            append: "".into(),
        })))
    );

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}    {:var.::othervar}19"),
        Ok(("19", ("name1".into(), RuleVariant {
            match_: header(),
            replace: Some(body),//once told me
            append: "".into(),
        })))
    );
}

fn match_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    let input = match_char(input, '%')?;
    let input = match_char(input, ':')?;
    match_inner_rule_definition(input)
}

/// matches the parts of a rule after '%:' (so that caller might scan for '%:' instead of calling this function everytime)
fn match_inner_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    // ruleName
    let input = match_whitespaces(input)?;
    let (input, rule_name) = match_ident(input).unwrap_or((input, ""));
    let rule_name = rule_name.into();
    let input = match_whitespaces(input)?;

    // {header with :rule:invocation.s} {body with :var.s}
    let header_start = input;
    let (input, header) = match_rule_part(input, match_invocation)?;
    let header = header.ok_or_else(|| MatchError::expected("Rule header", header_start))?;
    let input = match_whitespaces(input)?;
    let (input, body) = match_rule_part(input, match_var)?;
    
    Ok (
        (input, (rule_name, RuleVariant {match_: header, replace: body, append: String::new()}))
    )
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
                    match_: Header::literally("0"),
                    replace: None,
                    append: "".to_string()
                },
                RuleVariant {
                    match_: Header::literally("1"),
                    replace: None,
                    append: "".to_string(),
                }
            ]
        });
        let mut results = HashMap::new();
        results.insert("n1".to_string(), "1".to_string());
        results.insert("n2".to_string(), "0".to_string());
        let anon_results = vec!["1".to_string()];

        let (_, rule_head) = match_rule_part("{I have.::n1:number:n2:number.:apples......::number}", match_invocation).unwrap();
        let rule_head = rule_head.unwrap();

        assert_eq!(
            Self::match_rule_head(
                "I have:10:apples...1 and 2 bananas", &rule_head, &rules
            ),
            Ok((" and 2 bananas", (results, anon_results)))
        );
    }

    // match rule header with the start of "input", possibly invoking other rules
    // bind results of invocations to the specified variables (:var:rule)
    // bind results of anonymous invocations to vector in correct order (::rule)
    // return advanced input pointer or MatchError and bind results
    fn match_rule_head<'a>(input: &'a Input, rule_head: &Header, rules: &Rules)
                -> MatchResult<(&'a Input, (HashMap<String, String>, Vec<String>))> {
        let mut results = HashMap::new();
        let mut anon_results = vec![];
        let mut input: &'a Input = input;

        for (part, invocs) in rule_head.iter() {
            input = match_str(input, part)?;

            for &RuleInvocation(ref var, ref rule) in invocs {
                input = {
                    let (input, result) = apply_last(input, &rule, rules)?;

                    if !var.is_empty() {
                        // TODO: could panic here
                        assert!(!results.contains_key(&var.to_string()));
                        results.insert(var.to_string(), result);
                    } else {
                        anon_results.push(result);
                    }

                    input
                };
            }
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
        Self::replace_matches(text, match_var_, &(), |input, VarInvocation(ident)| {
            variables.get(&ident).ok_or(MatchError::unknown_variable(&ident, input))
        })
    }

    /// try matching one rule variant and resolve the result text
    /// return the remaining unconsumed input and the replacement string
    fn try_match<'a>(&self, input: &'a str, rules: &Rules, _name: impl AsRef<str>) -> MatchResult<(&'a str, String)> {
        let _btmatch = format!(" {{{}}}", self.match_);    
        //backtrace.push(format!("%: {} {} on '{}'", name.as_ref(), btmatch, firstline(input)));
       // println!("%: {} {} on '{}'", _name.as_ref(), btmatch, _firstline(input));

        // TODO: view compiled assembly
        (|| {
            let match_ = &self.match_;

            let (input_after_match, (results, anon_results)) = RuleVariant::match_rule_head(input, &match_, rules)?;
                
            match &self.replace {
                None => {
                    let mut anon_i = 0;
                    (input_after_match, self.match_.iter().fold(String::new(), |mut buf, (part, invocations)| {
                        buf.push_str(part);
                        for RuleInvocation(var, _) in invocations {
                            if !var.is_empty() {
                                buf.push_str(&results[var]);
                            } else {
                                buf.push_str(&anon_results[anon_i]);
                                anon_i += 1;
                            }
                        }
                        buf
                    }))
                }
                Some(replace) => {
                    (input_after_match, replace.iter().fold(String::new(), |mut buf, (part, invocations)| {
                        buf.push_str(part);
                        for VarInvocation(var) in invocations {
                            assert!(!var.is_empty());
                            buf.push_str(&results[var]);
                        }
                        buf
                    }))
                }
            }.tap(Ok)
        })().tap(|result| {
            //backtrace.pop();
            result
        })
        
    }
}

#[test]
fn test_apply_last() {
    let mut rules = HashMap::new();
    rules.insert("digit".to_string(), Rule {
        name: "digit".to_string(),
        variants: vec![
            RuleVariant {
                match_: Header::literally("0"), replace: None, append: "".to_string(),
            },
            RuleVariant {
                match_: Header::literally("1"), replace: None, append: "".to_string(),
            }
    ]});
    rules.insert("digits".to_string(), Rule {
        name: "digits".to_string(),
        variants: vec![
            RuleVariant {
                match_: parse_header("::digit").unwrap(),
                replace: None, 
                append: "".to_string(),
            },
            RuleVariant {
                match_: parse_header(":d:digit::digits").unwrap(), 
                replace: Some(parse_body("Two .times: :d:d").unwrap()), 
                append: "".to_string(),
            },
    ]});

    assert_eq!(apply_last("01110d01", "digits", &rules), Ok(("d01", "Two times: 00".to_string())));
    assert!(apply_last("abcde", "digits", &rules).is_err());
}

fn apply_last<'a>(input: &'a str, name: &str, rules: &Rules) -> Result<(&'a str, String), MatchError> {
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

fn apply_sequence(input: &str, name: &str, rules: &Rules, appleft: &mut MaybeInf<u32>) -> Result<String, MatchError> {
    let mut input = input.to_string();
    for variant in rules.get(name).map(|r| &r.variants).unwrap_or(&Vec::new()).iter().rev() {
        //backtrace.push(format!("%: {{{}}}", variant.match_.as_ref().unwrap_or(&"".to_string())));
        //let _f =  finally(|| {backtrace.pop();});

        if *appleft == MaybeInf::Finite(0u32) {
            break;
        }

        *appleft-= 1;

        let (unconsumed, replace) = variant.try_match(&input, rules, "")?;
        input = replace + unconsumed;
    }
    Ok(input)
}

fn process(input: &str, rules: &mut Rules, mut appleft: MaybeInf<u32>) -> Result<String, MatchError> {
    let mut result = String::new();
    let mut input = input.to_string();

    while let Some(start) = input.find("%:") {
        if appleft == MaybeInf::Finite(0) {
            break;
        }

        let (after_def, (name, variant)) = match_rule_definition(&input[start..])?;
        let def_length = input[start..].len() - after_def.len();
        let end = start + def_length;

        // add variant to definitions
        let name = || name.clone();
        rules.entry(name()).or_insert(Rule::new(name())).variants.push(variant);
        let name = name();

        // all text until the current rule definition remains untouched (because it is between the beginning/a rule definition and a rule definition)
        // so just push it to the result string
        // whether or not to remove rule definition from processed output ([..end] vs [..start])
        result.push_str(&input[..end]);

        if !name.is_empty() {
            // next portion to process is after the current rule definition
            input = input[end..].to_string();
        } else {
            // next portion to process is the output of application of the current rule definition (piped to all previous unnamed rule definitions)
            let new_input = apply_sequence(&input[end..], name, rules, &mut appleft)?;
            input = new_input;
        }
        
    }

    // the rest of the input contains no more rule definitions, so push it to the results
    result.push_str(&input);
    Ok(result)
}

use std::io::{self, Read, Write};
use std::fs::File;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut target = "input.txt".to_string();
    let mut userline = String::new();

    print!(" $ ");
    io::stdout().flush()?;

    while stdin.read_line(&mut userline).is_ok() {
        {
            let userline: Vec<&str> = userline.split(" ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if userline.len() == 0 {continue;}

            if userline[0] == "quit" {
                break;
            } else if userline[0] == "target" {
                target = userline.get(1).map(|s| s.to_string()).unwrap_or_else(|| {
                    println!("No target given.");
                    target
                });
            } else if userline[0] == "s" || userline[0] == "step" {
                File::open(&target)?.read_to_string(&mut buffer)?;
                let step_count: &str = userline.get(1).unwrap_or(&"1");
                let step_count: u32 = step_count.parse().unwrap();
                let result = process(&buffer, &mut HashMap::new(), MaybeInf::Finite(step_count))?;
                File::create(&target)?.write(result.as_bytes())?;
            } else {
                println!("unknown command '{}'", userline[0]);
            }
        }

        print!(" $ ");
        io::stdout().flush()?;
        userline.clear();
    }

    Ok(())
}

#[test]
fn test_input() {
    (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = String::new();
        File::open("testinput.txt")?.read_to_string(&mut buffer)?;

        let result = process(&buffer, &mut HashMap::new(), MaybeInf::Infinite)?;
        let last_line = result.lines().last().unwrap();
        assert_eq!(last_line, "success: testescape.)");

        Ok(())
    })().unwrap();
}
