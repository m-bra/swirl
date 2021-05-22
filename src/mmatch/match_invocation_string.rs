use crate::*;
use std::cell::Cell;

#[derive(PartialEq, Eq)]
pub struct InvocStrResult {
    pub result_str: VarInvocationString,
    /// named variable bounds of invocation results like :a:rule
    pub named_bounds: HashMap<String, String>,
    /// unnamed, but indexed variable bounds like ::rule
    pub indexed_bounds: Vec<String>,
}

// returned in the tuple is also the "result string" of evaluating the invocation string.
fn match_invocation_string_<'a>(
    maybe_input: Option<&'a Input>, invoc_str: &InvocationString, rules: &Rules,
    initial_vars: &HashMap<String, String>,
    match_var_invocs: bool,
) 
    -> MatchResult<(Option<&'a Input>, InvocStrResult)> 
{
    let (mut named_bounds, mut indexed_bounds) = (initial_vars.clone(), Vec::new());

    if !indexed_bounds.is_empty() {
        unimplemented!("
            indexed_bounds must be empty, because then, and only then, after having called this function and bound 
            anonymous rule invocations ::rule to indexed anonymous variables, bind_vars() can be used to bind their
            results to exactly where they where called.
        ");
    }

    let mut maybe_input = maybe_input;

    let mut var_invoc_str = InvocationString::new();

    for (part, invocs) in unsafe { invoc_str.iter() } {
        if !part.is_empty() {
            //println!("{}match '{}'", get_indent(), part);
        }
        if let Some(input) = maybe_input { maybe_input = Some(
            match_str(input, part)?
        )}
        var_invoc_str.add_str(part);

        for invoc in invocs {
            let (new_maybe_input, invoc) = match invoc {
                &Invocation::RuleInvocation(ref var, ref rule, ref param) => {
                    //println!("{}match ::{}", get_indent(), rule);
                    let param_result = match_invocation_string_pass(param, rules, &named_bounds)?.bind_vars()?;
    
                    let rule = rules.get(rule)
                        .ok_or_else(|| MatchError::unknown_rule(rule, "<>"))?;
                    let (input, result) = rule.match_last(maybe_input.unwrap_or(""), &param_result, rules)?;
    
                    if !var.is_empty() {
                        // TODO: could panic here
                        assert!(!named_bounds.contains_key(&var.to_string()));
                        named_bounds.insert(var.to_string(), result);
                    } else {
                        indexed_bounds.push(result);
                    }
    
                    (maybe_input.map(|_| input), Some(Invocation::new_var_invocation(var)))
                },
                &Invocation::VarInvocation(ref var) => if match_var_invocs {
                    // match var invocation with input.
                    let varcontent = named_bounds.get(var)
                        .ok_or(MatchError::unknown_variable(var, ""))?;
                    let input = maybe_input.unwrap_or("");
                    if input.starts_with(varcontent) {
                        (Some(&input[varcontent.len()..]), None)
                    } else {
                        return MatchError::expected(&format!(":{} aka '{}'", var, varcontent), input).tap(Err);
                    }
                } else {
                    (maybe_input, Some(Invocation::new_var_invocation(var)))
                },
            };

            if let Some(invoc) = invoc {
                var_invoc_str.add_invoc(invoc);
            }
            maybe_input = new_maybe_input;
        }
    }

    let result_str = var_invoc_str.seal().assume_only_var_invocs();

    Ok((maybe_input, InvocStrResult {
        result_str, named_bounds, indexed_bounds
    }))
}


pub struct NegatableMatchResult<'a, 'b>(&'a Input, &'b InvocationString, MatchResult<(&'a Input, InvocStrResult)>);

impl<'a, 'b> NegatableMatchResult<'a, 'b> {
    pub fn negated(self, negated: bool) -> MatchResult<(&'a Input, InvocStrResult)> {
        match self {
            NegatableMatchResult(input, rule_head, Ok(_)) if negated => {
                MatchError::expected(&format!("not {}", rule_head), input).tap(Err)
            },
            NegatableMatchResult(input, _, Err(_)) if negated => {
                Ok((input, InvocStrResult {
                    named_bounds: HashMap::new(), 
                    indexed_bounds: Vec::new(),
                    result_str: InvocationString::empty().assume_only_var_invocs(),
                }))
            }
            NegatableMatchResult(_, _, inner_result) /* if !negated */ => {
                inner_result
            }
        }
    }
}

// match rule header with the start of "input", possibly invoking other rules
// bind results of invocations to the specified variables (:var:rule)
// bind results of anonymous invocations to vector in correct order (::rule)
// return advanced input pointer or MatchError
/*
 * Match invocation string against input. 
 * 
 * Invocated rules will bind results to variable, which can be used by following invocations.
 * 
 * Variable invocs will, depending on "match_var_invocs", either match with the input,
 * or be passed to the result, so that they will later be substituted.
 * Old behaviour was match_var_invocs = false. Except when invoc_str is a body part, you want match_var_invocs = true.
 * 
 * This has the effect that results of rule invocations can be used in var invocations that precede them.
 */
pub fn match_invocation_string<'a, 'b>(
    input: &'a Input, 
    invoc_str: &'b InvocationString, 
    rules: &Rules,
    initial_vars: &HashMap<String, String>,
    match_var_invocs: bool
) -> NegatableMatchResult<'a, 'b> {
    NegatableMatchResult(input, invoc_str, {
        match match_invocation_string_(Some(input), invoc_str, rules, &initial_vars, match_var_invocs) {
            Ok((Some(input), isr)) => Ok((input, isr)),
            Ok((None, _)) => unreachable!(),
            Err(m) => Err(m)
        }
    })
}

/// like match_invocation_string, but without matching against some input, just resolving rule invocations,
/// which will throw an error if they expect any input, so they shouldn't.
/// thus assuming match_var_invocs = false.
pub fn match_invocation_string_pass<'b>(
    invoc_str: &'b InvocationString,
    rules: &Rules,
    initial_vars: &HashMap<String, String>,
) -> MatchResult<InvocStrResult> {
    let (option, isr) = match_invocation_string_(None, invoc_str, rules, initial_vars, false)?;
    assert!(option.is_none());
    Ok(isr)
}

#[test]
fn test_match_invocation_string() {
    /*let (_, rule_head) = match_invocation_string_def("{I have.::n1:number:n2:number.:apples......::number}", '{', '}').unwrap();
    let rule_head = rule_head.unwrap();

    assert_eq!(rule_head, {
        let mut head = InvocationString::new();
        head.add_str("I have:");
        head.add_invoc(Invocation::new_rule_invocation("n1", "number"));
        head.add_invoc(Invocation::new_rule_invocation("n2", "number"));
        head.add_str(":apples...");
        head.add_invoc(Invocation::new_rule_invocation("", "number"));
        head.seal()
    })*/
}

/// matches a rule header definition (including {}) or a rule body definition,
/// if input does not start with '{', no error is returned but just None.
pub fn match_invocation_string_def<'a>(input: &'a Input, rules: &Rules, wrap_begin: char, wrap_end: char, whitespace_handling_rule: &str)
        -> MatchResult<(&'a Input, Option<InvocationString>)> {
    let mut invocation_string = InvocationString::new();

    let mut input = match match_char(input, wrap_begin) {
        Ok(input) => input,
        Err(_) => return Ok((input, None)),
    };

    let level = Cell::new(1);
    let beginning = input;

    // whether the incoming text is just whitespace followed by either wrap_end or newline
    // special case: if input[0] == wrap_end, then false (see [1])
    let is_whitespace_end = |input: &'a Input, no_new_line: bool| {
        let is_whitespace_until = |index| input[..index].chars().all(char::is_whitespace);
        let is_1whitespace_until = |index| (input[..index].len() > 0 /*[1]*/) && is_whitespace_until(index);

        match input.find('\n') {
            None => match input.find(wrap_end) {
                Some(end_index) => is_1whitespace_until(end_index),
                None => false,
            },
            Some(newline_index) => match input.find(wrap_end) {
                Some(wrap_end_index) if no_new_line => is_1whitespace_until(wrap_end_index),
                Some(wrap_end_index) => is_whitespace_until(newline_index) || is_1whitespace_until(wrap_end_index),
                None => false 
            }
        }
    };

    let is_whitespace_line_end = |input| is_whitespace_end(input, false);
    let is_whitespace_all_end = |input| is_whitespace_end(input, true) && level.get() == 1;
    
    loop { input = {
            let maybe_escaped = match_quote(input).ok();
            
            if let Some((escaped_txt, input)) = maybe_escaped {
                invocation_string.add_str(escaped_txt);
                input
            } else {
                if let Ok((input, invo)) = match_invocation(input, rules) {
                    invocation_string.add_invoc(invo);
                    input
                } else if input.starts_with(wrap_begin) {
                    level.set(level.get() + 1);
                    invocation_string.add_char(wrap_begin);
                    skip_str(input, 1)
                } else if input.starts_with(wrap_end) {
                    level.set(level.get() - 1);
                    if level.get() == 0 {
                        break;
                    }
                    invocation_string.add_char(wrap_end);
                    skip_str(input, 1)
                } else if input.starts_with(char::is_whitespace) {
                    let whitespace_handling_rule: &Rule = match rules.get(whitespace_handling_rule) {
                        Some(rule) => rule,
                        None => return MatchError::new(format!("{}{}", "white space handler not defined at ", firstline(beginning))).tap(Err),
                    };

                    let (input_after, wh_count) = count_whitespaces(input)?;
                    let param = if input == beginning {
                        "begin"
                    } else if is_whitespace_all_end(input) {
                        "end"
                    } else if is_whitespace_line_end(input) {
                        "between lines"
                    } else {
                        "within line"
                    };
                    let result = whitespace_handling_rule.match_all(&input[..wh_count], param, rules)?;
                    let result = format!("({})", &result);
                    let (result_rest, result) = match_invocation_string_def(&result, rules, '(', ')', SWIRL_WHITESPACE_HANDLER_BODY)?;
                    let result = result.unwrap();
                    assert_eq!(result_rest, "");
                    invocation_string.add_invoc_str(&result);
                    input_after
                } else if input.len() > 0 {
                    invocation_string.add_char(input.chars().next().unwrap());
                    skip_str(input, 1)
                } else {
                    return MatchError::new("Unexpected end of file").tap(Err);
                }
            }
        }
    };
    let input = match_char(input, wrap_end).expect("Internal error: Next char after loop in match_invocation_string_def() has to be wrap_end!");
    Ok((input, Some(invocation_string.seal())))
}

impl InvocStrResult {
    pub fn bind_vars(&self) -> MatchResult<String> {
        self.result_str.bind_vars(&self.named_bounds, &self.indexed_bounds)
    }

    pub fn empty() -> InvocStrResult {
        InvocStrResult {
            indexed_bounds: Vec::new(),
            named_bounds: HashMap::new(),
            result_str: InvocationString::empty().assume_only_var_invocs(),
        }
    }
}
