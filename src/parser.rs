use std::io::{Reader, Buffer};

use common::{XmlDoc, XmlElem};
use util::{XmlError};
use lexer::{Lexer, XmlResult, NameToken, LessBracket, GreaterBracket};

/// Struct that represents what XML events
/// may be encountered during pull parsing
/// of documents
#[deriving(Clone, PartialEq, Eq, Show)]
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
    InStartTag
}

pub struct Parser<'r, R:'r> {
    pub depth: uint,
    pub elem: Option<XmlElem>,
    pub err: Option<XmlError>,
    peek: Option<XmlResult>,
    lexer: Lexer<'r,R>,
    state: State
}

pub struct ParserIterator<'i, 'p: 'i, R: 'p> {
    iter: &'i mut Parser<'p, R>
}

// The problem seems to be here
impl<'i,'r, R: Reader+Buffer> Iterator<XmlEvent> for ParserIterator<'i,'r, R> {
    fn next(&mut self) -> Option<XmlEvent> {
        self.iter.pull()
    }
}

impl<'r, R: Reader+Buffer> Parser<'r, R> {
    /// Constructs a new Parser from Reader `data`
    /// The Parser will use the given reader as the source for parsing.
    pub fn from_reader(data: &'r mut R)
                     -> Parser<'r, R> {
        Parser {
            depth: 0,
            elem: None,
            err: None,
            peek: None,
            lexer: Lexer::from_reader(data),
            state: OutsideTag
        }
    }

    pub fn nodes<'a>(&'a mut self) -> ParserIterator<'a, 'r, R> {
        ParserIterator {
            iter: self
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
        let token = self.peek_token();
        let mut event;
        // If end of token stream is found, return None
        if token.is_none() {
            return None;
        }

        // Otherwise it's time to see how states work
        event = match self.state {
            OutsideTag  => Some(self.parse_outside_tag(&token)),
            _           => None
        };

        event
    }

    fn peek_token(&mut self) -> Option<XmlResult> {
        if self.peek.is_none() {
            self.peek = self.lexer.pull();
        }
        self.peek.clone()
    }

    fn read_token(&mut self) -> Option<XmlResult> {
        if self.peek.is_none() {
            self.lexer.pull()
        } else {
            let token = self.peek.clone();
            self.peek = None;
            token
        }
    }

    fn parse_outside_tag(&mut self, token_peek: &Option<XmlResult> ) -> XmlEvent {

        match *token_peek {
            Some(Ok(LessBracket)) => {
                self.read_token();
                self.state = InStartTag;
                self.parse_start_tag()
            },
            _           => ErrEvent
        }

    }

    fn parse_start_tag(&mut self) -> XmlEvent {
        let mut event;
        let elem;
        match self.read_token() {
            Some(Ok(NameToken(x))) => {
                elem = Some(XmlElem::new(x.as_slice()));
                event = ElemStart;
            },
            // FIXME: Proper error handling
            _ => {
                elem = None;
                event = ErrEvent;
            }
        }

        match self.read_token() {
            Some(Ok(GreaterBracket)) => {
                self.elem = elem;
            },
            // FIXME: Proper error handling
            _ => {
                event = ErrEvent;
            }
        }

        event
    }

}


pub fn main() {

}

#[cfg(test)]
mod test {
    use super::{Parser};
    use lexer::{XmlResult, LessBracket, NameToken};

    use std::io::BufReader;

    #[test]
    fn read_token(){
        let mut read = BufReader::new(b"<XML>");
        let mut parser = Parser::from_reader(&mut read);

        assert_eq!(Some(Ok(LessBracket)),                       parser.read_token());
        assert_eq!(Some(Ok(NameToken("XML".into_string()))),    parser.read_token());
    }

    #[test]
    fn peek_token(){
        let mut read = BufReader::new(b"<XML>");
        let mut parser = Parser::from_reader(&mut read);

        assert_eq!(Some(Ok(LessBracket)),                       parser.peek_token());
        assert_eq!(Some(Ok(LessBracket)),                       parser.peek_token());
        assert_eq!(Some(Ok(LessBracket)),                       parser.read_token());
        assert_eq!(Some(Ok(NameToken("XML".into_string()))),    parser.peek_token());
        assert_eq!(Some(Ok(NameToken("XML".into_string()))),    parser.read_token());
    }
}

