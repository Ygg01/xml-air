// Crate linkage metadata
#![crate_name = "tests"]
#![crate_type="bin"]

extern crate "xml_air" as xml;

#[cfg(test)]
mod test_util;
#[cfg(test)]
mod test_lexer;
#[cfg(test)]
mod test_parser;
