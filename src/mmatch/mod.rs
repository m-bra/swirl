#![allow(dead_code)]

use crate::error::*;
use crate::*;

mod match_rule_part;
pub use match_rule_part::*;

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
            Ok(&input[1..])
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

pub fn match_ident<'a>(input: &'a Input) -> MatchResult<(&'a Input, &str)> {
    // todo: look at asm
    let len = input.chars().take_while(|c| c.is_alphabetic() || c.is_digit(10) || *c == '_').count();
    if len == 0 {
        MatchError::expected("identifier", input).tap(Err)
    } else {
        Ok((&input[len..], &input[..len]))
    }
}

pub fn match_var<'a>(input: &'a Input) -> MatchResult<(&'a Input, VarInvocation)> {
    match_char(input, ':').and_then(match_ident).map(|(input, ident)| (input, 
        VarInvocation(ident.into())
    ))
}

#[test]
pub fn test_match_invocation() {
    assert_eq!(
        match_invocation("::rule text"),
        Ok((" text", RuleInvocation::new("", "rule")))
    );
    assert_eq!(
        match_invocation(":name:rule text"),
        Ok((" text", RuleInvocation::new("name", "rule")))
    );
    assert!(
        match_invocation(":name:: text").is_err()
    );
    assert!(
        match_invocation("text :name:rule").is_err()
    );
}

pub fn match_invocation<'a>(input: &'a Input) -> MatchResult<(&'a Input, RuleInvocation)> {
    let input = match_char(input, RULE_INVOCATION_CHAR)?;
    let (input, variable_ident) = match_ident(input).unwrap_or((input, ""));
    let input = match_char(input, RULE_INVOCATION_CHAR)?;
    let (input, rule_ident) = match_ident(input)?;

    let (variable_ident, rule_ident) = (variable_ident.into(), rule_ident.into());
    (input, RuleInvocation(variable_ident, rule_ident)).tap(Ok)
}

#[test]
pub fn test_match_escapable_char() {
    assert_eq!(
        match_escapable_char("..text", '.'),
        Ok(("text", '.'))
    );
    assert_eq!(
        match_escapable_char("abtext", 'a'),
        Ok(("text", 'b'))
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
}

pub fn match_escapable_char(input: &Input, escape: char) -> MatchResult<(&Input, char)> {
    let mut input = input.chars();
    let c1 = input.next()
        .ok_or(MatchError::expected("some char", input.as_str()))?;

    if c1 == escape {
        let c2 = input.next()
            .ok_or(MatchError::expected("some char", input.as_str()))?;
        (input.as_str(), c2)
    } else {
        (input.as_str(), c1)
    }.tap(Ok)
}

pub fn match_whitespace(input: &Input) -> MatchResult<&Input> {
    let whitespace = &[' ', '\n', '\t'];
    let mut errors = vec![];
    for w in whitespace {
        errors.push(match match_char(input, *w) {
            Ok(input) => return Ok(input),
            Err(err) => err,
        });
    }
    MatchError::expected("whitespace", input).tap(Err)
}

pub fn match_whitespaces(mut input: &Input) -> MatchResult<&Input> {
    while let Ok(new_input) = match_whitespace(input) {
        input = new_input;
    }
    Ok(input)
}


pub fn match_invocation_<'a>(input: &'a Input, _: &()) -> MatchResult<(&'a Input , RuleInvocation)> {
    match_invocation(input)
}

pub fn match_var_<'a>(input: &'a Input, _: &()) -> MatchResult<(&'a Input, VarInvocation)> {
    match_var(input)
}