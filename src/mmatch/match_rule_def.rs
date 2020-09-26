
use crate::*;

use std::collections::HashSet;

#[test]
fn test_match_rule_def() {
    let header = || {
        let mut header = InvocationString::new();
        header.add_char('.');
        header.add_invoc(Invocation::new_rule_invocation("", "rule"));
        header.add_char('}');
        header.seal()
    };

    let body = {
        let mut body = InvocationString::new();
        body.add_invoc(Invocation::new_var_invocation("var"));
        body.add_char(':');
        body.add_invoc(Invocation::new_var_invocation("othervar"));
        body.seal()
    };

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}19"),
        Ok(("19", ("name1".into(), RuleVariant::new(header(), None))))
    );

    assert_eq!(
        match_rule_definition("%:  name1{..::rule.}}    {:var.::othervar}19"),
        Ok(("19", ("name1".into(), RuleVariant::new(
            header(),
            Some(body),//once told me
        ))))
    );
}

use std::char;

pub enum SwirlStatement {
    FileInvocation(String),
    VariantDefinition(String, RuleVariant),
    VarAssignment(String, String)
}

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

pub fn match_variant_flag(input: &Input) -> MatchResult<(&Input, &str)> {
    let input = match_char(input, '(')?;
    let (input, ident) = match_ident(input)?;
    let input = match_char(input, ')')?;
    Ok((input, ident))
}

/// matches the parts of a rule after '%:' (so that caller might scan for '%:' instead of calling this function everytime)
pub fn match_inner_rule_definition<'a>(input: &'a Input) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    // ruleName
    let input = match_whitespaces(input)?;
    let (input, rule_name) = match_ident(input).unwrap_or((input, ""));
    let rule_name: String = rule_name.into();
    let mut input = match_whitespaces(input)?;

    // flags
    let mut flags = HashSet::new();
    loop {
        input = match match_variant_flag(input) {
            Ok((input, flag)) => {flags.insert(flag.to_string()); input}
            Err(_) => break
        }.tap(match_whitespaces)?
    }

    if flags.contains("once") && !rule_name.is_empty() {
        return MatchError::new("Can only use flag (once) on unnamed rules").tap(Err);
    }

    // {parameter header} {input header with :rule:invocation.s} -> {body with :var.s} (catch unknown rule) {body with :var.s}

    let symbolic_whitespace = WhiteSpaceHandling::Substitute(Invocation::new_rule_invocation("", "swirl_inserted_whitespace"));

    let (input, parameter_header_option) = match_invocation_string_def(input, '(', ')', &symbolic_whitespace)?;
    let input = match_whitespaces(input)?;

    let header_start = input;
    let (input, input_header) = match_invocation_string_def(input, '{', '}', &symbolic_whitespace)?;
    let input_header = input_header.ok_or_else(|| MatchError::expected("Rule header", header_start))?;
    let input = match_whitespaces(input)?;

    match match_str(input, "->") {
        Err(_) => {
            let missing_arrow_warning: MatchResult<()> = try {
                let input = match_whitespaces(input)?;
                match_str(input, "{")?;
            };
            if missing_arrow_warning.is_ok() {
                println!("Warning: Rule '{}' is probably missing an arrow in its variant {{{}}}", rule_name, input_header);
            }
            RuleVariant::new(input_header)
                .parameter_header_option(parameter_header_option)
                .flags(flags)
                .verify(&rule_name)
                .and_then(|rule_variant| Ok((input, (rule_name, rule_variant))))
        },
        Ok(input) => {
            let input = match_whitespaces(input)?;

            let (input, body) = match_invocation_string_def(input, '{', '}', &WhiteSpaceHandling::TrimLineBegin)?;
            let body = body.ok_or(MatchError::expected("rule body", input))?;

            let input = match_whitespaces(input)?;
            let (input, catch_unknown_rule) = match match_str(input, "(catch unknown rule)") {
                Ok(input) => {
                    let input = match_whitespaces(input)?;
                    let catch_body_start = input;
                    let (input, catch_body) = match_invocation_string_def(input, '{', '}', &WhiteSpaceHandling::TrimLineBegin)?;
                    let catch_body = catch_body.ok_or_else(|| MatchError::expected("Catch Body", catch_body_start))?;
                    (input, Some(catch_body))
                }
                Err(_) => (input, None)
            };
        
            RuleVariant::new(input_header)
                .parameter_header_option(parameter_header_option)
                .body(body)
                .flags(flags)
                .catch_unknown_rule_option(catch_unknown_rule)
                .verify(&rule_name)
                .and_then(|rule_variant| {
                    Ok((input, (rule_name, rule_variant)))
                })
        }
    }
}
