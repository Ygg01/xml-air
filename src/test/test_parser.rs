use std::io::BufReader;

use xml::parser::{Parser, ElemStart};
use xml::node::{XmlElem};

#[test]
fn test_element(){
    let str1 = bytes!("<a>");
    let mut reader = BufReader::new(str1);
    let mut parser = Parser::from_reader(&mut reader);

    assert_eq!(Some(ElemStart),     parser.pull());
}