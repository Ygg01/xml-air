extern crate debug;

use std::fmt;
use std::string::String;
/// If an error occurs while parsing some XML, this is the structure which is
/// returned
#[deriving(PartialEq, Eq, Clone, Show)]
pub struct XmlError {
    /// The line number at which the error occurred
    pub line: uint,
    /// The column number at which the error occurred
    pub col: uint,
    /// A message describing the type of the error
    pub msg: String,
    /// Type of error
    //kind: ErrKind,
    /// Position and context of error in Context
    pub mark: Option<Mark>
}

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum ErrKind {
    NonDigitError,
    UnreadableChar,
    UnknownToken,
    IllegalChar,
    CharParsingError,
    NumParsingError,
    RestrictedCharError,
    MinMinInComment,
    PrematureEOF
}

pub enum ErrBehavior {
    Ignore,
    Warn,
    Fail
}

pub struct Config {
    unreadable_char:            Option<ErrBehavior>,
    double_minus_in_comment:    Option<ErrBehavior>,
    premature_eof:              Option<ErrBehavior>,
    non_digit_:                 Option<ErrBehavior>,
    num_parse:                  Option<ErrBehavior>,
    char_parse:                 Option<ErrBehavior>,
    illegal_char:               Option<ErrBehavior>
}

impl Config {
    pub fn default() -> Config {
        Config {
            unreadable_char:            Some(Warn),
            double_minus_in_comment:    Some(Warn),
            premature_eof:              Some(Warn),
            non_digit_:                 Some(Warn),
            num_parse:                  Some(Warn),
            char_parse:                 Some(Warn),
            illegal_char:               Some(Warn)
        }
    }
}

#[deriving(PartialEq, Eq, Clone)]
/// This struct models the pretty error output
pub struct Mark {
    /// Message displayed in first in marked message
    offset_msg: String,
    /// Position of error within context string
    pos: uint,
    /// Length of the erroneous underline in context
    length: uint,
    /// Context that describes where error occured
    context: String
}

impl fmt::Show for Mark {
    /// Displays the string represenation to error mark
    /// E.g.
    ///       Thes text isn't spelt properly
    ///       ^~~~
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut mark_str = self.offset_msg.clone();

        mark_str.push_str(self.context.as_slice());
        mark_str.push_char('\n');
        let margin = self.pos + self.offset_msg.as_slice().char_len();
        for _ in range(0, margin) {
            mark_str.push_char(' ');
        };

        let mut first_char = true;
        for _ in range(0, self.length) {
            if first_char {
                mark_str.push_char('^');
                first_char = false;
            } else {
                mark_str.push_char('~');
            }

        };
        mark_str.fmt(f)
    }
}

/// This methods removes all restricted character from a given String
/// It emites no error or causes failures
pub fn clean_restricted(input: String) -> String {
    let mut result = String::new();

    for c in input.as_slice().chars() {
        if !is_restricted_char(&c) {
            result.push_char(c);
        }
    }

    result
}
#[inline(always)]
/// Returns `true` if a character can be part of
/// a decimal number. False otherwise.
pub fn is_digit(input: &char) -> bool {
    match *input {
        '0'..'9' => true,
        _ => false
    }
}

#[inline(always)]
/// Returns `true` if a character can be part of
/// a hexadecimal number. False otherwise
pub fn is_hex_digit(input: &char) -> bool {
    match *input {
        '0'..'9'
        | 'a'..'f'
        | 'A'..'F' => true,
        _ => false
    }
}

#[inline]
/// Returns true  if a character belongs to a public id literal,
/// as defined in http://www.w3.org/TR/xml11/#NT-PubidChar
pub fn is_pubid_char(input: &char) -> bool {
    match *input {
        '\x20'
        | '\x0D'
        | '\x0A'
        | 'a'..'z'
        | 'A'..'Z'
        | '0'..'9' => true,
        a if("-'()+,./:=?;!*#@$_%".contains_char(a)) => true,
        _ => false
    }
}

#[inline]
/// Returns true if a character can be starting character of
/// XML document encoding.
///
/// See: http://www.w3.org/TR/xml11/#NT-EncName
pub fn is_encoding_start_char(input: &char) -> bool {
    match *input {
        'a'.. 'z'
        | 'A'..'Z' => true,
        _ => false
    }
}

#[inline]
/// Returns true if a character can be non-starting character of
/// XML document encoding.
///
/// See: http://www.w3.org/TR/xml11/#NT-EncName
pub fn is_encoding_char(input: &char) -> bool {
    match *input {
        'a'.. 'z'
        | 'A'..'Z'
        | '0'..'9'
        | '.' | '_'
        | '-' => true,
        _ => false
    }
}

/// Determines if a character can be a start of a name
/// token in XML. Name token can be attribute, tag and other names
///
/// See: http://www.w3.org/TR/xml11/#NT-NameStartChar
pub fn is_name_start(chr: &char) -> bool {
    match *chr {
        ':'
        | '_'
        | 'A'..'Z'
        | 'a'..'z'
        | '\xC0'..'\xD6'
        | '\xD8'..'\xF6'
        | '\xF8'..'\u02FF'
        | '\u0370'..'\u03FD'
        | '\u037F'..'\u1FFF'
        | '\u200C'..'\u200D'
        | '\u2070'..'\u218F'
        | '\u2C00'..'\u2FEF'
        | '\u3001'..'\uD7FF'
        | '\uF900'..'\uFDCF'
        | '\uFDF0'..'\uFFFD'
        | '\U00010000'..'\U000EFFFF' => true,
        _ => false
    }
}

