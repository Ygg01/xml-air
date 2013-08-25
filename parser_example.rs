pub use xml_parser::*;
pub use xml_node::*;

pub mod xml_parser;
pub mod xml_node;

fn main() {
    let parser = Parser::new();
    let xmlStr = ~"<root></root>";
    parser.parseDoc(xmlStr);
}