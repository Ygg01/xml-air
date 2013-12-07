use std::ascii::StrAsciiExt;
use std::io::{Reader, Buffer};
use std::char::from_u32;
use std::num::from_str_radix;

use util::{XmlError, is_whitespace, is_name_start, is_name_char};
use util::{is_char, is_restricted, clean_restricted};
use util::{ErrKind,UnreadableChar,UnexpectedChar};
use util::{RestrictedCharError,MinMinInComment,PrematureEOF};

mod util;

#[deriving(Eq, ToStr)]
pub enum XmlToken {
    ErrorToken(~str),   // Error token
    LessBracket,        // Symbol '<'
    GreaterBracket,     // Symbol '>'
    LeftSqBracket,      // Symbol '['
    RightSqBracket,     // Symbol ']'
    LeftParen,          // Symbol '('
    RightParen,         // Symbol ')'
    EqTok,              // Symbol '='
    Plus,               // Symbol '+'
    Pipe,               // Symbol '|'
    Star,               // Symbol '*'
    Amp,                // Symbol '&'
    QuestionMark,       // Symbol '?'
    Semicolon,          // Symbol ';'
    Percent,            // Percent '%'
    CloseTag,           // Symbol '</'
    EmptyTag,           // Symbol '/>'
    NameToken(~str),    // Tag name
    NMToken(~str),      // NMToken
    Text(~str),         // Various characters
    WhiteSpace(~str),   // Whitespace
    PI(~str),           // Processing instruction token
    PrologStart,        // Start of PI block '<?'
    PrologEnd,          // End of PI block '?>'
    CData(~str),        // CData token with inner structure
    DoctypeStart,       // Start of Doctype block '<!DOCTYPE'
    DoctypeOpen,        // Symbol '<!['
    DoctypeClose,       // Symbol ']]>'
    DoctypeEnd,         // End of Doctype block '!>'
    EntityType,         // Symbol <!ENTITY
    AttlistType,        // Symbol <!ATTLIST
    ElementType,        // Symbol <!ELEMENT
    NotationType,       // Symbol <!NOTATION
    Comment(~str),      // Comment token
    CharRef(char),      // Encoded char or '&#'
    QuotedString(~str), // Single or double quoted string
                        // e.g. 'example' or "example"
    RequiredDecl,       // Symbol #REQUIRED
    ImpliedDecl,        // Symbol #IMPLIED
    FixedDecl,          // Symbol #FIXED
    PCDataDecl,         // Symbol #PCDATA
    EndOfFile           // Denotes end of file
}

#[deriving(Eq,ToStr)]
pub enum Character {
    Char(char),
    RestrictedChar(char),
    EndFile
}

impl Character {
    pub fn extract_char(&self) -> Option<char> {
        match *self {
            Char(a)
            | RestrictedChar(a) => Some(a),
            EndFile             => None
        }
    }

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
            // If we encounter unknown character we replace it with
            // space.
            // TODO check if this is OK
            Char(' ')
        }
    }
}

pub enum ExpectElem {
    Attlist,
    Entity,
    Pubid,
    Encoding
}

pub struct XmlLexer<R> {
    line: uint,
    col: uint,
    priv peek_buf: ~str,
    priv err_buf: ~str,
    priv source: R
}

