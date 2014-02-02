
#[deriving(Eq, Clone, ToStr)]
/// If an error occurs while parsing some XML, this is the structure which is
/// returned
pub struct XmlError {
    /// The line number at which the error occurred
    line: uint,
    /// The column number at which the error occurred
    col: uint,
    /// A message describing the type of the error
    msg: ~str,
    /// Type of error
    //kind: ErrKind,
    /// Position and context of error in Context
    mark: Option<Mark>
}

#[deriving(Eq, Clone, ToStr)]
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

    pub fn decide(error: &ErrKind) -> ErrBehavior {
        match error {
            _ => Warn
        }
    }
}

#[deriving(Eq, Clone)]
/// This struct models the pretty error output
pub struct Mark {
    /// Message displayed in first in marked message
    offset_msg: ~str,
    /// Position of error within context string
    pos: uint,
    /// Length of the erroneous underline in context
    length: uint,
    /// Context that describes where error occured
    context: ~str
}

impl ToStr for Mark {
    /// Displays the string represenation to error mark
    /// E.g.
    ///       Thes text isn't spelt properly
    ///       ^~~~
    fn to_str(&self) -> ~str {
        let mut mark_str = self.offset_msg.clone();

        mark_str.push_str(self.context);
        mark_str.push_char('\n');
        let margin = self.pos + self.offset_msg.char_len();
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
        mark_str
    }
}

/// This methods removes all restricted character from a given ~str
/// It emites no error or causes failures
pub fn clean_restricted(input: ~str) -> ~str {
    let mut result = ~"";

    for c in input.chars() {
        if !is_restricted(&c) {
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
pub fn is_restricted(chr: &char) -> bool {
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

pub fn is_valid_char(chr: &char) -> bool {
    if is_restricted(chr) {
        false
    } else if is_char(chr) {
        true
    } else {
        false
    }
}

pub fn main() {
    let mark = Mark {
        offset_msg: ~"Error on line 1:  ",
        pos: 6,
        length: 4,
        context: ~" Well that went well"
    };
    println!("{}", mark.to_str());
}

#[cfg(test)]
mod tests{
    use super::{is_restricted};

    #[test]
    fn test_is_whitespace(){

    }

    #[test]
    fn test_is_char(){
    }

    #[test]
    fn test_is_restricted(){
        assert_eq!(true, is_restricted(&'\x0B'));
        assert_eq!(true, is_restricted(&'\x02'));
        assert_eq!(true, is_restricted(&'\x0C'));
        assert_eq!(true, is_restricted(&'\x0F'));
        assert_eq!(true, is_restricted(&'\x1F'));
        assert_eq!(true, is_restricted(&'\x7F'));
        assert_eq!(true, is_restricted(&'\x84'));
        assert_eq!(true, is_restricted(&'\x86'));
        assert_eq!(true, is_restricted(&'\x9A'));
        assert_eq!(true, is_restricted(&'\U0001FFFE'));
        assert_eq!(true, is_restricted(&'\U0001FFFF'));
        assert_eq!(true, is_restricted(&'\U0002FFFE'));
        assert_eq!(true, is_restricted(&'\U0002FFFF'));
        assert_eq!(true, is_restricted(&'\U0003FFFE'));
        assert_eq!(true, is_restricted(&'\U0003FFFF'));
        assert_eq!(true, is_restricted(&'\U0004FFFE'));
        assert_eq!(true, is_restricted(&'\U0004FFFF'));
        assert_eq!(true, is_restricted(&'\U0005FFFE'));
        assert_eq!(true, is_restricted(&'\U0005FFFF'));
        assert_eq!(true, is_restricted(&'\U0006FFFE'));
        assert_eq!(true, is_restricted(&'\U0006FFFF'));
        assert_eq!(true, is_restricted(&'\U0007FFFE'));
        assert_eq!(true, is_restricted(&'\U0007FFFF'));
        assert_eq!(true, is_restricted(&'\U0008FFFE'));
        assert_eq!(true, is_restricted(&'\U0008FFFF'));
        assert_eq!(true, is_restricted(&'\U0009FFFE'));
        assert_eq!(true, is_restricted(&'\U0009FFFF'));
        assert_eq!(true, is_restricted(&'\U000AFFFE'));
        assert_eq!(true, is_restricted(&'\U000AFFFF'));
        assert_eq!(true, is_restricted(&'\U000BFFFE'));
        assert_eq!(true, is_restricted(&'\U000BFFFF'));
        assert_eq!(true, is_restricted(&'\U000CFFFE'));
        assert_eq!(true, is_restricted(&'\U000CFFFF'));
        assert_eq!(true, is_restricted(&'\U000DFFFE'));
        assert_eq!(true, is_restricted(&'\U000DFFFF'));
        assert_eq!(true, is_restricted(&'\U000EFFFE'));
        assert_eq!(true, is_restricted(&'\U000EFFFF'));
        assert_eq!(true, is_restricted(&'\U000FFFFE'));
        assert_eq!(true, is_restricted(&'\U000FFFFF'));
    }
}