/// Determines if a character can be in a name token
///
/// See:http://www.w3.org/TR/xml11/#NT-NameChar
pub fn is_name_char(chr: &char) -> bool {
    if is_name_start(chr) {
        return true;
    }
    match *chr {
        '-'
        | '.'
        | '0'..'9'
        | '\xB7'
        | '\u0300'..'\u036F'
        | '\u203F'..'\u2040' => true,
        _ =>false
    }
}

#[inline]
/// Determines if the character is allowed, if not, returns false
pub fn is_char(chr : &char) -> bool {
    match *chr {
        '\x01' .. '\uD7FF'
        | '\uE000' .. '\uFFFD'
        | '\U00010000' .. '\U0010FFFF' => true,
        _ => false
    }
}

#[inline]
/// Determines if the character is one of allowed whitespaces
pub fn is_whitespace(chr: &char) -> bool {
    *chr == ' ' || *chr == '\t' || *chr == '\n' || *chr == '\r'
}

#[inline]
/// This method verifies if the character is a restricted character
/// According to http://www.w3.org/TR/xml11/#NT-Char
/// Restricted character include anything in the range of
/// [#x1-#x8], [#xB-#xC], [#xE-#x1F], [#x7F-#x84], [#x86-#x9F]
/// [#x1FFFE-#x1FFFF], [#x2FFFE-#x2FFFF], [#x3FFFE-#x3FFFF],
/// [#x4FFFE-#x4FFFF], [#x5FFFE-#x5FFFF], [#x6FFFE-#x6FFFF],
/// [#x7FFFE-#x7FFFF], [#x8FFFE-#x8FFFF], [#x9FFFE-#x9FFFF],
/// [#xAFFFE-#xAFFFF], [#xBFFFE-#xBFFFF], [#xCFFFE-#xCFFFF],
/// [#xDFFFE-#xDFFFF], [#xEFFFE-#xEFFFF], [#xFFFFE-#xFFFFF],
/// [#x10FFFE-#x10FFFF].
pub fn is_restricted_char(chr: &char) -> bool {
    match *chr {
        '\x01'..'\x08'
        | '\x0B'.. '\x0C'
        | '\x0E'.. '\x1F'
        | '\x7F'.. '\x84'
        | '\x86'.. '\x9F'
        | '\U0001FFFE' | '\U0001FFFF'
        | '\U0002FFFE' | '\U0002FFFF'
        | '\U0003FFFE' | '\U0003FFFF'
        | '\U0004FFFE' | '\U0004FFFF'
        | '\U0005FFFE' | '\U0005FFFF'
        | '\U0006FFFE' | '\U0006FFFF'
        | '\U0007FFFE' | '\U0007FFFF'
        | '\U0008FFFE' | '\U0008FFFF'
        | '\U0009FFFE' | '\U0009FFFF'
        | '\U000AFFFE' | '\U000AFFFF'
        | '\U000BFFFE' | '\U000BFFFF'
        | '\U000CFFFE' | '\U000CFFFF'
        | '\U000DFFFE' | '\U000DFFFF'
        | '\U000EFFFE' | '\U000EFFFF'
        | '\U000FFFFE' | '\U000FFFFF' => true,
        _ => false
    }
}
/// This trait is a temporary shim, until Rust readds
/// pop_char, shift_char to String or similar structure

pub trait PopShiftShim {
    fn pop_char_shim(&mut self) -> Option<char>;
    fn shift_char_shim(&mut self) -> Option<char>;
}

impl PopShiftShim for String {
    fn  shift_char_shim(&mut self) -> Option<char> {
        let mut result;
        if self.as_slice() == "" {
            result = None;
        } else {
            let s = self.clone();
            let mut pop = None;
            let mut rest = String::new();
            let mut is_first = true;

            for chr in s.as_slice().chars() {
                if is_first {
                    pop = Some(chr);
                    is_first = false;
                } else {
                    rest.push_char(chr);
                }
            }


            self.truncate(0);
            self.push_str(rest.as_slice());

            result = pop;
        }
        result
    }

    fn pop_char_shim(&mut self) -> Option<char> {
        let mut result;
        if self.as_slice() == "" {
            result = None;
        } else {
            let s = self.clone();
            let mut shift = None;
            let mut rest = String::new();


            let char_len = s.as_slice().char_len();
            let mut i = 0;

            for chr in s.as_slice().chars() {
                if i == char_len-1 {
                    shift = Some(chr);
                } else {
                    rest.push_char(chr);
                }
                i += 1;

            }

            self.truncate(0);
            self.push_str(rest.as_slice());

            result = shift;
        }
        result
    }
}



#[cfg(test)]
mod test {

    use super::{is_restricted_char};
    use super::{PopShiftShim};
    #[test]
    fn name(){
        assert_eq!(true, is_restricted_char(&'\x0B'));
    }

    #[test]
    fn test_pop_char(){
        let s = "华b¢€𤭢";
        let mut buf = String::from_str(s);
        let rez = buf.pop_char_shim();

        assert_eq!(Some('𤭢'), rez);
        assert_eq!("华b¢€", buf.as_slice());
    }

    #[test]
    fn test_shift_char(){
        let s = "华b¢€𤭢";
        let mut buf = String::from_str(s);
        let rez = buf.shift_char_shim();

        assert_eq!(Some('华'), rez);
        assert_eq!("b¢€𤭢", buf.as_slice());
    }

}