impl<R: Reader+Buffer> Iterator<XmlToken> for XmlLexer<R>{
    /// This method pulls tokens from Reader until it reaches
    /// end of file. From that point on, it will return None.
    ///
    /// Example:
    ///     let reader = Reader::new(bytes!("<a></a>"));
    ///     let mut lexer = XmlLexer::from_reader(reader);
    ///
    ///     // Calling lexer for each individual element
    ///     let token = lexer.next();
    ///
    ///     // Calling lexer in a loop
    ///     for tok in lexer {
    ///         println!(tok);
    ///     }
    ///     assert_eq!(None, lexer.next());
    fn next(&mut self) -> Option<XmlToken> {
        let chr_peek = self.peek_chr();

        //debug!(format!("Chr peek {}", chr_peek));

        let token = match chr_peek {

            Char(chr) if is_whitespace(&chr)
                      => self.get_whitespace_token(),
            Char(chr) if is_name_start(&chr)
                      => self.get_name_token(),
            Char(chr) if is_name_char(&chr)
                      => self.get_nmtoken(),
            Char('<') => self.get_left_bracket_token(),
            Char('?') => self.get_pi_end_token(),
            Char(']') => self.get_sqbracket_right_token(),
            Char('[') => self.get_sqbracket_left_token(),
            Char('(') => self.get_paren_left_token(),
            Char(')') => self.get_paren_right_token(),
            Char('|') => self.get_pipe_token(),
            Char('*') => self.get_star_token(),
            Char('+') => self.get_plus_token(),
            Char('&') => self.get_ref_token(),
            Char('%') => self.get_peref_token(),
            Char('!') => self.get_doctype_end_token(),
            Char('>') => self.get_right_bracket_token(),
            Char('/') => self.get_empty_tag_token(),
            Char(';') => self.get_semicolon_token(),
            Char('=') => self.get_equal_token(),
            Char('#') => self.get_entity_def_token(),
            Char('\'') | Char('"') => self.get_quote_token(),
            Char(_) => self.get_text_token(),
            _ => None
        };
        //debug!(fmt!("token: %?", token));
        token

    }
}

impl<R: Reader+Buffer> XmlLexer<R> {
    /// Constructs a new `XmlLexer` from data given.
    /// Data structure that is shared, must implement Reader
    /// and Writer traits.
    pub fn from_reader(data : R) -> XmlLexer<R> {
        XmlLexer {
            line: 1,
            col: 0,
            peek_buf: ~"",
            err_buf: ~"",
            source: data
        }
    }


    /// This method reads a string of given length skipping over any
    /// restricted character and adding an error for each such
    /// character.
    ///
    /// Restricted characters are *not included* into the output
    /// string.
    pub fn read_str(&mut self, len: uint) -> ~str {
        clean_restricted(self.read_raw_str(len))
    }

    /// Method that peeks incoming strings
    pub fn peek_str(&mut self, len: uint) -> ~str {
        let col = self.col;
        let line = self.line;

        let peek_result  = self.read_raw_str(len);
        self.col = col;
        self.line = line;

        //FIXME: With different access function
        for c in peek_result.chars_rev(){
             self.peek_buf.push_char(c);
        }

        clean_restricted(peek_result)
    }

    pub fn next_special(&mut self, expect: ExpectElem)
                        -> Option<XmlToken> {
        let result;

        match self.peek_chr() {
            Char('\'') | Char('"') => {
                result = match expect {
                    Attlist => self.get_attl_quote(),
                    Entity  => self.get_ent_quote(),
                    Pubid   => self.get_pubid_quote(),
                    Encoding => self.get_encoding_quote()
                };

            },
            _ => {result = self.next();}

        };
        result
    }

    fn get_encoding_quote(&mut self) -> Option<XmlToken> {
        let quote = self.read_str(1u);
        assert_eq!(true, (quote == ~"'" || quote == ~"\""));

        let text = self.read_until_peek(quote);

        self.read_str(1u);
        Some(QuotedString(text))
    }

    fn get_pubid_quote(&mut self) -> Option<XmlToken> {
        None
    }

    fn get_ent_quote(&mut self) -> Option<XmlToken> {
        None
    }

