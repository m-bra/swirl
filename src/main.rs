#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(core_intrinsics)]
#![feature(try_blocks)]

extern crate meval;


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

pub const ESCAPE_BRACE_OPEN: [&str; 2] = ["'", "{'"];
pub const ESCAPE_BRACE_CLOSE: [&str; 2] = ["`", "`}"];
pub const RULE_INVOCATION_CHAR: char = ':';
pub const RULE_DEFINITION_KEY: &str = "%:";

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

// first string in returned pair is the skipped text
pub fn find_statement(input: &Input) -> Option<(&str, &Input)> {
    input.find(RULE_DEFINITION_KEY).map(|i| (&input[..i], &input[i..]))
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

fn init_rules() -> Rules {
    let mut rules = HashMap::new();
    rules.insert("swirl_default_call_explicit_syntax".to_string(), {
        Rule::new("swirl_default_call_explicit_syntax".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_feature_undefine_rule".to_string(), {
        Rule::new("swirl_feature_undefine_rule".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_new_quote_signs".to_string(), {
        Rule::new("swirl_new_quote_signs".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_version_0_0_1".to_string(), {
        Rule::new("swirl_version_0_0_1".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_version_0_0_2".to_string(), {
        Rule::new("swirl_version_0_0_2".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_version_0_0_3".to_string(), {
        Rule::new("swirl_version_0_0_3".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_version_0_1_0".to_string(), {
        Rule::new("swirl_version_0_1_0".to_string())
            .variant(RuleVariant::empty())
    });
    rules.insert("swirl_version_0_2_0".to_string(), {
        Rule::new("swirl_version_0_2_0".to_string())
            .variant(RuleVariant::empty())
    });
    rules
}

pub fn process(input: &str, rules: &mut Rules, mut appleft: MaybeInf<u32>, remove_defs: bool, mut receive_output: impl FnMut(&str) -> MatchResult<()>) -> MatchResult<()> {
    let mut input = input.to_string();

    while let Some((skipped_text, statement_begin)) = find_statement(&input) {
        // todo: use swirl to sweeten this up to 
        // break if appleft == MaybeInf::Finite(0);
        if appleft == MaybeInf::Finite(0) {
            break;
        }

        let (statement_end, (name, maybe_variant)) = match_statement(statement_begin)?;
        // all text until the current rule definition remains untouched (because it is between the beginning/a rule definition and a rule definition)
        // so just push it to the result string
        receive_output(skipped_text)?;
        if !remove_defs {
            receive_output(&statement_begin[..(statement_begin.len() - statement_end.len())])?;
        }

        // add variant to definitions (or remove) (perhaps call it)
        if let Some(variant) = maybe_variant {
            let name = || name.clone();
            let rule_entry = rules.entry(name()).or_insert(Rule::new(name()));
            let name = name();

            // next portion to process is after the current rule definition
            input = statement_end.to_string();

            if variant.is_undefine() {
                rules.remove(&name);
            } else {
                rule_entry.variants.push(variant.clone());

                // empty name means invocation
                if name.is_empty() {
                    // next portion to process is the output of application of the current rule definition (piped to all previous unnamed rule definitions)
                    let new_input = rules[&name].match_sequence(&input, rules, &mut appleft)?;
                    // if this rule was just to be applied once, remove from definitions
                    if variant.shallow_call() {
                        rules.get_mut(&name).unwrap().variants.pop().unwrap();
                    }
                    input = new_input;
                }

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

    // the rest of the input contains no more rule definitions, so output it
    receive_output(&input)?;
    Ok(())
}

fn process_file(target: &str, steps: MaybeInf<u32>, remove_defs: bool) -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    File::open(&target)?.read_to_string(&mut buffer)?;
    
    let mut result = String::new();
    process(&buffer, &mut init_rules(), steps, remove_defs, |lines| result.push_str(lines).tap(Ok))?;

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

static mut VERBOSE: bool = false; 

pub fn is_verbose() -> bool {
    return unsafe {VERBOSE};
}

//#[cfg(not(debug_assertions))]
fn main() -> Result<(), ()>  {
    let mut args = std::env::args();
    let mut is_stepping = args.any(|s| s == "--step" || s == "-s");
    unsafe { VERBOSE = args.any(|s| s == "--verbose" || s == "-v") };

    let (steps, remove_defs) = if is_stepping {
        (MaybeInf::Finite(1), false)
    } else {
        (MaybeInf::Infinite, true)
    };

    if cfg!(debug_assertions) {
        println!(" -- Debug mode --");
        unsafe { ::std::intrinsics::breakpoint() }
        return process_file("input.txt", MaybeInf::Infinite, true).map_err(|e| eprintln!("{}", e));
    }

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)
        .map_err(|e| eprintln!("{}", e))?;
    process(&buffer, &mut init_rules(), steps, remove_defs, |lines| {
        println!("{}", lines);
        io::stdout().flush().unwrap();
        Ok(())
    }).map_err(|e| eprintln!("{}", e))

    /* let mut rules = init_rules();
    let mut userline = String::new();
    while io::stdin().read_line(&mut userline).is_ok() {
        process(&userline, &mut rules, steps, remove_defs, |lines| {
            println!("{}", lines);
            io::stdout().flush().unwrap();
            Ok(())
        }).map_err(|e| eprintln!("{}", e))?;
        userline.clear();
    }
    Ok(()) */
}

