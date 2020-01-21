use std::collections::{BTreeMap, HashMap};
use crate::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuleVariant {
    pub header_negated: bool,
    pub header: Header,
    pub body: Option<Body>,
    pub append: String,
}
