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
        mark_str.push('\n');
        let margin = self.pos + self.offset_msg.as_slice().char_len();
        for _ in range(0, margin) {
            mark_str.push(' ');
        };

        let mut first_char = true;
        for _ in range(0, self.length) {
            if first_char {
                mark_str.push('^');
                first_char = false;
            } else {
                mark_str.push('~');
            }

        };
        mark_str.fmt(f)
    }
}


