
pub fn _firstline<'a>(string: &str) -> &str {
    string.split("\n").next().unwrap_or("")
}