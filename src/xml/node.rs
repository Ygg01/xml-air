use std::vec::Vec;

/// A struct representing an XML root document
pub struct XmlDoc {
    // The document's root
    root: XmlElem,
    // The document's processing instructions
    pi: Vec<PINode>
}



/// A struct representing an XML processing instruction
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct PINode {
    /// The processing instruction's target
    target: String,
    /// The processing instruction's value
    /// Must not contain ?>
    value: String
}

#[deriving(Clone, PartialEq, Eq, Show)]
pub struct Doctype {
    /// Doctype name
    name: String
    // Internal Doctype definition
    //internal: Vec<DoctypeDecl>
}

pub enum DoctypeDecl {
    /// Element declaration
    ElementDecl(DTDElem),
    /// Attlist declaration
    AttDecl(DTDAttlist),
    /// Entity declaration
    EntityDecl(DTDEntity),
    /// Notation declaration
    NotationDecl(DTDNota)
}

pub struct DTDElem {
    name: String,
    spec: ContentSpec
}

pub enum ContentSpec {
    Empty,
    Any,
    Mixed(Vec<MixedSpec>),
    Children(ChildSpec)
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
    ChildName(String, Multi),
    ChildChoice(CPList, Multi),
    ChildSeq(CPList, Multi)
}

pub enum Multi {
    Single,
    OneOrZero,
    ZeroOrMany,
    Many
}
pub struct DTDAttlist {
    name: String,
    defs: Vec<AttDef>
}

pub struct AttDef {
    name: String,
    att_type: AttType,
    default: DefaultVal
}

pub enum DefaultVal {
    Required,
    Implied,
    Fixed(Vec<AttVal>)
}

pub enum AttVal {
    AttText(String),
    AttRef(String)
}

pub enum EntVal {
    EntText(String),
    PERef(String),
    EntRef(String)
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
    Notation(Vec<String>),
    Enumeration(Vec<String>)
}

pub enum DTDEntity {

}

pub enum DTDNota {

}


/// A struct representing an XML element
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct XmlElem {
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
    XDoctype(Doctype),
    /// An XML Element
    XElem(XmlElem),
    /// Character Data
    XText(String),
    /// CDATA
    XCdata(String),
    /// A XML Comment
    XComment(String),
    /// Processing Information
    XPi(PINode)
}

fn main() {

}



impl XmlDoc {
    pub fn new() -> XmlDoc {
        XmlDoc {
            root: XmlElem {
                    name: String::new(),
                    namespace: XmlNS{name: String::new(), uri: String::new()},
                    attributes: Vec::new(),
                    children: Vec::new()
            },
            pi: Vec::new()
        }
    }

    pub fn to_str(&self) -> String {
        let mut ret = String::new();
        ret
    }
}

impl XmlElem {
    pub fn new(new_name : &str) -> XmlElem {
        XmlElem {
            name: new_name.to_owned(),
            namespace: XmlNS{name: String::new(), uri: String::new()},
            attributes: Vec::new(),
            children: Vec::new()
        }
    }
}

impl PINode {
    pub fn to_str(&self) -> String {
       format!("<?{} {} ?>", self.target, self.value)
    }
}


impl XmlNS {
    pub fn to_str(&self) -> String {
        String::new()
    }
}
