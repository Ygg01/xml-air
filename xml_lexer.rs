use std::io::*;
use util::*;

mod util;

#[deriving(Eq)]
enum XmlToken {
    LeftBracket,        // Symbol '<'
    RightBracket,       // Symbol '>'
    EndTag,             // Symbol '</'
    Text(~str),         // Various characters
    Whitespace(int),    // Whitespace
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
    EndOfFile
}

#[deriving(Eq)]
pub enum Character {
    Char(char),
    NewLine,
    RestrictedChar
}

pub struct XmlLexer {
    line: uint,
    col: uint,
    depth: uint,
    token: Option<XmlToken>,
    priv buf: ~str,
    priv source: @Reader
}

impl XmlLexer {
    /// Constructs a new `XmlLexer` from @Reader `data`
    /// The `XmlLexer` will use the given string as the source for parsing.
    pub fn from_reader(data : @Reader)
                     -> XmlLexer {
        XmlLexer {
            line: 1,
            col: 0,
            depth: 0,
            token: None,
            buf: ~"",
            source: data
        }
    }
    /// This method reads a character and returns an enum that might be
    /// either a value of character, a new-line sign or a restricted character.
    /// If it finds a restricted character the method will still update
    /// position accordingly.
    fn read(&mut self)
            -> Character {
        let chr = self.raw_read();
        let retVal;

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

                retVal = NewLine;

            },
            // A regular single character new line is found same as previous
            // section without the need to peek the next character.
            '\x85'
            | '\u2028' => {
                self.line += 1u;
                self.col = 0u;
                retVal = NewLine;
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



    #[test]
    /// Tests if it reads a restricted character
    /// and recognize a char correctly
    fn test_restricted_char(){
        let r1 = @BytesReader {
                bytes : "\x01\x04\x08a\x0B\x0Cb\x0E\x10\x1Fc\x7F\x80\x84d\x86\x90\x9F".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlLexer::from_reader(r1);

        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(Char('a'),           parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(Char('b'),           parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(Char('c'),           parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(Char('d'),           parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
        assert_eq!(RestrictedChar,      parser.read());
    }

    #[test]
    fn test_read_newline(){
        let r1 = @BytesReader {
                bytes : "a\r\nt".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlLexer::from_reader(r1);

        assert_eq!(Char('a'),   parser.read());
        assert_eq!(1,           parser.line);
        assert_eq!(1,           parser.col);
        assert_eq!(NewLine,     parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(0,           parser.col);
        assert_eq!(Char('t'),   parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(1,           parser.col);

        let r2= @BytesReader {
                bytes : "a\rt".as_bytes(),
                pos: @mut 0
        } as @Reader;

        parser = XmlLexer::from_reader(r2);
        assert_eq!(Char('a'),   parser.read());
        assert_eq!(1,           parser.line);
        assert_eq!(1,           parser.col);
        assert_eq!(NewLine,     parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(0,           parser.col);
        assert_eq!(Char('t'),   parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(1,           parser.col);

        let r3 = @BytesReader {
                bytes : "a\r\x85t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        parser = XmlLexer::from_reader(r3);
        assert_eq!(Char('a'),   parser.read());
        assert_eq!(1,           parser.line);
        assert_eq!(1,           parser.col);
        assert_eq!(NewLine,     parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(0,           parser.col);
        assert_eq!(Char('t'),   parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(1,           parser.col);


        let r4 = @BytesReader {
                bytes : "a\x85t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlLexer::from_reader(r4);
        assert_eq!(Char('a'),   parser.read());
        assert_eq!(1,           parser.line);
        assert_eq!(1,           parser.col);
        assert_eq!(NewLine,     parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(0,           parser.col);
        assert_eq!(Char('t'),   parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(1,           parser.col);
      

        let r5 = @BytesReader {
                bytes : "a\u2028t".as_bytes(),
                pos: @mut 0
        } as @Reader;

        let mut parser = XmlLexer::from_reader(r5);
        assert_eq!(Char('a'),   parser.read());
        assert_eq!(1,           parser.line);
        assert_eq!(1,           parser.col);
        assert_eq!(NewLine,     parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(0,           parser.col);
        assert_eq!(Char('t'),   parser.read());
        assert_eq!(2,           parser.line);
        assert_eq!(1,           parser.col);
    }
}