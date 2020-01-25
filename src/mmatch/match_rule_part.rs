use crate::*;

fn match_positive_rule_head<'a>(input: &'a Input, rule_head: &Header, rules: &Rules)
            -> MatchResult<(&'a Input, (HashMap<String, String>, Vec<String>))> {
    let mut results = HashMap::new();
    let mut anon_results = vec![];
    let mut input: &'a Input = input;

    for (part, invocs) in rule_head.iter() {
        input = match_str(input, part)?;

        for &RuleInvocation(ref var, ref rule) in invocs {
            input = {
                let rule = rules.get(rule)
                    .ok_or_else(|| {
                        panic!("unknown rule {}", rule)//MatchError::unknown_rule(rule, "<>")
                    })?;
                let (input, result) = rule.match_last(input, rules)?;

                if !var.is_empty() {
                    // TODO: could panic here
                    assert!(!results.contains_key(&var.to_string()));
                    results.insert(var.to_string(), result);
                } else {
                    anon_results.push(result);
                }

                input
            };
        }
    }

    Ok((input, (results, anon_results)))
}

// match rule header with the start of "input", possibly invoking other rules
// bind results of invocations to the specified variables (:var:rule)
// bind results of anonymous invocations to vector in correct order (::rule)
// return advanced input pointer or MatchError
pub fn match_rule_head<'a>(input: &'a Input, rule_head: &Header, negated: bool, rules: &Rules)
            -> MatchResult<(&'a Input, (HashMap<String, String>, Vec<String>))> {

    let inner_result = match_positive_rule_head(input, rule_head, rules);

    if negated {
        match inner_result {
            Ok(_) => {
                MatchError::expected(&format!("not {}", rule_head), input).tap(Err)
            },
            Err(_) => {
                Ok((input, (HashMap::new(), Vec::new())))
            }
        }
    } else {
        inner_result
    }
}

#[test]
fn test_match_rule_part() {
    let (_, rule_head) = match_rule_part("{I have.::n1:number:n2:number.:apples......::number}", match_invocation).unwrap();
    let rule_head = rule_head.unwrap();

    assert_eq!(rule_head, {
        let mut head = Header::new();
        head.add_str("I have:");
        head.add_invoc(RuleInvocation::new("n1", "number"));
        head.add_invoc(RuleInvocation::new("n2", "number"));
        head.add_str(":apples...");
        head.add_invoc(RuleInvocation::new("", "number"));
        head.seal()
    })
}

/// matches a rule header (including {}) or a rule body,
/// where `Invocation` is either RuleInvocation or VarInvocation
/// and `match_invocation` either match_invocation or match_var
/// if input does not start with '{', no error is returned but just None.
pub fn match_rule_part<'a, Invocation: Clone>(input: &'a Input, mut match_invocation: impl FnMut(&'a Input) -> MatchResult<(&'a Input, Invocation)>)
        -> MatchResult<(&'a Input, Option<RulePart<Invocation>>)> {
    let mut rulepart = RulePart::new();

    let mut input = match match_char(input, '{') {
        Ok(input) => input,
        Err(_) => return Ok((input, None)),
    };

    loop { input =
        if let Ok((input, invo)) = match_invocation(input) {
            rulepart.add_invoc(invo);
            input
        } else if let Some('}') = input.chars().next() {
            break;
        } else {
            let is_unescaped = input.chars().next().map(|c| c != ESCAPE_CHAR).unwrap_or(false);
            let (input, c) = match_escapable_char(input, ESCAPE_CHAR)?;
            if !(is_unescaped && c.is_whitespace()) {
                rulepart.add_char(c);
            }
            input
        }
    };
    let input = match_char(input, '}').expect("Internal error: Next char after loop in match_rule_part() has to be '}'!");
    Ok((input, Some(rulepart.seal())))
}

#[test]
fn _test_match_rule_head() {
    let mut rules = HashMap::new();
    rules.insert("number".to_string(), Rule {
        name: "number".to_string(),
        variants: vec![
            RuleVariant {
                header: Header::literally("0"),
                header_negated: false,
                once: false,
                body: None,
                append: "".to_string()
            },
            RuleVariant {
                header: Header::literally("1"),
                header_negated: false, once: false,
                body: None,
                append: "".to_string(),
            }
        ]
    });
    let mut results = HashMap::new();
    results.insert("n1".to_string(), "1".to_string());
    results.insert("n2".to_string(), "0".to_string());
    let anon_results = vec!["1".to_string()];

    let (_, rule_head) = match_rule_part("{I. have.::n1:number:n2:number.:apples......::number}", match_invocation).unwrap();
    let rule_head = rule_head.unwrap();

    assert_eq!(
        match_rule_head(
            "I have:10:apples...1 and 2 bananas", &rule_head, false, &rules
        ),
        Ok((" and 2 bananas", (results, anon_results)))
    );
}
