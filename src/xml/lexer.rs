use std::ascii::StrAsciiExt;
use std::io::{Reader, Buffer};
use std::char::from_u32;
use std::str::from_char;
use std::num::from_str_radix;

use util::{is_whitespace, is_name_start, is_name_char};
use util::{ErrKind, Config};
use util::{is_restricted, clean_restricted, is_char};
use util::{RestrictedCharError,MinMinInComment,PrematureEOF,NonDigitError};
use util::{NumParsingError,CharParsingError,IllegalChar,UnknownToken};

mod util;

#[deriving(Eq, ToStr, Clone)]
pub enum XmlToken {
    /// Processing instruction token
    /// First string represents target and the second string
    /// represents text
    PI(~str, ~str),
    /// Start of PI block '<?'
    PrologStart,
    /// End of PI block '?>'
    PrologEnd,
    /// XML declaration encoding token
    Encoding(~str),
    /// XML declaration standalone token
    Standalone(bool),
    /// XML declaration version
    Version(~str),
    /// Error token
    ErrorToken(~str),
    /// Symbol '<'
    LessBracket,
    /// Symbol '>'
    GreaterBracket,
    /// Symbol '['
    LeftSqBracket,
    /// Symbol ']'
    RightSqBracket,
    /// Symbol '('
    LeftParen,
    /// Symbol ')'
    RightParen,
    /// Symbol '='
    Eq,
    /// Symbol '+'
    Plus,
    /// Symbol '|'
    Pipe,
    /// Symbol '*'
    Star,
    /// Symbol '&'
    Amp,
    /// Symbol '?'
    QuestionMark,
    /// Symbol '!'
    ExclamationMark,
    /// Symbol ','
    Comma,
    /// Percent '%'
    Percent,
    /// Symbol '</'
    CloseTag,
    /// Symbol '/>'
    EmptyTag,
    /// Tag or attribute name
    NameToken(~str),
    /// Qualified name token
    /// first string is prefix, second is local-part
    QNameToken(~str,~str),
    /// NMToken
    NMToken(~str),
    /// Various characters
    Text(~str),
    /// Whitespace
    WhiteSpace(~str),
    /// CData token with inner structure
    CData(~str),
    /// Start of Doctype block '<!DOCTYPE'
    DoctypeStart,
    /// Symbol '<!['
    DoctypeOpen,
    /// Symbol ']]>
    DoctypeClose,
    /// Symbol <!ENTITY
    EntityType,
    /// Symbol <!ATTLIST
    AttlistType,
    /// Symbol <!ELEMENT
    ElementType,
    /// Symbol <!NOTATION
    NotationType,
    /// Comment token
    Comment(~str),
    /// Encoded char or '&#'
    CharRef(char),
    /// Attribute reference
    Ref(~str),
    /// Parsed entity reference
    ParRef(~str),
    /// Single or double quoted string
    /// e.g. 'example' or "example"
    QuotedString(~str),
    /// Quote token
    Quote,
    /// Symbol #REQUIRED
    RequiredDecl,
    /// Symbol #IMPLIED
    ImpliedDecl,
    /// Symbol #FIXED
    FixedDecl,
    /// Symbol #PCDATA
    PCDataDecl

}

#[deriving(Eq,ToStr)]
pub enum Character {
    Char(char),
    RestrictedChar(char),
}

impl Character {

    pub fn is_valid_char(&self) -> bool {
        match *self {
            Char(_) => true,
            _       => false
        }
    }

    pub fn is_char(&self, c: char) -> bool {
        match *self {
            Char(a) => a==c,
            _       => false
        }
    }

    pub fn from_char(chr: char) -> Character {
        if is_restricted(&chr) {
            RestrictedChar(chr)
        } else if is_char(&chr) {
            Char(chr)
        } else {
            RestrictedChar(chr)
        }
    }
}
#[deriving(Eq,ToStr)]
enum State {
    OutsideTag,
    // Attlist takes quote, because attributes are mixed content and to
    // correctly display it, it treats each Quote as a special symbol
    // so for example "text&ref;" becomes `Quote Text(text) Ref(ref) Quote`
    Attlist(Quotes),
    EntityList(Quotes),
    InDoctype,
    InElementType,
    InEntityType,
    Pubid,
    InProlog,
    InStartTag,
    InternalSubset,
    Doctype,
    ExpectEncoding,
    ExpectStandalone,
    ExpectVersion
}
#[deriving(Eq,ToStr)]
enum Quotes {
    Single,
    Double
}

impl Quotes {
    pub fn to_char(&self) -> char {
        match *self {
            Single => '\'',
            Double => '"'
        }
    }

    pub fn from_str(quote: ~str) -> Quotes {
        if quote == ~"'" {
            Single
        } else if quote == ~"\"" {
            Double
        } else {
            println!(" Expected single (`'`) or double quotes (`\"`) got `{:?}` instead ", quote);
            fail!("fail");
        }
    }

    pub fn from_chr(quote: &char) -> Quotes {
        Quotes::from_str(from_char(*quote))
    }
}

pub struct Lexer<R> {
    line: uint,
    col: uint,
    config: Config,
    priv state: State,
    priv peek_buf: ~str,
    priv buf: ~str,
    priv source: R
}

// Struct to help with the Iterator pattern emulating Rust native libraries
pub struct TokenIterator <'b,R> {
    priv iter: &'b mut Lexer<R>
}

// The problem seems to be here
impl<'b,R: Reader+Buffer> Iterator<XmlToken> for TokenIterator<'b,R> {
    // Apparently I can't have &'b mut
    fn next(&mut self) -> Option<XmlToken> {
        self.iter.pull()
    }
}

impl<'iter,R: Reader+Buffer> Lexer<R> {
    pub fn tokens(&'iter mut self) -> TokenIterator<'iter,R>{
        TokenIterator{ iter: self}
    }
}

impl<R: Reader+Buffer> Lexer<R> {
    /// This method pulls tokens from Reader until it reaches
    /// end of file. From that point on, it will return None.
    ///
    /// Example:
    ///
    ///     let reader = Reader::new(bytes!("<a></a>"));
    ///     let mut lexer = Lexer::from_reader(reader);
    ///
    ///     // Calling lexer for each individual element
    ///     let token = lexer.pull();
    ///
    ///     // Calling lexer in a loop
    ///     for tok in lexer.tokens() {
    ///         println!(tok);
    ///     }
    ///     assert_eq!(None, lexer.pull());
    pub fn pull(&mut self) -> Option<XmlToken> {
        self.buf = ~"";

        let read_chr = match self.read_chr() {
            Some(a) => a,
            None => return None
        };

        match read_chr {
            Char(a) => self.buf.push_char(a),
            _ => {}
        }

        let token = match read_chr {
            RestrictedChar(_) => {
                Some(self.handle_errors(RestrictedCharError, None))
            },
            Char(chr) if is_whitespace(&chr)
                      => self.get_whitespace_token(),
            Char(a) => self.parse_char(&a)
        };

        token

    }

