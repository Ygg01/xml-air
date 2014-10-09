use std::io::{Buffer, IoError, EndOfFile};
use std::str::{CharEq};
use super::{XToken, StartTag, EOFToken};

/// A struct representing states of an XML ER parser
enum StateEr {
    Data,
    Tag,
    EndTag,
    EndTagName,
    EndTagNameAfter,
    Pi,
    PiTarget,
    PiTargetAfter,
    PiContent,
    PiAfter,
    MarkupDecl,
    Comment,
    CommentDash,
    CommentEnd,
    Cdata,
    CdataBracket,
    CdataEnd,
    Doctype,
    DoctypeRootNameBefore,
    DoctypeRootName,
    DoctypeRootNameAfter,
    DoctypeIdentDoubleQ,
    DoctypeIdentSingleQ,
    DoctypeIntSubset,
    DoctypeIntSubsetAfter,
    DoctypeTag,
    DoctypeMarkupDecl,
    DoctypeComment,
    DoctypeCommentDash,
    DoctypeCommentEnd,
    DoctypeEnt,
    DoctypeEntTypeBefore,
    DoctypeEntParamBefore,
    DoctypeEntParam,
    DoctypeEntName,
    DoctypeEntNameAfter,
    DoctypeEntValDoubleQ,
    DoctypeEntValSingleQ,
    DoctypeEntValAfter,
    DoctypeEntIdent,
    DoctypeEntIdentDoubleQ,
    DoctypeEntIdentSingleQ,
    DoctypeAttlist,
    DoctypeAttlistNameBefore,
    DoctypeAttlistName,
    DoctypeAttlistNameAfter,
    DoctypeAttlistAttrname,
    DoctypeAttlistAttrnameAfter,
    DoctypeAttlistAttrtype,
    DoctypeAttlistAttrtypeAfter,
    DoctypeAttlistAttrdeclBefore,
    DoctypeAttlistAttrdecl,
    DoctypeAttlistAttrdeclAfter,
    DoctypeAttlistAttrvalDoubleQuoted,
    DoctypeAttlistAttrvalSingleQuoted,
    DoctypeNotation,
    DoctypeNotationIdent,
    DoctypeNotationIdentDoubleQ,
    DoctypeNotationIdentSingleQ,
    DoctypePi,
    DoctypeAfterPi,
    DoctypeBogusComment,
    TagName,
    EmptyTag,
    TagAttrNameBefore,
    TagAttrNameAfter,
    TagAttrValueBefore,
    TagAttrValueDoubleQuoted,
    TagAttrValueSingleQuoted,
    TagAttrValueUnquoted,
    BogusComment
}

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

pub struct XmlReader<'r,R :'r> {
    pub line: u64,
    pub col: u64,
    peek_buf: Option<char>,
    source: &'r mut R
}
#[deriving(PartialEq, Eq, Show)]
enum ReadChar {
    CharErr(IoError),
    CharEOF,
    Char(char)
}

impl<'r, R: Buffer> XmlReader<'r,R> {
    /// Function used for constructing XmlReader from field `data`
    /// that is both a reader and a buffer. One such element is
    /// `BufferedReader`
    pub fn from_reader(data: &'r mut R)
                        -> XmlReader<'r,R> {
        XmlReader {
            line: 1,
            col: 0,
            peek_buf: None,
            source: data
        }
    }

    /// A function that determines current line and column in
    /// given `XmlReader`
    pub fn position(&self) -> (u64, u64) {
        (self.line, self.col)
    }

    /// A function that reads and returns a single char, normalizing
    /// standard XML new lines into `\n`. Null characters '\x00' are
    /// normalized into '\uFFFD'.
    ///
    /// According to XML-ER implementation supported line endings are:
    /// `\n`, `\r`, `\r \n`.
    fn read_norm_char(&mut self) -> ReadChar {
        let chr;

        if self.peek_buf.is_none() {
            chr = self.source.read_char();
        } else {
            chr = Ok(self.peek_buf.unwrap());
            self.peek_buf = None;
        }

        let retval = match chr {
            Err(IoError{kind: EndOfFile, ..}) => CharEOF,
            Err(err)=> CharErr(err),
            Ok(chr) if "\r\n".contains_char(chr) => {
                self.line += 1;
                self.col = 0;

                if chr == '\r' {
                    match self.source.read_char() {
                        Ok(a) if a != '\n' => {
                            self.peek_buf = Some(a);
                        },
                        _ => {}
                    }
                }
                Char('\n')
            },
            Ok(a)   => {
                self.col += 1;
                if a == '\x00' {
                    return Char('\uFFFD')
                } else{
                    return Char(a)
                }
                Char(a)
            }
        };
        retval
    }