    fn get_attl_quote(&mut self) -> Option<XmlToken> {
        None
    }
    /// This method reads a character and returns an enum that
    /// might be either a value of character, a new-line sign or a
    /// restricted character. Encountering Restricted characters
    /// by default will not result in an error, only a warning.
    /// Position will still be updated upon finding Restricted
    /// characters. Characters that are neither restricted nor
    /// allowed will be ignored.
    ///
    /// Method short-circuits if the End of file has been reached.
    ///
    /// Note: This method will normalize all accepted newline
    /// characters into '\n' character. Encountered will not be
    /// preserved.
    /// See http://www.w3.org/TR/xml11/#sec-line-ends for more
    /// information
    fn read_chr(&mut self) -> Character {

        let chr;

        if self.peek_buf.is_empty() {

            if self.source.eof() {
                return EndFile
            }

            let temp_chr = self.source.read_char();
            match temp_chr {
                Some(a) => chr = a,
                None => {
                    self.handle_errors(UnreadableChar);
                    // If error on read is encountered handle errors
                    // method should fail, but if it doesn't
                    // then value of restricted char is `\x01`
                    chr = '\x01';
                }
            }
        } else {
            chr = self.peek_buf.pop_char();
        }

        if "\r\u2028\x85".contains_char(chr) {
           return self.process_newline(chr)
        } else {
           return self.process_char(chr)
        }

    }



    fn peek_chr(&mut self) -> Character {
        let col = self.col;
        let line = self.line;

        let peek_char = self.read_chr();
        self.col = col;
        self.line = line;
        match peek_char.extract_char() {
            Some(a) => self.peek_buf.push_char(a),
            None => {}
        };

        peek_char
    }


    //TODO Doc
    fn read_until_fn(&mut self, filter_fn: |Character|-> bool )
                     -> ~str {
        let mut col = 0u;
        let mut line = 1u;
        let mut peek_char = self.peek_chr();
        let mut ret_str = ~"";

        while(filter_fn(peek_char)){
            match peek_char {
                Char(a) => {
                    ret_str.push_char(a);
                    self.read_chr();
                    col = self.col;
                    line = self.line;
                    peek_char = self.peek_chr();
                },
                _ => {}
            }
        }
        self.col = col;
        self.line = line;
        //TODO error checking
        ret_str
    }

    fn read_until_peek(&mut self, peek_look: &str) -> ~str {
        let mut peek = self.peek_str(peek_look.char_len());
        let mut result = ~"";
        while peek != peek_look.to_owned() {

            let extracted_char = self.read_chr().extract_char();
            match extracted_char {
                None => {/* FIXME: Error processing*/},
                Some(a) => {result.push_char(a)}
            }

            peek = self.peek_str(peek_look.char_len());
        }
        result
    }

    /// This method reads a string of given length, adding any
    /// restricted char  into the error section.
    /// Restricted character are *included* into the output string
    fn read_raw_str(&mut self, len: uint) -> ~str {
        let mut raw_str = ~"";
        let mut eof = false;
        let mut l = 0u;

        while (l < len && !eof) {
            let chr = self.read_chr();
            l += 1;
            match chr {
                Char(a) => raw_str.push_char(a),
                EndFile => {
                    self.handle_errors(PrematureEOF);
                    eof = true;
                },
                RestrictedChar(a) =>{
                    self.handle_errors(RestrictedCharError);
                    raw_str.push_char(a);
                }
            };

        };
        raw_str
    }


    fn handle_errors(&self, kind: ErrKind) {

    }