    fn parse_char(&mut self, c: &char ) -> Option<XmlToken> {
        match self.state {
            InStartTag => {
                match c {
                    chr if is_name_start(chr)
                            => self.get_qname_token(),
                    &'='    => self.get_equal_token(),
                    &'>'    => self.get_right_bracket_token(),
                    quote if quote == &'\'' || quote == &'"'
                            => {
                                self.state = Attlist(Quotes::from_chr(quote));
                                self.get_spec_quote()
                            },
                    &'<'    => self.get_left_bracket_token(),
                    &'/'    => self.get_empty_tag_token(),
                    _       => Some(self.handle_errors(IllegalChar, None))
                }
            },
            Attlist(quotes) => {
                match c {
                    &'&'    => self.get_ref_token(),
                    &'<'    => self.get_attl_error_token(),
                    &'\'' | &'"' if *c == quotes.to_char()
                            => {
                                self.state = InStartTag;
                                self.get_spec_quote()
                            },
                    _       => self.get_attl_text(&quotes.to_char())
                }
            },
            InDoctype => {
                match c {
                    &'>'    => {
                        let res = self.get_right_bracket_token();
                        self.state = OutsideTag;
                        res
                    },
                    c if is_name_char(c)
                            => self.get_name_token(),
                    &'\'' | &'"'
                            => self.get_quote_token(),
                    &'['    => {
                        let res = self.get_sqbracket_left_token();
                        self.state = InternalSubset;
                        res
                    },
                    // TODO change to error
                    _       => self.get_text_token()
                }
            },

            InternalSubset => {
                match c {
                    &'<' => {
                        let res = self.get_left_bracket_token();
                        if res == Some(EntityType) {
                            self.state = InEntityType;
                        } else if res == Some(ElementType) {
                            self.state = InElementType;
                        }
                        res
                    },
                    &']' => {
                        let res = self.get_sqbracket_right_token();
                        self.state = InDoctype;
                        res
                    },
                    &'%'  => self.get_peref_token(),
                    // TODO change to error
                    _   => self.get_text_token()
                }
            }
            InElementType => {
                match c {
                    chr if is_name_start(chr) => {
                        self.get_name_token()
                    },
                    &'>' => {
                        self.state = InternalSubset;
                        self.get_right_bracket_token()
                    },
                    &'(' => {
                        self.get_paren_left_token()
                    },
                    &')' => {
                        self.get_paren_right_token()
                    },
                    &'*' => {
                        self.get_star_token()
                    },
                    &'+' => {
                        self.get_plus_token()
                    },
                    &'|' => {
                        self.get_pipe_token()
                    },
                    &'%' => self.get_peref_token(),
                    &',' => self.get_comma_token(),
                    &'?' => self.get_question_mark_token(),
                    &'#' => self.get_pcdata_token(),
                    // TODO Change to proper error
                    _     => {
                        Some(self.handle_errors(IllegalChar, None))
                    }
                }
            },
            InEntityType => {
                match c {
                    chr if is_name_start(chr)
                         => self.get_name_token(),
                    &'>' => {
                        self.state = InternalSubset;
                        self.get_right_bracket_token()
                    },
                    &'%' => self.get_percent_token(),
                    quote if quote == &'\'' || quote == &'"'
                         => {
                        self.state = EntityList(Quotes::from_chr(quote));
                        self.get_spec_quote()
                    },
                    // TODO Change to proper error
                    _     => {
                        Some(self.handle_errors(IllegalChar, None))
                    }
                }
            },
            EntityList(quotes) => {
                match c {
                    &'&'    => self.get_ref_token(),
                    &'\'' | &'"' if *c == quotes.to_char()
                            => {
                                self.state = InEntityType;
                                self.get_spec_quote()
                            },
                    _       => self.get_attl_text(&quotes.to_char())
                }
            },
            _ => {
                match c {
                    chr if is_name_start(chr)
                              => self.get_name_token(),
                    chr if is_name_char(chr)
                              => self.get_nmtoken(),
                    &'<'  => self.get_left_bracket_token(),
                    &'&'  => self.get_ref_token(),
                    &'%'  => self.get_peref_token(),
                    &'>'  => self.get_right_bracket_token(),
                    &'?'  => self.get_pi_end_token(),
                    &'/'  => self.get_empty_tag_token(),
                    &'='  => self.get_equal_token(),
                    &'\''  | &'"'  => self.get_quote_token(),
                    _  => self.get_text_token(),
                }
            }
        }
    }
    /// Constructs a new `Lexer` from data given.
    /// Parameter `data` represents source for parsing,
    /// that must implement Reader and Buffer traits.
    /// Example
    /// ```rust
    ///    let bytes = bytes!("<an:elem />");
    ///    let lexer = xml::Lexer::from_reader(BufReader::new(bytes));
    /// ````
    pub fn from_reader(data : R) -> Lexer<R> {
        Lexer {
            line: 1,
            col: 0,
            config: Config::default(),
            peek_buf: ~"",
            state: OutsideTag,
            buf: ~"",
            source: data
        }
    }

    /// This method reads a string of given length skipping over any
    /// restricted character and adding an error for each such
    /// character encountered.
    ///
    /// Restricted characters are *not included* into the output
    /// string.
    pub fn read_str(&mut self, len: uint) -> ~str {
        clean_restricted(self.read_raw_str(len))
    }

    /// Method that peeks incoming strings
    /// Removes this function
    fn peek_str(&mut self, len: uint) -> ~str {
        let col = self.col;
        let line = self.line;

        let peek_result  = self.read_raw_str(len);

        self.col = col;
        self.line = line;

        for c in peek_result.chars_rev(){
             self.peek_buf.push_char(c);
        }

        clean_restricted(peek_result)
    }

    /// FIXME: Remove this function
    fn peek_chr(&mut self) -> Option<Character> {
        let col = self.col;
        let line = self.line;

        let peek_char = self.read_chr();
        self.col = col;
        self.line = line;

        match peek_char {
            Some(Char(a))
            | Some(RestrictedChar(a)) => self.peek_buf.push_char(a),
            None => {}
        }

        peek_char
    }
    #[inline(always)]
    /// FIXME: Replace this with in built rewind before read operations
    fn rewind(&mut self, col: uint, line: uint, peeked: ~str) {
        self.col = col;
        self.line = line;

        for c in peeked.chars_rev(){
             self.peek_buf.push_char(c);
        }
    }

    /// This method reads a character and returns an enum that
    /// might be either a value of character, or a
    /// restricted character. Encountering Restricted characters
    /// by default will not result in an error, only a warning.
    /// Position will still be updated upon finding Restricted
    /// characters. Characters that are neither restricted nor
    /// allowed will be ignored.
    ///
    /// If method reaches end of file it will return `None`.
    ///
    /// Note: This method will normalize all accepted newline
    /// characters into '\n' character. Encountered will not be
    /// preserved.
    /// See http://www.w3.org/TR/xml11/#sec-line-ends for more
    /// information
    pub fn read_chr(&mut self) -> Option<Character> {

        let chr;

        if self.peek_buf.is_empty() {

            let read_chr = self.source.read_char();

            match read_chr {
                Ok(a) => chr = a,
                // If an error occurs we abort further iterations
                Err(_) => {
                    return None
                }
            }
        } else {
            chr = self.peek_buf.pop_char();
        }

        if "\r\u2028\x85".contains_char(chr) {
           return Some(self.process_newline(chr))
        } else {
           return Some(self.process_char(chr))
        }

    }

