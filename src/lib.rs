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
pub use lexer::{Lexer,XmlToken};
pub use util::{XmlError, is_whitespace, is_name_start};
pub use util::{is_name_char, Config,ErrBehavior};

pub mod lexer;
pub mod parser;
pub mod common;
pub mod util;
