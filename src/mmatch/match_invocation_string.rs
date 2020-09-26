use crate::*;

#[derive(PartialEq, Eq)]
pub struct InvocStrResult {
    pub result_str: VarInvocationString,
    /// named variable bounds of invocation results like :a:rule
    pub named_bounds: HashMap<String, String>,
    /// unnamed, but indexed variable bounds like ::rule
    pub indexed_bounds: Vec<String>,
}

// returned in the tuple is also the "result string" of evalualiting the invocation string.
fn match_invocation_string_<'a>(
    maybe_input: Option<&'a Input>, invoc_str: &InvocationString, rules: &Rules,
    initial_vars: &HashMap<String, String>,
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
    
                    (maybe_input.map(|_| input), Invocation::new_var_invocation(var))
                },
                &Invocation::VarInvocation(ref var) => (maybe_input, Invocation::new_var_invocation(var)),
            };

            var_invoc_str.add_invoc(invoc);
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
 * invocated rules will bind results to variable, which can be used by following invocations.
 * variable invocs are ignored; those can be substituted with the result vars.
 * 
 * this has the effect that results of rule invocations can be used in var invocations that precede them.
 */
pub fn match_invocation_string<'a, 'b>(
    input: &'a Input, 
    invoc_str: &'b InvocationString, 
    rules: &Rules,
    initial_vars: &HashMap<String, String>,
) -> NegatableMatchResult<'a, 'b> {
    NegatableMatchResult(input, invoc_str, {
        match match_invocation_string_(Some(input), invoc_str, rules, &initial_vars) {
            Ok((Some(input), isr)) => Ok((input, isr)),
            Ok((None, _)) => unreachable!(),
            Err(m) => Err(m)
        }
    })
}