    fn get_error (&mut self, err: ~str) -> XmlError {
        XmlError {
            line: self.line,
            col: self.col,
            msg: err,
            mark: None
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

        if(c == '\r'){
            let chrPeek = self.source.read_char();
            match chrPeek {
                // If the read character isn't a double
                // new-line character (\r\85 or \n),
                // it's added to peek buffer
                Some(a) if a != '\x85' && a != '\n'
                        => self.peek_buf.push_char(a),
                _ => {}

            }
        }

        Char('\n')
    }

    /// This method expects to takes an input `char` *c* that isn't a
    /// newline sigil. According to it, it then processes the given
    /// *c*, increasing position in reader.
    fn process_char(&mut self, c: char) -> Character {
        self.col += 1u;
        Character::from_char(c)
    }

    fn process_name_token(&mut self) -> ~str {
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => util::is_name_char(&v)
            }
        })
    }

    fn process_name_token2(&mut self) -> ~str {
        let mut str_buf = ~"";

        match self.read_chr() {
            Char(a) if(util::is_name_start(&a))
                    => str_buf.push_char(a),
            Char(_) => {
                self.handle_errors(UnexpectedChar);
            }
            RestrictedChar(_) => {
                str_buf = ~"";
                self.handle_errors(RestrictedCharError);
            },
            EndFile => {
                str_buf = ~"";
                self.handle_errors(PrematureEOF);
            }
        }
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => util::is_name_char(&v)
            }
        });

        str_buf
    }

    fn process_digits(&mut self, is_hex: &bool) -> ~str {
        if *is_hex { self.read_chr();}
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => util::is_digit(&v)
            }
        })
    }

    /// If we find a whitespace character this method
    /// consumes all following whitespace characters until it
    /// reaches a non white space character be it Restricted char,
    /// EndFile or  a non-white space char.
    fn get_whitespace_token(&mut self) -> Option<XmlToken> {
        let ws = self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => util::is_whitespace(&v)
            }
        });
        Some(WhiteSpace(ws))
    }

    /// If we find a namespace start character this method
    /// consumes all namespace token until it reaches a non-name
    /// character.
    fn get_name_token(&mut self) -> Option<XmlToken> {
        let mut name = ~"";
        let start_char = self.read_chr();
        match start_char.extract_char() {
            Some(a) if(util::is_name_start(&a))
              => name.push_char(a),
            _ => fail!(~"Expected name start token")
        };

        let result = self.process_name_token();
        name.push_str(result);

        Some(NameToken(name))
    }

    fn get_nmtoken(&mut self) -> Option<XmlToken> {
        let mut name = ~"";
        let start_char = self.peek_chr();
        match start_char.extract_char() {
            Some(a) if(util::is_name_start(&a))
              => {},
            _ => fail!(~"Expected name start token")
        };

        let result = self.process_name_token();
        name.push_str(result);

        Some(NameToken(name))
    }

    fn get_left_bracket_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('<'),   self.peek_chr());

        let peek_first = self.peek_str(2u);
        let result;

        //debug!(fmt!("peek first: %?", peek_first));

        if peek_first  == ~"<?" {
            result = self.get_pi_token();
        } else if peek_first == ~"</" {
            result = self.get_close_tag_token();
        } else if peek_first == ~"<!" {
            let peek_sec = self.peek_str(3u);

            if peek_sec == ~"<!-" {
                result = self.get_comment_token();
            } else if peek_sec == ~"<![" {
                result = self.get_cdata_token();
            } else if peek_sec == ~"<!D" {
                result = self.get_doctype_start_token();
            } else if peek_sec == ~"<!E" {
                result = self.get_entity_or_element_token();
            } else if peek_sec == ~"<!A" {
                result = self.get_attlist_token();
            } else if peek_sec == ~"<!N" {
                result = self.get_notation_token();
            } else {
                result = Some(ErrorToken(~""));
            }
        } else {
            self.read_chr();
            result = Some(LessBracket);
        }
        result
    }

    //FIX THIS: possible element ignore section
    fn get_cdata_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<![",       self.read_str(3u));

        let peek = self.peek_str(6u);
        let result;
        if(peek == ~"CDATA["){

            self.read_str(6u);
            let text = self.read_until_peek("]]>");
            self.read_str(3u);

            result = Some(CData(text));
        } else {
            result = Some(DoctypeOpen);
        }
        result
    }

    fn get_doctype_start_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!D",       self.read_str(3u));
        let peeked_str = self.peek_str(6u);
        let result;
        if peeked_str == ~"OCTYPE" {
            self.read_str(6u);
            result = Some(DoctypeStart);
        } else {
            result = Some(ErrorToken(~"<!D"));
        }
        result
    }

    fn get_ref_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('&'),       self.read_chr());
        let peek_char = self.peek_chr();

        let token = match peek_char {
            Char('#') => {
                self.get_char_ref_token()
            },
            Char(_) => {
                Some(Amp)
            },
            _ => {
                Some(EndOfFile)
            }
        };
        token
    }

    fn get_peref_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('%'),       self.read_chr());
        Some(Percent)
    }

    fn get_equal_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('='),       self.read_chr());
        Some(EqTok)
    }

    fn get_char_ref_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('#'),       self.read_chr());
        let peek_char = self.peek_chr();

        let radix;
        match peek_char {
            Char('x') => {
                radix = 16;
            },
            Char(_) => radix = 10,
            _ => return {
                Some(EndOfFile)
            }
        }

        let is_radix = (radix == 16);
        let char_ref = self.process_digits(&is_radix);

        let end_char_ref = self.peek_chr();

        match end_char_ref {
            Char(';') => {
                self.read_chr();
            },
            _ => {
                let mut err_token = ~"#";
                match peek_char.extract_char() {
                    Some(a) => err_token.push_char(a),
                    _ => {}
                }
                err_token.push_str(char_ref);
                return Some(ErrorToken(~""))
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
                    _ => Some(EndOfFile)
                }
            },
            None => {
                Some(EndOfFile)
            }
        }
    }

    fn get_sqbracket_left_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('['),       self.read_chr());
        Some(LeftSqBracket)
    }


    fn get_sqbracket_right_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char(']'),       self.peek_chr());
        let result;
        if ~"]]>" == self.peek_str(3u) {
            self.read_str(3u);
            result = Some(DoctypeClose);
        } else {
            self.read_chr();
            result = Some(RightSqBracket);
        }
        result
    }

    fn get_paren_left_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('('),       self.read_chr());
        Some(LeftParen)
    }

    fn get_paren_right_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char(')'),       self.read_chr());
        Some(RightParen)
    }

    fn get_semicolon_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char(';'),       self.read_chr());
        Some(Semicolon)
    }

    fn get_entity_def_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('#'),       self.read_chr());
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

    fn get_entity_or_element_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!", self.read_str(2u));

        let result;
        if self.peek_str(7u) == ~"ELEMENT" {
            self.read_str(7u);
            result = Some(ElementType);
        } else if self.peek_str(6u) == ~"ENTITY" {
            self.read_str(6u);
            result = Some(EntityType);
        } else {
            result = Some(ErrorToken(~"<!"));
        }
        result
    }

    fn get_attlist_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!", self.read_str(2u));
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
        assert_eq!(~"<!", self.read_str(2u));
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
        assert_eq!(Char('*'),       self.read_chr());
        Some(Star)
    }

    fn get_plus_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('+'),       self.read_chr());
        Some(Plus)
    }

    fn get_pipe_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('|'),       self.read_chr());
        Some(Pipe)
    }

    fn get_quote_token(&mut self) -> Option<XmlToken> {
        let quote = self.read_str(1u);
        assert_eq!(true, (quote == ~"'" || quote == ~"\""));

        let text = self.read_until_peek(quote);

        self.read_str(1u);
        Some(QuotedString(text))
    }

    fn get_text_token(&mut self) -> Option<XmlToken> {
        let mut peek;
        let mut text = ~"";
        let mut run_loop = true;
        while run_loop {
            peek = self.peek_str(3u);
            run_loop = !peek.starts_with("&")
                    && !peek.starts_with("<")
                    && peek != ~"]]>";
            if run_loop {
                text.push_str(self.read_str(1u));
            }
        }
        Some(Text(text))
    }

    fn get_pi_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<?",       self.read_str(2u));

        let name = self.peek_str(3u);

        if name.eq_ignore_ascii_case("xml") {
            self.read_str(3u);
            return Some(PrologStart);
        } else {
            let text = self.read_until_peek("?>");
            self.read_str(2u);
            return Some(PI(text));
        }
    }

    fn get_doctype_end_token(&mut self) -> Option<XmlToken> {
        let peek_str = self.peek_str(2u);

        if peek_str == ~"!>" {
            self.read_str(2u);
            return Some(DoctypeEnd)
        } else {
            let text = self.read_str(1u);
            return Some(ErrorToken(text))
        }
    }

    fn get_right_bracket_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('>'), self.read_chr());
        return Some(GreaterBracket)
    }

    fn get_comment_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"<!-", self.read_str(3u));

        let peek_str = self.peek_str(1u);

        if peek_str == ~"-" {
            self.read_str(1u);

            let text = self.process_comment();
            return Some(Comment(text))
        } else {
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
                    self.handle_errors(MinMinInComment);
                }

                let extracted_char = self.read_chr().extract_char();
                match extracted_char {
                    None => {},
                    Some(a) => {result.push_char(a)}
                }
                peek = self.peek_str(3u);
            }
        }
        result
    }

    fn get_close_tag_token(&mut self) -> Option<XmlToken> {
        assert_eq!(~"</",  self.read_str(2u));
        return Some(CloseTag)
    }

    fn get_empty_tag_token(&mut self) -> Option<XmlToken> {
        assert_eq!(Char('/'), self.read_chr());

        let result;
        if self.read_str(1u) == ~">" {
            result = Some(EmptyTag);
        } else {
            result = Some(ErrorToken(~"/"));
        }
        result
    }

    fn get_pi_end_token(&mut self) -> Option<XmlToken> {
        let chr_assert = self.read_chr();
        assert_eq!(Char('?'),   chr_assert);

        let chr_peek = self.peek_chr();
        let result = match chr_peek {
            Char('>')    => {
                self.read_chr();
                Some(PrologEnd)
            },
            _=> Some(QuestionMark)
        };
        result
    }
}

