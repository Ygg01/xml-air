use std::io::*;
use util::*;

mod util;

#[deriving(Eq)]
pub enum XmlToken {
    LeftBracket,        // Symbol '<'
    RightBracket,       // Symbol '>'
    Equal,              // Symbol '='
    EndTag,             // Symbol '</'
    NameToken(~str),    // Tag name
    Text(~str),         // Various characters
    WhiteSpace,         // Whitespace
    PIStart,            // Start of PI block '<?'
    PIEnd,              // End of PI block '?>'
    CDataStart,         // Start of CDATA block '<![CDATA'
    CDataEnd,           // End of CDATA block ']]>'
    DoctypeStart,       // Start of Doctype block '<!DOCTYPE'
    DoctypeEnd,         // End of Doctype block '!>'
    CommentStart,       // Comment start <!--
    CommentEnd,         // Comment start --!>
    EntityRef,          // Entity refernce, symbol '&'
    PERef,              // Entity refernce, symbol '%'
    CharRef,            // Encoded char or '&#'
    Encoding(~str),     // Encoding and it's respective value e.g. Encoding(~"UTF-8")
    Standalone(bool),   // Standalone declaration, yes or no
    EndOfFile           // Denotes end of file
}

#[deriving(Eq,ToStr)]
pub enum Character {
    Char(char),
    RestrictedChar,
    EndFile
}


pub struct XmlLexer {
    line: uint,
    col: uint,
    token: Option<XmlToken>,
    priv buf: ~str,
    priv err_buf: ~str,
    priv source: @Reader
}

impl Iterator<XmlResult<XmlToken>> for XmlLexer {
    /// This method pulls tokens from stream until it reaches end of file.
    /// From that point on, it will return None.
    ///
    /// Example:
    /// TODO
    fn next(&mut self)
            -> Option<XmlResult<XmlToken>>{
        let chr_read = self.read();
        let token = match chr_read {
            // If we find a whitespace character  the following method
            // consumes all following whitespace characters until it
            // reaches a non white space character be it Restricted char,
            // EndFile or  a non-white space char
            Char(chr) if(is_whitespace(chr)) => {
                self.read_until_fn( |val| {
                    match val {
                        RestrictedChar => false,
                        EndFile => false,
                        Char(v) => is_whitespace(v)
                    }
                });
                Some(XmlResult{data: WhiteSpace, errors: ~[]})
            },
            Char('<') => {
                let chr_peek = self.peek_str(1u).data;
                match chr_peek {
                    ~"?" => {
                        self.read();
                        Some(XmlResult{data: PIStart, errors: ~[]})
                    }
                    ~"!" => {
                        self.read();
                        let sec_chr_peek = self.peek_str(1u).data;
                        match sec_chr_peek {
                            ~"[" => {
                                let peek_cdata = self.peek_str(6u).data;
                                if(peek_cdata == ~"[CDATA"){
                                    self.read_str(6u);
                                    Some(XmlResult {data: CDataStart, errors:~[]})
                                }else{
                                    //TODO peek
                                    Some(XmlResult {data: CDataStart, errors: ~[]})
                                }
                            },
                            _ => None
                        }
                    }
                    _   => Some(XmlResult{data: LeftBracket, errors: ~[]})
                }
            }
            _ => None
        };
        token

    }
}

