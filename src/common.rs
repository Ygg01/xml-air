use std::vec::Vec;
use std::fmt;
use std::string::String;

/// A struct representing an XML root document
pub struct XDoc {
    // The document's root
    root: XElem,
    // The document's processing instructions
    pi: Vec<XPi>
}



/// A struct representing an XML processing instruction
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct XPi {
    /// The processing instruction's target
    target: String,
    /// The processing instruction's value
    /// Must not contain ?>
    value: String
}

#[deriving(Clone, PartialEq, Eq, Show)]
pub struct XDoctype {
    /// Doctype name
    name: String
}


/// A struct representing an XML element
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct XElem {
    /// The element's name
    pub name: String,
    /// The element's namespace
    pub namespace: XmlNS,
    /// The element's `Attribute`s
    pub attributes: Vec<XmlAttr>,
    /// The element's child `XmlNode` nodes
    pub children: Vec<XNode>
}



/// A struct representing an XML attribute
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct XmlAttr {
    /// The attribute's name
    pub name: String,
    /// The attribute's value
    pub value: String,
    /// The attribute's namespace
    pub namespace: XmlNS
}

#[deriving(Clone, PartialEq, Eq, Show)]
/// A struct that models an XML namespace
pub struct XmlNS {
    /// The namespace's shorthand name
    pub name: String,
    /// The namespace's uri value
    pub uri: String
}


/// General types
/// An Enum describing a XML Node
#[deriving(Clone, PartialEq, Eq, Show)]
pub enum XNode {
}


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
    ///       Thes text contains an error.
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

fn main() {

}

