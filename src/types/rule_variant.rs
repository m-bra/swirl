use std::collections::{BTreeMap, HashMap};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleVariant {
    pub once: bool, // only valid flag for unnamed rules
    pub header_negated: bool,
    pub header: Header,
    pub body: Option<Body>,
    pub append: String,
}
