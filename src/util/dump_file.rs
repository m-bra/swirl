use std::io::{self, Read, Write, Result};
use std::fs::File;
use std::error::Error;

pub fn dump_file(filename: &str) -> Result<String> {
    let mut buffer = String::new();
    File::open(filename)?.read_to_string(&mut buffer)?;
    Ok(buffer)
}
