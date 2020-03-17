use std::collections::{BTreeMap, HashMap};
use crate::*;

// for clarification: matches/applies a rule, not its definition (it has already been defined and read by swirl) 

impl Rule {
    /// start trying to apply rule variants from the bottom up, skipping a number of variants
    pub fn match_last_skip<'a>(&self, input: &'a str, param: &str, rules: &Rules, skip: usize, candidate_errors: Vec<MatchError>) -> MatchResult<(&'a str, String)> {
        //let variants = &rules.get(name).ok_or_else(|| MatchError::new(format!("Rule '{}' does not exist.", name), &mut vec![]))?.variants;
        let mut candidate_errors = candidate_errors;
        for (i, v) in self.variants.iter().rev().enumerate().skip(skip) {
            let verbose = self.name != "09" && self.name != "az" && self.name != "aZ" && self.name != "AZ" && self.name != "alnum"
                && self.name != "specialchar" && false;
            if verbose {
                println!("{}-Try %: {} {{{}}} on '{}'", get_indent(), self.name, v.header, firstline(input));
                push_indent();
            }

            match v.try_match(input, param, rules, &self.name, i) {
                Ok((input, result)) => {
                    if verbose {
                        pop_indent();
                        println!("{}-> success!", get_indent());
                    }
                    return Ok((input, result))
                },
                Err(err) => {
                    if verbose {
                        pop_indent();
                        println!("{}-> fail.", get_indent());
                    }
                    candidate_errors.push(err);
                },
            }
        }
        return MatchError::compose(format!("No variant of '{}' matched.", self.name), candidate_errors).tap(Err);
    }

    pub fn match_last<'a>(&self, input: &'a str, param: &str, rules: &Rules) -> Result<(&'a str, String), MatchError> {
        self.match_last_skip(input, param, rules, 0, vec![])
    }

    pub fn match_sequence(&self, input: &str, rules: &Rules, appleft: &mut MaybeInf<u32>) -> Result<String, MatchError> {
        let mut input = input.to_string();
        for (i, variant) in self.variants.iter().rev().enumerate() {
            //backtrace.push(format!("%: {{{}}}", variant.header.as_ref().unwrap_or(&"".to_string())));
            //let _f =  finally(|| {backtrace.pop();});

            if *appleft == MaybeInf::Finite(0u32) {
                break;
            }

            *appleft-= 1;

            let (unconsumed, replace) = variant.try_match(&input, "", rules, "", i)?;
            input = replace + unconsumed;
        }
        Ok(input)
    }
}

#[test]
fn test_match_last() {
    let mut rules = HashMap::new();
    let ruleDigit = Rule {
        name: "digit".to_string(),
        variants: vec![
            RuleVariant::new(
                InvocationString::literally("0"), None,
            ),
            RuleVariant::new(
                InvocationString::literally("1"), None,
            )
    ]};
    let ruleDigits = Rule {
        name: "digits".to_string(),
        variants: vec![
            RuleVariant::new(
                parse_header("::digit").unwrap(), None
            ),
            RuleVariant::new(
                parse_header(":d:digit::digits").unwrap(),
                Some(parse_body("Two .times: :d:d").unwrap()),
            ),
    ]};
    rules.insert("digit".to_string(), ruleDigit.clone());
    rules.insert("digits".to_string(), ruleDigits.clone());

    assert_eq!(ruleDigits.match_last("01110d01", "", &rules), Ok(("d01", "Two times: 00".to_string())));
    assert!(ruleDigits.match_last("abcde", "", &rules).is_err());
}
