
use regex::Regex;
use std::collections::HashMap;

mod tap;
use tap::*;

mod util;
use util::*;

#[macro_use]
extern crate lazy_static;

mod error;
use error::*;

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
type Input = str;

const ESCAPE_CHAR: char = '.';
const RULE_INVOCATION_CHAR: char = ':';


#[test]
fn test_process_escapes() {
    // ESCAPE_CHAR == .
    assert_eq!(process_escapes(".{abc.d.e.}.."), Some("{abcde}.".to_string()));
    assert_eq!(process_escapes("aaaa..."), None);
}

/// none if string ends in non-escaped "."
fn process_escapes(text: &str) -> Option<String> {
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
    static ref rule_re: Regex = {
        assert_eq!(ESCAPE_CHAR, '.'); // all regexes in lazy_static! use . as escape char
        assert_eq!(RULE_INVOCATION_CHAR, ':'); // and : as rule invocation
        Regex::new(r"%:(?:\s*([a-zA-Z0-9_]+))?(?:\s*\{((?:[^\.\}]|(?:\..))*)\}(?:\s*\{((?:[^\.\}]|(?:\..))*)\}(?:\s*\{((?:[^\.\}]|(?:\..))*)\})?)?)?").unwrap()
        // https://regex101.com/r/Jlvyng/2
    };
}
    rule_re.captures_iter(&grammar).next().map(|capture| {
        let name = capture.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        ((capture.get(0).unwrap().start(), capture.get(0).unwrap().end()), name, RuleVariant {
            match_:  capture.get(2).map(|x| (x.as_str().to_string())),
            replace: capture.get(3).map(|x| (x.as_str().to_string())),
            append:  capture.get(4).map(|x| (x.as_str().to_string())).unwrap_or("".to_string())
        })
    })
}

struct Invocation {
    start: usize,
    end: usize,
    rule: String,
    // variable to bind the result of the rule application
    binding_var: String
}

#[test]
fn test() {
    // in my mind it works like that
    assert_eq!(&"01234"[..2], "01");
}

#[test]
fn test_match_rule_head() {
    RuleVariant::test_match_rule_head();
}

#[test]
fn test_replace_vars() {
    RuleVariant::test_replace_vars();
}

fn match_char(input: &str, expect: char) -> MatchResult<&str> {
    // todo: look at asm
    let err = || MatchError::expected(&expect.to_string(), input).tap(Err);
    if let Some(c) = input.chars().next() {
        if c == expect {
            Ok(&input[1..])
        } else {
            err()
        }
    } else {
        err()
    }
}

fn match_ident(input: &Input) -> MatchResult<(&Input, &str)> {
    // todo: look at asm
    let len = input.chars().take_while(|c| c.is_alphabetic() || c.is_digit(10) || *c == '_').count();
    if len == 0 {
        MatchError::expected("identifier", input).tap(Err)
    } else {
        Ok((&input[len..], &input[..len]))
    }
}

fn match_invocation(input: &Input) -> MatchResult<(&Input, &str, &str)> {
    let input = match_char(input, RULE_INVOCATION_CHAR)?;
    let (input, variable_ident) = match_ident(input)?;
    let input = match_char(input, RULE_INVOCATION_CHAR)?;
    let (input, rule_ident) = match_ident(input)?;
    (input, variable_ident, rule_ident).tap(Ok)
}

fn match_escapable_char(input: &Input, escape: char) -> MatchResult<(&Input, char)> {
    let c1 = input.chars().next()
        .ok_or(MatchError::expected("some char", input))?;

    if c1 == escape {
        let c2 = input.chars().next()
            .ok_or(MatchError::expected("some char", input))?;
        (&input[2..], c2)
    } else {
        (&input[1..], c1)
    }.tap(Ok)
}

impl RuleVariant {
    fn test_match_rule_head() {
        let mut results = HashMap::new();
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
        assert_eq!(
            Self::match_rule_head(
                "I have:10:apples... and 2 bananas", "I have.::n1:number:n2:number.:apples......", &mut results, &rules, &mut vec![]
            ),
            Ok(" and 2 bananas")
        );
        assert_eq!(results.get("n1"), Some("1".to_string()).as_ref());
        assert_eq!(results.get("n2"), Some("0".to_string()).as_ref());
    }

