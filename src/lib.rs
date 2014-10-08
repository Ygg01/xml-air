// Crate linkage metadata
#![crate_name = "xml_air"]


//Metadata
#![comment = "XML pull parser for rust"]
#![license = "MIT/LGPL"]
#![crate_type = "lib"]


// Forbidden things
#![forbid(non_camel_case_types)]
#![forbid(non_uppercase_statics)]
#![forbid(unreachable_code)]


// Warn on missing docs
#![warn(unnecessary_qualification)]
//#[warn(missing_doc)];
//#[warn(owned_heap_memory)];

// Ignore dead code
#![allow(dead_code)]

// Import lexer
pub mod parser;
pub mod common;
pub mod util;

pub enum XToken {
    StartTag
}