// Crate linkage metadata
#[crate_id(name = "xml", vers = "0.1", author = "DanielFath", package_id="xml")];

//Metadata
#[desc = "XML pull parser for rust"];
#[crate_type = "lib"];


// Forbidden things
#[forbid(non_camel_case_types)];
#[forbid(non_uppercase_statics)];
#[forbid(unreachable_code)];


// Warn on missing docs
#[warn(unnecessary_qualification)];
#[warn(managed_heap_memory)];
//#[warn(missing_doc)];
//#[warn(owned_heap_memory)];

// Ignore dead code
#[allow(dead_code)];

pub use util::{XmlError, is_whitespace, is_name_start, is_name_char};

mod xml_lexer;
mod xml_parser;
mod xml_node;
mod util;


fn main() {
    error!("This is an error log");
    warn!("This is a warn log");
    info!("this is an info log");
    debug!("This is a debug log");
}
