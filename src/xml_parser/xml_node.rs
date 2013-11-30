#[deriving(Clone,Eq)]
/// A struct representing an XML root document
pub struct XmlDoc {
    // The document's root
    root: ~XmlElem,
    // The document's processing instructions
    pi: ~[PINode]
}

#[deriving(Clone,Eq,ToStr)]
/// A struct representing an XML processing instruction
pub struct PINode {
    /// The processing instruction's target
    target: ~str,
    /// The processing instruction's value
    /// Must not contain ?>
    value: ~str
}

#[deriving(Clone,Eq,ToStr)]
/// A struct representing an XML element
pub struct XmlElem {
    /// The element's name
    name: ~str,
    /// The element's namespace
    namespace: ~XmlNS,
    /// The element's `Attribute`s
    attributes: ~[XmlAttr],
    /// The element's child `XmlNode` nodes
    children: ~[XNode]
}


#[deriving(Clone,Eq,ToStr)]
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


// General types
#[deriving(Clone,Eq,ToStr)]
/// An Enum describing a XML Node
pub enum XNode {
    /// An XML Element
    XElem(~XmlElem),
    /// Character Data
    XText(~str),
    /// CDATA
    XCdata(~str),
    /// A XML Comment
    XComment(~str),
    /// Processing Information
    XPi(~PINode)
}

fn main() {
    
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

impl XmlElem {
    pub fn new(new_name : ~str) -> XmlElem {
        XmlElem {
                    name:new_name,
                    namespace:~XmlNS{name: ~"", uri: ~""},
                    attributes: ~[],
                    children: ~[]
        }
    }
}

impl PINode {
    pub fn to_str(&self) -> ~str {
       format!("<?{} {} ?>", self.target, self.value)
    }
}


impl XmlNS {
    pub fn to_str(&self) -> ~str {
        ~""
    }
}


#[cfg(test)]
mod tests{
    use super::{PINode};

    #[test]
    fn test_pi_to_str(){
        let pi = ~PINode { target: ~"php", value: ~"echo"};
        assert_eq!(~"<?php echo ?>",pi.to_str())
    }

    #[test]
    fn test_cdata_to_str(){

    }
}