    /// Processes the input `char` as it was a newline
    /// Note if char read is `\r` it must peek to check if
    /// `\x85` or `\n` are next, because they are part of same
    /// newline group.
    /// See to `http://www.w3.org/TR/xml11/#sec-line-ends`
    /// for details. This method updates column and line position
    /// accordingly.
    ///
    /// Note: Lines and column start at 1 but the read character
    /// will be update after a new character is read.
    fn process_newline(&mut self, c: char) -> Character {
        self.line += 1u;
        self.col = 0u;

        if c == '\r' {
            let chrPeek = self.source.read_char();
            match chrPeek {
                // If the read character isn't a double
                // new-line character (\r\85 or \n),
                // it's added to peek buffer
                Ok(a) if a != '\x85' && a != '\n'
                        => self.peek_buf.push_char(a),
                _ => {}

            }
        }

        Char('\n')
    }

    /// This method expects to takes an input `char` *c* that isn't a
    /// newline sigil. According to it, it then processes the given
    /// *c*, increasing position in reader.
    #[inline(always)]
    fn process_char(&mut self, c: char) -> Character {
        self.col += 1u;
        Character::from_char(c)
    }

    /// This method reads a string of given length, adding any
    /// restricted char  into the error section.
    /// Restricted character are *included* into the output string
    fn read_raw_str(&mut self, len: uint) -> ~str {
        let mut raw_str = ~"";
        let mut eof = false;
        let mut l = 0u;

        while l < len && !eof {
            let chr = self.read_chr();
            l += 1;
            match chr {
                Some(a) => {
                    match a {
                        Char(a) => raw_str.push_char(a),
                        RestrictedChar(a) => {
                            self.handle_errors(RestrictedCharError, None);
                            raw_str.push_char(a);
                        }
                    }
                },
                None => {
                    self.handle_errors(PrematureEOF, None);
                    eof = true;
                }
            };

        };
        raw_str
    }

    //TODO Doc
    // TODO rewrite this function to take a Filter trait, which will
    // deal with various queries behind screen
    fn read_while_fn(&mut self, fn_while: |Option<Character>|-> bool )
                     -> ~str {
        let mut col = self.col;
        let mut line = self.line;
        let mut ret_str = ~"";
        let mut chr = self.read_chr();

        while fn_while (chr) {
            match chr {
                None => break,
                Some(Char(a)) => {
                    ret_str.push_char(a);
                    col = self.col;
                    line = self.line;
                    chr = self.read_chr();
                },
                Some(RestrictedChar(_)) => {
                    col = self.col;
                    line = self.line;
                    chr = self.read_chr();
                }
            }
        }

        //After encountering wrong char
        // we 'unread' the last character
        self.col = col;
        self.line = line;
        match chr {
            Some(Char(a))
            | Some(RestrictedChar(a)) => {
                 self.peek_buf.push_char(a);
            },
            None => {}
        }

        ret_str
    }

    fn read_until_peek(&mut self, peek_look: &str) -> ~str {
        let mut peek = self.peek_str(peek_look.char_len());
        let mut result = ~"";
        while peek != peek_look.to_owned() {

            let extracted_char = self.read_chr();

            match extracted_char {
                None => {/* FIXME: Error processing*/},
                Some(Char(a))
                | Some(RestrictedChar(a)) => {result.push_char(a)}
            }

            peek = self.peek_str(peek_look.char_len());
        }
        result
    }


    fn handle_errors(&self, kind: ErrKind,
                     pass: Option<XmlToken>)
                     -> XmlToken {
        if kind == IllegalChar  {
            //println!("ERROR!");
        }
        ErrorToken(~"")
    }

    fn process_namechars(&mut self) -> ~str {
        self.read_while_fn( |val| {
            match val {
                Some(Char(v))             => util::is_name_char(&v),
                _ => false
            }
        })
    }

    fn process_name(&mut self) -> ~str {
        let mut result = ~"";
        match self.read_chr() {
            Some(Char(a)) if util::is_name_start(&a) => {
                result.push_char(a);
            },
            Some(Char(_)) => {
                self.handle_errors(IllegalChar, None);
            },
            Some(RestrictedChar(_)) => {
                self.handle_errors(RestrictedCharError, None);
            },
            None => {
                self.handle_errors(PrematureEOF, None);
            }
        }
        result.push_str(self.process_namechars());
        result
    }

    /// It will attempt to consume all digits until it reaches a non-digit
    /// numeral. If value `is_hex` is true it will consume all hexadecimal
    /// digits including values 0-9 a-f or A-F. If value `is_hex` is false it
    /// will only consume decimal digits
    fn process_digits(&mut self, is_hex: &bool) -> ~str {
         self.read_while_fn( |val| {
                match val {
                    Some(Char(v)) => {
                        if *is_hex  {
                            util::is_hex_digit(&v)
                        } else {
                            util::is_digit(&v)
                        }
                    },
                    _ => false
                }
            })
    }

    /// If we find a whitespace character this method
    /// consumes all following whitespace characters until it
    /// reaches a non white space character be it Restricted char,
    /// EndFile or  a non-white space char.
    fn get_whitespace_token(&mut self) -> Option<XmlToken> {

        let ws = self.read_while_fn( |val| {
            match val {
                Some(Char(v))             => util::is_whitespace(&v),
                _   => false
            }
        });

        self.buf.push_str(ws);

        Some(WhiteSpace(self.buf.clone()))
    }

    /// If we find a name start character this method
    /// consumes all name token until it reaches a non-name
    /// character.
    fn get_name_token(&mut self) -> Option<XmlToken> {
        let result = self.process_namechars();
        self.buf.push_str(result);

        // Prolog has three special type of quotes
        // these types are encoding, standalone and version.
        // Quotes that have specific behavior usually are handled
        // by lexer because they can have subtle errors.
        if self.state == InProlog && self.buf == ~"encoding" {
            self.state = ExpectEncoding
        } else if self.state == InProlog && self.buf == ~"standalone" {
            self.state = ExpectStandalone
        } else if self.state == InProlog && self.buf == ~"version" {
            self.state = ExpectVersion
        }

        Some(NameToken(self.buf.clone()))
    }

    /// If we find a name start character this method consumes
    /// all name characters until it reaches a non-name character.
    /// This method also handles qualfied names, as defined in
    /// [Namespace specification](http://www.w3.org/TR/xml-names11/)
    fn get_qname_token(&mut self) -> Option<XmlToken> {
        let result;
        let namechars = self.process_namechars();

        self.buf.push_str(namechars);
        if self.buf.contains_char(':'){
            if self.buf.char_at(0) == ':'
            || self.buf.char_at(self.buf.len()-1) == ':'{
                result = Some(NameToken(self.buf.clone()));
            } else {
                let split_name = self.buf.split(':').to_owned_vec();

                if split_name.len() == 2 {
                    result = Some(
                        QNameToken(split_name[0].to_owned(),
                                   split_name[1].to_owned())
                    );
                } else {
                    result = Some(NameToken(self.buf.clone()));
                }
            }
        } else {
            result = Some(NameToken(self.buf.clone()));
        }

        result

    }

