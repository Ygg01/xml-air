use std::io::*;
use util::*;
use std::char::*;
use std::num::*;
use std::ascii::StrAsciiExt;

mod util;

#[deriving(Eq)]
pub enum XmlToken {
    ErrorToken(~str),   // Error token
    LessBracket,        // Symbol '<'
    GreaterBracket,     // Symbol '>'
    LeftSqBracket,      // Symbol '['
    RightSqBracket,     // Symbol ']'
    LeftParen,          // Symbol '('
    RightParen,         // Symbol ')'
    Equal,              // Symbol '='
    Plus,               // Symbol '+'
    Pipe,               // Symbol '|'
    Star,               // Symbol '*'
    QuestionMark,       // Symbol '?'
    CloseTag,           // Symbol '</'
    EmptyTag,           // Symbol '/>'
    NameToken(~str),    // Tag name
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
    EntityRef(~str),    // Entity reference, symbol '&'
    PERef(~str),        // Entity reference, symbol '%'
    CharRef(char),      // Encoded char or '&#'
    SingleQuoted(~str), // Single quoted string e.g. 'example'
    DoubleQuoted(~str), // Single quoted string e.g. "example"
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

pub enum CharData {
    No,
    PIChar,
    CDataChar,
    ElemData
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
        if(is_restricted(&chr)){
            RestrictedChar(chr)
        }else if(is_char(&chr)){
            Char(chr)
        }else{
            // If we encounter unknown character we replace it with
            // space.
            // TODO check if this is OK
            Char(' ')
        }
    }
}


pub struct XmlLexer {
    line: uint,
    col: uint,
    token: Option<XmlToken>,
    priv peek_buf: ~str,
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
        let chr_peek = self.peek_chr();

        //println(fmt!("Chr peek %?", chr_peek));

        let token = match chr_peek {

            Char(chr) if(is_whitespace(chr)) => self.get_whitespace_token(),
            Char(chr) if(is_name_start(&chr))=> self.get_name_token(),
            Char('<') => self.get_left_bracket_token(),
            Char('?') => self.get_pi_end_token(),
            Char(']') => self.get_sqbracket_right_token(),
            Char('[') => self.get_sqbracket_left_token(),
            Char('(') => self.get_paren_left_token(),
            Char(')') => self.get_paren_right_token(),
            Char('*') => self.get_star_token(),
            Char('+') => self.get_plus_token(),
            Char('&') => self.get_ref_token(),
            Char('%') => self.get_peref_token(),
            Char('!') => self.get_doctype_end_token(),
            Char('>') => self.get_right_bracket_token(),
            Char('/') => self.get_empty_tag_token(),
            Char('\'') | Char('"') => self.get_quote_token(),
            Char('#') => self.get_entity_def_token(),
            Char(_) => self.get_text_token(),
            _ => None
        };
        //println(fmt!("token: %?", token));
        token

    }
}

impl XmlLexer {
    /// Constructs a new `XmlLexer` from @Reader `data`
    /// The `XmlLexer` will use the given reader as the source for parsing.
    pub fn from_reader(data : @Reader) -> XmlLexer {
        XmlLexer {
            line: 1,
            col: 0,
            token: None,
            peek_buf: ~"",
            err_buf: ~"",
            source: data
        }
    }


    /// This method reads a string of given length skipping over any
    /// restricted character and adding an error for each such
    /// character.
    /// Restricted characters are *not included* into the output
    /// string.
    fn read_str(&mut self, len: uint) -> XmlResult<~str> {
        XmlLexer::clean_restricted(self.read_str_raw(len))
    }

