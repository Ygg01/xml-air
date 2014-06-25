use std::ascii::StrAsciiExt;
use std::io::{Reader, Buffer};
use std::char::from_u32;
use std::str::from_char;
use std::num::from_str_radix;
use std::string::String;

use util::{is_whitespace, is_name_start, is_name_char};
use util::{XmlError, ErrKind, PopShiftShim};
use util::{is_hex_digit, is_digit};
use util::{is_restricted_char, clean_restricted, is_char};
use util::{RestrictedCharError,MinMinInComment,PrematureEOF,NonDigitError};
use util::{NumParsingError,CharParsingError,IllegalChar,UnknownToken};



pub type XmlResult = Result<XmlToken,(XmlError, Option<XmlToken>)>;


#[deriving(PartialEq, Eq, Show, Clone)]
pub enum XmlToken {
    /// Processing instruction token
    /// First string represents target and the second string
    /// represents text
    PI(String, String),
    /// Start of PI block '<?'
    PrologStart,
    /// End of PI block '?>'
    PrologEnd,
    /// Error token
    ErrorToken(String),
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
    NameToken(String),
    /// Qualified name token
    /// first string is prefix, second is local-part
    QNameToken(String,String),
    /// NMToken
    NMToken(String),
    /// Various characters
    Text(String),
    /// Whitespace
    WhiteSpace(String),
    /// CData token with inner structure
    CData(String),
    /// Start of Doctype block '<!DOCTYPE'
    DoctypeStart,
    /// Start of inner IGNORE/INCLUDE block
    /// uses symbol '<!['
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
    Comment(String),
    /// Encoded char or '&#'
    CharRef(char),
    /// Attribute reference
    Ref(String),
    /// Parsed entity reference
    ParRef(String),
    /// Single or double quoted string
    /// e.g. 'example' or "example"
    QuotedString(String),
    /// Quote token
    Quote,
    /// Symbol #FIXED
    FixedDecl,
    /// Symbol #PCDATA
    PCDataDecl,
    /// Symbol #REQUIRED
    RequiredDecl,
    /// Symbol #IMPLIED
    ImpliedDecl,
    /// FIXME token is temporarily
    FIXME
}

#[deriving(PartialEq, Eq, Show)]
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
        if is_restricted_char(&chr) {
            RestrictedChar(chr)
        } else if is_char(&chr) {
            Char(chr)
        } else {
            RestrictedChar(chr)
        }
    }
}
#[deriving(PartialEq, Eq, Show)]
enum State {
    OutsideTag,
    // Attlist takes quote, because attributes are mixed content and to
    // correctly display it, it treats each Quote as a special symbol
    // so for example "text&ref;" becomes `Quote Text(text) Ref(ref) Quote`
    Attlist(Quotes),
    /// Similar as above but for Attlist in DTD
    TypeAttlist(Quotes),
    EntityList(Quotes),
    InDoctype,
    InElementType,
    InEntityType,
    InNotationType,
    InAttlistType,
    InExternalId,
    InProlog,
    InStartTag,
    InternalSubset,
    Doctype
}
#[deriving(PartialEq, Eq, Show)]
enum Quotes {
    Single,
    Double
}

pub struct Checkpoint {
    col: u64,
    line: u64
}

impl Quotes {
    pub fn to_char(&self) -> char {
        match *self {
            Single => '\'',
            Double => '"'
        }
    }

    pub fn from_str(quote: String) -> Quotes {
        if quote == String::from_str("'") {
            Single
        } else if quote == String::from_str("\"") {
            Double
        } else {
            println!(" Expected single (`'`) or double quotes (`\"`) got `{}` instead ", quote);
            fail!("fail");
        }
    }

    pub fn from_chr(quote: &char) -> Quotes {
        Quotes::from_str(from_char(*quote))
    }
}

pub struct Lexer<'r, R> {
    pub line: u64,
    pub col: u64,
    token: Option<XmlToken>,
    err: Option<XmlError>,
    checkpoint: Option<Checkpoint>,
    state: State,
    // TODO change these to borrowed str
    peek_buf: String,
    buf: String,
    source: &'r mut R
}

// Struct to help with the Iterator pattern emulating Rust native libraries
pub struct TokenIterator <'b,'r, R> {
    iter: &'b mut Lexer<'r, R>
}

