
#[test]
fn test_find_first_definition() {
    assert_eq!(
        find_first_definition("0123%:  name1{...}}19"),

        Some(((4, 19), "name1".to_string(), RuleVariant {
            match_: Some("...}".to_string()),
            replace: None,
            append: "".to_string(),
        }
    )));
}

fn find_first_definition(grammar: &str) -> Option<((usize, usize), String, RuleVariant)> {
    lazy_static! {
    static ref RULE_RE: Regex = {
        assert_eq!(ESCAPE_CHAR, '.'); // all regexes in lazy_static! use . as escape char
        assert_eq!(RULE_INVOCATION_CHAR, ':'); // and : as rule invocation
        Regex::new(r"%:(?:\s*([a-zA-Z0-9_]+))?(?:\s*\{((?:[^\.\}]|(?:\..))*)\}(?:\s*\{((?:[^\.\}]|(?:\..))*)\}(?:\s*\{((?:[^\.\}]|(?:\..))*)\})?)?)?").unwrap()
        // https://regex101.com/r/Jlvyng/2
    };
}
    RULE_RE.captures_iter(&grammar).next().map(|capture| {
        let name = capture.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        ((capture.get(0).unwrap().start(), capture.get(0).unwrap().end()), name, RuleVariant {
            match_:  capture.get(2).map(|x| (x.as_str().to_string())),
            replace: capture.get(3).map(|x| (x.as_str().to_string())),
            append:  capture.get(4).map(|x| (x.as_str().to_string())).unwrap_or("".to_string())
        })
    })
}