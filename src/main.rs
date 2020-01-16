#![allow(dead_code)]
#![allow(unused_imports)]

use regex::Regex;
use std::collections::HashMap;

mod util;
use util::*;

mod error;
use error::*;

mod mmatch;
use mmatch::*;

#[macro_use]
extern crate lazy_static;

pub const ESCAPE_CHAR: char = '.';
pub const RULE_INVOCATION_CHAR: char = ':';

// todo: idea:
// program watches input file
// and then whenever it changes it updates input.txt.1 with the first stage
// input.txt.2 with the second stage
// and input.txt.n with the last stage (both literally 'input.txt.n' and 'input.txt.5' if n=5)
// enables interactive usage and viewing stages directly with shortcuts

// todo: remove unescaped whitespace

mod types;
use types::*;

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
            header: header(),
            body: None,
            append: "".into(),
        })))
    );

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}    {:var.::othervar}19"),
        Ok(("19", ("name1".into(), RuleVariant {
            header: header(),
            body: Some(body),//once told me
            append: "".into(),
        })))
    );
}

pub fn match_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    let input = match_char(input, '%')?;
    let input = match_char(input, ':')?;
    match_inner_rule_definition(input)
}

/// matches the parts of a rule after '%:' (so that caller might scan for '%:' instead of calling this function everytime)
pub fn match_inner_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
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
        (input, (rule_name, RuleVariant {header: header, body: body, append: String::new()}))
    )
}


#[test]
fn _test_replace_vars() {
    let mut vars = HashMap::new();
    vars.insert("v1".to_string(), ".!".to_string());
    vars.insert("v2".to_string(), "not used".to_string());
    assert_eq!(
        replace_vars(":v1.:..:v1", &vars), Ok(".!:..!".to_string()),
    );
    assert!(
        replace_vars(":nonexistingvariable", &vars).is_err()
    )
}

/// processes and escapes `escape_text`, replacing occurences of `match_fn` with its output piped into the `replace` function
/// yes we are processing escapes here, because that needs to be coupled with searching matches
/// so that occurences can always be escaped and 'hidden' from this function
/// errors in `match_fn` will be treated as "cant match; ignore", while errors in `replace` will be returned by this function
pub fn replace_matches<'a, 'b, In, Out, AR>(escape_text: &'a str, mut match_fn: impl MatchFn<'a, &'b In, Out>, param: &'b In,
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
pub fn replace_vars(text: &str, variables: &HashMap<String, String>) -> MatchResult<String> {
    replace_matches(text, match_var_, &(), |input, VarInvocation(ident)| {
        variables.get(&ident).ok_or(MatchError::unknown_variable(&ident, input))
    })
}

#[test]
fn test() {
    // in my mind it works like that
    assert_eq!(&"01234"[..2], "01");
}

#[test]
fn test_apply_last() {
    let mut rules = HashMap::new();
    let ruleDigit = Rule {
        name: "digit".to_string(),
        variants: vec![
            RuleVariant {
                header: Header::literally("0"), body: None, append: "".to_string(),
            },
            RuleVariant {
                header: Header::literally("1"), body: None, append: "".to_string(),
            }
    ]};
    let ruleDigits = Rule {
        name: "digits".to_string(),
        variants: vec![
            RuleVariant {
                header: parse_header("::digit").unwrap(),
                body: None,
                append: "".to_string(),
            },
            RuleVariant {
                header: parse_header(":d:digit::digits").unwrap(),
                body: Some(parse_body("Two .times: :d:d").unwrap()),
                append: "".to_string(),
            },
    ]};
    rules.insert("digit".to_string(), ruleDigit.clone());
    rules.insert("digits".to_string(), ruleDigits.clone());

    assert_eq!(ruleDigits.apply_last("01110d01", &rules), Ok(("d01", "Two times: 00".to_string())));
    assert!(ruleDigits.apply_last("abcde", &rules).is_err());
}

pub fn process(input: &str, rules: &mut Rules, mut appleft: MaybeInf<u32>) -> Result<String, MatchError> {
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
            let new_input = rules[&name].apply_sequence(&input[end..], rules, &mut appleft)?;
            input = new_input;
        }

    }

    // the rest of the input contains no more rule definitions, so push it to the results
    result.push_str(&input);
    Ok(result)
}

fn process_file(target: &str, steps: MaybeInf<u32>) -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    File::open(&target)?.read_to_string(&mut buffer)?;

    let result = process(&buffer, &mut HashMap::new(), steps)?;

    File::create(&target)?.write(result.as_bytes())?;

    Ok(())
}

use std::io::{self, Read, Write};
use std::fs::File;
use std::error::Error;

fn repl() -> Result<(), Box<dyn Error>> {
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
            } else if userline[0] == "s_unsupported" || userline[0] == "step_unsupported" {
                let step_count: &str = userline.get(1).unwrap_or(&"1");
                let step_count: u32 = step_count.parse().unwrap();
                process_file(&target, MaybeInf::Finite(step_count))?;
            } else if userline[0] == "r" || userline[0] == "run" {
                process_file(&target, MaybeInf::Infinite)?;
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

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(arg) = std::env::args().skip(1).next() {
        process_file(arg, MaybeInf::Infinite);
    } else {
        process_file("input.txt", MaybeInf::Infinite);
    }
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
