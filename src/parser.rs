use std::io::{Buffer, IoError, EndOfFile};

//use util::{XmlError};

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

struct XmlReader<'r,R :'r> {
    pub line: u64,
    pub col: u64,
    pub offset: u64,
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
            offset: 0,
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
    /// standard XML new lines into `\n`. Reading double newlines will
    /// increment offset by 2.
    ///
    /// According to XML-ER implementation supported line endings are:
    /// `\n`, `\r`, `\r \n`.
    pub fn read_norm_char(&mut self) -> ReadChar {
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
                        Ok('\n') => {
                            self.offset += 2
                        },
                        Ok(a) => {
                            self.peek_buf = Some(a);
                        },
                        Err(_) => {}
                    }
                } else {
                    self.offset += 1
                }
                Char('\n')
            },
            Ok(a)   => {
                self.offset += 1;
                self.col += 1;
                Char(a)
            }
        };
        retval
    }

}


pub struct Parser<'r, R:'r> {
    pub depth: uint,
    reader: XmlReader<'r,R>,
    state: StateEr
}

impl<'r, R: Buffer> Parser<'r, R> {
    /// Constructs a new Parser from Reader `data`
    /// The Parser will use the given reader as the source for parsing.
    pub fn from_reader(data: &'r mut R)
                     -> Parser<'r, R> {
        Parser {
            depth: 0,
            reader: XmlReader::from_reader(data),
            state: Data
        }
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
        let mut read = BufReader::new(b"ab\r\n\n");
        let mut xml_read = XmlReader::from_reader(&mut read);
        assert_eq!(Char('a'),       xml_read.read_norm_char());
        assert_eq!((1u64,1u64),     xml_read.position());
        assert_eq!(1u64,            xml_read.offset);
        assert_eq!(Char('b'),       xml_read.read_norm_char());
        assert_eq!((1u64,2u64),     xml_read.position());
        assert_eq!(2u64,            xml_read.offset);
        assert_eq!(Char('\n'),      xml_read.read_norm_char());
        assert_eq!((2u64,0u64),     xml_read.position());
        assert_eq!(4u64,            xml_read.offset);
        assert_eq!(Char('\n'),      xml_read.read_norm_char());
        assert_eq!((3u64,0u64),     xml_read.position());
        assert_eq!(5u64,            xml_read.offset);
    }
}

