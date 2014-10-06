use std::io::BufReader;

use xml::parser::{Parser, ElemStart};
use xml::common::{XmlElem};

#[test]
fn test_element(){
    let mut reader = BufReader::new(b"<a>");
    let mut parser = Parser::from_reader(&mut reader);
    let a = XmlElem::new("a");

    assert_eq!(Some(ElemStart),     parser.pull());
    assert_eq!(Some(a),             parser.elem);
}

#[test]
fn test_empty(){
    let mut reader = BufReader::new(b"");
    let mut parser = Parser::from_reader(&mut reader);

    assert_eq!(None,    parser.pull());
}