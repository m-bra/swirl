use std::collections::{BTreeMap, HashMap};
use crate::*;
use std::str::FromStr;

static mut first: bool = true;

impl RuleVariant {

    fn catch_unknown_rule<'a>(
        result: MatchResult<(&'a Input, InvocStrResult)>, 
        catch_body: Option<&InvocationString>,
        rules: &Rules,
        input: &'a Input // the input to return when catch body is used
    ) 
         -> MatchResult<(&'a Input, InvocStrResult)> {
        match result {
            Ok(x) => Ok(x),
            Err(err) if err.is_unknown_rule() && catch_body.is_some() => {
                let catch_body = catch_body.unwrap();
                let catch_body_result = match_invocation_string_pass(catch_body, rules, &HashMap::new())?;
                Ok((input, catch_body_result))
            }
            Err(err) => Err(err), 
        }
    }

    fn match_param(param_header: Option<&InvocationString>, param: &Input, rules: &Rules) -> MatchResult<HashMap<String, String>> {
        let (param_rest, param_result) = match param_header {
            Some(param_header) => {
                let (param_rest, res) = match_invocation_string(param, param_header, rules, &HashMap::new()).negated(false)?;
                (param_rest, res.named_bounds)
            }
            None => (param, HashMap::new())
        };

        if !param_rest.is_empty() {
            MatchError::new(
                format!("Rule parameter must match its whole input, but '{}' is still left.", param_rest)
            ).tap(Err)
        } else {
            Ok(param_result)
        }
    }

    /// takes the result of matching the rule header, and transfers the variable binds to the body,
    /// or, if the body does not exist, into the rule header itself
    fn make_result(&self, header_result: InvocStrResult, rules: &Rules) -> MatchResult<String> {
        match &self.body() {
            None => header_result.bind_vars(),
            Some(body) => match_invocation_string_pass(&body, rules, &header_result.named_bounds)?.bind_vars()
        }
    }

    /// try matching one rule variant and resolve the result text
    /// return the remaining unconsumed input and the replacement string
    /// variant_index is 0 if it is the last variant (which is the first to be applied)
    pub fn try_match<'a>(&self, input: &'a str, param: &str, rules: &Rules, name: impl AsRef<str>, variant_index: usize) -> MatchResult<(&'a str, String)> { 
        (|| {
            unsafe { 
                if first {
                    //breakpoint();
                }
                first = false;
            }

            let name = name.as_ref();

            let optimize_tail_recursion = {
                self.header().end_invocation().map(|end_invoc| match end_invoc {
                        Invocation::RuleInvocation(_, n, _) if n == name => true,
                        _ => false
                    }).unwrap_or(false)
                    && variant_index == 0
            };

            // doing this for both branches of optimize_tail_recursion,
            // even though it is not really accurate for `if optimize_tail_recursion` 
            // (because it would have to call match_param multiple times),
            // but it will check if the param input is non-empty (see [2])
            let param_result = Self::match_param(self.parameter_header(), param, rules)?; // [3]

            if !optimize_tail_recursion {
                if self.is_any() {
                    assert!(self.header().is_empty(), "(any) rule variants must have an empty header.");
                    if input.len() > 0 {
                        (&input[1..], String::from_str(&input[0..1]).unwrap())
                    } else {
                        return MatchError::expected("Any char", input).tap(Err)
                    }
                } else {
                    let header_result = match_invocation_string(input, &self.header(), rules, &param_result)
                        .negated(self.header_negated());
                    let (input, header_result) = Self::catch_unknown_rule(header_result, self.unknown_rule_catch_body(), rules, input)?;
                    (input, self.make_result(header_result, rules)?)
                }
            } else {
                let (recursion_var, recursive_param) = match self.header().end_invocation().unwrap() {
                    Invocation::RuleInvocation(result_var, _, param) => (result_var, param),
                    Invocation::VarInvocation(_) => unreachable!(),
                };

                if self.header_negated() // [1]
                || self.unknown_rule_catch_body().is_some()
                || self.parameter_header().is_some() // [2]
                || !recursive_param.is_empty() // [4] a not-supported example would be %: rec {::other::rec(recursive param)}
                || self.flags().len() > 0 // no flags allowed
                {
                    unimplemented!()
                }

                self.header().without_tail_recursion(name, |header| {
                    // apply this (which is the last) variant (without tail recursion) 0 or more times
                    let mut input_before = input; // read "input before rule application in frame"
                    let mut frame_stack = Vec::new();
                    let mut frame_result = Ok(("", InvocStrResult::empty())); // result of current frame
                    while {
                        frame_result = match_invocation_string(input_before, header, rules, &HashMap::new())
                            .negated(false /*see [1]*/);
                        frame_result.is_ok()
                    } {
                        let (input_after, results) = frame_result.unwrap();
                        frame_stack.push( (input_before, results));
                        input_before = input_after;
                    }

                    let mut input = input_before;

                    // in the frame we're currently in, this variant (`header`) failed.
                    // so try the other variants. if none of them match,
                    // unwind the frame one time and try again.
                    let mut frame_result: MatchResult<(&str, String)> = Err(frame_result.err().unwrap());
                    while {
                        // see [3] (no param given to initial call to this variant)
                        // and [4] (no param given to recursive call by this variant)
                        assert!(param.is_empty()); 
                        frame_result = header.match_last_skip(input, "", rules, 1, vec![frame_result.err().unwrap()]);
                        frame_result.is_err()
                    } {
                        input = frame_stack.pop().ok_or_else(|| frame_result.clone().err().unwrap())?.0;
                    }

                    let (input, result_str) = frame_result.ok().unwrap();


                    // bind rule results in reverse
                    let mut result_str: String = result_str;
                    if let Some(body) = &self.body() {
                        unimplemented!("
                            this program crashed because the coder was too lazy to implement a specific case. sorry.
                            okay but alright to be fair, recursive rules are unefficient when they have bodies.
                        "); 

                        /*
                            // the following code needs to be fixed by ...
                            for (_, mut invoc_str_result) in frame_stack.drain(..).rev() {
                                if recursion_var.is_empty() {
                                    invoc_str_result.indexed_bounds.push(result_str);
                                    // inserting the string to invoc_str_result.result_str here, 
                                } else {
                                    assert!(invoc_str_result.named_bounds.get(recursion_var).is_none());
                                    invoc_str_result.named_bounds.insert(recursion_var.to_string(), result_str);
                                    // and here.
                                }
                                result_str = self.make_result(invoc_str_result, rules)?;
                            }
                        */
                    } else { // this is just an optimization for the if-branch above, which can do the same as this code
                        for (_, mut invoc_str_result) in frame_stack.drain(..).rev() {
                            let partial_result = invoc_str_result.bind_vars()?;
                            let mut concatenation = String::with_capacity(result_str.len() + partial_result.len());
                            concatenation.push_str(&partial_result);
                            concatenation.push_str(&result_str);
                            result_str = concatenation;
                        }
                    }
                
                    (input, result_str).tap(Ok)
                })?
            }.tap(Ok)

        })().trace({
            format!("%: {} {{{}}} on '{}'", name.as_ref(), self.header(), firstline(input))
        })
    }
}
