
use crate::*;

use std::collections::HashSet;

use std::char;

pub fn match_file_invocation<'a>(input: &'a Input, rules: &Rules) -> MatchResult<(&'a Input, &'a str)> {
    if match_rule_definition(input, rules).is_ok() {
        return MatchError::expected("file invocation (instead got rule definition)", input)
            .tap(Err);
    }

    let input = match_str(input, RULE_DEFINITION_KEY)?;
    let input = match_whitespaces(input)?;
    let filename_end = input.find(char::is_whitespace).unwrap_or(input.len());
    let filename = &input[..filename_end];
    let input = &input[filename_end..];

    Ok((input, filename))
}

pub fn match_rule_definition<'a>(input: &'a Input, rules: &Rules) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    let input = match_str(input, RULE_DEFINITION_KEY)?;
    let input = match_whitespaces(input)?;
    match_inner_rule_definition(input, rules)
}

pub fn match_variant_flag(input: &Input) -> MatchResult<(&Input, &str)> {
    let input = match_char(input, '(')?;
    let (input, ident) = match_ident(input)?;
    let input = match_char(input, ')')?;
    Ok((input, ident))
}

/// matches the parts of a rule after '%:' (so that caller might scan for '%:' instead of calling this function everytime)
pub fn match_inner_rule_definition<'a>(mut input: &'a Input, rules: &Rules) -> MatchResult<(&'a Input, (String, RuleVariant))> {
    trace(input_view(input).to_string(), || {
        // flags
        let mut flags = HashSet::new();
        loop {
            input = match match_variant_flag(input) {
                Ok((input, flag)) => {flags.insert(flag.to_string()); input}
                Err(_) => break
            }.tap(match_whitespaces)?
        }

        // ruleName
        let (input, rule_name) = match_ident(input).unwrap_or((input, ""));
        let rule_name: String = rule_name.into();
        let mut input = match_whitespaces(input)?;

        loop {
            input = match match_variant_flag(input) {
                Ok((input, flag)) => {flags.insert(flag.to_string()); input}
                Err(_) => break
            }.tap(match_whitespaces)?
        }

        // (parameter header) {input header with :rule:invocation.s} -> {body with :var.s} (catch unknown rule) {body with :var.s}

        let (input, parameter_header_option) = match_invocation_string_def(input, rules, '(', ')', SWIRL_WHITESPACE_HANDLER_PARAM_HEADER)?;
        let input = match_whitespaces(input)?;
        
        let (input, input_header) = match_invocation_string_def(input, rules, '{', '}', SWIRL_WHITESPACE_HANDLER_HEADER)?;
        let input_header_is_implicit = input_header.is_none();
        let input_header = input_header.unwrap_or(InvocationString::empty());
        let input = match_whitespaces(input)?;

        match match_str(input, "->") {
            Err(_) if !input_header_is_implicit => {
                RuleVariant::new(input_header)
                    .parameter_header_option(parameter_header_option)
                    .flags(flags)
                    .verify(&rule_name)
                    .and_then(|rule_variant| Ok((input, (rule_name, rule_variant))))
            },
            Err(_) => Err(
                MatchError::new("This is not a rule definition.")
            ),
            Ok(input) => {
                let input = match_whitespaces(input)?;
                
                let (input, body) = match_invocation_string_def(input, rules, '{', '}', SWIRL_WHITESPACE_HANDLER_BODY)?;
                let body = body.ok_or(MatchError::expected("rule body", input))?;

                let input = match_whitespaces(input)?;
                let (input, catch_unknown_rule) = match match_str(input, "(catch unknown rule)") {
                    Ok(input) => {
                        let input = match_whitespaces(input)?;
                        let catch_body_start = input;
                        let (input, catch_body) = match_invocation_string_def(input, rules, '{', '}', SWIRL_WHITESPACE_HANDLER_CATCH_BODY)?;
                        let catch_body = catch_body.ok_or_else(|| MatchError::expected("Catch Body", catch_body_start))?;
                        //let input = match_whitespaces(input)?;
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
    })
}
