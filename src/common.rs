use std::vec::Vec;


pub enum XToken {
}

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

fn main() {

}