    /// This method reads a character and returns an enum that might be
    /// either a value of character, a new-line sign or a restricted character.
    /// 
    /// Encountering Restricted characters will not result in an error,
    /// Instead the position will be update but no information about such
    /// characters will not be preserved.
    ///
    /// Method short-circuits if the End of file has been reached
    ///
    /// Note: This method will normalize all accepted newline characters into
    /// '\n' character.
    /// encountered will not be preserved.
    ///TODO add line char buffer
    fn read(&mut self) -> Character {

        let chr;

        if(self.peek_buf.is_empty()){

            if(self.source.eof()){
                return EndFile
            }

            chr= self.raw_read();
        }else{
            chr = self.peek_buf.pop_char();
        }

        if("\r\u2028\x85".contains_char(chr)){
           return self.process_newline(chr)
        }else{
           return self.process_char(chr)
        }

    }

    pub fn peek_chr(&mut self) -> Character {
        let col = self.col;
        let line = self.line;

        let peek_char = self.read();
        self.col = col;
        self.line = line;
        match peek_char.extract_char() {
            Some(a) => self.peek_buf.push_char(a),
            None => {}
        };

        peek_char
    }

    /// Method that peeks incoming strings
    pub fn peek_str(&mut self, len: uint) -> XmlResult<~str>{
        let col = self.col;
        let line = self.line;

        let peek_result  = self.read_str_raw(len);
        self.col = col;
        self.line = line;

        for c in peek_result.data.rev_iter(){
             self.peek_buf.push_char(c);
        }

        XmlLexer::clean_restricted(peek_result)
    }

