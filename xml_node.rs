

// General types
#[deriving(Clone,Eq)]
/// An Enum describing a XML Node
pub enum XmlNode {
    /// An XML Element
    XmlElem(~XmlElem),
    /// Character Data
    XmlText(~str),
    /// CDATA
    CDATANode(~str),
    /// A XML Comment
    CommentNode(~str),
    /// Processing Information
    PINode(~str)
}

#[deriving(Clone,Eq)]
/// A struct representing an XML element
pub struct XmlElem {
    /// The element's name
    name: ~str,
    /// The element's namespace
    namespace: ~XmlNS,
    /// The element's `Attribute`s
    attributes: ~[XmlAttr],
    /// The element's child `XmlNode` nodes
    children: ~[XmlNode]
}


#[deriving(Clone,Eq)]
/// A struct representing an XML root document
pub struct XmlDoc {
    // The document's root
    root: ~XmlElem,
    // The document's processing instructions
    pi: ~[XmlNode]
}

impl XmlDoc {
    pub fn new() -> XmlDoc {
        XmlDoc {
            root: ~XmlElem {
                    name:~"",
                    namespace:~XmlNS{name: ~"", uri: ~""}, 
                    attributes: ~[],
                    children: ~[]
            },
            pi: ~[]
        }
    }
}

#[deriving(Clone,Eq)]
/// A struct representing an XML attribute
pub struct XmlAttr {
    /// The attribute's name
    name: ~str,
    /// The attribute's value
    value: ~str,
    /// The attribute's namespace
    namespace: ~XmlNS
}

#[deriving(Clone,Eq)]
/// A struct that models an XML namespace
pub struct XmlNS {
    /// The namespace's shorthand name
    name: ~str,
    /// The namespace's uri value
    uri: ~str
}

#[deriving(Eq)]
/// Events returned by the Parser
pub enum Events {
    Document,
    /// Event indicating a start tag was found
    ElementStart {
        name: ~str, 
        attributes : ~[XmlAttr],
        namespace : ~XmlNS
    },
    /// Event indicating an end tag was found
    ElementEnd {
        name: ~str
    },
    /// Event indicating processing information was found
    ProcessInstruction(~str),
    /// Event indicating character data was found
    Text (~str),
    /// Event indicating CDATA was found
    CDATA (~str),
    /// Event indicating a comment was found
    Comment (~str)
    //EndOfFile
}


fn main() {
    
}