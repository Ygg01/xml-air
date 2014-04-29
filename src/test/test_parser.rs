use std::io::BufReader;

use xml::parser::{Parser, ElemStart};
use xml::node::{XmlElem};

#[test]
fn test_element(){
    let str1 = bytes!("<a>");
    let mut reader = BufReader::new(str1);
    let mut parser = Parser::from_reader(&mut reader);
    let a = XmlElem::new("a");

    assert_eq!(Some(ElemStart),     parser.pull());
    assert_eq!(Some(a),             parser.elem);
}

#[test]
fn test_empty(){
    let str1 = bytes!("");
    let mut reader = BufReader::new(str1);
    let mut parser = Parser::from_reader(&mut reader);

    assert_eq!(None,    parser.pull());
}