    // TODO Write test
    fn get_nmtoken(&mut self) -> Option<XmlToken> {
        let mut name = ~"";

        match self.peek_chr() {
            Some(Char(a)) if(util::is_name_start(&a)) => {
                name.push_char(a);
            }
            _ => {
                self.handle_errors(IllegalChar, None);
            }
        };

        let result = self.process_namechars();
        name.push_str(result);

        Some(NameToken(name))
    }

    fn get_left_bracket_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<",   self.buf);

        let result;
        let col = self.col;
        let line = self.line;
        let chr = self.read_chr();
        let rew;

        match chr {
            Some(Char(a)) => {
                self.buf.push_char(a);
                rew = from_char(a);
            }
            Some(RestrictedChar(_)) => {
                return Some(self.handle_errors(IllegalChar, None));
            },
            None => {
                return Some(self.handle_errors(PrematureEOF, None));
            }
        }

        if self.buf == ~"</"{
            result = self.get_close_tag_token();
        } else if self.buf == ~"<?" {
            result = self.get_pi_token();
        } else if self.buf == ~"<!" {
            result = self.get_amp_excl();
        } else {
            // Only elements inside start tag can have
            // attributes, so we look if we are outside
            // tag, to make sure we don't activate it for
            // Doctype Declaration for example
            if self.state == OutsideTag {
                self.state = InStartTag;
            }
            self.rewind(col, line, rew);
            result = Some(LessBracket);
        }

