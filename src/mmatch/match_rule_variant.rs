use std::collections::{BTreeMap, HashMap};
use crate::*;

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
            let (input, (results, anon_results)) = match_rule_head(input, header, self.header_negated, rules)?;
            if self.header_negated {
                (input, String::new())
            } else {
                (input, body.bind_vars(&results, &anon_results)?)
            }
        } else {

            if self.header_negated {
                unimplemented!()
            }

            let recursion_var = self.header.end_invocation().unwrap().result_var();
            self.header.without_tail_recursion(name, |header| {
                // apply this (which is the last) variant (without tail recursion) 0 or more times
                let mut current_input = input;
                let mut frame_stack = Vec::new(); // each frame contains the input before matching its rule
                let mut frame_result = Ok(("", (HashMap::new(), Vec::new()))); // result of current frame
                while {
                    frame_result = match_rule_head(current_input, header, self.header_negated, rules);
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
                    frame_result = rules[name].match_last_skip(current_input, rules, 1, vec![frame_result.err().unwrap()]);
                    frame_result.is_err()
                } {
                    current_input = frame_stack.pop().ok_or_else(|| frame_result.clone().err().unwrap())?.0;
                }

                let (input, result_str) = frame_result.ok().unwrap();


                // bind rule results in reverse
                let mut result_str: String = result_str;
                for (_, (mut results, mut anon_results)) in frame_stack.drain(..).rev() {
                    if recursion_var.is_empty() {
                        anon_results.push(result_str);
                    } else {
                        assert!(results.get(recursion_var).is_none());
                        results.insert(recursion_var.to_string(), result_str);
                    }
                    result_str = body.bind_vars(&results, &anon_results)?;
                }
                (input, result_str).tap(Ok)
            })?
        }.tap(Ok)


        })().trace({
            format!("%: {} {{{}}} on '{}'", name.as_ref(), self.header, firstline(input))
        })
    }
}
