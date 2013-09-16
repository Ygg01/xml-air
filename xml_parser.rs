use xml_node::*;
use std::io::*;

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


pub struct XmlParser {
    line: uint,
    col: uint,
    depth: uint,
    elem: Option<XmlElem>,
    priv pos: uint,
    priv pushback: Option<char>,
    priv source: @Reader,
    priv buf: ~str,
    priv name: ~str,
    priv attrName: ~str,
    priv attributes: ~[XmlAttr],
    priv delim: char,
    priv state: State

}

impl XmlParser {
    /// std::io is on way out, so for time being I'm not making a from_str method
    /// once that part is stabilized, I'll implement convenience methods for 
    /// using 


    /// Constructs a new XmlParser from Reader `data`
    /// The Xmlparser will use the given string as the source for parsing.
    /// Best used for small examples.
    /// ~~~
    /// let mut p = XmlParser::from_read(stdin)
    /// p.parse_doc() => XmlDoc { root: XmlElem {name: "root"} ... }
    /// ~~~
    pub fn from_reader(data : @Reader)
                     -> XmlParser {
        XmlParser {
            line: 1,
            col: 0,
            buf: ~"",
            name: ~"",
            elem: None,
            pos: 0,
            pushback: None,
            source: data,
            attrName: ~"",
            attributes: ~[],
            delim: 0 as char,
            state: OutsideTag,
            depth: 0
        }
    }

    /// This method will parse entire document into memory as a tree of 
    /// XmlElem. It retuns an XmlDoc if it parses correctly or an Error
    /// if the parsing wasn't succesful.
    // TODO IMPLEMENT
    pub fn parse_doc(&mut self) 
                     -> Result<XmlDoc,Error> { 
        Ok(XmlDoc::new())
    }
    /// This method pulls tokens in similar way `parse_doc`  does, but 
    /// it also takes an callback to function to execute on each iteration.
    pub fn parse_call(&mut self, cb: &fn (Result<Events,Error>))
                      -> Result<XmlDoc,Error>{
        //TODO IMPLEMENT
        Ok(XmlDoc::new())
    }
    /// This method pulls tokens until it reaches a fully formed XML node
    /// once it's finished a node, it stops returning said node or error
    /// if it encountered one. 
    ///
    /// This method should be used similar to an outer iterator.
    pub fn next(&mut self) 
                -> Result<XmlNode,Error>{
        //TODO IMPLEMENT
        let retVal = Ok(XmlCDATA(~"CDATA"));

        retVal

    }

    fn read(&mut self) -> char {
        //Before reading we set the char to current position in stream
        let chr = self.source.read_char();
        let chrPeek = self.source.read_char();
        let vec: [char, ..2] = [chr, chrPeek];
        match vec {
            // We found a double character newline, thus we neeed to
            // update position
            ['\r', '\n']
            | ['\n', '\r'] => {
                self.line += 1u;
                self.col = 0u;
            },
            // We found a single character newline, thus we neeed to
            // unread a chrPeek and then update position
            ['\r', _ ]
            | ['\n', _ ] => {
                self.raw_unread();
                self.line += 1u;
                self.col = 0u;
            },
            // We found no extra char, just unread the character and update
            // line
            [_,_] => {
                self.raw_unread();
                self.col += 1u;
            }
        };
        chr
    }

    fn unread(&mut self, unr_str : &str) {

    }


    #[inline]
    /// This method reads the source and simply updates position
    /// This method WILL NOT update new col or row
    fn raw_read(&mut self) -> char {
        self.pos += 1u;
        self.source.read_char()
    }

    /// This method unreads the source and simply updates position
    /// This method WILL NOT update new col or row
    fn raw_unread(&mut self) {
        //REMOVING negative causes overflow test then fix this
        self.pos -= 1u;
        let pos = self.pos as int;
        self.source.seek(pos, SeekSet);
    }
}


pub fn main() {
    error!("This is an error log");
    warn!("This is a warn log");
    info!("this is an info log");
    debug!("This is a debug log");
}
