use std::io::{Buffer, IoError, EndOfFile};
use std::num::{from_str_radix};
use std::char::{from_u32};
use super::{is_digit, is_hex_digit};

/// A struct representing states of an XML ER parser
#[deriving(PartialEq, Eq)]
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
    ErrEvent,
    FixMeEvent
}

pub struct XmlReader<'r,R :'r> {
    /// Line field denotes on which line of reader are we currently on.
    pub line: u64,
    /// Col fields denotes current column, or more precisely, how many
    /// characters from new line are we apart
    pub col: u64,
    /// `eof` field notifies parser it has reached end of file.
    pub eof: bool,
    peek_buf: Option<char>,
    source: &'r mut R
}
#[deriving(PartialEq, Eq, Show)]
pub enum ReadChar {
    CharErr(IoError),
    CharEOF,
    Char(char)
}

pub trait Filter {
    fn is_match(&self, char) -> bool;
}

impl Filter for char {
    fn is_match(&self, c: char) -> bool {
        return *self == c;
    }
}

impl Filter for fn(char) -> bool {
    fn is_match(&self, c: char) -> bool {
        (*self)(c)
    }
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
            eof: false,
            peek_buf: None,
            source: data
        }
    }

    /// A function that returns current line and column in
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
    pub fn read_nchar(&mut self) -> ReadChar {
        let chr;

        if self.peek_buf.is_none() {
            chr = self.source.read_char();
        } else {
            chr = Ok(self.peek_buf.unwrap());
            self.peek_buf = None;
        }

        let retval = match chr {
            Err(IoError{kind: EndOfFile, ..}) => {
                self.eof = true;
                CharEOF
            },
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

    fn peek(&mut self) -> Option<char> {
        if self.peek_buf.is_none() {
            let (line,col) = self.position();
            let old_flag = self.eof;

            match self.read_nchar() {
                Char(a) => self.peek_buf = Some(a),
                _       => self.peek_buf = None,
            }
            // If we peeked and saw flag by accident
            // this resets it back
            self.eof = old_flag;
            self.line = line;
            self.col = col;
        };
        self.peek_buf
    }

    pub fn read_until<Cond: Filter>(&mut self,  cond: Cond, opp: bool)
                                    -> String {
        let mut retval = String::new();

        loop {
            match self.read_nchar() {
                Char(c) => {
                    if cond.is_match(c) == opp {
                        self.peek_buf = Some(c);
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
    buf: String,
    state: StateEr,
    event: Option<XmlEvent>,
}

impl<'r, R: Buffer> Parser<'r, R> {
    /// Constructs a new Parser from Reader `data`
    /// The Parser will use the given reader as the source for parsing.
    pub fn from_reader(data: &'r mut R)
                     -> Parser<'r, R> {
        Parser {
            depth: 0,
            reader: XmlReader::from_reader(data),
            buf: String::new(),
            state: Data,
            event: None
        }
    }

    /// Consumes elements from reader until it is ready to emit a token.
    /// Upon consuming token the values of parsers can be looked for values
    pub fn pull(&mut self) -> Option<XmlEvent> {
        while self.event.is_none() {
            match self.state {
                Data    => self.data_state(),
                _       => self.event = Some(FixMeEvent),
            };
        }
        self.event
    }

    fn data_state(&mut self) {
        let chr = self.reader.read_nchar();
        match chr {
            Char('&')   => self.data_state(),
            Char('<')   => self.state = Tag,
            Char(a)     => {/*TODO */},
            _           => self.event = None,
        };
    }

    fn consume_entity(&mut self) {
        let chr = self.reader.read_nchar();
        match chr {
            Char('#') => {
                self.buf.push_str("&#");
                match self.reader.read_nchar() {
                    Char('x') => {
                        match self.reader.peek(){
                            Some(a) if is_hex_digit(a) => {
                                self.consume_num(true)
                            }
                            _ =>  self.buf.push('x')
                        }
                    },
                    Char(a) if is_digit(a) => {
                        self.buf = String::new();
                        self.buf.push(a);
                    },
                    Char(_) => {
                        //TODO
                    }
                    CharErr(_)
                    | CharEOF => {
                        self.event = None;
                    }
                }
                let text = self.buf.clone();
                //TODO
            },
            Char(_) => {
                //TODO
            }
            CharErr(_)
            | CharEOF => {
                //TODO
            },
        }
    }

    fn consume_num(&mut self, is_hex: bool) {
        let radix;
        let filter = if is_hex {
            radix = 16;
            is_hex_digit
        } else {
            radix = 10;
            is_digit
        };
        let digits = self.reader.read_until(filter, false);
        self.buf.push_str(digits.as_slice());
        let chr = match from_str_radix::<u32>(self.buf.as_slice(), radix){
            Some(x) => {
                let conv_chr = from_u32(x);
                match conv_chr {
                    Some(c) => c,
                    // For now just make a dummy value
                    None => '\uFFFD',
                }
            },
            // For now just make a dummy value
            None => '\uFFFD'
        };
        self.buf = String::from_char(1u,chr);
    }
}


#[cfg(test)]
mod test {
    use super::{XmlReader, Char};

    use std::io::BufReader;
    #[test]
    fn test_eof() {
        let mut read = BufReader::new(b"ab\r\n");
        let mut xml_read = XmlReader::from_reader(&mut read);
        xml_read.read_nchar();
        assert!(!xml_read.eof);

        xml_read.read_nchar();
        assert!(!xml_read.eof);

        xml_read.read_nchar();
        assert!(!xml_read.eof);

        xml_read.peek();
        assert!(!xml_read.eof);

        xml_read.read_nchar();
        assert!(xml_read.eof);

    }
    #[test]
    fn test_norm_char() {
        let mut read = BufReader::new(b"ab\r\n\na\ra\x00");
        let mut xml_read = XmlReader::from_reader(&mut read);
        assert_eq!(Char('a'),       xml_read.read_nchar());
        assert_eq!((1u64,1u64),     xml_read.position());
        assert_eq!(Char('b'),       xml_read.read_nchar());
        assert_eq!((1u64,2u64),     xml_read.position());
        assert_eq!(Char('\n'),      xml_read.read_nchar());
        assert_eq!((2u64,0u64),     xml_read.position());
        assert_eq!(Char('\n'),      xml_read.read_nchar());
        assert_eq!((3u64,0u64),     xml_read.position());
        assert_eq!(Char('a'),       xml_read.read_nchar());
        assert_eq!((3u64,1u64),     xml_read.position());
        assert_eq!(Char('\n'),      xml_read.read_nchar());
        assert_eq!((4u64,0u64),     xml_read.position());
        assert_eq!(Char('a'),       xml_read.read_nchar());
        assert_eq!((4u64,1u64),     xml_read.position());
        assert_eq!(Char('\uFFFD'),  xml_read.read_nchar());
        assert_eq!((4u64,2u64),     xml_read.position());
    }

    #[test]
    fn test_peek_char() {
        let mut read = BufReader::new(b"abc");
        let mut xml_read = XmlReader::from_reader(&mut read);
        assert_eq!(Some('a'),       xml_read.peek());
        assert_eq!((1u64,0u64),     xml_read.position());
        assert_eq!(Some('a'),       xml_read.peek());
        assert_eq!((1u64,0u64),     xml_read.position());
        assert_eq!(Some('a'),       xml_read.peek());
        assert_eq!((1u64,0u64),     xml_read.position());
        assert_eq!(Char('a'),       xml_read.read_nchar());
        assert_eq!((1u64,1u64),     xml_read.position());
    }
    #[test]
    fn test_read_until() {
        let mut read = BufReader::new(b"aaab");
        let mut xml_read = XmlReader::from_reader(&mut read);
        assert_eq!("aaa".to_string(),  xml_read.read_until('a', false));

        let mut read2 = BufReader::new(b"aaab");
        xml_read = XmlReader::from_reader(&mut read2);
        assert_eq!("".to_string(),     xml_read.read_until('a', true));

        let mut read3 = BufReader::new(b"aaab");
        xml_read = XmlReader::from_reader(&mut read3);
        assert_eq!("aaa".to_string(),  xml_read.read_until('b', true));

        let mut read4 = BufReader::new(b"aaab");
        xml_read = XmlReader::from_reader(&mut read4);
        assert_eq!("".to_string(),   xml_read.read_until('b', false));
    }
}
