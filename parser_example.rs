pub use xml_parser::*;

pub mod xml_parser;

fn main() {
    let parser = Parser::new();
    let xmlStr = ~"<root></root>";
    parser.parseDoc()
}