impl XmlLexer {
    /// Constructs a new `XmlLexer` from @Reader `data`
    /// The `XmlLexer` will use the given reader as the source for parsing.
    pub fn from_reader(data : @Reader)
                     -> XmlLexer {
        XmlLexer {
            line: 1,
            col: 0,
            token: None,
            buf: ~"",
            err_buf: ~"",
            source: data
        }
    }
    /// This method reads a character and returns an enum that might be
    /// either a value of character, a new-line sign or a restricted character.
    /// 
    /// Encountering Restricted characters will not result in an error,
    /// Instead the position will be update but no information about such
    /// characters will not be preserved.
    ///
    /// Method shortcircuits if the End of file has been reached
    ///
    /// Note: This method will normalize all accepted newline characters into
    /// '\n' character.
    /// encountered will not be preserved.
    ///TODO add buffer for character/
    ///TODO add line char buffer
    fn read(&mut self) -> Character {


        if(self.source.eof()){
            return EndFile
        }
        let chr;
        let retVal;
        if(self.err_buf.is_empty()){
            chr= self.raw_read();
        }else{
            chr = self.err_buf.pop_char();
        }

        // This pattern matcher decides what to do with found character.
        match chr {
            // If char read is `\r` it must peek tocheck if `\x85` or `\n` are
            // next,  because they are part of same newline group.
            // According to `http://www.w3.org/TR/xml11/#sec-line-ends`
            // definition. This method updates column and line position.
            // Note: Lines and column start at 1 but the read character will be
            // update after a new character is read.
            '\r' => {
                self.line += 1u;
                self.col = 0u;

                let chrPeek = self.raw_read();
                if(chrPeek != '\x85' && chrPeek != '\n'){
                    self.raw_unread();
                }

                retVal = Char('\n');

            },
            // A regular single character new line is found same as previous
            // section without the need to peek the next character.
            '\x85'
            | '\u2028' => {
                self.line += 1u;
                self.col = 0u;
                retVal = Char('\n');
            },
            // If we encounter a restricted character as specified in
            // `http://www.w3.org/TR/xml11/#charsets` the compiler is notified
            // that such character has been found.
            // Restricted chars still but increase column number because
            // they might be ignored by the parser.
            a if (!is_char(&a) || is_restricted(&a)) => {
                self.col += 1u;
                retVal = RestrictedChar;
            },
            // A valid non-restricted char was found,
            // so we update the column position.
            _ => {
                self.col += 1u;
                retVal = Char(chr);
            }

        }
        retVal
    }

    //TODO Doc
    pub fn read_until_fn(&mut self, true_fn: &fn(Character)-> bool ) -> XmlResult<~str>{
        let mut col = 0u;
        let mut line = 1u;
        let mut char_read = self.read();
        let mut ret_str = ~"";

        while(true_fn(char_read)){
            match char_read {
                Char(a) => {
                    col = self.col;
                    line = self.line;
                    ret_str.push_char(a);
                    char_read = self.read();

                }
                _ => {}
            }
        }
        self.raw_unread();
        self.col = col;
        self.line = line;
        //TODO error checking
        XmlResult{ data: ret_str, errors: ~[]}
    }

    /// This method reads a string of given length skipping over any restricted char
    ///
    //TODO return XMLrestul
    pub fn read_str(&mut self, len: uint) -> XmlResult<~str> {
        let mut string = ~"";
        let mut found_errs = ~[];
        let mut eof = false;
        let mut l = 0u;

        while (l < len && !eof) {
            let chr = self.read();
            l += 1;
            match chr {
                Char(a) => string.push_char(a),
                EndFile => {
                    found_errs = ~[self.get_error(@~"Unexpected end of file")];
                    eof = true;
                },
                RestrictedChar =>{
                    found_errs = ~[self.get_error(@~"Illegal character")];
                }
            };

        };
        XmlResult{ data: string, errors:found_errs}
    }

    pub fn get_error(&mut self, err: @~str) -> XmlError {
        XmlError {
            line: self.line,
            col: self.col,
            msg: err,
            context: None,
            mark: None
        }
    }

    /// Method that peeks incoming strings
    pub fn peek_str(&mut self, len: uint) -> XmlResult<~str>{
        let col = self.col;
        let line = self.line;
        let offset = len as int;

        let peekStr  = self.read_str(len);
        self.col = col;
        self.line = line;
        self.source.seek(-offset, SeekCur);

        peekStr
    }

    #[inline]
    /// This method reads the source and updates position of
    /// pointer in said structure.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::*;
    use util::*;

