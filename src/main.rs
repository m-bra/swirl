#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(core_intrinsics)]
#![feature(try_blocks)]


use regex::Regex;
use std::collections::HashMap;

mod util;
use util::*;

mod error;
use error::*;

mod mmatch;
use mmatch::*;

mod example_input;

#[macro_use]
extern crate lazy_static;

pub const ESCAPE_BRACE_OPEN: &str = "'";
pub const ESCAPE_BRACE_CLOSE: &str = "`";
pub const RULE_INVOCATION_CHAR: char = ':';

static mut INDENT: usize = 0;

fn push_indent() {unsafe {INDENT += 2;}}
fn pop_indent() {unsafe {INDENT -= 2;}}
fn get_indent() -> String {unsafe {"  ".repeat(INDENT)}}

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
fn test() {
    // in my mind it works like that
    assert_eq!(&"01234"[..2], "01");
}

// first string in returned pair is the skipped text
pub fn find_statement(input: &Input) -> Option<(&str, &Input)> {
    input.find("%:").map(|i| (&input[..i], &input[i..]))
}

use std::fs::File;

pub fn match_statement(input: &Input) -> MatchResult<(&Input, (String, Option<RuleVariant>))> {
    match match_rule_definition(input) {
        Ok((input, (name, variant))) => Ok((input, (name, Some(variant)))),
        Err(def_err) => match match_file_invocation(input) {
            Ok((input, name)) => match File::open(name) {
                Ok(_) => Ok((input, (name.to_string(), None))),
                Err(file_err) => {
                    let msg = "Ill-formed rule definition or invocation to unexisting file.";
                    let file_err = MatchError::new(format!("Error loading file '{}': {} ", name, file_err));
                    MatchError::compose(msg, vec![def_err, file_err]).tap(Err)
                }
            },
            Err(file_err) => {
                let msg = format!("Expected statement, got {}", error_region(input));
                MatchError::compose(msg, vec![def_err, file_err]).tap(Err)
            }
        }
    }
}

fn init_rules() -> &mut Rules {
    let mut rules = HashMap::new();
    rules.insert("", v: V)
}

pub fn process(input: &str, rules: &mut Rules, mut appleft: MaybeInf<u32>, remove_defs: bool) -> MatchResult<String> {
    let mut result = String::new();
    let mut input = input.to_string();

    while let Some((skipped_text, statement_begin)) = find_statement(&input) {
        if appleft == MaybeInf::Finite(0) {
            break;
        }

        let (statement_end, (name, maybe_variant)) = match_statement(statement_begin)?;
        // all text until the current rule definition remains untouched (because it is between the beginning/a rule definition and a rule definition)
        // so just push it to the result string
        result.push_str(skipped_text);
        if !remove_defs {
            result.push_str(&statement_begin[..(statement_begin.len() - statement_end.len())]);
        }

        // add variant to definitions
        if let Some(variant) = maybe_variant {
            let once = variant.flags.contains("call");
            let name = || name.clone();
            rules.entry(name()).or_insert(Rule::new(name())).variants.push(variant);
            let name = name();

            if !name.is_empty() {
                // next portion to process is after the current rule definition
                input = statement_end.to_string();
            } else {
                // next portion to process is the output of application of the current rule definition (piped to all previous unnamed rule definitions)
                let new_input = rules[&name].match_sequence(statement_end, rules, &mut appleft)?;
                // if this rule was just to be applied once, remove from definitions
                if once {
                    rules.get_mut(&name).unwrap().variants.pop().unwrap();
                }
                input = new_input;
            }
        }
        // invoke file
        else {
            // insert file contents before rest of this file
            let filecontent = dump_file(&name)
                .map_err(|err| MatchError::new(format!("Error loading '{}': {}", name, err)))?;
            input = format!("{}{}", filecontent, statement_end);
        }

    }

    // the rest of the input contains no more rule definitions, so push it to the results
    result.push_str(&input);
    Ok(result)
}

fn process_file(target: &str, steps: MaybeInf<u32>, remove_defs: bool) -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    File::open(&target)?.read_to_string(&mut buffer)?;

    let result = process(&buffer, &mut HashMap::new(), steps, remove_defs)?;

    File::create(format!("{}.out", target))?.write(result.as_bytes())?;

    Ok(())
}

use std::io::{self, Read, Write};
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
                process_file(&target, MaybeInf::Finite(step_count), false)?;
            } else if userline[0] == "r" || userline[0] == "run" {
                process_file(&target, MaybeInf::Infinite, true)?;
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

#[cfg(debug_assertions)]
fn main() -> Result<(), ()> {
    println!(" -- Debug mode --");
    unsafe { ::std::intrinsics::breakpoint() }
    return process_file("input.txt", MaybeInf::Infinite, true).map_err(|e| eprintln!("{}", e))
}

#[cfg(not(debug_assertions))]
fn main() -> Result<(), ()>  {
    let mut is_stepping = std::env::args().any(|s| s == "--step" || s == "-s");

    let (steps, remove_defs) = if is_stepping {
        (MaybeInf::Finite(1), false)
    } else {
        (MaybeInf::Infinite, true)
    };

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)
        .map_err(|e| eprintln!("{}", e))?;
    let result = process(&buffer, &mut HashMap::new(), steps, remove_defs)
        .map_err(|e| eprintln!("{}", e))?;
    println!("{}", result);
    Ok(())
}

#[test]
fn test_input() {
    (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = String::new();
        File::open("testinput.txt")?.read_to_string(&mut buffer)?;

        let result = process(&buffer, &mut HashMap::new(), MaybeInf::Infinite, true)?;
        let last_line = result.lines().last().unwrap();
        assert_eq!(last_line, "success: testescape.)");

        Ok(())
    })().unwrap();
}