pub fn main() {

}

#[cfg(test)]
mod tests {

    use super::{XmlLexer, Char, EndFile, RestrictedChar};
    use super::{PrologEnd,PrologStart,PI,CData,WhiteSpace};
    use super::{DoctypeOpen,DoctypeStart,DoctypeEnd,CharRef};
    use super::{Percent,NameToken,DoctypeClose,Amp, Semicolon};
    use super::{EntityType,NotationType,Comment};
    use super::{AttlistType,GreaterBracket,LessBracket,ElementType};
    use super::{CloseTag,EqTok,Star,QuestionMark,Plus,Pipe};
    use super::{LeftParen,RightParen,EmptyTag,QuotedString,Text};

    use std::io::mem::BufReader;

    use util::{XmlError};

    #[test]
    fn test_iteration() {
        let r = BufReader::new(bytes!("<a>"));
        let mut lexer = XmlLexer::from_reader(r);
        for token in lexer {

        }
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_tokens() {
        let r = BufReader::new(bytes!("<?xml?> <?php stuff?><![CDATA[<test>]]>\t"));
        let mut lexer =         XmlLexer::from_reader(r);


        assert_eq!(Some(PrologStart),               lexer.next());
        assert_eq!(Some(PrologEnd),                 lexer.next());
        assert_eq!(Some(WhiteSpace(~" ")),          lexer.next());
        assert_eq!(Some(PI(~"php stuff")),          lexer.next());
        assert_eq!(Some(CData(~"<test>")),          lexer.next());
        assert_eq!(Some(WhiteSpace(~"\t")),         lexer.next());


        let r2 = BufReader::new(bytes!("<![]]><!DOCTYPE &#x3123;&#212;%name;&name2;"));
        lexer = XmlLexer::from_reader(r2);

        assert_eq!(Some(DoctypeOpen),           lexer.next());
        assert_eq!(Some(DoctypeClose),          lexer.next());
        assert_eq!(Some(DoctypeStart),          lexer.next());
        assert_eq!(Some(WhiteSpace(~" ")),      lexer.next());
        assert_eq!(Some(CharRef('\u3123')),     lexer.next());
        assert_eq!(Some(CharRef('\xD4')),       lexer.next());
        assert_eq!(Some(Percent),               lexer.next());
        assert_eq!(Some(NameToken(~"name")),    lexer.next());
        assert_eq!(Some(Semicolon),             lexer.next());
        assert_eq!(Some(Amp),                   lexer.next());
        assert_eq!(Some(NameToken(~"name2")),   lexer.next());
        assert_eq!(Some(Semicolon),             lexer.next());

        let r3 = BufReader::new(bytes!("<!ENTITY<!NOTATION<!ELEMENT<!ATTLIST!><br>"));
        lexer = XmlLexer::from_reader(r3);

        assert_eq!(Some(EntityType),        lexer.next());
        assert_eq!(Some(NotationType),      lexer.next());
        assert_eq!(Some(ElementType),       lexer.next());
        assert_eq!(Some(AttlistType),       lexer.next());
        assert_eq!(Some(DoctypeEnd),        lexer.next());
        assert_eq!(Some(LessBracket),       lexer.next());
        assert_eq!(Some(NameToken(~"br")),  lexer.next());
        assert_eq!(Some(GreaterBracket),    lexer.next());

        let r4 = BufReader::new(bytes!("</br><e/><!-- -->()|+?*="));
        lexer = XmlLexer::from_reader(r4);

        assert_eq!(Some(CloseTag),          lexer.next());
        assert_eq!(Some(NameToken(~"br")),  lexer.next());
        assert_eq!(Some(GreaterBracket),    lexer.next());
        assert_eq!(Some(LessBracket),       lexer.next());
        assert_eq!(Some(NameToken(~"e")),   lexer.next());
        assert_eq!(Some(EmptyTag),          lexer.next());
        assert_eq!(Some(Comment(~" ")),     lexer.next());
        assert_eq!(Some(LeftParen),         lexer.next());
        assert_eq!(Some(RightParen),        lexer.next());
        assert_eq!(Some(Pipe),              lexer.next());
        assert_eq!(Some(Plus),              lexer.next());
        assert_eq!(Some(QuestionMark),      lexer.next());
        assert_eq!(Some(Star),              lexer.next());
        assert_eq!(Some(EqTok),             lexer.next());

        let r5 = BufReader::new(bytes!("'quote'\"funny\"$BLA<&apos;"));
        lexer = XmlLexer::from_reader(r5);

        assert_eq!(Some(QuotedString(~"quote")),    lexer.next());
        assert_eq!(Some(QuotedString(~"funny")),    lexer.next());
        assert_eq!(Some(Text(~"$BLA")),             lexer.next());
        assert_eq!(Some(LessBracket),               lexer.next());
        assert_eq!(Some(Amp),                       lexer.next());
        assert_eq!(Some(NameToken(~"apos")),        lexer.next());
        assert_eq!(Some(Semicolon),                 lexer.next());

    }

    #[test]
    fn test_multi_peek() {
        let r = BufReader::new(bytes!("123"));
        let mut lexer =             XmlLexer::from_reader(r);

        assert_eq!(~"12",           lexer.peek_str(2u));
        assert_eq!(~"12",           lexer.peek_str(2u));
        assert_eq!(~"1",            lexer.read_str(1u));
        assert_eq!(~"23",           lexer.peek_str(2u));
        assert_eq!(~"23",           lexer.peek_str(2u));
    }

    #[test]
    fn test_peek_restricted() {
        let r = BufReader::new(bytes!("1\x0123"));
        let mut lexer =             XmlLexer::from_reader(r);

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
        let r = BufReader::new(bytes!("012345"));
        let mut lexer =         XmlLexer::from_reader(r);

        lexer.peek_str(6u);
        assert_eq!(~"012345",       lexer.read_str(6u));
    }

    #[test]
    fn test_whitespace() {
        let r = BufReader::new(bytes!("  \t\n  a "));
        let mut lexer =         XmlLexer::from_reader(r);

        assert_eq!(Some(WhiteSpace(~"  \t\n  ")),      lexer.next());
        assert_eq!(6u,                                 lexer.col);
        assert_eq!(1u,                                 lexer.line);
        assert_eq!(Some(NameToken(~"a")),              lexer.next());
    }

    #[test]
    fn test_peek_str() {
        let r = BufReader::new(bytes!("as"));
        let mut lexer = XmlLexer::from_reader(r);

        assert_eq!(~"as",               lexer.peek_str(2u));
        assert_eq!(0u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);
        assert_eq!(Char('a'),           lexer.read_chr());
        assert_eq!(1u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);
        assert_eq!(~"s",                lexer.read_str(1u));
        assert_eq!(2u,                  lexer.col);
        assert_eq!(1u,                  lexer.line);
    }

    #[test]
    fn test_eof() {
        let r = BufReader::new(bytes!("a"));
        let mut lexer = XmlLexer::from_reader(r);

        assert_eq!(Char('a'),           lexer.read_chr());
        assert_eq!(EndFile,             lexer.read_chr())
    }

    #[test]
    fn test_read_until() {
        let r = BufReader::new(bytes!("aaaab"));
        let mut lexer = XmlLexer::from_reader(r);

        let result = lexer.read_until_fn(|c|{
            match c {
                Char('a') => true,
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
        let r1 = BufReader::new(bytes!("\x01\x04\x08a\x0B\x0Cb\x0E\x10\x1Fc\x7F\x80\x84d\x86\x90\x9F"));
        let mut lexer = XmlLexer::from_reader(r1);

        assert_eq!(RestrictedChar('\x01'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x04'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x08'),      lexer.read_chr());
        assert_eq!(Char('a'),                   lexer.read_chr());
        assert_eq!(RestrictedChar('\x0B'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x0C'),      lexer.read_chr());
        assert_eq!(Char('b'),                   lexer.read_chr());
        assert_eq!(RestrictedChar('\x0E'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x10'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x1F'),      lexer.read_chr());
        assert_eq!(Char('c'),                   lexer.read_chr());
        assert_eq!(RestrictedChar('\x7F'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x80'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x84'),      lexer.read_chr());
        assert_eq!(Char('d'),                   lexer.read_chr());
        assert_eq!(RestrictedChar('\x86'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x90'),      lexer.read_chr());
        assert_eq!(RestrictedChar('\x9F'),      lexer.read_chr());
    }

    #[test]
    fn test_read_newline() {
        let r1 = BufReader::new(bytes!("a\r\nt"));
        let mut lexer = XmlLexer::from_reader(r1);

        assert_eq!(Char('a'),   lexer.read_chr());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);

        let r2 = BufReader::new(bytes!("a\rt"));
        lexer = XmlLexer::from_reader(r2);

        assert_eq!(Char('a'),   lexer.read_chr());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);

        let r3 = BufReader::new(bytes!("a\r\x85t"));
        lexer = XmlLexer::from_reader(r3);

        assert_eq!(Char('a'),   lexer.read_chr());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);

        let r4 = BufReader::new(bytes!("a\x85t"));
        let mut lexer = XmlLexer::from_reader(r4);

        assert_eq!(Char('a'),   lexer.read_chr());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);


        let r5 = BufReader::new(bytes!("a\u2028t"));
        let mut lexer = XmlLexer::from_reader(r5);

        assert_eq!(Char('a'),   lexer.read_chr());
        assert_eq!(1,           lexer.line);
        assert_eq!(1,           lexer.col);
        assert_eq!(Char('\n'),  lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(0,           lexer.col);
        assert_eq!(Char('t'),   lexer.read_chr());
        assert_eq!(2,           lexer.line);
        assert_eq!(1,           lexer.col);
    }

}
