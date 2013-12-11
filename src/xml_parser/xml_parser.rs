use std::io::{Reader, Buffer};

use xml_node::{XmlDoc, XmlElem, XNode};
use util::{XmlError};
use xml_lexer::{XmlLexer};

mod xml_node;
mod util;
mod xml_lexer;

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

pub struct XmlParser<R> {
    line: uint,
    col: uint,
    depth: uint,
    elem: Option<XmlElem>,
    priv lexer: XmlLexer<R>,
    priv name: ~str,
    priv state: State

}

impl<R: Reader+Buffer> Iterator<XNode> for XmlParser<R> {
    /// This method pulls tokens, until it reaches a fully formed XML node.
    /// Once it finds a node, it stops returning said node or error
    /// if it there was an error during processing.
    ///
    /// This method should be used similar to an outer iterator.
    fn next(&mut self)
            -> Option<XNode>{
        None

    }
}

impl<R: Reader+Buffer> XmlParser<R> {
    /// Constructs a new XmlParser from Reader `data`
    /// The XmlParser will use the given reader as the source for parsing.
    /// ~~~
    /// let mut p = XmlParser::from_read(stdin)
    /// p.parse_doc() => XmlDoc { root: XmlElem {name: "root"} ... }
    /// ~~~
    pub fn from_reader(data: R)
                     -> XmlParser<R> {
        XmlParser {
            line: 1,
            col: 0,
            depth: 0,
            elem: None,
            lexer: XmlLexer::from_reader(data),
            name: ~"",
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

}


pub fn main() {
    error!("This is an error log");
    warn!("This is a warn log");
    info!("this is an info log");
    debug!("This is a debug log");
}

/*
#[cfg(test)]
mod tests{
    use super::*;
    use std::io::*;
    use xml_node::*;

    #[test]
    fn parse_simple(){
        let r1 = @BytesReader {
                bytes : "<a>".as_bytes(),
                pos: @mut 0
        } as ~Reader;

        let mut parser = XmlParser::from_reader(r1);
        let node = parser.next();
        match node {
            Some(Ok(a)) => {
                println(fmt!("PRINT: %?", a));
                assert_eq!(XElem(~XmlElem::new(~"a")), a);
            }
            _ => {
                fail!(~"No element found");
            }
        }

    }


}
*/
