// Crate linkage metadata
#![crate_id = "xml-test"]
#![crate_type="bin"]

extern crate xml;

#[cfg(test)]
mod test_util;
#[cfg(test)]
mod test_lexer;
#[cfg(test)]
mod test_parser;

fn main() {
}
