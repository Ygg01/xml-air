

// General types
#[deriving(Clone,Eq)]
/// An Enum describing a XML Node
pub enum XmlNode {
    /// An XML Element
    XmlElem(~XmlElem),
    /// Character Data
    XmlText(~str),
    /// CDATA
    XmlCDATA(~str),
    /// A XML Comment
    XmlComment(~str),
    /// Processing Information
    PINode(~PINode)
}

#[deriving(Clone,Eq)]
/// A struct representing an XML processing instruction
pub struct PINode {
    /// The processing instruction's target
    target: ~str,
    /// The processing instruction's value
    /// Must not contain ?>
    value: ~str
}

impl PINode {
    pub fn to_str(&self) -> ~str {
       fmt!("<?%s %s ?>", self.target, self.value)
    }
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
    pi: ~[PINode]
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

    pub fn to_str(&self) -> ~str {
        let mut ret = ~"";
        for e in self.pi.iter() {
            ret = ret + e.to_str();
        }
        ret
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

impl XmlNS {
    pub fn to_str() -> ~str {
        ~""
    }
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


#[test]
fn test_pi(){
    let pi = ~PINode { target: ~"php", value: ~"echo"};
    assert_eq!(~"<?php echo ?>",pi.to_str())
}