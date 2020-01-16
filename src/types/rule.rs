use std::collections::{BTreeMap, HashMap};
use crate::*;

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
        //let variants = &rules.get(name).ok_or_else(|| MatchError::new(format!("Rule '{}' does not exist.", name), &mut vec![]))?.variants;
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
