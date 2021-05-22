use std::collections::{BTreeMap, HashMap};
use crate::*;
use std::str::FromStr;

static mut first: bool = true;

impl RuleVariant {
    pub fn backtraceline(&self, name: &str, input: &str) -> String {
        let input_view = input_view(input);
        let mut param_header_str =  self.parameter_header().map(|ph| format!("({})", ph)).unwrap_or("".to_string());
        if param_header_str != "" {
            param_header_str += " ";
        }
        if name == "" {
            format!("%: {}{{{}}} on '{}'", param_header_str, self.header(), input_view)
        } else {
            format!("%: {}{} {{{}}} on '{}'", name, param_header_str, self.header(), input_view)
        }
    }

    // Returns None if no "unknown rule" error was thrown
    // Returns Some if such an error has been caught, with the result of the catch body.
    fn catch_unknown_rule<'a>(
        result: &MatchResult<(&'a Input, InvocStrResult)>, 
        catch_body: Option<&InvocationString>,
        rules: &Rules
    ) -> MatchResult<Option<String>> {
        match result {
            Ok(_) => None,
            Err(err) if err.is_unknown_rule() && catch_body.is_some() => {
                let catch_body = catch_body.unwrap();
                let catch_body_result = match_invocation_string_pass(catch_body, rules, &HashMap::new())?;
                Some(catch_body_result.bind_vars()?)
            }
            Err(_) => None, 
        }.tap(Ok)
    }

    fn match_param(param_header: Option<&InvocationString>, param: &Input, rules: &Rules) -> MatchResult<HashMap<String, String>> {
        let (param_rest, param_result) = match param_header {
            Some(param_header) => {
                let (param_rest, res) = match_invocation_string(param, param_header, rules, &HashMap::new(), true).negated(false)?;
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
        trace(self.backtraceline(name.as_ref(), input), || {
            unsafe { 
                if first {
                    //breakpoint();
                }
                first = false;
            }

            let name = name.as_ref();

            //TODO: in the optimized tail recursion loop, test if we are in a user-caused infinite loop!
            let optimize_tail_recursion = unsafe {
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
                        let x: (&str, String) = (skip_str(input, 1), substr(input, 0, 1).to_string());
                        x
                    } else {
                        return MatchError::expected("Any char", input).tap(Err)
                    }
                } else {
                    let header_result = match_invocation_string(input, &self.header(), rules, &param_result, true)
                        .negated(self.header_negated());
                    if let Some(result) = Self::catch_unknown_rule(&header_result, self.unknown_rule_catch_body(), rules)? {
                        (input, result)
                    } else {
                        let (input, header_result) = header_result?;
                        (input, self.make_result(header_result, rules)?)
                    }
                }
            } else {
                let (_, recursive_param) = match unsafe { self.header().end_invocation().unwrap() } {
                    Invocation::RuleInvocation(result_var, _, param) => (result_var, param),
                    Invocation::VarInvocation(_) => unreachable!(),
                };

                if self.header_negated() // [1]
                || self.unknown_rule_catch_body().is_some()
                || self.parameter_header().is_some() // [2] // @ENSURE("tail optimization allows no")
                || !recursive_param.is_empty() // [4] a not-supported example would be %: rec {::other::rec(recursive param)}
                || self.flags().len() > 0 // no flags allowed
                {
                    unimplemented!(
                        "Rule variant {} of {} is to be tail-optimized, but it violates one of the following conditions:\n\
                         1. header must not be negated\n\
                         2. there must be no catch body\n\
                         3. there must be no usage of parameters\n\
                         4. no passing of parameters to itself 
                         5. no flags allowed", variant_index, name);
                }

                self.header().without_tail_recursion(name, |header| {
                    // apply this (which is the last) variant (without tail recursion) 0 or more times
                    let mut input_before = input; // read "input before rule application in frame"
                    let mut frame_stack = Vec::new();
                    let mut frame_result = Ok(("", InvocStrResult::empty())); // result of current frame
                    let mut i = 0;
                    while {
                        frame_result = match_invocation_string(input_before, header, rules, &HashMap::new(), true)
                            .negated(false /*see [1]*/);
                        frame_result.is_ok()
                    } {
                        let (input_after, results) = frame_result.unwrap();
                        frame_stack.push( (input_before, results));
                        input_before = input_after;

                        i += 1;
                        if i > 1000000 {
                            panic!("Infinite tail recursion...");
                        }
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
                        frame_result = rules[name].match_last_skip(input, "", rules, 1, vec![frame_result.err().unwrap()]);
                        frame_result.is_err()
                    } {
                        input = frame_stack.pop().ok_or_else(|| frame_result.clone().err().unwrap())?.0;
                    }

                    let (input, result_str) = frame_result.ok().unwrap();


                    // bind rule results in reverse
                    let mut result_str: String = result_str;
                    if let Some(_) = &self.body() {
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
                        for (_, invoc_str_result) in frame_stack.drain(..).rev() {
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
        })
    }
}
