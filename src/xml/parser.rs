use std::io::{Reader, Buffer};

use node::{XmlDoc, XmlElem};
use util::{XmlError};
use lexer::{Lexer};

/// Struct that represents what XML events
/// may be encountered during pull parsing
/// of documents
#[deriving(Clone,Eq,Show)]
pub enum XmlEvent {
    DeclEvent,
    ElemStart,
    ElemEnd,
    EmptyElem,
    PIEvent,
    TextEvent,
    CDataEvent,
    ErrEvent
}

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

pub struct Parser<'r, R> {
    pub depth: uint,
    pub elem: Option<XmlElem>,
    pub err: Option<XmlError>,
    lexer: Lexer<'r,R>,
    state: State

}

// Struct to help with the Iterator pattern emulating Rust native libraries
pub struct XmlIterator <'i, 'r, R> {
    iter: &'i mut Parser<'r, R>
}

// The problem seems to be here
impl<'i, 'r, R: Reader+Buffer> Iterator<XmlEvent> for XmlIterator<'i, 'r, R> {
    fn next(&mut self) -> Option<XmlEvent> {
        self.iter.pull()
    }
}

impl<'i, 'r, R: Reader+Buffer> Parser<'r, R> {
    pub fn elems(&'i mut self) -> XmlIterator<'i, 'r, R>{
        XmlIterator{ iter: self}
    }
}

impl<'r, R: Reader+Buffer> Parser<'r, R> {
    /// Constructs a new Parser from Reader `data`
    /// The Parser will use the given reader as the source for parsing.
    /// ~~~
    /// let mut p = Parser::from_read(stdin)
    /// p.parse_doc() => XmlDoc { root: XmlElem {name: "root"} ... }
    /// ~~~
    pub fn from_reader(data: &'r mut R)
                     -> Parser<'r, R> {
        Parser {
            depth: 0,
            elem: None,
            err: None,
            lexer: Lexer::from_reader(data),
            state: OutsideTag
        }
    }

    /// This method will parse entire document into memory as a tree of
    /// XmlElem. It retuns an XmlDoc if it parses correctly or an Error
    /// if the parsing wasn't succesful.
    // TODO IMPLEMENT
    pub fn parse_doc(&mut self)
                     -> Result<XmlDoc,XmlError> {
        Ok(XmlDoc::new())
    }

    /// This method will pull next event and the result
    pub fn pull(&mut self)
                -> Option<XmlEvent> {
        //FIXME
        Some(ElemStart)
    }
}


pub fn main() {

}