// The problem seems to be here
impl<'b,'r, R: Reader+Buffer> Iterator<XmlResult> for TokenIterator<'b, 'r, R> {
    fn next(&mut self) -> Option<XmlResult> {
        self.iter.pull()
    }
}

impl<'iter, 'r, R: Reader+Buffer> Lexer<'r, R> {
    pub fn tokens(&'iter mut self) -> TokenIterator<'iter,'r,R>{
        TokenIterator{ iter: self}
    }
}

impl<'r, R: Reader+Buffer> Lexer<'r, R> {
    /// Constructs a new `Lexer` from data given.
    /// Parameter `data` represents source for parsing,
    /// that must implement Reader and Buffer traits.
    /// Example
    /// ```rust
    ///    let bytes = bytes!("<an:elem />");
    ///    let lexer = xml::Lexer::from_reader(BufReader::new(bytes));
    /// ````
    pub fn from_reader(data : &'r mut R) -> Lexer<'r, R> {
        Lexer {
            line: 1,
            col: 0,
            peek_buf: String::new(),
            checkpoint: None,
            state: OutsideTag,
            buf: String::new(),
            err: None,
            token: None,
            source: data
        }
    }
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
    pub fn pull(&mut self) -> Option<XmlResult> {
        self.buf = String::new();

        let read_chr = match self.read_chr() {
            Some(a) => a,
            None => return None
        };

        match read_chr {
            Char(a) => self.buf.push_char(a),
            _ => {}
        }

        match read_chr {
            RestrictedChar(_) => {
                self.handle_errors(RestrictedCharError, None);
            },
            Char(chr) if is_whitespace(&chr)
                      => self.get_whitespace_token(),
            Char(a) => self.parse_char(&a)
        };

        let result = match self.token {
            Some(ref token) => Ok(token.clone()),
            None => {
                //FIXME: Do real error checking
                let err = XmlError {
                        line: 0,
                        col: 0,
                        msg: String::new(),
                        mark: None
                };
                Err((err, None))
            }
        };

        Some(result)

    }

    fn parse_char(&mut self, c: &char ) {
        //FIXME: This must be removed and emitting token
        // will be per case basis (possible macro!)
        self.token = match self.state {
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
                    &'<'    => {
                                let tok = self.get_left_bracket_token();
                                if tok == Some(PrologStart) {
                                    self.state = InProlog;
                                }
                                tok
                            }
                    &'/'    => self.get_empty_tag_token(),
                    _       => Some(FIXME)
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
            TypeAttlist(quotes) => {
                match c {
                    &'&'    => self.get_ref_token(),
                    &'<'    => self.get_attl_error_token(),
                    &'\'' | &'"' if *c == quotes.to_char()
                            => {
                                self.state = InAttlistType;
                                self.get_spec_quote()
                            },
                    _       => self.get_attl_text(&quotes.to_char())
                }
            }
            InDoctype => {
                match c {
                    &'<'    => {
                                let tok = self.get_left_bracket_token();
                                if tok == Some(PrologStart) {
                                    self.state = InProlog;
                                }
                                tok
                            },
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
                    &']'    =>  self.get_doctype_end_token(),
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
                        } else if res == Some(NotationType) {
                            self.state = InNotationType;
                        } else if res == Some(AttlistType) {
                            self.state = InAttlistType;
                        } else if res == Some(PrologStart) {
                            self.state = InProlog;
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
            },
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
                    &'#' => self.get_hash_token(),
                    // TODO Change to proper error
                    _     => {
                        Some(FIXME)
                    }
                }
            },
            InNotationType => {
                match c {
                    &'>' => {
                        self.state = InternalSubset;
                        self.get_right_bracket_token()
                    },
                    chr if is_name_start(chr) => {
                        self.get_name_token()
                    },
                    &'\''
                    | &'"'  => self.get_quote_token(),
                    _    => {
                        Some(FIXME)
                    }
                }
            },
            InAttlistType => {
                match c {
                    &'>' => {
                        self.state = InternalSubset;
                        self.get_right_bracket_token()
                    },
                    &'(' => self.get_paren_left_token(),
                    &')' => self.get_paren_right_token(),
                    &'|' => self.get_pipe_token(),
                    &'#' => self.get_hash_token(),
                    chr if is_name_char(chr) =>  self.get_name_token(),
                    quote if quote == &'\'' || quote == &'"' => {
                        self.state = TypeAttlist(Quotes::from_chr(quote));
                        self.get_spec_quote()
                    },
                    _ => {
                        Some(FIXME)
                    }
                }
            },
            InEntityType => {
                match c {
                    chr if is_name_start(chr)
                         => {
                            let res = self.get_name_token();
                            if res == Some(NameToken(String::from_str("PUBLIC"))) ||
                               res == Some(NameToken(String::from_str("SYSTEM"))) {
                                self.state = InExternalId;
                            }
                            res
                    },
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
                        Some(FIXME)
                    }
                }
            },
            InExternalId => {
                match c {
                    chr if is_name_start(chr)
                              => self.get_name_token(),
                    &'\''
                    | &'"' => self.get_quote_token(),
                    &'>' => {
                        self.state = InDoctype;
                        self.get_right_bracket_token()
                    },
                    _     => {
                        Some(FIXME)
                    }
                }
            }
            EntityList(quotes) => {
                match c {
                    &'&'    => self.get_ref_token(),
                    &'%'    => self.get_peref_token(),
                    &'\''
                    | &'"' if *c == quotes.to_char()
                            => {
                                self.state = InEntityType;
                                self.get_spec_quote()
                            },
                    _       => self.get_ent_text(&quotes.to_char())
                }
            },
            OutsideTag => {
                match c {
                    chr if is_name_start(chr) || is_name_char(chr)
                              => self.get_name_token(),
                    &'<'  => {
                        let tok = self.get_left_bracket_token();
                        if tok == Some(LessBracket) {
                            self.state = InStartTag
                        } else if tok == Some(DoctypeStart) {
                            self.state = InDoctype;
                        }
                        tok
                    },
                    &'&'  => self.get_ref_token(),
                    &'%'  => self.get_peref_token(),
                    &'>'  => self.get_right_bracket_token(),
                    &'?'  => self.get_pi_end_token(),
                    &'/'  => self.get_empty_tag_token(),
                    &'='  => self.get_equal_token(),
                    &'\''
                    | &'"'  => self.get_quote_token(),
                    _  => self.get_text_token(),
                }
            },
            InProlog
            | Doctype => {
                match c {
                    chr if is_name_start(chr)
                              => self.get_name_token(),
                    chr if is_name_char(chr)
                              => self.get_name_token(),
                    &'<'  => self.get_left_bracket_token(),
                    &'&'  => self.get_ref_token(),
                    &'%'  => self.get_peref_token(),
                    &'>'  => self.get_right_bracket_token(),
                    &'?'  => self.get_pi_end_token(),
                    &'/'  => self.get_empty_tag_token(),
                    &'='  => self.get_equal_token(),
                    &'\''
                    | &'"'  => self.get_quote_token(),
                    _  => self.get_text_token(),
                }
            }
        };
    }


    /// This method reads a string of given length skipping over any
    /// restricted character and adding an error for each such
    /// character encountered.
    ///
    /// Restricted characters are *not included* into the output
    /// string.
    pub fn read_str(&mut self, len: u64) -> String {
        clean_restricted(self.read_raw_str(len))
    }

    #[inline]
    fn rewind(&mut self, peeked: String) {
        match self.checkpoint {
            Some(cp) => {
                self.rewind_to(peeked, cp);
            },
            _ => {}
        }
    }

    #[inline]
    fn rewind_to(&mut self, peeked: String, cp: Checkpoint) {
        self.col  = cp.col;
        self.line = cp.line;

        for c in peeked.as_slice().chars().rev(){
            self.peek_buf.push_char(c);
        }
    }

    fn save_checkpoint (&mut self) -> Checkpoint {
        let checkpoint  = Checkpoint {
            col: self.col,
            line: self.line
        };
        self.checkpoint = Some(checkpoint);
        checkpoint
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
            println!("read char {}", read_chr);

            match read_chr {
                Ok(a) => chr = a,
                // If an error occurs we abort further iterations
                Err(_) => {
                    return None
                }
            }
        } else {
            chr = self.peek_buf.pop_char_shim().unwrap();
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
        self.line += 1;
        self.col = 0;

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
        self.col += 1;
        Character::from_char(c)
    }

    /// This method reads a string of given length, adding any
    /// restricted char  into the error section.
    /// Restricted character are *included* into the output string
    fn read_raw_str(&mut self, len: u64) -> String {
        let mut raw_str = String::new();
        let mut eof = false;
        let mut l = 0;

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
        raw_str.into_owned()
    }

    //TODO Doc
    // TODO rewrite this function to take a Filter trait, which will
    // deal with various queries behind screen
    fn read_while_fn(&mut self, fn_while: |Option<Character>|-> bool )
                     -> String {
        let mut col = self.col;
        let mut line = self.line;
        let mut ret_str = String::new();
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

        ret_str.into_owned()
    }

    fn read_until_peek(&mut self, peek_look: String) -> String {
        let mut peek_found = false;
        let mut result = String::new();
        let peek_len = (peek_look.as_slice().char_len() - 1) as u64;

        while !peek_found {
            let pre_cp = self.save_checkpoint();
            let extracted_char = self.read_chr();

            match extracted_char {
                None          => {},
                Some(Char(a)) => {
                    self.save_checkpoint();
                    let mut rew = String::new();
                    let mut peek = String::from_char(1,a);

                    if peek_len > 0 {
                        rew = String::from_str(self.read_str(peek_len).as_slice());
                        peek.push_str(rew.clone().as_slice());
                    }

                    if peek == peek_look {
                        peek_found = true;
                    } else {
                        result.push_char(a);
                    }

                    if !rew.len() > 0{
                        if peek_found {
                            self.rewind_to(peek.into_owned(), pre_cp);
                        } else {
                            self.rewind(rew.into_owned());
                        }
                    }
                },
                Some(RestrictedChar(_)) => {
                    self.handle_errors(RestrictedCharError, None);
                }
            }
        }
        result.into_owned()
    }


    fn handle_errors(&self, kind: ErrKind,
                     pass: Option<XmlToken>)
                      {
        if kind == IllegalChar  {
            //println!("ERROR!");
        }
    }

    fn process_namechars(&mut self) -> String {
        self.read_while_fn( |val| {
            match val {
                Some(Char(v))             => is_name_char(&v),
                _ => false
            }
        })
    }

    fn process_name(&mut self) -> String {
        let mut result = String::new();
        match self.read_chr() {
            Some(Char(a)) if is_name_start(&a) => {
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
        result.push_str(self.process_namechars().as_slice());
        result.into_owned()
    }

    /// It will attempt to consume all digits until it reaches a non-digit
    /// numeral. If value `is_hex` is true it will consume all hexadecimal
    /// digits including values 0-9 a-f or A-F. If value `is_hex` is false it
    /// will only consume decimal digits
    fn process_digits(&mut self, is_hex: &bool) -> String {
         self.read_while_fn( |val| {
                match val {
                    Some(Char(v)) => {
                        if *is_hex  {
                            is_hex_digit(&v)
                        } else {
                            is_digit(&v)
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
    fn get_whitespace_token(&mut self) {

        let ws = self.read_while_fn( |val| {
            match val {
                Some(Char(v))             => is_whitespace(&v),
                _   => false
            }
        });

        self.buf.push_str(ws.as_slice());
        self.token = Some(WhiteSpace(self.buf.clone().into_owned()));
    }

    /// If we find a name start character this method
    /// consumes all name token until it reaches a non-name
    /// character.
    fn get_name_token(&mut self) -> Option<XmlToken> {
        assert_eq!(1, self.buf.len());


        let result;

        let temp = self.process_namechars();
        self.buf.push_str(temp.as_slice());

        let buf_slice = self.buf.as_slice();
        let start_char = buf_slice.char_at(0);

        if is_name_start(&start_char) {
            result = Some(NameToken(self.buf.clone()));
        } else if is_name_char(&start_char) {
            result = Some(NMToken(self.buf.clone()));
        } else {
            result = Some(FIXME);
        }

        result
    }

    /// If we find a name start character this method consumes
    /// all name characters until it reaches a non-name character.
    /// This method also handles qualfied names, as defined in
    /// [Namespace specification](http://www.w3.org/TR/xml-names11/)
    fn get_qname_token(&mut self) -> Option<XmlToken> {
        let result;
        let namechars = self.process_namechars();

        self.buf.push_str(namechars.as_slice());
        if self.buf.as_slice().contains_char(':'){
            if self.buf.as_slice().char_at(0) == ':'
            || self.buf.as_slice().char_at(self.buf.len()-1) == ':'{
                result = Some(NameToken(self.buf.clone()));
            } else {
                let split_name: Vec<&str> = self.buf.as_slice().split(':').collect();

                if split_name.len() == 2 {
                    let ns = (*split_name.get(0)).into_string();
                    let name = (*split_name.get(1)).into_string();
                    result = Some(
                        QNameToken(ns, name)
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

    fn get_left_bracket_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<",   self.buf.as_slice());

        let result;
        self.save_checkpoint();
        let chr = self.read_chr();
        let rew;

        match chr {
            Some(Char(a)) => {
                self.buf.push_char(a);
                rew = from_char(a);
            }
            Some(RestrictedChar(_)) => {
                return Some(FIXME);
            },
            None => {
                return Some(FIXME);
            }
        }

        if self.buf.as_slice() == "</"{
            result = self.get_close_tag_token();
        } else if self.buf.as_slice() == "<?" {
            result = self.get_pi_token();
        } else if self.buf.as_slice() == "<!" {
            result = self.get_amp_excl();
        } else {
            self.rewind(rew);
            result = Some(LessBracket);
        }

        result
    }

    fn get_amp_excl(&mut self) -> Option<XmlToken> {
        assert_eq!("<!",   self.buf.as_slice());
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
            },
            Some(Char('N')) => {
                self.buf.push_char('N');
                self.get_notation_token()
            },
            Some(Char('A')) => {
                self.buf.push_char('A');
                self.get_attlist_token()
            }
            None => Some(Text("<!".into_string())),
            _ => Some(Text("NON IMPLEMENTED".into_string()))
        };

        result
    }

    fn get_cdata_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<![",       self.buf.as_slice());

        self.save_checkpoint();
        let cdata = self.read_str(6);
        let result;

        if cdata == "CDATA[".into_string() {
            let text = self.read_until_peek("]]>".into_string());
            self.read_str(3);

            result = Some(CData(text));
        } else {
            self.rewind(cdata);

            result = Some(DoctypeOpen);
        }
        result
    }

    fn get_doctype_start_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<!D",       self.buf.as_slice());
        self.save_checkpoint();
        let peeked_str  = self.read_str(6);
        let result;

        if peeked_str == "OCTYPE".into_string() {
            result = Some(DoctypeStart);
        } else {
            self.rewind(peeked_str);
            result = Some(FIXME);
        }
        result
    }

    fn get_attlist_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<!A",       self.buf.as_slice());
        self.save_checkpoint();

        let peeked_str = self.read_str(6);
        let result;

        if peeked_str == "TTLIST".into_string() {
            result = Some(AttlistType);
        } else {
            self.rewind(peeked_str);
            result = Some(FIXME);
        }
        result
    }

    #[inline(always)]
    fn get_equal_token(&mut self) -> Option<XmlToken> {
        assert_eq!("=",       self.buf.as_slice());
        Some(Eq)
    }

    fn get_ref_token(&mut self) -> Option<XmlToken> {
        assert_eq!("&",  self.buf.as_slice());
        self.save_checkpoint();
        let chr = self.read_chr();

        let token = match chr {
            Some(Char('#')) => {
                self.buf.push_char('#');
                self.get_char_ref_token()
            },
            Some(Char(a)) => {
                self.rewind(from_char(a));
                self.get_entity_ref_token(true)
            },
            Some(RestrictedChar(a)) => {
                self.rewind(from_char(a));
                self.handle_errors(
                    RestrictedCharError,
                    None
                );
                Some(Text("&".into_string()))
            },
            None => {
                Some(FIXME)
            }
        };
        token
    }

    fn get_peref_token(&mut self) -> Option<XmlToken> {
        assert_eq!("%",       self.buf.as_slice());
        self.get_entity_ref_token(false)
    }

    fn get_char_ref_token(&mut self) -> Option<XmlToken> {
        assert_eq!("&#",       self.buf.as_slice());
        self.save_checkpoint();
        let next_char = self.read_chr();


        let radix;
        match next_char {
            Some(Char('x')) => {
                radix = 16;
            },
            Some(Char(a)) if (is_digit(&a)) => {
                self.rewind(from_char(a));
                radix = 10;
            },
            Some(Char(_))
            | Some(RestrictedChar(_)) => {
                return Some(FIXME);
            },
            None => {
                return Some(FIXME);
            }
        }

        let is_radix = radix == 16;
        let char_ref = self.process_digits(&is_radix);

        self.save_checkpoint();
        let read_chr = self.read_chr();

        match read_chr {
            Some(Char(';')) => {
            },
            Some(Char(a))
            | Some(RestrictedChar(a)) => {
                self.rewind(from_char(a));
            }
            _ => {
                return Some(ErrorToken(self.buf.clone()));
            }
        }

        let parse_char = from_str_radix::<u64>(char_ref.as_slice(),radix);

        match parse_char {
            Some(a) => {
                let ref_char = from_u32(a as u32);

                match ref_char {
                    Some(a) => {
                         Some(CharRef(a))
                    }
                    _ => {
                        Some(FIXME)
                    }
                }
            },
            None => {
                Some(FIXME)
            }
        }
    }

    fn get_entity_ref_token(&mut self, is_ent: bool) -> Option<XmlToken> {

        let ref_name = self.process_name();

        self.save_checkpoint();
        let expect_semi = self.read_chr();

        let result = match expect_semi {
            Some(Char(';')) => {
                if is_ent {
                    Some(Ref(ref_name))
                } else {
                    Some(ParRef(ref_name))
                }
            },
            Some(Char(a)) => {
                self.rewind(from_char(a));
                self.handle_errors(IllegalChar, None);
                if is_ent {
                    Some(Ref(ref_name))
                } else {
                    Some(ParRef(ref_name))
                }
            },
            Some(RestrictedChar(a)) => {
                self.rewind(from_char(a));
                self.handle_errors(IllegalChar, None);
                if is_ent {
                    Some(Ref(ref_name))
                } else {
                    Some(ParRef(ref_name))
                }
            },
            None => {
                Some(FIXME)
            }
        };
        result
    }

    #[inline(always)]
    fn get_sqbracket_left_token(&mut self) -> Option<XmlToken> {
        assert_eq!("[",       self.buf.as_slice());
        Some(LeftSqBracket)
    }

    fn get_doctype_end_token(&mut self) -> Option<XmlToken> {
        assert_eq!("]",        self.buf.as_slice());
        self.save_checkpoint();
        let rew  = self.read_str(2);

        if rew == "]>".into_string() {
            Some(DoctypeClose)
        } else {
            self.rewind(rew);
            Some(RightSqBracket)
        }
    }

    #[inline(always)]
    fn get_sqbracket_right_token(&mut self) -> Option<XmlToken> {
        assert_eq!("]",       self.buf.as_slice());
        Some(RightSqBracket)
    }

    #[inline(always)]
    fn get_paren_left_token(&mut self) -> Option<XmlToken> {
        assert_eq!("(",       self.buf.as_slice());
        Some(LeftParen)
    }

    #[inline(always)]
    fn get_paren_right_token(&mut self) -> Option<XmlToken> {
        assert_eq!(")",       self.buf.as_slice());
        Some(RightParen)
    }

    #[inline(always)]
    fn get_percent_token(&mut self) -> Option<XmlToken> {
        assert_eq!("%",       self.buf.as_slice());
        Some(Percent)
    }

    fn get_hash_token(&mut self) -> Option<XmlToken> {
        assert_eq!("#",    self.buf.as_slice());
        self.save_checkpoint();
        let mut rew = String::from_owned_str(self.read_str(5));
        let result;

        if rew.as_slice() == "FIXED" {
            result = Some(FixedDecl);
        } else if rew.as_slice() == "PCDAT" {

            rew.push_str(self.read_str(1).as_slice());
            if rew.as_slice() == "PCDATA" {
                result = Some(PCDataDecl);
            } else {
                result = Some(ErrorToken("#".into_string()));
            }
        } else if rew.as_slice() ==  "IMPLI" {

            rew.push_str(self.read_str(2).as_slice());
            if rew.as_slice() == "IMPLIED" {
                result = Some(ImpliedDecl);
            } else {
                result = Some(ErrorToken("#".into_string()));
            }
        } else if rew.as_slice() == "REQUI" {

            rew.push_str(self.read_str(3).as_slice());
            if rew.as_slice() == "REQUIRED" {
                result = Some(RequiredDecl);
            } else {
                result = Some(ErrorToken("#".into_string()));
            }
        } else {
            result = Some(ErrorToken("#".into_string()));
        }

        match result {
            Some(ErrorToken(_)) => self.rewind(rew.into_owned()),
            _ => {}
        }

        result
    }

    fn get_entity_or_element_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<!E", self.buf.as_slice());

        let mut result = Some(Text("<!E".into_string()));
        self.save_checkpoint();
        let mut read = self.read_str(6);

        if read.as_slice() == "LEMENT" {
            result = Some(ElementType);
        } else {
            self.rewind(read);
        }

        read = self.read_str(5);

        if read.as_slice()  == "NTITY" {
            result = Some(EntityType);
        } else {
            self.rewind(read);
        }

        result
    }

    fn get_notation_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<!N", self.buf.as_slice());

        self.save_checkpoint();
        let result;

        let read = self.read_str(7);

        if read.as_slice()  == "OTATION" {
            result = Some(NotationType);
        } else {
            self.rewind(read);
            result = Some(Text("<!N".into_string()));
        }
        result
    }

    fn get_star_token(&mut self) -> Option<XmlToken> {
        assert_eq!("*",       self.buf.as_slice());
        Some(Star)
    }

    fn get_plus_token(&mut self) -> Option<XmlToken> {
        assert_eq!("+",       self.buf.as_slice());
        Some(Plus)
    }

    fn get_pipe_token(&mut self) -> Option<XmlToken> {
        assert_eq!("|",       self.buf.as_slice());
        Some(Pipe)
    }

    fn get_comma_token(&mut self) -> Option<XmlToken> {
        assert_eq!(",",       self.buf.as_slice());
        Some(Comma)
    }


    fn get_quote_token(&mut self) -> Option<XmlToken> {
        let quote = self.buf.clone();
        assert!(quote.as_slice() == "'" || quote.as_slice()  == "\"");

        Some(self.process_quotes(quote))
    }


    fn process_quotes(&mut self, quote: String) -> XmlToken {
        let text = self.read_until_peek(quote.clone());
        self.save_checkpoint();
        let peek = self.read_str(1);

        if peek != quote.clone() {
            self.rewind(peek);
            self.handle_errors(
                IllegalChar,
                Some(QuotedString(text.clone()))
            );
        }

        QuotedString(text.clone())
    }

    #[inline]
    fn get_spec_quote(&mut self) -> Option<XmlToken> {
        assert!(self.buf.as_slice() == "'"
                || self.buf.as_slice() == "\"");
        Some(Quote)
    }

    fn get_attl_text(&mut self, quote: &char) -> Option<XmlToken> {
        let text = self.read_while_fn( |val| {
            match val {
                Some(Char(a))  => (a != '<' && a != '&' && a != *quote),
                _ => false
            }
        });
        let result = self.buf.clone().append(text.as_slice());
        Some(Text(result))
    }

    fn get_ent_text(&mut self, quote: &char) -> Option<XmlToken> {
        let text = self.read_while_fn( |val| {
            match val {
                Some(Char(a))  => (a != '<' && a != '%' && a != *quote),
                _ => false
            }
        });
        let result = self.buf.clone().append(text.as_slice());
        Some(Text(result))
    }

    #[inline(always)]
    fn get_attl_error_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<", self.buf.as_slice());
        Some(FIXME)
    }

    fn get_text_token(&mut self) -> Option<XmlToken> {
        let mut peek = String::new();
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
                        peek.shift_char_shim();
                        peek.push_char(a);
                    }
                    if peek.as_slice() == "]]>" {
                        run_loop = false;
                        // if we found this, it means we already took `]]`
                        text.pop_char_shim();
                        text.pop_char_shim();
                    }

                    if run_loop {
                        text.push_char(a);
                    }
                }
            }

        }
        Some(Text(text.into_owned()))
    }

    fn get_pi_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<?",       self.buf.as_slice());

        // Process target name
        let target = self.process_name();
        let result;

        if target.as_slice().eq_ignore_ascii_case("xml") {
            result = Some(PrologStart);
        } else {
            // We skip a possible whitespace token
            // to get to text of PI
            self.get_whitespace_token();

            let text = self.read_until_peek("?>".into_string());
            self.read_str(2);
            result = Some(PI(target,text));
        }
        result
    }

    fn get_right_bracket_token(&mut self) -> Option<XmlToken> {
        assert_eq!(">", self.buf.as_slice());
        return Some(GreaterBracket)
    }

    fn get_comment_token(&mut self) -> Option<XmlToken> {
        assert_eq!("<!-", self.buf.as_slice());
        self.save_checkpoint();
        let rewind_str = self.read_str(1);

        if rewind_str.as_slice() == "-" {

            let text = self.process_comment();
            return Some(Comment(text))
        } else {

            self.rewind(rewind_str);
            return Some(ErrorToken("<!-".into_string()))
        }
    }

    fn process_comment(&mut self) -> String {
        self.save_checkpoint();
        let mut peek = self.read_str(3);
        let mut result = String::new();
        let mut found_end = false;

        while !found_end {

            if peek.as_slice().starts_with("--") && peek.as_slice() == "-->" {
                found_end = true;
            } else {
                if peek.as_slice().starts_with("--") && peek.as_slice() != "-->" {
                    self.handle_errors(
                        MinMinInComment,
                        Some(Comment(self.buf.clone()))
                    );
                }

                self.rewind(peek);
                match self.read_chr() {
                    None
                    | Some(RestrictedChar(_)) => {},
                    Some(Char(a)) => {
                        result.push_char(a)
                    }
                }
                self.save_checkpoint();
                peek = self.read_str(3);
            }
        }
        result.into_owned()
    }

    fn get_close_tag_token(&mut self) -> Option<XmlToken> {
        assert_eq!("</",  self.buf.as_slice());
        return Some(CloseTag)
    }

    fn get_empty_tag_token(&mut self) -> Option<XmlToken> {
        assert_eq!("/", self.buf.as_slice());

        let result;
        if self.read_str(1).as_slice() == ">" {
            result = Some(EmptyTag);
        } else {
            result = Some(ErrorToken("/".into_string()));
        }
        result
    }

    fn get_pi_end_token(&mut self) -> Option<XmlToken> {
        assert_eq!("?",   self.buf.as_slice());
        self.save_checkpoint();

        let chr = self.read_chr();
        let result = match chr {
            Some(Char('>')) => {
                Some(PrologEnd)
            },
            Some(Char(a)) => {
                self.rewind(a.to_str());
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

    #[inline(always)]
    fn get_question_mark_token(&mut self) -> Option<XmlToken> {
        assert_eq!("?", self.buf.as_slice());
        Some(QuestionMark)
    }
}

pub fn main() {

}

#[cfg(test)]
mod test {
    use super::{Lexer, Char};

    use std::io::BufReader;

    #[test]
    fn lexer_read_until() {
        let mut read = BufReader::new(b"aaaab");
        let mut lexer = Lexer::from_reader(&mut read);

        let result = lexer.read_while_fn(|c|{
            match c {
                Some(Char('a')) => true,
                _ => false
            }
        });

        assert_eq!("aaaa".into_string(),    result);
        assert_eq!(1,                       lexer.line);
        assert_eq!(4,                       lexer.col);
        assert_eq!("b".into_string(),       lexer.read_str(1));
        assert_eq!(1,                       lexer.line);
        assert_eq!(5,                       lexer.col);
    }

    #[test]
    fn test_rewind(){
        let mut read = BufReader::new(b"abcd");
        let mut lexer = Lexer::from_reader(&mut read);

        lexer.save_checkpoint();
        let read = lexer.read_str(3);
        assert_eq!("abc".into_string(), read);

        lexer.rewind(read);

        let after = lexer.read_str(3);
        assert_eq!("abc".into_string(), after);
    }

}
