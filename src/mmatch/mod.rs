#![allow(dead_code)]

use crate::error::*;
use crate::*;

mod match_invocation_string;
pub use match_invocation_string::*;

mod match_rule_def;
pub use match_rule_def::*;

mod match_rule_variant;
pub use match_rule_variant::*;

mod match_rule;
pub use match_rule::*;

pub type Input = str;

// function that receives input string pointer and some in params,
// then advances input pointer and returns some out params
pub trait MatchFn<'a, In, Out>: FnMut(&'a Input, In) -> MatchResult<(&'a Input, Out)> {}
impl<'a, F, In, Out> MatchFn<'a, In, Out> for F where F: FnMut(&'a Input, In) -> MatchResult<(&'a Input, Out)> {}

pub fn match_char(input: &str, expect: char) -> MatchResult<&str> {
    // todo: look at asm
    let err = || MatchError::expected(&expect.to_string(), input).tap(Err);
    if let Some(c) = input.chars().next() {
        if c == expect {
            Ok(skip_str(input, 1))
        } else {
            err()
        }
    } else {
        err()
    }
}

pub fn match_str(input: &str, expect: impl AsRef<str>) -> MatchResult<&str> {
    let expect = expect.as_ref();
    if input.starts_with(expect) {
        Ok(&input[expect.len()..])
    } else {
        MatchError::expected(&expect.to_string(), input).tap(Err)
    }
}

pub fn match_maybe_str(input: &str, expect: impl AsRef<str>) -> (&str, bool) {
    let expect = expect.as_ref();
    if input.starts_with(expect) {
        (&input[expect.len()..], true)
    } else {
        (input, false)
    }
}


pub fn match_ident<'a>(input: &'a Input) -> MatchResult<(&'a Input, &str)> {
    // todo: look at asm
    let len = input.chars().take_while(|c| c.is_alphabetic() || c.is_digit(10) || *c == '_').count();
    if len == 0 {
        MatchError::expected("identifier", input).tap(Err)
    } else {
        Ok((&input[len..], &input[..len]))
    }
}

pub fn match_var<'a>(input: &'a Input) -> MatchResult<(&'a Input, Invocation)> {
    match_char(input, ':').and_then(match_ident).map(|(input, ident)| (input,
        Invocation::new_var_invocation(ident)
    ))
}

#[test]
pub fn test_match_invocation() {
    assert_eq!(
        match_rule_invoc("::rule text"),
        Ok((" text", Invocation::new_rule_invocation("", "rule")))
    );
    assert_eq!(
        match_rule_invoc(":name:rule text"),
        Ok((" text", Invocation::new_rule_invocation("name", "rule")))
    );
    assert!(
        match_rule_invoc(":name:: text").is_err()
    );
    assert!(
        match_rule_invoc("text :name:rule").is_err()
    );
}

pub fn match_rule_invoc<'a>(input: &'a Input, rules: &Rules) -> MatchResult<(&'a Input, Invocation)> {
    let input = match_char(input, RULE_INVOCATION_CHAR)?;
    let (input, variable_ident) = match_ident(input).unwrap_or((input, ""));
    let input = match_char(input, RULE_INVOCATION_CHAR)?;
    let (input, rule_ident) = match_ident(input)?;
    let (input, invoc) = match_invocation_string_def(input, rules, '(', ')', SWIRL_WHITESPACE_HANDLER_PARAM_INPUT)?;
    let invoc = invoc.unwrap_or(InvocationString::empty());

    (input, Invocation::new_rule_invoc_with_param(variable_ident, rule_ident, invoc)).tap(Ok)
}

pub fn match_invocation<'a>(input: &'a Input, rules: &Rules) -> MatchResult<(&'a Input, Invocation)> {
    if let Ok((input, invoc)) = match_rule_invoc(input, rules) {
        (input, invoc).tap(Ok)
    } else {
        match_var(input)
    }
}

/*
#[test]
pub fn test_match_escapable_char() {
    assert_eq!(
        match_escapable_char("{.}text", '{', '}'),
        Ok(("text", '.'))
    );
    assert_eq!(
        match_escapable_char("abtext", 'a', 't'),
        Ok(("ext", 'b'))
    );
    assert_eq!(
        match_escapable_char("P,text", ','),
        Ok((",text", 'P'))
    );
    assert!(
        match_escapable_char("", '.').is_err()
    );
    assert!(
        match_escapable_char(".", '.').is_err()
    );
    assert_eq!(
        match_escapable_char("a", '.'),
        Ok(("", 'a'))
    );
}*/

pub fn match_escapable_char_old(input: &Input, escape: char) -> MatchResult<(&Input, char)> {
    let mut input = input.chars();
    let c1 = input.next()
        .ok_or_else(|| MatchError::expected("some char", input.as_str()))?;

    if c1 == escape {
        let c2 = input.next()
            .ok_or_else(|| MatchError::expected("some char", input.as_str()))?;
        (input.as_str(), c2)
    } else {
        (input.as_str(), c1)
    }.tap(Ok)
}

