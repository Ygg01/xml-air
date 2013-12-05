use std::str::{with_capacity,replace};


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
enum ErrKind {
    RestrictedCharError,
    EndOfFileError
}

#[deriving(Eq, Clone)]
pub struct Mark {
    pos: uint,
    length: uint,
    context: ~str
}

impl ToStr for Mark {
    fn to_str(&self) -> ~str {
        let mut mark_str = ~"";
        mark_str.push_str(self.context);
        mark_str.push_char('\n');
        self.pos.times(|| {
            mark_str.push_char(' ');
        });
        let mut first_char = true;
        self.length.times (|| {
            if first_char {
                mark_str.push_char('^');
                first_char = false;
            } else {
                mark_str.push_char('~');
            }

        });
        mark_str
    }
}

    Data(D),
    Warning(D, ~[XmlError]),
    Failure(D, ~[XmlError]),
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
    let mut result = with_capacity(input.len());

    for c in input.chars() {
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

    /// This methods removes all restricted character from a given XmlResult<~str>,
    /// Without changing errors
    pub fn clean_restricted(input: XmlResult<~str>) -> XmlResult<~str> {
        let mut clean_str = ~"";

        for c in input.data.chars() {
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

#[inline]
/// Unescapes all valid XML entities in a string.
pub fn unescape(input: &str) -> ~str {
    let tmp = replace(input, "&quot;", "\"");
    let tmp = replace(tmp, "&apos;", "'");
    let tmp = replace(tmp, "&gt;", ">");
    let tmp = replace(tmp, "&lt;", "<");
    replace(tmp, "&amp;", "&")
}

#[inline]
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
pub fn is_encoding_start_char(input: &char) -> bool {
    match *input {
        'a'.. 'z'
        | 'A'..'Z' => true,
        _ => false
    }
}

#[inline]
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


pub fn main() {
    let mark = Mark {
        pos: 6,
        length: 4,
        context: ~" Well that went well"
    };
    println(mark.to_str());
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