        result
    }

    fn get_amp_excl(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!",   self.buf);
        let read = self.read_chr();

        let result = match read {
            Some(Char('[')) => {
                self.buf.push_char('[');
                self.get_cdata_token()
            },
            Some(Char('-')) => {
                self.buf.push_char('-');
                self.get_comment_token()
            },
            Some(Char('D')) => {
                self.buf.push_char('D');
                self.get_doctype_start_token()
            },
            Some(Char('E')) => {
                self.buf.push_char('E');
                self.get_entity_or_element_token()
            }
            None => Some(Text(~"<!")),
            _ => Some(Text(~"NON IMPLEMENTED"))
        };

        result
    }

    fn get_cdata_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<![",       self.buf);

        let col = self.col;
        let line = self.line;
        let cdata = self.read_str(6u);
        let result;

        if cdata == ~"CDATA[" {
            let text = self.read_until_peek("]]>");
            self.read_str(3u);

            result = Some(CData(text));
        } else {
            self.rewind(col, line, cdata);

            result = Some(DoctypeOpen);
        }
        result
    }

    fn get_doctype_start_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!D",       self.buf);
        let col         = self.col;
        let line        = self.line;
        let peeked_str  = self.read_str(6u);
        let result;

        if peeked_str == ~"OCTYPE" {
            self.state = InDoctype;
            result = Some(DoctypeStart);
        } else {
            self.rewind(col, line, peeked_str);
            result = Some(self.handle_errors(
                            UnknownToken,
                            Some(Text(~"<!D"))
                            )
                    );
        }
        result
    }


    #[inline(always)]
    fn get_equal_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"=",       self.buf);
        Some(Eq)
    }

    fn get_ref_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"&",  self.buf);
        let col = self.col;
        let line = self.line;
        let chr = self.read_chr();

        let token = match chr {
            Some(Char('#')) => {
                self.buf.push_char('#');
                self.get_char_ref_token()
            },
            Some(Char(a)) => {
                self.rewind(col,line, from_char(a));
                self.get_entity_ref_token(true)
            },
            Some(RestrictedChar(a)) => {
                self.rewind(col,line, from_char(a));
                self.handle_errors(
                    RestrictedCharError,
                    None
                );
                Some(Text(~"&"))
            },
            None => {
                Some(self.handle_errors(
                    PrematureEOF,
                    None
                ))
            }
        };
        token
    }

    fn get_peref_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"%",       self.buf);
        self.get_entity_ref_token(false)
    }

    fn get_char_ref_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"&#",       self.buf);
        let col = self.col;
        let line = self.line;
        let next_char = self.read_chr();


        let radix;
        match next_char {
            Some(Char('x')) => {
                radix = 16;
            },
            Some(Char(a)) if (util::is_digit(&a)) => {
                self.rewind(col,line, from_char(a));
                radix = 10;
            },
            Some(Char(_))
            | Some(RestrictedChar(_)) => {
                return Some(self.handle_errors(
                                NonDigitError,
                                Some(ErrorToken(self.buf.clone()))
                            )
                       );
            },
            None => {
                return Some(self.handle_errors(
                                PrematureEOF,
                                Some(ErrorToken(self.buf.clone()))
                            )
                        );
            }
        }

        let is_radix = (radix == 16);
        let char_ref = self.process_digits(&is_radix);

        match self.peek_chr() {
            Some(Char(';')) => {
                self.read_chr();
            },
            _ => {
                return Some(ErrorToken(self.buf.clone()));
            }
        }

        let parse_char = from_str_radix::<uint>(char_ref,radix);

        match parse_char {
            Some(a) => {
                let ref_char = from_u32(a as u32);

                match ref_char {
                    Some(a) => {
                         Some(CharRef(a))
                    }
                    _ => {
                        Some(self.handle_errors(
                                CharParsingError,
                                Some(ErrorToken(self.buf.clone()))
                            )
                        )
                    }
                }
            },
            None => {
                Some(self.handle_errors(
                        NumParsingError,
                        Some(ErrorToken(self.buf.clone()))
                    )
                )
            }
        }
    }

    fn get_entity_ref_token(&mut self, is_ent: bool) -> Option<XmlToken> {

        let ref_name = self.process_name();

        let col = self.col;
        let line = self.line;
        let expect_semi = self.read_chr();

        let result = match expect_semi {
            Some(Char(';')) => {
                if is_ent {
                    Some(Ref(ref_name))
                } else {
                    Some(ParRef(ref_name))
                }
            },
            Some(Char(_)) => {
                self.handle_errors(IllegalChar, None);
                if is_ent {
                    Some(Ref(ref_name))
                } else {
                    Some(ParRef(ref_name))
                }
            },
            Some(RestrictedChar(_)) => {
                self.handle_errors(IllegalChar, None);
                if is_ent {
                    Some(Ref(ref_name))
                } else {
                    Some(ParRef(ref_name))
                }
            },
            None => {
                Some(self.handle_errors(
                        PrematureEOF,
                        Some(ErrorToken(self.buf.clone()))
                    )
                )
            }
        };
        result
    }

    #[inline(always)]
    fn get_sqbracket_left_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"[",       self.buf);
        Some(LeftSqBracket)
    }

    #[inline(always)]
    fn get_sqbracket_right_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"]",       self.buf);
        Some(RightSqBracket)
    }

    #[inline(always)]
    fn get_paren_left_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"(",       self.buf);
        Some(LeftParen)
    }

    #[inline(always)]
    fn get_paren_right_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~")",       self.buf);
        Some(RightParen)
    }

    #[inline(always)]
    fn get_percent_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"%",       self.buf);
        Some(Percent)
    }

    fn get_entity_def_token(&mut self) -> Option<XmlToken> {
        //assert_eq!(Some(Char('#')),       self.read_chr());
        let result;
        if self.peek_str(8u) == ~"REQUIRED" {
            result = Some(RequiredDecl);
        } else if self.peek_str(7u) == ~"IMPLIED" {
            result = Some(ImpliedDecl);
        } else if self.peek_str(6u) == ~"PCDATA" {
            result = Some(PCDataDecl);
        } else if self.peek_str(5u) == ~"FIXED" {
            result = Some(FixedDecl);
        } else {
            result = Some(Text(~"#"));
        }
        result
    }

    fn get_pcdata_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"#",    self.buf);
        let col = self.col;
        let line = self.line;
        let rew = self.read_str(6u);
        let result;

        if rew == ~"PCDATA" {
            result = Some(PCDataDecl);
        } else {
            self.rewind(col, line, rew);
            result = Some(ErrorToken(~"#"));
        }

        result
    }

    fn get_entity_or_element_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!E", self.buf);

        let mut result = Some(Text(~"<!E"));
        let col = self.col;
        let line = self.line;
        let mut read = self.read_str(6);

        if read == ~"LEMENT" {
            result = Some(ElementType);
        } else {
            self.rewind(col, line, read);
        }

        read = self.read_str(5);

        if read == ~"NTITY" {
            result = Some(EntityType);
        } else {
            self.rewind(col, line, read);
        }

        result
    }

    fn get_attlist_token(&mut self) -> Option<XmlToken> {
        //assert_eq!(~"<!", self.read_str(2u));
        let result;

        if self.peek_str(7u) == ~"ATTLIST" {
            self.read_str(7u);
            result = Some(AttlistType);
        } else {
            result = Some(ErrorToken(~"<!"));
        }
        result
    }

    fn get_notation_token(&mut self) -> Option<XmlToken> {
        //assert_eq!(~"<!", self.read_str(2u));
        let result;
        if self.peek_str(8u) == ~"NOTATION" {
            self.read_str(8u);
            result = Some(NotationType);
        } else {
            result = Some(ErrorToken(~"<!"));
        }
        result
    }

    fn get_star_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"*",       self.buf);
        Some(Star)
    }

    fn get_plus_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"+",       self.buf);
        Some(Plus)
    }

    fn get_pipe_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"|",       self.buf);
        Some(Pipe)
    }

    fn get_comma_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~",",       self.buf);
        Some(Comma)
    }


    fn get_quote_token(&mut self) -> Option<XmlToken> {
        let quote = self.buf.clone();
        assert!(quote == ~"'" || quote == ~"\"");

        let quote_char = if quote == ~"'" { '\''} else { '"'};

        // TODO only keep encoding quote, it has special rules, others are
        // Better reported by a parser.
        let result = match self.state {
            ExpectEncoding      => self.process_encoding_quote(&quote_char),
            ExpectStandalone    => self.proces_standalone(quote),
            ExpectVersion       => self.proces_version(quote),
            _                   => self.process_quotes(quote)
        };

        Some(result)
    }

    fn proces_standalone(&mut self, quote: ~str) -> XmlToken {
        assert_eq!(ExpectStandalone, self.state);
        let quote = self.process_quotes(quote);
        self.state = InProlog;

        let result = match quote {
            QuotedString(~"yes")    => Standalone(true),
            QuotedString(~"no")     => Standalone(false),
            _                       => quote
        };
        result
    }

    fn proces_version(&mut self, quote: ~str) -> XmlToken {
        assert_eq!(ExpectVersion, self.state);
        let quote = self.process_quotes(quote);
        self.state = InProlog;

        let result = match quote {
            QuotedString(~"1.1")    => Version(~"1.1"),
            QuotedString(~"1.0")    => Version(~"1.0"),
            _                       => quote
        };
        result
    }

    fn process_quotes(&mut self, quote: ~str) -> XmlToken {
        let text = self.read_until_peek(quote);
        let peek = self.peek_str(1u);

        if peek != quote {

            self.handle_errors(
                IllegalChar,
                Some(QuotedString(text.clone()))
            );
        } else {
            self.read_str(1u);
        }

        QuotedString(text.clone())
    }

    fn process_encoding_quote(&mut self, quote: &char) -> XmlToken {
        assert_eq!(ExpectEncoding, self.state);
        assert!(*quote == '\'' || *quote == '"');
        self.state = InProlog;

        let result;
        let mut chr = self.read_chr();
        let mut first_char = true;

        //Clear buffer
        self.buf = ~"";

        while chr != Some(Char(*quote)) {
            match chr {
                Some(Char(c)) => {
                    if first_char {
                        if !util::is_encoding_start_char(&c) {
                           return self.handle_errors(
                                    IllegalChar,
                                    Some(Encoding(self.buf.clone()))
                            );
                        }
                        first_char = false;
                    } else {
                        if !util::is_encoding_char(&c) {
                            return self.handle_errors(
                                    IllegalChar,
                                    Some(Encoding(self.buf.clone()))
                            );
                        }
                    }
                    self.buf.push_char(c);
                },
                Some(RestrictedChar(_)) => {
                    return self.handle_errors(
                                IllegalChar,
                                Some(Encoding(self.buf.clone()))
                    )
                },
                None => return self.handle_errors(
                                PrematureEOF,
                                Some(Encoding(self.buf.clone()))
                    )
            }

            chr = self.read_chr();
        }
        result = Encoding(self.buf.clone());
        result
    }

    fn get_pubid_quote(&mut self) -> Option<XmlToken> {
        let quote = self.read_str(1u);
        let b = quote == ~"'" || quote == ~"\"";
        //assert!(b);

        let result = self.process_quotes(quote.clone());

        match result {
            QuotedString(ref text) => {
                for c in text.chars() {
                    if util::is_pubid_char(&c) {
                        return Some(self.handle_errors(
                                        IllegalChar,
                                        Some(result.clone())
                                ));
                    }
                }
            },
            _ => {}
        }

        Some(result)
    }

    fn get_ent_quote(&mut self) -> Option<XmlToken> {
        None
    }

    #[inline]
    fn get_spec_quote(&mut self) -> Option<XmlToken> {
        assert!(self.buf == ~"'" || self.buf == ~"\"");
        Some(Quote)
    }

    fn get_attl_text(&mut self, quote: &char) -> Option<XmlToken> {
        let text = self.read_while_fn( |val| {
            match val {
                Some(Char(a))  => (a != '<' && a != '&' && a != *quote),
                _ => false
            }
        });
        let result = self.buf.clone().append(text);
        Some(Text(result))
    }

    #[inline(always)]
    fn get_attl_error_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<", self.buf);
        Some(self.handle_errors(IllegalChar, None))
    }

    fn get_text_token(&mut self) -> Option<XmlToken> {
        let mut peek = ~"";
        let mut text = self.buf.clone();
        let mut run_loop = true;
        while run_loop {
            let read = self.read_chr();

            match read {
                None                    => run_loop = false,
                Some(RestrictedChar(_)) => {},
                Some(Char('&'))         => run_loop = false,
                Some(Char('<'))         => run_loop = false,
                Some(Char(a))           => {
                    if peek.len() == 3 {
                        peek.shift_char();
                        peek.push_char(a);
                    }
                    if peek == ~"]]>" {
                        run_loop = false;
                        // if we found this, it means we already took `]]`
                        text.pop_char();
                        text.pop_char();
                    }

                    if run_loop {
                        text.push_char(a);
                    }
                }
            }

        }
        Some(Text(text))
    }

    fn get_pi_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<?",       self.buf);

        // Process target name
        let target = self.process_name();


        if target.eq_ignore_ascii_case("xml") {
            self.state = InProlog;
            return Some(PrologStart);
        } else {
            // We skip a possible whitespace token
            // to get to text of PI
            self.get_whitespace_token();

            let text = self.read_until_peek("?>");
            self.read_str(2u);
            return Some(PI(target,text));
        }
    }

    fn get_right_bracket_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~">", self.buf);
        return Some(GreaterBracket)
    }

    fn get_comment_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!-", self.buf);
        let col = self.col;
        let line = self.line;
        let rewind_str = self.read_str(1u);

        if rewind_str == ~"-" {

            let text = self.process_comment();
            return Some(Comment(text))
        } else {

            self.rewind(col,line, rewind_str);
            return Some(ErrorToken(~"<!-"))
        }
    }

    fn process_comment(&mut self) -> ~str {
        let mut peek = self.peek_str(3u);
        let mut result = ~"";
        let mut found_end = false;

        while !found_end {
            if peek.starts_with("--") && peek == ~"-->" {
                self.read_str(3u);
                found_end = true;
            } else {
                if peek.starts_with("--") && peek != ~"-->" {
                    self.handle_errors(
                        MinMinInComment,
                        Some(Comment(result.clone()))
                    );
                }

                match self.read_chr() {
                    None => {},
                    Some(Char(a))
                    | Some(RestrictedChar(a)) => {result.push_char(a)}
                }
                peek = self.peek_str(3u);
            }
        }
        result
    }

    fn get_close_tag_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"</",  self.buf);
        return Some(CloseTag)
    }

    fn get_empty_tag_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"/", self.buf);

        let result;
        if self.read_str(1u) == ~">" {
            result = Some(EmptyTag);
        } else {
            result = Some(ErrorToken(~"/"));
        }
        result
    }

    fn get_pi_end_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"?",   self.buf);
        let line = self.line;
        let col  = self.col;

        let chr = self.read_chr();
        let result = match chr {
            Some(Char('>')) => {
                self.state = OutsideTag;
                Some(PrologEnd)
            },
            Some(Char(a)) => {
                self.rewind(col, line, a.to_str());
                Some(QuestionMark)
            },
            Some(RestrictedChar(_)) => {
                self.handle_errors(RestrictedCharError, Some(QuestionMark));
                Some(QuestionMark)
            },
            None => {
                Some(QuestionMark)
            }
        };
        result
    }

    fn get_question_mark_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"?", self.buf);
        Some(QuestionMark)
    }
}