// finds the matching closing brace to the opening brace that is located at input[-1]
// returns (if no error):
//     1. the new input pointer as beginning at the matched closed brace
//     2. the text between opening brace and corresponding (!) closing brace.
fn select_until_matching_brace<'a>(input: &'a Input, open: &str, close: &str) -> MatchResult<(&'a Input, &'a str)> {
    // level is now at 1
    // return the closing brace that brings level back to 0
    let mut level = 1;

    let input_start = input;
    let mut input = input;
    let brace_error = || MatchError::expected(&format!("Closing brace: '{}'", close), input_start);

    // ensure open != close [1]
    if open == close {
        unimplemented!();
    }

    loop {
        // branch [a] and branch [b] are mutually exclusive because of [1]
        if input.starts_with(open) { // [a]
            level += 1;
        }
        if input.starts_with(close) { // [b]
            level -= 1;
            if level == 0 {
                let length = input_start.len() - input.len();
                return Ok((input, &input_start[..length]))
            }
        }
        
        if input.is_empty() {
            return Err(brace_error());
        }
        input = skip_str(input, 1);
    }

    // old implementation
    /* 
        let get_next_brace = |input: &Input| {
            let s = input.matches(open).next();
            let t = input.matches(close).next();
            match (s, t) {
                (Some(s), Some(t)) if s.len() > t.len() => Some(s),
                (_, Some(t)) => Some(t),
                (Some(s), None) => Some(s),
                (None, None) => None
            }
            .map(|s| s.as_ptr() as usize - input.as_ptr() as usize)
        };
    
        loop {
            let i = get_next_brace(input).ok_or_else(brace_error)?;
            input = &input[i..];
    
            if input.starts_with(open) {
                level += 1;
            } else {
                level -= 1;
                if level == 0 {
                    let length = input_start.len() - input.len();
                    return Ok((input, &input_start[..length]));
                }
            }
        }
    */
}

// deprecated name.
fn find_matching_brace<'a>(input: &'a Input, open: &str, close: &str) -> MatchResult<(&'a Input, &'a str)> {
    return select_until_matching_brace(input, open, close);
}

// either matches one character, or escaped text that is enclosed in the given strings.
// the boolean returns whether the string was escaped or not
pub fn match_escapable_char<'a>(input: &'a Input, open: &str, close: &str) -> MatchResult<(&'a Input, &'a str, bool)> {
    let open_l = open.len();
    let close_l = close.len();
    if input.starts_with(open) {
        let (closing_brace, brace_contents) 
            = select_until_matching_brace(&input[open_l..], open, close)
            .ok().ok_or_else(|| 
                MatchError::expected(&format!("End of escape string: '{}'", close), "<end of file>"))?;
        
        Ok((&closing_brace[close_l..], brace_contents, true))
    } else {
        if input.is_empty() {
            MatchError::expected("some char", input).tap(Err)
        } else {
            Ok((skip_str(input, 1), skip_str(input, 1), false))
        }
    }
}

/// returns contained text
pub fn match_quote(input: &Input) -> MatchResult<(&str, &Input)> {
    // higher escape brace indices take precedence
    let (input_after, s, is_escaped) = match_escapable_char(input, ESCAPE_BRACE_OPEN[1], ESCAPE_BRACE_CLOSE[1])?;
    if is_escaped {
        Ok((s, input_after))
    } else {
        // try again with other escape braces
        let (input_after, s, is_escaped) = match_escapable_char(input, ESCAPE_BRACE_OPEN[0], ESCAPE_BRACE_CLOSE[0])?;
        if is_escaped {
            Ok((s, input_after))
        } else {
            Err(MatchError::expected("Quote", input))
        }
    }
}

pub fn match_whitespace(input: &Input) -> MatchResult<&Input> {
    let whitespace = &[' ', '\n', '\t'];
    for w in whitespace {
        match match_char(input, *w) {
            Ok(input) => return Ok(input),
            Err(_) => ()
        }
    }
    MatchError::expected("whitespace", input).tap(Err)
}

pub fn match_whitespaces(input: &Input) -> MatchResult<&Input> {
    let (input, _) = count_whitespaces(input)?;
    return Ok(input);
}

pub fn count_whitespaces(mut input: &Input) -> MatchResult<(&Input, usize)> {
    let mut count = 0usize;
    while let Ok(new_input) = match_whitespace(input) {
        input = new_input;
        count += 1;
    }
    Ok((input, count))
}


pub fn match_rule_invoc_<'a>(input: &'a Input, _: &(), rules: &Rules) -> MatchResult<(&'a Input , Invocation)> {
    match_rule_invoc(input, rules)
}

pub fn match_var_<'a>(input: &'a Input, _: &()) -> MatchResult<(&'a Input, Invocation)> {
    match_var(input)
}
