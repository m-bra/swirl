use std::collections::{BTreeMap, HashMap};
use crate::*;

mod invocation;
pub use invocation::*;

mod rulepart;
pub use rulepart::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleVariant {
    pub header: Header,
    pub body: Option<Body>,
    pub append: String,
}

impl RuleVariant {
    /// try matching one rule variant and resolve the result text
    /// return the remaining unconsumed input and the replacement string
    /// variant_index is 0 if it is the last variant (which is the first to be applied)
    pub fn try_match<'a>(&self, input: &'a str, rules: &Rules, name: impl AsRef<str>, variant_index: usize) -> MatchResult<(&'a str, String)> { (|| {
        let name = name.as_ref();

        let optimize_tail_recursion = {
            self.header.end_invocation().map(|end_invoc| end_invoc.rule() == name)
                .unwrap_or(false)
                && variant_index == 0
        };

        let body = self.body.clone().unwrap_or_else(|| self.header.as_body());

        if !optimize_tail_recursion {
            let header = &self.header;
            let (input, (results, anon_results)) = match_rule_head(input, header, rules)?;
            (input, body.bind_vars(&results, &anon_results))
        } else {
            let recursion_var = self.header.end_invocation().unwrap().result_var();
            self.header.without_tail_recursion(name, |header| {
                // apply this (which is the last) variant (without tail recursion) 0 or more times
                let mut current_input = input;
                let mut frame_stack = Vec::new(); // each frame contains the input before matching its rule
                let mut frame_result = Ok(("", (HashMap::new(), Vec::new()))); // result of current frame
                while {
                    frame_result = match_rule_head(current_input, header, rules); 
                    frame_result.is_ok()
                } {
                    let (input, results) = frame_result.unwrap();
                    frame_stack.push((current_input, results));
                    current_input = input;
                }

                // in the frame we're currently in, this variant (`header`) failed.
                // so try the other variants. if none of them match,
                // unwind the frame one time and try again.
                let mut frame_result: MatchResult<(&str, String)> = Err(frame_result.err().unwrap());
                while {
                    // todo:
                    // i feel like in this line, if rules[name] is self, we'll get into problems (see doc of Header::without_tail_recursion())
                    // which means that rules with tail recursion that also recurse besides at the tail will crash lol but lets ignore that hahaahha
                    frame_result = rules[name].apply_last_skip(current_input, rules, 1, vec![frame_result.err().unwrap()]);
                    frame_result.is_err()
                } {
                    current_input = frame_stack.pop().ok_or_else(|| frame_result.clone().err().unwrap())?.0;
                }

                let (input, result_str) = frame_result.ok().unwrap();


                // bind rule results in reverse
                let mut result_str = result_str;
                for (_, (mut results, mut anon_results)) in frame_stack.drain(..).rev() {
                    if recursion_var.is_empty() {
                        anon_results.push(result_str);
                    } else {
                        assert!(results.get(recursion_var).is_none());
                        results.insert(recursion_var.to_string(), result_str);
                    }
                    result_str = body.bind_vars(&results, &anon_results);
                }
                (input, result_str).tap(Ok)
            })?
        }.tap(Ok)
        
        
        })().trace({
            format!("%: {} {{{}}} on '{}'", name.as_ref(), self.header, firstline(input))
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub variants: Vec<RuleVariant>,
}

impl Rule {
    pub fn new(name: String) -> Rule {
        Rule {
            name: name,
            variants: Vec::new(),
        }
    }

    pub fn variant(mut self, v: RuleVariant) -> Rule {
        self.variants.push(v);
        self
    }
}

impl Rule {
    /// start trying to apply rule variants from the bottom up, skipping a number of variants
    pub fn apply_last_skip<'a>(&self, input: &'a str, rules: &Rules, skip: usize, candidate_errors: Vec<MatchError>) -> MatchResult<(&'a str, String)> {
        //let variants = &rules.get(name).ok_or(MatchError::new(format!("Rule '{}' does not exist.", name), &mut vec![]))?.variants;
        let mut candidate_errors = candidate_errors;
        for (i, v) in self.variants.iter().rev().enumerate().skip(skip) {
            match v.try_match(input, rules, &self.name, i) {
                Ok((input, result)) => return Ok((input, result)),
                Err(err) => candidate_errors.push(err),
            }
        }
        return Err(MatchError::compose(format!("No variant of '{}' matched.", self.name), candidate_errors));
    }

    pub fn apply_last<'a>(&self, input: &'a str, rules: &Rules) -> Result<(&'a str, String), MatchError> {
        self.apply_last_skip(input, rules, 0, vec![])
    }

    pub fn apply_sequence(&self, input: &str, rules: &Rules, appleft: &mut MaybeInf<u32>) -> Result<String, MatchError> {
        let mut input = input.to_string();
        for (i, variant) in self.variants.iter().rev().enumerate() {
            //backtrace.push(format!("%: {{{}}}", variant.header.as_ref().unwrap_or(&"".to_string())));
            //let _f =  finally(|| {backtrace.pop();});

            if *appleft == MaybeInf::Finite(0u32) {
                break;
            }

            *appleft-= 1;

            let (unconsumed, replace) = variant.try_match(&input, rules, "", i)?;
            input = replace + unconsumed;
        }
        Ok(input)
    }
}

pub type Rules = HashMap<String, Rule>;

mod display;