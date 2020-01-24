
use crate::*;

#[test]
fn test_match_rule_def() {
    let header = || {
        let mut header = Header::new();
        header.add_char('.');
        header.add_invoc(RuleInvocation("".into(), "rule".into()));
        header.add_char('}');
        header.seal()
    };

    let body = {
        let mut body = Body::new();
        body.add_invoc(VarInvocation ("var".into()));
        body.add_char(':');
        body.add_invoc(VarInvocation ("othervar".into()));
        body.seal()
    };

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}19"),
        Ok(("19", ("name1".into(), RuleVariant {
            header: header(),
            header_negated: false,
            body: None,
            append: "".into(),
        })))
    );

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}    {:var.::othervar}19"),
        Ok(("19", ("name1".into(), RuleVariant {
            header: header(),
            header_negated: false,
            body: Some(body),//once told me
            append: "".into(),
        })))
    );
}

use std::char;

pub fn match_file_invocation<'a>(input: &'a Input) -> MatchResult<(&'a Input, &'a str)> {
    if match_rule_definition(input).is_ok() {
        return MatchError::expected("file invocation (instead got rule definition)", input)
            .tap(Err);
    }

    let input = match_char(input, '%')?;
    let input = match_char(input, ':')?;
    let input = match_whitespaces(input)?;
    let filename_end = input.find(char::is_whitespace).unwrap_or(input.len());
    let filename = &input[..filename_end];
    let input = &input[filename_end..];

    Ok((input, filename))
}

pub fn match_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    let input = match_char(input, '%')?;
    let input = match_char(input, ':')?;
    match_inner_rule_definition(input)
}

/// matches the parts of a rule after '%:' (so that caller might scan for '%:' instead of calling this function everytime)
pub fn match_inner_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    // ruleName
    let input = match_whitespaces(input)?;
    let (input, rule_name) = match_ident(input).unwrap_or((input, ""));
    let rule_name = rule_name.into();
    let input = match_whitespaces(input)?;

    // maybe not
    let (input, negated) = if input.starts_with("not") {
        (match_whitespaces(&input[3..])?, true)
    } else {
        (input, false)
    };

    // {header with :rule:invocation.s} {body with :var.s}
    let header_start = input;
    let (input, header) = match_rule_part(input, match_invocation)?;
    let header = header.ok_or_else(|| MatchError::expected("Rule header", header_start))?;
    let input = match_whitespaces(input)?;
    let (input, body) = match_rule_part(input, match_var)?;

    Ok (
        (input, (rule_name, RuleVariant {header: header, header_negated: negated, body: body, append: String::new()}))
    )
}
