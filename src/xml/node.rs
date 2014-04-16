use std::vec::Vec;

/// A struct representing an XML root document
pub struct XmlDoc {
    // The document's root
    root: ~XmlElem,
    // The document's processing instructions
    pi: Vec<PINode>
}

/// Struct that represents what XML events
/// may be encountered during pull parsing
/// of documents
#[deriving(Clone,Eq,Show)]
pub enum XmlEvent {
    DeclEvent,
    ElemStart,
    ElemEnd,
    EmptyElem,
    PIEvent,
    TextEvent,
    CDataEvent
}


/// A struct representing an XML processing instruction
#[deriving(Clone,Eq,Show)]
pub struct PINode {
    /// The processing instruction's target
    target: ~str,
    /// The processing instruction's value
    /// Must not contain ?>
    value: ~str
}

#[deriving(Clone,Eq,Show)]
pub struct Doctype {
    /// Doctype name
    name: ~str
    // Internal Doctype definition
    //internal: Vec<DoctypeDecl>
}

pub enum DoctypeDecl {
    /// Element declaration
    ElementDecl(~DTDElem),
    /// Attlist declaration
    AttDecl(~DTDAttlist),
    /// Entity declaration
    EntityDecl(~DTDEntity),
    /// Notation declaration
    NotationDecl(~DTDNota)
}

pub struct DTDElem {
    name: ~str,
    spec: ContentSpec
}

pub enum ContentSpec {
    Empty,
    Any,
    Mixed(Vec<MixedSpec>),
    Children(~ChildSpec)
}

pub enum MixedSpec {
    PCData,
    Name
}

pub struct ChildSpec {
    multi: Multi,
    children: CPList,
    is_choice: bool
}

pub struct CPList {
    elems: Vec<ElemType>
}

pub enum ElemType {
    ChildName(~str, Multi),
    ChildChoice(~CPList, Multi),
    ChildSeq(~CPList, Multi)
}

pub enum Multi {
    Single,
    OneOrZero,
    ZeroOrMany,
    Many
}
pub struct DTDAttlist {
    name: ~str,
    defs: Vec<AttDef>
}

pub struct AttDef {
    name: ~str,
    att_type: AttType,
    default: DefaultVal
}

pub enum DefaultVal {
    Required,
    Implied,
    Fixed(Vec<AttVal>)
}

pub enum AttVal {
    AttText(~str),
    AttRef(~str)
}

pub enum EntVal {
    EntText(~str),
    PERef(~str),
    EntRef(~str)
}

pub enum AttType {
    CData,
    Id,
    Idref,
    Idrefs,
    Entity,
    Entities,
    Nmtoken,
    Nmtokens,
    Notation(Vec<~str>),
    Enumeration(Vec<~str>)
}

pub enum DTDEntity {

}

pub enum DTDNota {

}


/// A struct representing an XML element
#[deriving(Clone,Eq,Show)]
pub struct XmlElem {
    /// The element's name
    pub name: ~str,
    /// The element's namespace
    pub namespace: ~XmlNS,
    /// The element's `Attribute`s
    pub attributes: Vec<XmlAttr>,
    /// The element's child `XmlNode` nodes
    pub children: Vec<XNode>
}



/// A struct representing an XML attribute
#[deriving(Clone,Eq,Show)]
pub struct XmlAttr {
    /// The attribute's name
    pub name: ~str,
    /// The attribute's value
    pub value: ~str,
    /// The attribute's namespace
    pub namespace: ~XmlNS
}

#[deriving(Clone,Eq,Show)]
/// A struct that models an XML namespace
pub struct XmlNS {
    /// The namespace's shorthand name
    pub name: ~str,
    /// The namespace's uri value
    pub uri: ~str
}


/// General types
/// An Enum describing a XML Node
#[deriving(Clone,Eq,Show)]
pub enum XNode {
    XDoctype(~Doctype),
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
                    attributes: Vec::new(),
                    children: Vec::new()
            },
            pi: Vec::new()
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
    pub fn new(new_name : &str) -> XmlElem {
        XmlElem {
            name: new_name.to_owned(),
            namespace:~XmlNS{name: ~"", uri: ~""},
            attributes: Vec::new(),
            children: Vec::new()
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
