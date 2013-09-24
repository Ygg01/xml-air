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

#[deriving(Eq)]
pub enum Character {
    Chars(char),
    NewLine
}

pub struct XmlParser {
    line: uint,
    col: uint,
    depth: uint,
    elem: Option<XmlElem>,
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

    /// This method reads a character and returns an enum that might be
    /// either a value of character, a new-line sign or a restricted
    /// character.
    fn read(&mut self)
            -> Character {
        //TODO implement docs and restricted chars
        let chr = self.raw_read();
        let retVal;
        match chr {
            '\r' => {
                self.line += 1u;
                self.col = 0u;
                retVal = NewLine;
                let chrPeek = self.raw_read();
                if(chrPeek != '\x85' && chrPeek != '\n'){
                    self.raw_unread();
                }
            },
            '\x85'
            | '\u2028' => {
                self.line += 1u;
                self.col = 0u;
                retVal = NewLine;
            },
            _ => {
                self.col += 1u;
                retVal = Chars(chr);
            }


        }
        retVal
    }

    #[inline]
    /// This method reads the source andBb simply updates position
    /// This method WILL NOT update new col or row
    fn raw_read(&mut self) -> char {
        self.source.read_char()
    }

    #[inline]
    /// This method unreads the source and simply updates position
    /// This method WILL NOT update new col or row
    fn raw_unread(&mut self) {
        self.source.seek(-1, SeekCur);
    }
}


pub fn main() {
    error!("This is an error log");
    warn!("This is a warn log");
    info!("this is an info log");
    debug!("This is a debug log");
}


#[cfg(test)]
mod tests{
    use super::*;
    use std::io::*;


    #[test]
    fn test_read_newline(){
        let r1 = @BytesReader {
                bytes : "a\r\nt".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlParser::from_reader(r1);
        assert_eq!(Chars('a'), parser.read());
        assert_eq!(1,   parser.line);
        assert_eq!(1,   parser.col);
        assert_eq!(NewLine,parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(0,   parser.col);
        assert_eq!(Chars('t'),parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(1,   parser.col);

        let r2= @BytesReader {
                bytes : "a\rt".as_bytes(),
                pos: @mut 0
        } as @Reader;

        parser = XmlParser::from_reader(r2);
        assert_eq!(Chars('a'), parser.read());
        assert_eq!(1,   parser.line);
        assert_eq!(1,   parser.col);
        assert_eq!(NewLine,parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(0,   parser.col);
        assert_eq!(Chars('t'),parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(1,   parser.col);

        let r3 = @BytesReader {
                bytes : "a\r\x85t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        parser = XmlParser::from_reader(r3);
        assert_eq!(Chars('a'), parser.read());
        assert_eq!(1,   parser.line);
        assert_eq!(1,   parser.col);
        assert_eq!(NewLine, parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(0,   parser.col);
        assert_eq!(Chars('t'),parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(1,   parser.col);


        let r4 = @BytesReader {
                bytes : "a\x85t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlParser::from_reader(r4);
        assert_eq!(Chars('a'), parser.read());
        assert_eq!(1,   parser.line);
        assert_eq!(1,   parser.col);
        assert_eq!(NewLine,parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(0,   parser.col);
        assert_eq!(Chars('t'),parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(1,   parser.col);
      

        let r5 = @BytesReader {
                bytes : "a\u2028t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlParser::from_reader(r5);
        assert_eq!(Chars('a'), parser.read());
        assert_eq!(1,   parser.line);
        assert_eq!(1,   parser.col);
        assert_eq!(NewLine,parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(0,   parser.col);
        assert_eq!(Chars('t'),parser.read());
        assert_eq!(2,   parser.line);
        assert_eq!(1,   parser.col);
    }

}