    // match "match_" with the start of "input", possibly invoking other rules
    // "match_" is an unescaped string.
    // bind results of invocations to the specified variables
    // return rest of input (that was not consumed) or MatchError and bind results
    fn match_rule_head<'a>(input: &'a str, rule_head: &str, results: &mut HashMap<String, String>, rules: &Rules, backtrace: &mut Vec<String>)
                -> Result<&'a str, MatchError> {

        fn to_invocation(capture: regex::Match) -> Invocation {
            let second_colon = capture.as_str().rfind(RULE_INVOCATION_CHAR).unwrap();
            // we have to search from the right, because the first char in the match is an arbitrary character before the invocation,
            // which could also be a colon.
            let first_colon = capture.as_str()[..second_colon].rfind(RULE_INVOCATION_CHAR).unwrap();
            Invocation {
                start: capture.start() + first_colon,
                end: capture.end(),
                binding_var: capture.as_str()[first_colon+1..second_colon].to_string(),
                rule: capture.as_str()[second_colon+1..].to_string(),
            }
        };

        lazy_static! {
            // also matches with the character right before the invocation to check if its not ESCAPE_CHAR
            static ref invocation_re: Regex = Regex::new(r"(?x)
                :([a-zA-Z0-9_]*):([a-zA-Z0-9_]+)
            ").unwrap();
        }


        // find first/next invocation in the match string
        match invocation_re.find(rule_head).map(to_invocation) {
            None => {
                let pmatch = process_escapes(rule_head)
                    .ok_or(MatchError::new("Internal error: Unescaped '.' followed by nothing", backtrace))?;
                if input.starts_with(&pmatch) {
                    Ok(&input[pmatch.len()..])
                } else {
                    let got = input.chars().take(pmatch.len()).collect::<String>();
                    Err(MatchError::new(format!("Expected '{}', got '{}'", pmatch, got), backtrace))
                }
            },
            Some(invocation) => {
                // test if this is really an invocation / that the leading : was not escaped
                let escaped = process_escapes(&rule_head[..invocation.start]).is_none();
                if escaped {
                    // match until escaped colon (at invocation.start inclusively)
                    let pmatch = process_escapes(&rule_head[..invocation.start+1]).ok_or(MatchError::new("", backtrace))?;
                    if input.starts_with(&pmatch) {
                        Self::match_rule_head(&input[pmatch.len()..], &rule_head[invocation.start+1..], results, rules, backtrace)
                    } else {
                        Err(MatchError::new(format!("Expected '{}', got '{}'", pmatch, &input[..pmatch.len()]), backtrace))
                    }
                } else {
                    // match text before invocation...
                    let input = RuleVariant::match_rule_head(input, &rule_head[..invocation.start], results, rules, backtrace)?;
                    assert_ne!(invocation.rule, "");
                    // .. then invoke rule...
                    let (input, result) = apply_named_rule(input, &invocation.rule, rules, backtrace)?;
                    if !invocation.binding_var.is_empty() {
                        // TODO: could panic here
                        assert!(!results.contains_key(&invocation.binding_var));
                        results.insert(invocation.binding_var, result);
                    }

                    // ... then match the rest recursively
                    let input = RuleVariant::match_rule_head(input, &rule_head[invocation.end..], results, rules, backtrace)?;
                    Ok(input)
                }
            }
        }
    }

    fn test_replace_vars() {
        let mut vars = HashMap::new();
        vars.insert("v1".to_string(), ".!".to_string());
        vars.insert("v2".to_string(), "not used".to_string());
        assert_eq!(
            RuleVariant::replace_vars(":v1.:..:v1", &vars, &mut vec![]), Ok(".!:..!".to_string()),
        );
        assert!(
            RuleVariant::replace_vars(":nonexistingvariable", &vars, &mut vec![]).is_err()
        )
    }

    /// replaces occurences of :var with the associated string
    /// taking into account escaped ".:", then processing escapes
    fn replace_vars(text: &str, variables: &HashMap<String, String>, backtrace: &mut Vec<String>) -> Result<String, MatchError> {
        lazy_static! {
            static ref use_var_re: Regex = Regex::new(r"(?x)
                (  ) #1 in front, there has to be either a non-escape or beginning of text
                :( [a-zA-Z0-9_]+ ) #2 variable name
            ").unwrap();
        }

        match use_var_re.captures_iter(text).next() {
            None => Ok(process_escapes(text).ok_or(MatchError::new("Internal error @replace_vars()", backtrace))?),
            Some(capture) => {
                let nameMatch = capture.get(2).unwrap();
                let varname = nameMatch.as_str();
                let (varstart, varend) = (nameMatch.start() - 1, nameMatch.end());

                // the text before variable
                let front = process_escapes(&text[..varstart]);
                match front {
                    Some(front) => {
                        let var_replace = variables.get(varname)
                            .ok_or(MatchError::new(format!("Undefined variable '{}'", varname), backtrace))?;
                        Ok(front + var_replace + &Self::replace_vars(&text[varend..], variables, backtrace)?)
                    }
                    None => {
                        // front is escaping the colon, so this is not actually a variable but taken to be literal text!
                        // so instead process escapes from beginning to "variable" end
                        let escaped_text = (&text[..varend]).tap(process_escapes)
                            .ok_or(MatchError::new("Internal error @replace_vars()", backtrace))?;
                        Ok(escaped_text + &Self::replace_vars(&text[varend..], variables, backtrace)?)
                    }
                }
                
            }
        }
    }

    /// try matching one rule variant and resolve the result text
    /// return the remaining unconsumed input and the replacement string
    fn try_match<'a>(&self, input: &'a str, rules: &Rules, backtrace: &mut Vec<String>, name: impl AsRef<str>) -> Result<(&'a str, String), MatchError> {
        let btmatch = self.match_.as_ref().map(|s| format!(" {{{}}}", s)).unwrap_or("".to_string());    
        backtrace.push(format!("%: {} {} on '{}'", name.as_ref(), btmatch, firstline(input)));

        // TODO: view compiled assembly
        (|| {
            let match_ = self.match_.as_ref()
                    .ok_or(MatchError::new(format!("Variant has no header, can't match.", ), backtrace))?;

            let mut results = HashMap::new();
            let input_after_match = RuleVariant::match_rule_head(input, match_, &mut results, rules, backtrace)?;
                
            match &self.replace {
                None => {
                    // result is matched text
                    let amount_consumed = input.len() - input_after_match.len();
                    (input_after_match, input[..amount_consumed].to_string())
                }
                Some(replace) => {
                    (input_after_match, RuleVariant::replace_vars(&replace, &results, backtrace)?)
                }
            }.tap(Ok)
        })().tap(|result| {
            backtrace.pop();
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

    assert_eq!(apply_named_rule("01110d01", "digits", &rules, &mut vec![]), Ok(("d01", "Two times: 00".to_string())));
    assert!(apply_named_rule("abcde", "digits", &rules, &mut vec![]).is_err());
}

fn apply_named_rule<'a>(input: &'a str, name: &str, rules: &Rules, backtrace: &mut Vec<String>) -> Result<(&'a str, String), MatchError> {
    let variants = &rules.get(name).ok_or(MatchError::new(format!("Rule '{}' does not exist.", name), backtrace))?.variants;
    let mut candidate_errors = Vec::new();
    for v in variants.iter().rev() {
        match v.try_match(input, rules, backtrace, name) {
            Ok((input, result)) => return Ok((input, result)),
            Err(err) => candidate_errors.push(err),
        }
    }
    return Err(MatchError::compose(format!("No variant of '{}' matched.", name), candidate_errors, backtrace));
}

fn apply_unnamed_rules(input: &str, rules: &Rules) -> Result<String, MatchError> {
    let mut input = input.to_string();
    let mut backtrace = Vec::new();
    for variant in rules.get("").map(|r| &r.variants).unwrap_or(&Vec::new()).iter().rev() {
        //backtrace.push(format!("%: {{{}}}", variant.match_.as_ref().unwrap_or(&"".to_string())));
        //let _f =  finally(|| {backtrace.pop();});

        let (unconsumed, replace) = variant.try_match(&input, rules, &mut backtrace, "")?;
        input = replace + unconsumed;
    }
    Ok(input)
}

fn process(input: &str, rules: &mut Rules) -> Result<String, MatchError> {
    if let Some(((start, end), name, variant)) = find_first_definition(input) {
        let name = || name.clone();
        rules.entry(name()).or_insert(Rule::new_empty(name())).variants.push(variant);
        let name = name();
        input[..start].to_string() + &{
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
    let mut handle = stdin.lock();

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
        let lastLine = result.lines().last().unwrap();
        assert_eq!(lastLine, "success: testescape.)");

        Ok(())
    })().unwrap();
}