pub fn main() {

}

#[cfg(test)]
mod tests {

    use super::{Lexer, Char, RestrictedChar};
    use super::{PrologEnd,PrologStart,PI,CData,WhiteSpace};
    use super::{DoctypeStart,CharRef};
    use super::{Percent,NameToken};
    use super::{EntityType,Comment};
    use super::{AttlistType,GreaterBracket,LessBracket,ElementType};
    use super::{CloseTag,Eq,Star,QuestionMark,Plus,Pipe};
    use super::{LeftParen,RightParen,EmptyTag,QuotedString,Text};
    use super::{Encoding, Standalone, Version, Ref, Quote, QNameToken};
    use super::{LeftSqBracket, RightSqBracket, InEntityType,PCDataDecl};
    use super::{Comma,ParRef};

    use std::io::BufReader;


    #[test]
    fn test_iteration() {
        let bytes = bytes!("<a>");
        let r = BufReader::new(bytes);
        let mut lexer = Lexer::from_reader(r);
        for token in lexer.tokens() {
        }

        assert_eq!(None,                lexer.pull());
    }

    #[test]
    fn test_pi() {
        let str0 = bytes!("<?php var = echo()?><?php?>");
        let buf0 = BufReader::new(str0);

        let mut lexer = Lexer::from_reader(buf0);

        assert_eq!(Some(PI(~"php", ~"var = echo()")),   lexer.pull());
        assert_eq!(Some(PI(~"php", ~"")),               lexer.pull());

        let str1 = bytes!("<?xml encoding = 'UTF-8'?>");
        let buf1 =BufReader::new(str1);

        lexer = Lexer::from_reader(buf1);

        assert_eq!(Some(PrologStart),                   lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(NameToken(~"encoding")),        lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Eq),                            lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Encoding(~"UTF-8")),            lexer.pull());
        assert_eq!(Some(PrologEnd),                     lexer.pull());

        let str3 = bytes!("<?xml standalone = 'yes'?>");
        let buf3 = BufReader::new(str3);

        lexer = Lexer::from_reader(buf3);

        assert_eq!(Some(PrologStart),                   lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(NameToken(~"standalone")),      lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Eq),                            lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Standalone(true)),              lexer.pull());
        assert_eq!(Some(PrologEnd),                     lexer.pull());

        let str4 = bytes!("<?xml standalone = 'no'?>");
        let buf4 =BufReader::new(str4);

        lexer = Lexer::from_reader(buf4);

        assert_eq!(Some(PrologStart),                   lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(NameToken(~"standalone")),      lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Eq),                            lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Standalone(false)),             lexer.pull());
        assert_eq!(Some(PrologEnd),                     lexer.pull());

        let str5 = bytes!("<?xml version = '1.0'?>");
        let buf5 =BufReader::new(str5);

        lexer = Lexer::from_reader(buf5);

        assert_eq!(Some(PrologStart),                   lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(NameToken(~"version")),         lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Eq),                            lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Version(~"1.0")),               lexer.pull());
        assert_eq!(Some(PrologEnd),                     lexer.pull());
    }

    #[test]
    fn test_cdata() {

        let str1  = bytes!("<![CDATA[various text data like <a>]]>!");
        let read1 = BufReader::new(str1);

        let mut lexer = Lexer::from_reader(read1);

        assert_eq!(Some(CData(~"various text data like <a>")),  lexer.pull());
        assert_eq!(Some(Char('!')),                     lexer.read_chr());

        let str2 = bytes!("<![C!");
        let read2 = BufReader::new(str2);

        lexer = Lexer::from_reader(read2);

        lexer.pull();
        assert_eq!(Some(Char('C')),                     lexer.read_chr());
    }

    #[test]
    fn test_comment(){
        let str1  = bytes!("<!-- Nice comments --><>");
        let read1 = BufReader::new(str1);

        let mut lexer = Lexer::from_reader(read1);

        assert_eq!(Some(Comment(~" Nice comments ")), lexer.pull());
        assert_eq!(Some(LessBracket), lexer.pull());
        assert_eq!(Some(GreaterBracket), lexer.pull());
    }

    #[test]
    fn test_element(){
        let str1  = bytes!("<elem attr='something &ref;bla&#35;&#x2A;'></elem><br/>");
        let read1 = BufReader::new(str1);

        let mut lexer = Lexer::from_reader(read1);
        assert_eq!(Some(LessBracket),           lexer.pull());
        assert_eq!(Some(NameToken(~"elem")),    lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),      lexer.pull());
        assert_eq!(Some(NameToken(~"attr")),    lexer.pull());
        assert_eq!(Some(Eq),                    lexer.pull());
        assert_eq!(Some(Quote),                 lexer.pull());
        assert_eq!(Some(Text(~"something ")),   lexer.pull());
        assert_eq!(Some(Ref(~"ref")),           lexer.pull());
        assert_eq!(Some(Text(~"bla")),          lexer.pull());
        assert_eq!(Some(CharRef('#')),          lexer.pull());
        assert_eq!(Some(CharRef('*')),          lexer.pull());
        assert_eq!(Some(Quote),                 lexer.pull());
        assert_eq!(Some(GreaterBracket),        lexer.pull());
        assert_eq!(Some(CloseTag),              lexer.pull());
        assert_eq!(Some(NameToken(~"elem")),    lexer.pull());
        assert_eq!(Some(GreaterBracket),        lexer.pull());
        assert_eq!(Some(LessBracket),           lexer.pull());
        assert_eq!(Some(NameToken(~"br")),      lexer.pull());
        assert_eq!(Some(EmptyTag),              lexer.pull());
    }

    #[test]
    fn test_qname(){
        let str1 = bytes!("<book:elem book:iso= '11231A'");
        let read1 = BufReader::new(str1);

        let mut lexer = Lexer::from_reader(read1);
        assert_eq!(Some(LessBracket),                   lexer.pull());
        assert_eq!(Some(QNameToken(~"book",~"elem")),   lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(QNameToken(~"book",~"iso")),    lexer.pull());
        assert_eq!(Some(Eq),                            lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),              lexer.pull());
        assert_eq!(Some(Quote),                         lexer.pull());
        assert_eq!(Some(Text(~"11231A")),               lexer.pull());
        assert_eq!(Some(Quote),                         lexer.pull());
    }

    #[test]
    fn test_quote_terminating(){
        let str1 = bytes!("<el name=\"test");
        let read = BufReader::new(str1);
        let mut lexer = Lexer::from_reader(read);

        assert_eq!(Some(LessBracket),               lexer.pull());
        assert_eq!(Some(NameToken(~"el")),          lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(NameToken(~"name")),        lexer.pull());
        assert_eq!(Some(Eq),                        lexer.pull());
        assert_eq!(Some(Quote),                     lexer.pull());
        assert_eq!(Some(Text(~"test")),             lexer.pull());
    }

    #[test]
    fn test_doctype() {
        let str1 = bytes!("<!DOCTYPE stuff SYSTEM 'pubid' [
        <!ELEMENT (name|(#PCDATA,%div;))?+*>
        ]>");
        let read = BufReader::new(str1);
        let mut lexer =             Lexer::from_reader(read);

        assert_eq!(Some(DoctypeStart),              lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(NameToken(~"stuff")),       lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(NameToken(~"SYSTEM")),      lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(QuotedString(~"pubid")),    lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(LeftSqBracket),             lexer.pull());
        assert_eq!(Some(WhiteSpace(~"\n        ")), lexer.pull());
        assert_eq!(Some(ElementType),               lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(LeftParen),                 lexer.pull());
        assert_eq!(Some(NameToken(~"name")),        lexer.pull());
        assert_eq!(Some(Pipe),                      lexer.pull());
        assert_eq!(Some(LeftParen),                 lexer.pull());
        assert_eq!(Some(PCDataDecl),                lexer.pull());
        assert_eq!(Some(Comma),                     lexer.pull());
        assert_eq!(Some(ParRef(~"div")),            lexer.pull());
        assert_eq!(Some(RightParen),                lexer.pull());
        assert_eq!(Some(RightParen),                lexer.pull());
        assert_eq!(Some(QuestionMark),              lexer.pull());
        assert_eq!(Some(Plus),                      lexer.pull());
        assert_eq!(Some(Star),                      lexer.pull());
        assert_eq!(Some(GreaterBracket),            lexer.pull());
        assert_eq!(Some(WhiteSpace(~"\n        ")), lexer.pull());
        assert_eq!(Some(RightSqBracket),            lexer.pull());
        assert_eq!(Some(GreaterBracket),            lexer.pull());

        let str2 = bytes!("<!DOCTYPE PUBLIC [
        <!ENTITY % 'text%ent;&x;&#94;&#x7E;'>
        ]>");
        let read2 = BufReader::new(str2);
        lexer = Lexer::from_reader(read2);

        assert_eq!(Some(DoctypeStart),              lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(NameToken(~"PUBLIC")),      lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(LeftSqBracket),             lexer.pull());
        assert_eq!(Some(WhiteSpace(~"\n        ")), lexer.pull());
        assert_eq!(Some(EntityType),                lexer.pull());
        assert_eq!(InEntityType,                    lexer.state);
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(Percent),                   lexer.pull());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.pull());
        assert_eq!(Some(Quote),                     lexer.pull());
        assert_eq!(Some(Text(~"text")),             lexer.pull());
        assert_eq!(Some(ParRef(~"ent")),            lexer.pull());
        assert_eq!(Some(Ref(~"x")),                 lexer.pull());
        assert_eq!(Some(CharRef('^')),             lexer.pull());
        assert_eq!(Some(CharRef('~')),             lexer.pull());
        assert_eq!(Some(Quote),                     lexer.pull());
        assert_eq!(Some(GreaterBracket),            lexer.pull());
        assert_eq!(Some(WhiteSpace(~"\n        ")), lexer.pull());
    }

    #[test]
    fn test_multi_peek() {
        let str1 = bytes!("123");
        let read = BufReader::new(str1);
        let mut lexer =             Lexer::from_reader(read);

        assert_eq!(~"12",           lexer.peek_str(2u));
        assert_eq!(~"12",           lexer.peek_str(2u));
        assert_eq!(~"1",            lexer.read_str(1u));
        assert_eq!(~"23",           lexer.peek_str(2u));
        assert_eq!(~"23",           lexer.peek_str(2u));
    }

    #[test]
    fn test_peek_restricted() {
        let str1 = bytes!("1\x0123");
        let r1 = BufReader::new(str1);
        let mut lexer =             Lexer::from_reader(r1);

        assert_eq!(~"1",            lexer.peek_str(2u));
        assert_eq!(~"12",           lexer.peek_str(3u));
    }

    #[test]
    /// This method tests buffer to ensure that adding characters
    /// into it will not cause premature end of line.
    /// If lexer takes six characters and then peeks six
    /// character the reader will be moved, and those characters
    /// added to peek buffer.
    /// If reader doesn't check peek buffer before the reader field
    /// it will cause premature end of file
    fn test_premature_eof() {
        let str1 = bytes!("012345");
        let read = BufReader::new(str1);
        let mut lexer =         Lexer::from_reader(read);

        lexer.peek_str(6u);
        assert_eq!(~"012345",       lexer.read_str(6u));
    }

    #[test]
    fn test_whitespace() {
        let str1 = bytes!("  \t\n  a");
        let read = BufReader::new(str1);
        let mut lexer =         Lexer::from_reader(read);

        assert_eq!(Some(WhiteSpace(~"  \t\n  ")),      lexer.pull());
        assert_eq!(6u,                                 lexer.col);
        assert_eq!(1u,                                 lexer.line);
        assert_eq!(Some(NameToken(~"a")),              lexer.pull());
    }

    #[test]
    fn test_peek_str() {
        let str1 = bytes!("as");
        let read = BufReader::new(str1);
        let mut lexer = Lexer::from_reader(read);

        assert_eq!(~"as",               lexer.peek_str(2u));
        assert_eq!(0u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);
        assert_eq!(Some(Char('a')),     lexer.read_chr());
        assert_eq!(1u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);
        assert_eq!(~"s",                lexer.read_str(1u));
        assert_eq!(2u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);
    }

    #[test]
    fn test_eof() {
        let str1 = bytes!("a");
        let read = BufReader::new(str1);
        let mut lexer = Lexer::from_reader(read);

        assert_eq!(Some(Char('a')),     lexer.read_chr());
        assert_eq!(None,                lexer.read_chr())
    }

    #[test]
    fn test_read_until() {
        let str1 = bytes!("aaaab");
        let read = BufReader::new(str1);
        let mut lexer = Lexer::from_reader(read);

        let result = lexer.read_while_fn(|c|{
            match c {
                Some(Char('a')) => true,
                _ => false
            }
        });

        assert_eq!(~"aaaa",      result);
        assert_eq!(1,            lexer.line);
        assert_eq!(4,            lexer.col);
        assert_eq!(~"b",         lexer.read_str(1u));
        assert_eq!(1,            lexer.line);
        assert_eq!(5,            lexer.col);
    }

    #[test]
    /// Tests if it reads a restricted character
    /// and recognize a char correctly
    fn test_restricted_char() {
        let str1 = bytes!("\x01\x04\x08a\x0B\x0Cb\x0E\x10\x1Fc\x7F\x80\x84d\x86\x90\x9F");
        let read = BufReader::new(str1);
        let mut lexer = Lexer::from_reader(read);

        assert_eq!(Some(RestrictedChar('\x01')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x04')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x08')),      lexer.read_chr());
        assert_eq!(Some(Char('a')),                   lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x0B')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x0C')),      lexer.read_chr());
        assert_eq!(Some(Char('b')),                   lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x0E')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x10')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x1F')),      lexer.read_chr());
        assert_eq!(Some(Char('c')),                   lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x7F')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x80')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x84')),      lexer.read_chr());
        assert_eq!(Some(Char('d')),                   lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x86')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x90')),      lexer.read_chr());
        assert_eq!(Some(RestrictedChar('\x9F')),      lexer.read_chr());
    }

    #[test]
    fn test_read_newline() {
        let str1  = bytes!("a\r\nt");
        let read1 = BufReader::new(str1);
        let mut lexer = Lexer::from_reader(read1);

        assert_eq!(Some(Char('a')), lexer.read_chr());
        assert_eq!(1,               lexer.line);
        assert_eq!(1,               lexer.col);
        assert_eq!(Some(Char('\n')),lexer.read_chr());
        assert_eq!(2,               lexer.line);
        assert_eq!(0,               lexer.col);
        assert_eq!(Some(Char('t')), lexer.read_chr());
        assert_eq!(2,               lexer.line);
        assert_eq!(1,               lexer.col);

        let str2  = bytes!("a\rt");
        let read2 = BufReader::new(str2);
        lexer = Lexer::from_reader(read2);

        assert_eq!(Some(Char('a')), lexer.read_chr());
        assert_eq!(1,               lexer.line);
        assert_eq!(1,               lexer.col);
        assert_eq!(Some(Char('\n')),lexer.read_chr());
        assert_eq!(2,               lexer.line);
        assert_eq!(0,               lexer.col);
        assert_eq!(Some(Char('t')), lexer.read_chr());
        assert_eq!(2,               lexer.line);
        assert_eq!(1,               lexer.col);

        let str3  = bytes!("a\r\x85t");
        let read3 = BufReader::new(str3);
        lexer = Lexer::from_reader(read3);

        assert_eq!(Some(Char('a')),     lexer.read_chr());
        assert_eq!(1,                   lexer.line);
        assert_eq!(1,                   lexer.col);
        assert_eq!(Some(Char('\n')),    lexer.read_chr());
        assert_eq!(2,                   lexer.line);
        assert_eq!(0,                   lexer.col);
        assert_eq!(Some(Char('t')),     lexer.read_chr());
        assert_eq!(2,                   lexer.line);
        assert_eq!(1,                   lexer.col);

        let str4  = bytes!("a\x85t");
        let read4 = BufReader::new(str4);
        let mut lexer = Lexer::from_reader(read4);

        assert_eq!(Some(Char('a')),     lexer.read_chr());
        assert_eq!(1,                   lexer.line);
        assert_eq!(1,                   lexer.col);
        assert_eq!(Some(Char('\n')),    lexer.read_chr());
        assert_eq!(2,                   lexer.line);
        assert_eq!(0,                   lexer.col);
        assert_eq!(Some(Char('t')),     lexer.read_chr());
        assert_eq!(2,                   lexer.line);
        assert_eq!(1,                   lexer.col);

        let str5  = bytes!("a\u2028t");
        let read5 = BufReader::new(str5);
        let mut lexer = Lexer::from_reader(read5);

        assert_eq!(Some(Char('a')), lexer.read_chr());
        assert_eq!(1,               lexer.line);
        assert_eq!(1,               lexer.col);
        assert_eq!(Some(Char('\n')),lexer.read_chr());
        assert_eq!(2,               lexer.line);
        assert_eq!(0,               lexer.col);
        assert_eq!(Some(Char('t')), lexer.read_chr());
        assert_eq!(2,               lexer.line);
        assert_eq!(1,               lexer.col);
    }

    #[test]
    fn test_rewind(){
        let str1 = bytes!("abcd");
        let read = BufReader::new(str1);

        let mut lexer = Lexer::from_reader(read);
        let read = lexer.read_str(3);
        assert_eq!(~"abc", read);

        lexer.rewind(0,1, read);

        let after = lexer.read_str(3);
        assert_eq!(~"abc", after);
    }

}