    #[test]
    /// This method test buffer to ensure that adding characters into it
    /// will not cause premature end of line. 
    /// If program has six characters and lexer peeks 6 the reader will
    /// be moved, and those characters added to buffer.
    /// If reader isn't set back the read() method will end prematurely
    /// because it encountered an EOF sign, but it hasn't read all characters
    fn test_premature_eof(){
        let r = @BytesReader {
            bytes: "012345".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer =         XmlLexer::from_reader(r);
        lexer.peek_str(6u);
        assert_eq!(~"012345",       lexer.read_str(6u).data);
    }

    #[test]
    fn test_tokens(){
        let r = @BytesReader {
            bytes: "<?xml> <![CDATA]]> <!DOCTYPE !><a></a><a/><!-- --!>".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer =         XmlLexer::from_reader(r);

        assert_eq!(Some(XmlResult{ data: PIStart, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NameToken(~"xml"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: PIEnd, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CDataStart, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CDataEnd, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: DoctypeStart, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: DoctypeEnd, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: LeftBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NameToken(~"a"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: RightBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: LeftBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NameToken(~"a"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: EndTag, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CommentStart, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CommentEnd, errors: ~[] }),
                   lexer.next());

    }

    #[test]
    fn test_whitespace(){
        let r = @BytesReader {
            bytes: "   \t\n  a ".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r);
        let whitespace = XmlResult{ data: WhiteSpace, errors: ~[]};
        assert_eq!(Some(whitespace),    lexer.next());
        assert_eq!(7u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);

    }

    #[test]
    fn test_peek_str(){
        let r = @BytesReader {
            bytes: "as".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r);
        assert_eq!(~"as",                       lexer.peek_str(2u).data);
        assert_eq!(0u,                          lexer.col);
        assert_eq!(1u,                          lexer.line);
        assert_eq!(~"as",                       lexer.read_str(2u).data);
        assert_eq!(2u,                          lexer.col);
        assert_eq!(1u,                          lexer.line);
    }

    #[test]
    fn test_read_str(){
        let r = @BytesReader {
            bytes: "as".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r);
        assert_eq!(XmlResult{ data: ~"as", errors :~[]},               lexer.read_str(2u));
        r.seek(0, SeekSet);
        lexer = XmlLexer::from_reader(r);
        assert_eq!(XmlResult{ data: ~"as", errors: ~[XmlError{ line: 1u, col: 2u, msg: @~"Unexpected end of file", context: None, mark: None}]},
                    lexer.read_str(3u));
    }

    #[test]
    fn test_eof(){
        let r = @BytesReader {
            bytes: "a".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r);
        assert_eq!(Char('a'),           lexer.read());
        assert_eq!(EndFile,             lexer.read())
    }

    #[test]
    /// Tests if it reads a restricted character
    /// and recognize a char correctly
    fn test_restricted_char(){
        let r1 = @BytesReader {
                bytes : "\x01\x04\x08a\x0B\x0Cb\x0E\x10\x1Fc\x7F\x80\x84d\x86\x90\x9F".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r1);

        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(Char('a'),           lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(Char('b'),           lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(Char('c'),           lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(Char('d'),           lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
        assert_eq!(RestrictedChar,      lexer.read());
    }

    #[test]
    fn test_read_newline(){
        let r1 = @BytesReader {
                bytes : "a\r\nt".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r1);

        assert_eq!(Char('a'),   lexer.read());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);

        let r2= @BytesReader {
                bytes : "a\rt".as_bytes(),
                pos: @mut 0
        } as @Reader;

        lexer = XmlLexer::from_reader(r2);
        assert_eq!(Char('a'),   lexer.read());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);

        let r3 = @BytesReader {
                bytes : "a\r\x85t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        lexer = XmlLexer::from_reader(r3);
        assert_eq!(Char('a'),   lexer.read());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);


        let r4 = @BytesReader {
                bytes : "a\x85t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r4);
        assert_eq!(Char('a'),   lexer.read());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);
      

        let r5 = @BytesReader {
                bytes : "a\u2028t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r5);
        assert_eq!(Char('a'),   lexer.read());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);
    }
}
