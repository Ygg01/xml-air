use xml_node::*;
use std::io::ReaderUtil;

mod xml_node;

enum State {
    OutsideTag,
    TagOpened,
    InProcessingInstructions,
    InTagName,
    InCloseTagName,
    InTag,
    InAttrName,
    InAttrValue,
    ExpectDelimiter,
    ExpectClose,
    ExpectSpaceOrClose,
    InExclamationMark,
    InCDATAOpening,
    InCDATA,
    InCommentOpening,
    InComment1,
    InComment2,
    InDoctype,
    Namespace
}

pub struct Parser {
    priv line: uint,
    priv col: uint,
    priv buf: ~str,
    priv name: ~str,
    priv attrName: ~str,
    priv attributes: ~[XmlAttr],
    priv delim: char,
    priv state: State,
    priv level: uint
}

impl Parser {
    // Returns a new Parser
    pub fn new() -> Parser {
        let p = Parser {
            line: 1,
            col: 0,
            buf: ~"",
            name: ~"",
            attrName: ~"",
            attributes: ~[],
            delim: 0 as char,
            state: OutsideTag,
            level: 0
        };
        p
    }
    // This method parses a document from the result
    // TODO IMPLEMENT
    pub fn parseStr(&self, inStr: &str) -> XmlDoc{ XmlDoc::new() }

    // TODO IMPLEMENT
    pub fn parseIO(&self, input: &ReaderUtil) -> XmlDoc{ XmlDoc::new() }

    }
}


pub fn main() {
    let p = Parser::new();
    println(p.parseDoc("bla"));
}