/// like match_invocation_string, but without matching against some input, just resolving rule invocations,
/// which will throw an error if they expect any input, so they shouldn't.
pub fn match_invocation_string_pass<'b>(
    invoc_str: &'b InvocationString,
    rules: &Rules,
    initial_vars: &HashMap<String, String>
) -> MatchResult<InvocStrResult> {
    let (option, isr) = match_invocation_string_(None, invoc_str, rules, initial_vars)?;
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

/// how to handle whitespace in invocation strings
pub enum WhiteSpaceHandling {
    Remove,
    // trims line beginnings (to remove indentation) and the beginning and ending of the whole invocation string
    TrimLineBegin,
    /// trims like TrimLineBegin, and all other whitespaces (namely, whitespaces between non-whitespace characters)
    /// are substituted with the given invocation
    Substitute(Invocation),
    LeaveUnchanged,
}

impl WhiteSpaceHandling {
    pub fn trim_begin_end(&self) -> bool {
        match self {
            WhiteSpaceHandling::TrimLineBegin => true,
            WhiteSpaceHandling::Substitute(_) => true,
            // for WhiteSpaceHandling::Remove, the end and the beginning will already be "trimmed"
            _ => false
        }
    }
}

/// matches a rule header definition (including {}) or a rule body definition,
/// if input does not start with '{', no error is returned but just None.
pub fn match_invocation_string_def<'a>(input: &'a Input, wrap_begin: char, wrap_end: char, whitespace_handling: &WhiteSpaceHandling)
        -> MatchResult<(&'a Input, Option<InvocationString>)> {
    let mut invocation_string = InvocationString::new();

    let mut input = match match_char(input, wrap_begin) {
        Ok(input) => input,
        Err(_) => return Ok((input, None)),
    };

    let mut level = 1;
    let beginning = input;

    // whether the incoming text is just whitespace followed by wrap_end [or newline]
    // special case: if input[0] == wrap_end, then false (see [1])
    let is_whitespace_end = |input: &'a Input, no_new_line: bool| {
        let is_whitespace_until = |index| input[..index].chars().all(char::is_whitespace);
        let is_1whitespace_until = |index| (input[..index].len() > 0 /*[1]*/) && is_whitespace_until(index);

        match input.find('\n') {
            None => match input.find(wrap_end) {
                Some(end_index) => is_whitespace_until(end_index),
                None => false,
            },
            Some(newline_index) => match input.find(wrap_end) {
                Some(wrap_end_index) if no_new_line => is_1whitespace_until(wrap_end_index),
                Some(wrap_end_index) => is_whitespace_until(newline_index) || is_1whitespace_until(wrap_end_index),
                None => false 
            }
        }
    };

              //}        // }           // * }

    //N        x         true           

    // N
    
    // * N
    

    //breakpoint();

    loop { input =
        if let Ok((input, invo)) = match_invocation(input) {
            invocation_string.add_invoc(invo);
            input
        }
        // end of definition 
        else if whitespace_handling.trim_begin_end() && level == 1 && is_whitespace_end(input, true) {
            input = match_whitespaces(input)?;
            level -= 1;
            break;
        }
        // beginning of definition or between lines
        else if whitespace_handling.trim_begin_end() && is_whitespace_end(input, false) {
            // this is not only skipping until line end, but also all the leading whitespace of the next line
            // which suits are quite very well :))
            if let WhiteSpaceHandling::Substitute(invoc) = whitespace_handling {
                let is_end = is_whitespace_end(input, true) && level == 1;
                if input != beginning && !is_end {
                    invocation_string.add_invoc(invoc.clone());                    
                }
            }
            match_whitespaces(input)?
        } else {
            // higher escape brace indices take precedence
            let input_start = input;
            let (input, s, is_escaped) = match_escapable_char(input, ESCAPE_BRACE_OPEN[1], ESCAPE_BRACE_CLOSE[1])?;
            let (input, s, is_escaped) = if is_escaped {
                (input, s, true)
            } else {
                match_escapable_char(input_start, ESCAPE_BRACE_OPEN[0], ESCAPE_BRACE_CLOSE[0])?
            };
            let mut input = input;

            if !is_escaped {
                if s == format!("{}", wrap_begin) {
                    level += 1;
                } else if s == format!("{}", wrap_end) {
                    level -= 1;
                    if level == 0 {
                        break;
                    }
                }
            }

            if s.chars().all(char::is_whitespace) && !is_escaped {
                match whitespace_handling {
                    WhiteSpaceHandling::LeaveUnchanged => invocation_string.add_str(s),
                    WhiteSpaceHandling::Remove => (),
                    WhiteSpaceHandling::Substitute(invoc) => {
                        invocation_string.add_invoc(invoc.clone());
                        // also skip the next whitespace characters
                        input = match_whitespaces(input)?;
                    },
                    WhiteSpaceHandling::TrimLineBegin => invocation_string.add_str(s) // trimming already occurs above
                }
            } else {
                invocation_string.add_str(s);
            }
            
            input
        }
    };
    let input = match_char(input, wrap_end).expect("Internal error: Next char after loop in match_invocation_string_def() has to be wrap_end!");
    Ok((input, Some(invocation_string.seal())))
}

#[test]
fn _test_match_rule_head() {
    let mut rules = HashMap::new();
    rules.insert("number".to_string(), Rule {
        name: "number".to_string(),
        variants: vec![
            RuleVariant::new(
                InvocationString::literally("0"),
                None
            ),
            RuleVariant::new(
                InvocationString::literally("1"),
                None
            )
        ]
    });
    let mut named_bounds = HashMap::new();
    named_bounds.insert("n1".to_string(), "1".to_string());
    named_bounds.insert("n2".to_string(), "0".to_string());
    let indexed_bounds = vec!["1".to_string()];

   /* let (_, rule_head) = match_invocation_string_def("{I. have.::n1:number:n2:number.:apples......::number}", '{', '}').unwrap();
    let rule_head = rule_head.unwrap();

    assert_eq!(
        match_invocation_string_(
            "I have:10:apples...1 and 2 bananas", &rule_head, false, &rules
        ),
        Ok((" and 2 bananas", (named_bounds, indexed_bounds)))
    );*/
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