    fn read_until<Cond: CharEq>(&mut self,  cond: &mut Cond, opp: bool)
                                    -> String {
        let mut retval = String::new();

        loop {
            match self.read_norm_char() {
                Char(c) => {
                    if cond.matches(c) == opp {
                        break
                    } else {
                        retval.push(c)
                    }
                },
                _ => break
            }
        }
        retval
    }
}


pub struct Parser<'r, R:'r> {
    pub depth: uint,
    reader: XmlReader<'r,R>,
    state: StateEr,
    token: Option<XToken>
}

impl<'r, R: Buffer> Parser<'r, R> {
    /// Constructs a new Parser from Reader `data`
    /// The Parser will use the given reader as the source for parsing.
    pub fn from_reader(data: &'r mut R)
                     -> Parser<'r, R> {
        Parser {
            depth: 0,
            reader: XmlReader::from_reader(data),
            state: Data,
            token: None
        }
    }

    /// Consumes elements from reader until it is ready to emit a token.
    /// Upon consuming token the values of parsers can be looked for values
    pub fn pull(&mut self) -> Option<XToken> {
        while self.token.is_none() {
            match self.state {
                Data => self.data(),
                // FIXME: This part needs to go away
                _ => {self.token = Some(EOFToken);},
            };
        }
        self.token
    }

    fn data(&mut self) {
        let chr = self.reader.read_norm_char();
        match chr {
            Char('&')   => {self.token = Some(EOFToken)},
            Char('<')   => self.state = Tag,
            _   => self.token = Some(EOFToken),
        };
    }
}

// FIXME REMOVE THIS
pub fn main() {

}

#[cfg(test)]
mod test {
    use super::{XmlReader, Char};

    use std::io::BufReader;
    #[test]
    fn test_norm_char() {
        let mut read = BufReader::new(b"ab\r\n\na\ra\x00");
        let mut xml_read = XmlReader::from_reader(&mut read);
        assert_eq!(Char('a'),       xml_read.read_norm_char());
        assert_eq!((1u64,1u64),     xml_read.position());
        assert_eq!(Char('b'),       xml_read.read_norm_char());
        assert_eq!((1u64,2u64),     xml_read.position());
        assert_eq!(Char('\n'),      xml_read.read_norm_char());
        assert_eq!((2u64,0u64),     xml_read.position());
        assert_eq!(Char('\n'),      xml_read.read_norm_char());
        assert_eq!((3u64,0u64),     xml_read.position());
        assert_eq!(Char('a'),       xml_read.read_norm_char());
        assert_eq!((3u64,1u64),     xml_read.position());
        assert_eq!(Char('\n'),      xml_read.read_norm_char());
        assert_eq!((4u64,0u64),     xml_read.position());
        assert_eq!(Char('a'),       xml_read.read_norm_char());
        assert_eq!((4u64,1u64),     xml_read.position());
        assert_eq!(Char('\uFFFD'),  xml_read.read_norm_char());
        assert_eq!((4u64,2u64),     xml_read.position());
    }
    #[test]
    fn test_read_until() {
        let mut read = BufReader::new(b"aaab");
        let mut xml_read = XmlReader::from_reader(&mut read);
        assert_eq!("aaa".to_string(),  xml_read.read_until(&mut 'a', false));

        let mut read2 = BufReader::new(b"aaab");
        xml_read = XmlReader::from_reader(&mut read2);
        assert_eq!("".to_string(),     xml_read.read_until(&mut 'a', true));

        let mut read3 = BufReader::new(b"aaab");
        xml_read = XmlReader::from_reader(&mut read3);
        assert_eq!("aaa".to_string(),  xml_read.read_until(&mut 'b', true));

        let mut read4 = BufReader::new(b"aaab");
        xml_read = XmlReader::from_reader(&mut read4);
        assert_eq!("".to_string(),   xml_read.read_until(&mut 'b', false));
    }
}