    //TODO Doc
    fn read_until_fn(&mut self, filter_fn: &fn(Character)-> bool ) -> XmlResult<~str>{
        let mut col = 0u;
        let mut line = 1u;
        let mut char_read = self.read();
        let mut ret_str = ~"";

        while(filter_fn(char_read)){
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

    fn read_until_peek(&mut self, peek_look: &str) -> XmlResult<~str>{
        let mut peek = self.peek_str(peek_look.char_len());
        let mut result = ~"";
        while(peek.data != peek_look.to_owned()){
            
            let extracted_char = self.read().extract_char();
            match extracted_char {
                None => {/* FIXME: Error processing*/},
                Some(a) => {result.push_char(a)}
            }
            peek = self.peek_str(peek_look.char_len());
        }
        XmlResult{ data: result, errors: ~[]}
    }

    /// This method reads a string of given length, adding any
    /// restricted char  into the error section.
    /// Restricted character are *included* into the output string
    fn read_str_raw(&mut self, len: uint) -> XmlResult<~str> {
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
                    found_errs = ~[self.get_error(~"Unexpected end of file")];
                    eof = true;
                },
                RestrictedChar(a) =>{
                    found_errs = ~[self.get_error(~"Illegal character")];
                    string.push_char(a);
                }
            };

        };
        XmlResult{ data: string, errors:found_errs}
    }

    fn get_error(&mut self, err: ~str) -> XmlError {
        XmlError {
            line: self.line,
            col: self.col,
            msg: err,
            mark: None
        }
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

    /// This methods removes all restricted character from a given XmlResult<~str>,
    /// Without changing errors
    fn clean_restricted(input: XmlResult<~str>) -> XmlResult<~str> {
        let mut clean_str = ~"";

        for c in input.data.iter() {
            if (!is_restricted(&c)){
                clean_str.push_char(c);
            }
        }

        let result = XmlResult {
                        data: clean_str,
                        errors: input.errors.clone()
        };
        result
    }

    /// Processes the input `char` as it was a newline
    /// Note if char read is `\r` it must peek to check if `\x85` or `\n`
    /// are next, because they are part of same newline group.
    /// See to `http://www.w3.org/TR/xml11/#sec-line-ends` for details
    /// This method updates column and line position accordingly.
    ///
    /// Note: Lines and column start at 1 but the read character will be
    /// update after a new character is read.
    fn process_newline(&mut self, c: char) -> Character {
        self.line += 1u;
        self.col = 0u;

        if(c == '\r'){
            let chrPeek = self.raw_read();
            if(chrPeek != '\x85' && chrPeek != '\n'){
                self.raw_unread();
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

    fn process_name_token(&mut self) -> XmlResult<~str> {
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => is_name_char(&v)
            }
        })
    }

    fn process_name_token2(&mut self) -> XmlResult<~str> {
        let mut str_buf = ~"";
        let mut errs = ~[];
        match self.read() {
            Char(a) if(is_name_start(&a)) => str_buf.push_char(a),
            _ => {str_buf = ~""; errs.push(XmlError{line: 0u, col: 0u, msg: ~"", mark:None}); }
        }
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => is_name_char(&v)
            }
        });

        XmlResult{ data: str_buf, errors: errs}
    }

    fn process_hex_digit(&mut self) -> XmlResult<~str> {
        self.read();
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => is_hex_digit(&v)
            }
        })
    }

    fn process_digit(&mut self) -> XmlResult<~str> {
        self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => is_digit(v)
            }
        })
    }

    /// If we find a whitespace character this method
    /// consumes all following whitespace characters until it
    /// reaches a non white space character be it Restricted char,
    /// EndFile or  a non-white space char.
    fn get_whitespace_token(&mut self) -> Option<XmlResult<XmlToken>> {
        let whitespace = self.read_until_fn( |val| {
            match val {
                RestrictedChar(_)   => false,
                EndFile             => false,
                Char(v)             => is_whitespace(v)
            }
        }).data;
        Some(XmlResult{data: WhiteSpace(whitespace), errors: ~[]})
    }

    /// If we find a namespace start character this method
    /// consumes all namespace token until it reaches a non-name
    /// character.
    fn get_name_token(&mut self) -> Option<XmlResult<XmlToken>> {
        let start_char = self.read();
        assert_eq!(true, start_char.is_valid_char());


        let mut name = ~"";
        match start_char.extract_char() {
            Some(a) if(is_name_start(&a)) => name.push_char(a),
            _                             => fail!(~"Expected name start token")
        }


        let result = self.process_name_token();
        name.push_str(result.data);

        Some(XmlResult{data: NameToken(name), errors: result.errors.clone()})
    }

    fn get_left_bracket_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('<'),   self.peek_chr());

        let peek_first = self.peek_str(2u);
        let result;

        //println(fmt!("peek first: %?", peek_first.data));

        if(peek_first.data == ~"<?") {
            result = self.get_pi_token();
        } else if(peek_first.data == ~"</"){
            result = self.get_close_tag_token();
        } else if(peek_first.data == ~"<!"){
            let peek_sec = self.peek_str(3u).data;

            if(peek_sec == ~"<!-"){
                result = self.get_comment_token();
            }else if( peek_sec == ~"<!["){
                result = self.get_cdata_token();
            }else if( peek_sec == ~"<!D"){
                result = self.get_doctype_start_token();
            }else if(peek_sec == ~"<!E"){
                result = self.get_entity_or_element_token();
            }else if(peek_sec == ~"<!A"){
                result = self.get_attlist_token();
            }else if(peek_sec == ~"<!N"){
                result = self.get_notation_token();
            }else{
                result = Some(XmlResult{data: ErrorToken(~""), errors: ~[]});
            }
        } else {
            self.read();
            result = Some(XmlResult{data: LessBracket, errors: ~[]});
        }
        result
    }

    //FIX THIS: possible element ignore section
    fn get_cdata_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<![",       self.read_str(3u).data);
        let peek = self.peek_str(6u).data;
        let result; 
        if(peek == ~"CDATA["){
            self.read_str(6u);
            let text = self.read_until_peek("]]>").data;
            self.read_str(3u);
            result = Some(XmlResult{data: CData(text), errors: ~[]});
        }else{
            result = Some(XmlResult{data: DoctypeOpen, errors: ~[]});
        }
        result
    }

    fn get_doctype_start_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<!D",       self.read_str(3u).data);
        let peeked_str = self.peek_str(6u).data;
        let result;
        if(peeked_str == ~"OCTYPE"){
            self.read_str(6u);
            result = Some(XmlResult{data: DoctypeStart, errors: ~[]});
        }else{
            result = Some(XmlResult{data: ErrorToken(~"<!D"), errors: ~[]});
        }
        result
    }

    fn get_ref_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('&'),       self.read());
        let peek_char = self.peek_chr();

        let token = match peek_char {
            Char('#') => {
                self.get_char_ref_token()
            },
            Char(_) => {
                self.get_entity_ref_token()
            },
            _ => {
                Some(XmlResult{ data: EndOfFile, errors: ~[self.get_error(~"mock error")]})
            }
        };
        token
    }

    fn get_entity_ref_token(&mut self) -> Option<XmlResult<XmlToken>> {
        let text = self.process_name_token().data;
        let result;
        let entity_ref = EntityRef(text);
        match self.peek_chr() {
                Char(';') => {
                    self.read();
                    result = Some(XmlResult{ data: entity_ref, errors: ~[]});
                },
                _ => {
                    result = Some(XmlResult{ data: entity_ref, errors: ~[]})
                }
        };
        result
    }

    fn get_peref_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('%'),       self.read());
        let text = self.process_name_token().data;
        let result;
        let peref = PERef(text);
        match self.peek_chr() {
                Char(';') => {
                    self.read();
                    result = Some(XmlResult{ data: peref, errors: ~[]});
                },
                _ => {
                    result = Some(XmlResult{ data: peref, errors: ~[]})
                }
        };
        result
    }

    fn get_char_ref_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('#'),       self.read());
        let peek_char = self.peek_chr();

        let radix;
        match peek_char {
            Char('x') => {
                radix = 16;
            },
            Char(_) => radix = 10,
            _ => return Some(XmlResult{
                                data: EndOfFile,
                                errors: ~[self.get_error(~"mock error")]
                            })
        }

        let is_radix = (radix == 16);
        let char_ref;

        if(is_radix){
            char_ref = self.process_hex_digit();
        }else{
            char_ref = self.process_digit();
        }
        let end_char_ref = self.peek_chr();

        match end_char_ref {
            Char(';') => { self.read();},
            _ => return Some(
                    XmlResult{
                        data: EndOfFile,
                        errors: ~[self.get_error(~"mock error char ref")]
                    })
        }

        let parse_char = from_str_radix::<uint>(char_ref.data, radix);

        match parse_char {
            Some(a) => {
                let ref_char = from_u32(a as u32);


                match ref_char {
                    Some(a) => { Some(XmlResult{
                                        data: CharRef(a),
                                        errors: ~[]})
                    }
                    _ => Some(XmlResult{
                                data: EndOfFile,
                                errors: ~[self.get_error(~"unparsable stuff")]
                            })
                }
            },
            None =>Some(XmlResult{
                            data: EndOfFile,
                            errors: ~[self.get_error(~"invalid stuff")]})
        }
    }

    fn get_sqbracket_left_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('['),       self.read());
        Some(XmlResult{data: LeftSqBracket, errors: ~[]})
    }


    fn get_sqbracket_right_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char(']'),       self.peek_chr());
        let result;
        if(~"]]>" == self.peek_str(3u).data){
            self.read_str(3u);
            result = Some(XmlResult{ data: DoctypeClose, errors: ~[]});
        }else{
            self.read();
            result = Some(XmlResult{data: RightSqBracket, errors: ~[]});
        }
        result
    }

    fn get_paren_left_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('('),       self.read());
        Some(XmlResult{data: LeftParen, errors: ~[]})
    }

    fn get_paren_right_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char(')'),       self.read());
        Some(XmlResult{data: LeftParen, errors: ~[]})
    }

    fn get_entity_def_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('#'),       self.read());
        let result;
        if(self.peek_str(8u).data == ~"REQUIRED"){
            result = Some(XmlResult{data: RequiredDecl, errors: ~[]});
        }else if(self.peek_str(7u).data == ~"IMPLIED"){
            result = Some(XmlResult{data: ImpliedDecl, errors: ~[]});
        }else if(self.peek_str(6u).data == ~"PCDATA"){
            result = Some(XmlResult{data: PCDataDecl, errors: ~[]});
        }else if(self.peek_str(5u).data == ~"FIXED"){
            result = Some(XmlResult{data: FixedDecl, errors: ~[]});
        }else{
            result = Some(XmlResult{data: Text(~"#"), errors: ~[]});
        }
        result
    }

    fn get_entity_or_element_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<!", self.read_str(2u).data);

        let result;
        if(self.peek_str(7u).data == ~"ELEMENT"){
            self.read_str(7u);
            result = Some(XmlResult{ data: ElementType, errors: ~[]});
        }else if(self.peek_str(6u).data == ~"ENTITY"){
            self.read_str(6u);
            result = Some(XmlResult{ data: EntityType, errors: ~[]});
        }else{
            result = Some(XmlResult{
                            data: ErrorToken(~"<!"),
                            errors: ~[self.get_error(~"Error in g_e_o_e_t")]
                         });
        }
        result
    }

    fn get_attlist_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<!", self.read_str(2u).data);
        let result;
        println(fmt!("Peek_str: %s", self.peek_str(7u).data));
        if(self.peek_str(7u).data == ~"ATTLIST"){
            self.read_str(7u);
            result = Some(XmlResult{ data: AttlistType, errors: ~[]});
        }else{
            result = Some(XmlResult{
                data: ErrorToken(~"<!"),
                errors: ~[self.get_error(~"Error in get_attlist_token")]
            });
        }
        result
    }

    fn get_notation_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<!", self.read_str(2u).data);
        let result;
        if(self.peek_str(8u).data == ~"NOTATION"){
            self.read_str(8u);
            result = Some(XmlResult{ data: NotationType, errors: ~[]});
        }else{
            result = Some(XmlResult{
                data: ErrorToken(~"<!"),
                errors: ~[self.get_error(~"Error in get_attlist_token")]
            });
        }
        result
    }

    fn get_star_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('*'),       self.read());
        Some(XmlResult{data: Star, errors: ~[]})
    }

    fn get_plus_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('+'),       self.read());
        Some(XmlResult{data: Plus, errors: ~[]})
    }

    fn get_pipe_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('|'),       self.read());
        Some(XmlResult{data: Pipe, errors: ~[]})
    }

    fn get_quote_token(&mut self) -> Option<XmlResult<XmlToken>> {
        None
    }

    fn get_text_token(&mut self) -> Option<XmlResult<XmlToken>> {
        None
    }

    fn get_pi_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<?",       self.read_str(2u).data);

        let name = self.peek_str(3u).data;

        if(name.eq_ignore_ascii_case("xml")){
            self.read_str(3u);
            return Some(XmlResult{ data: PrologStart, errors: ~[]});
        }else{
            let text = self.read_until_peek("?>").data;
            self.read_str(2u);
            return Some(XmlResult{ data: PI(text), errors: ~[]});
        }
    }

    fn get_doctype_end_token(&mut self) -> Option<XmlResult<XmlToken>> {
        let peek_str = self.peek_str(2u);

        if(peek_str.data == ~"!>"){
            self.read_str(2u);
            return Some(XmlResult{ data: DoctypeEnd, errors: ~[]})
        }else{
            let text = self.read_str(1u);
            return Some(XmlResult{ data: ErrorToken(text.data), errors: ~[self.get_error(~"mock error")]})
        }
    }

    fn get_right_bracket_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('>'), self.read());
        return Some(XmlResult{ data: GreaterBracket, errors: ~[]})
    }

    fn get_comment_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"<!-", self.read_str(3u).data);

        let peek_str = self.peek_str(1u).data;

        if(peek_str == ~"-"){
            self.read_str(1u);

            let text = self.process_comment().data;
            return Some(XmlResult{ data: Comment(text), errors: ~[]})
        }else{
            return Some(XmlResult{ data: ErrorToken(~"<!-"), errors: ~[self.get_error(~"mock error")]})
        }
    }

    fn process_comment(&mut self) -> XmlResult<~str> {
        let mut peek = self.peek_str(3u);
        let mut result = ~"";
        let mut found_end = false;
        let mut found_errs = ~[];

        while(!found_end){
            if(peek.data.starts_with("--") && peek.data == ~"-->"){
                self.read_str(3u);
                found_end = true;
            }else{
                if(peek.data.starts_with("--") && peek.data != ~"-->"){

                    found_errs.push(self.get_error(~"Can't have -- in comments"));
                }

                let extracted_char = self.read().extract_char();
                    match extracted_char {
                            None => {},
                            Some(a) => {result.push_char(a)}
                }
                peek = self.peek_str(3u);
            }
        }
        XmlResult{ data: result, errors: ~[]}
    }

    fn get_close_tag_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(~"</",  self.read_str(2u).data);
        return Some(XmlResult{ data: CloseTag, errors: ~[]})
    }

    fn get_empty_tag_token(&mut self) -> Option<XmlResult<XmlToken>> {
        assert_eq!(Char('/'), self.read());

        let result;
        if(self.read_str(1u).data == ~">"){
            result = Some(XmlResult{ data: EmptyTag, errors: ~[]});
        }else{
            result = Some(XmlResult{ data: ErrorToken(~"/"), errors: ~[]});
        }
        result
    }

    fn get_pi_end_token(&mut self) -> Option<XmlResult<XmlToken>> {
        let chr_assert = self.read();
        assert_eq!(Char('?'),   chr_assert);

        let chr_peek = self.peek_chr();
        match chr_peek {
           Char('>')    => {
                self.read();
                Some(XmlResult{ data: PrologEnd, errors: ~[]})
            },
           //TODO real error handling
            Char(a)
            | RestrictedChar(a)          => {
                let mut str_buff = ~"";
                str_buff.push_char(a);
                Some(XmlResult{
                        data: ErrorToken(str_buff),
                        errors: ~[self.get_error(~"mock error")]
                    })
            },
            EndFile => {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::*;
    use util::*;

    #[test]
    fn test_tokens(){
        let r = @BytesReader {
            bytes: "<?xml?> <?php stuff?><![CDATA[<test>]]>\t".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer =         XmlLexer::from_reader(r);

        assert_eq!(Some(XmlResult{ data: PrologStart, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: PrologEnd, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace(~" "), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: PI(~"php stuff"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CData(~"<test>"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace(~"\t"), errors: ~[] }),
                   lexer.next());

        let r2 = @BytesReader {
            bytes: "<![]]><!DOCTYPE &#x3123;&#212;%name;&name2;".as_bytes(),
            pos: @mut 0
        } as @Reader;

        lexer = XmlLexer::from_reader(r2);

        assert_eq!(Some(XmlResult{ data: DoctypeOpen, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: DoctypeClose, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: DoctypeStart, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: WhiteSpace(~" "), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CharRef('\u3123'), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: CharRef('\xD4'), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: PERef(~"name"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: EntityRef(~"name2"), errors: ~[] }),
                   lexer.next());

        let r3 = @BytesReader {
            bytes: "<!ENTITY<!NOTATION<!ELEMENT<!ATTLIST!><br>".as_bytes(),
            pos: @mut 0
        } as @Reader;

        lexer = XmlLexer::from_reader(r3);

        assert_eq!(Some(XmlResult{ data: EntityType, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NotationType, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: ElementType, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: AttlistType, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: DoctypeEnd, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: LessBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NameToken(~"br"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: GreaterBracket, errors: ~[] }),
                   lexer.next());
       

        let r4 = @BytesReader {
            bytes: "</br><e/><!-- -->()|+?*".as_bytes(),
            pos: @mut 0
        } as @Reader;

        lexer = XmlLexer::from_reader(r4);

        assert_eq!(Some(XmlResult{ data: CloseTag, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NameToken(~"br"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: GreaterBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: LessBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: NameToken(~"e"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: EmptyTag, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: Comment(~" "), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: LeftParen, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: RightParen, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: Pipe, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: QuestionMark, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: Star, errors: ~[] }),
                   lexer.next());

        let r5 = @BytesReader {
            bytes: "'quote'\"funny\"$BLA<&apos;".as_bytes(),
            pos: @mut 0
        } as @Reader;

        lexer = XmlLexer::from_reader(r5);

        assert_eq!(Some(XmlResult{ data: SingleQuoted(~"quote"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: DoubleQuoted(~"funny"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: Text(~"$BLA"), errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: LessBracket, errors: ~[] }),
                   lexer.next());
        assert_eq!(Some(XmlResult{ data: Text(~"'"), errors: ~[] }),
                   lexer.next());
    }

    #[test]
    fn test_multi_peek(){
        let r = @BytesReader {
            bytes: "123".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer =             XmlLexer::from_reader(r);
        assert_eq!(~"12",           lexer.peek_str(2u).data);
        assert_eq!(~"12",           lexer.peek_str(2u).data);
        assert_eq!(~"1",            lexer.read_str(1u).data);
        assert_eq!(~"23",           lexer.peek_str(2u).data);
        assert_eq!(~"23",           lexer.peek_str(2u).data);
    }

    #[test]
    fn test_peek_restricted(){
        let r = @BytesReader {
            bytes: "1\x0123".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer =             XmlLexer::from_reader(r);
        assert_eq!(~"1",            lexer.peek_str(2u).data);
        assert_eq!(~"12",           lexer.peek_str(3u).data);
    }

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
    fn test_whitespace(){
        let r = @BytesReader {
            bytes: "   \t\n  a ".as_bytes(),
            pos: @mut 0
        } as @Reader;

        let mut lexer = XmlLexer::from_reader(r);
        let whitespace = XmlResult{ data: WhiteSpace(~"   \t\n  "), errors: ~[]};
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
        assert_eq!(Char('a'),                   lexer.read());
        assert_eq!(1u,                          lexer.col);
        assert_eq!(1u,                          lexer.line);
        assert_eq!(~"s",                        lexer.read_str(1u).data);
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
        assert_eq!(XmlResult{ data: ~"as", errors: ~[XmlError{ line: 1u, col: 2u, msg: ~"Unexpected end of file", mark: None}]},
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

        assert_eq!(RestrictedChar('\x01'),      lexer.read());
        assert_eq!(RestrictedChar('\x04'),      lexer.read());
        assert_eq!(RestrictedChar('\x08'),      lexer.read());
        assert_eq!(Char('a'),                   lexer.read());
        assert_eq!(RestrictedChar('\x0B'),      lexer.read());
        assert_eq!(RestrictedChar('\x0C'),      lexer.read());
        assert_eq!(Char('b'),                   lexer.read());
        assert_eq!(RestrictedChar('\x0E'),      lexer.read());
        assert_eq!(RestrictedChar('\x10'),      lexer.read());
        assert_eq!(RestrictedChar('\x1F'),      lexer.read());
        assert_eq!(Char('c'),                   lexer.read());
        assert_eq!(RestrictedChar('\x7F'),      lexer.read());
        assert_eq!(RestrictedChar('\x80'),      lexer.read());
        assert_eq!(RestrictedChar('\x84'),      lexer.read());
        assert_eq!(Char('d'),                   lexer.read());
        assert_eq!(RestrictedChar('\x86'),      lexer.read());
        assert_eq!(RestrictedChar('\x90'),      lexer.read());
        assert_eq!(RestrictedChar('\x9F'),      lexer.read());
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
