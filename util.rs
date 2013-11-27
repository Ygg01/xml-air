use std::str::*;

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
    ///TODO Type of error

    /// Position and context of error in Context
    mark: Option<Mark>
}

#[deriving(Eq, Clone, ToStr)]
enum ErrKind {
    RestrictedCharError,
    EndOfFileError
}

#[deriving(Eq, Clone, ToStr)]
pub struct Mark {
    pos: uint,

    context: ~str
}

//TODO replace this with
pub enum XmlResult2<'self, T> {
    Data(T),
    Recoverable(T, &'self [XmlError]),
    FatalError(XmlError)
}

#[deriving(Eq, Clone, ToStr)]
pub struct XmlResult<T> {
    data: T,
    errors: ~[XmlError]
}

#[inline]
/// Escapes unallowed character //TODO CHECK WHICH
pub fn escape(input: &str) -> ~str {
    let mut result = str::with_capacity(input.len());

    for c in input.iter() {
        match c {
            '&'     => result.push_str("&amp;"),
            '<'     => result.push_str("&lt;"),
            '>'     => result.push_str("&gt;"),
            '\''    => result.push_str("&apos;"),
            '"'     => result.push_str("&quot;"),
            chr     => result.push_char(chr)
        }
    }
    result
}

#[inline]
/// Unescapes all valid XML entities in a string.
pub fn unescape(input: &str) -> ~str {
    let tmp = str::replace(input, "&quot;", "\"");
    let tmp = str::replace(tmp, "&apos;", "'");
    let tmp = str::replace(tmp, "&gt;", ">");
    let tmp = str::replace(tmp, "&lt;", "<");
    str::replace(tmp, "&amp;", "&")
}

pub fn is_digit(input: &char) -> bool {
    match *input {
        '0'..'9' => true,
        _ => false
    }
}
#[inline]
pub fn is_hex_digit(input: &char) -> bool {
    match *input {
        '0'..'9'
        | 'A'..'F' => true,
        _ => false
    }
}

#[inline]
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

pub fn is_name_char(chr: &char) -> bool {
    if(is_name_start(chr)){
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
pub fn is_whitespace(chr: char) -> bool {
    chr == ' ' || chr == '\t' || chr == '\n' || chr == '\r'
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


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_is_whitespace(){
        
    }

    #[test]
    fn test_is_char(){
        
    }

    #[test]
    fn test_is_restricted(){
        assert_eq!(true, is_restricted(~'\x0B'));
        assert_eq!(true, is_restricted(~'\x02'));
        assert_eq!(true, is_restricted(~'\x0C'));
        assert_eq!(true, is_restricted(~'\x0F'));
        assert_eq!(true, is_restricted(~'\x1F'));
        assert_eq!(true, is_restricted(~'\x7F'));
        assert_eq!(true, is_restricted(~'\x84'));
        assert_eq!(true, is_restricted(~'\x86'));
        assert_eq!(true, is_restricted(~'\x9A'));
        assert_eq!(true, is_restricted(~'\U0001FFFE'));
        assert_eq!(true, is_restricted(~'\U0001FFFF'));
        assert_eq!(true, is_restricted(~'\U0002FFFE'));
        assert_eq!(true, is_restricted(~'\U0002FFFF'));
        assert_eq!(true, is_restricted(~'\U0003FFFE'));
        assert_eq!(true, is_restricted(~'\U0003FFFF'));
        assert_eq!(true, is_restricted(~'\U0004FFFE'));
        assert_eq!(true, is_restricted(~'\U0004FFFF'));
        assert_eq!(true, is_restricted(~'\U0005FFFE'));
        assert_eq!(true, is_restricted(~'\U0005FFFF'));
        assert_eq!(true, is_restricted(~'\U0006FFFE'));
        assert_eq!(true, is_restricted(~'\U0006FFFF'));
        assert_eq!(true, is_restricted(~'\U0007FFFE'));
        assert_eq!(true, is_restricted(~'\U0007FFFF'));
        assert_eq!(true, is_restricted(~'\U0008FFFE'));
        assert_eq!(true, is_restricted(~'\U0008FFFF'));
        assert_eq!(true, is_restricted(~'\U0009FFFE'));
        assert_eq!(true, is_restricted(~'\U0009FFFF'));
        assert_eq!(true, is_restricted(~'\U000AFFFE'));
        assert_eq!(true, is_restricted(~'\U000AFFFF'));
        assert_eq!(true, is_restricted(~'\U000BFFFE'));
        assert_eq!(true, is_restricted(~'\U000BFFFF'));
        assert_eq!(true, is_restricted(~'\U000CFFFE'));
        assert_eq!(true, is_restricted(~'\U000CFFFF'));
        assert_eq!(true, is_restricted(~'\U000DFFFE'));
        assert_eq!(true, is_restricted(~'\U000DFFFF'));
        assert_eq!(true, is_restricted(~'\U000EFFFE'));
        assert_eq!(true, is_restricted(~'\U000EFFFF'));
        assert_eq!(true, is_restricted(~'\U000FFFFE'));
        assert_eq!(true, is_restricted(~'\U000FFFFF'));
    }
}