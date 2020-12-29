use displaydoc::Display;
use std::error::Error;

#[derive(Debug, PartialEq, Display)]
pub enum InvalidRule {
    /// Only CONSTRUCT statements can be converted to rify rules.
    MustBeConstruct,
    /// FROM statements are not allowed.
    IllegalFrom,
    /// Base iri is not allowed.
    IllegalBaseIri,
    /// Only Basic Graph Patterns are allowed.
    MustBeBasicGraphPattern,
    /// Path patterns are not allowed.
    IllegalPathPattern,
    #[doc = "A variable exists in the construct clause that does not exist in the WHERE clause. \
             Rify does not allow this. The variable in question is called \"{name}\"."]
    UnboundImplied { name: String },
    #[doc = "An unbound node exists with the same name as a blank node. This is not allowed \
             because blank nodes are implicitly converted to unbound nodes. Consider renaming \
             the blank node \"_:{name}\"."]
    NameCollision { name: String },
    #[doc = "A blank node called \"{name}\" was found in the output portion of the CONSTRUCT \
             clause. Blank nodes in the output of a rule are a footgun so they are not allowed."]
    BlankNodeImplied { name: String },
}

impl Error for InvalidRule {}

pub type Iri = String;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RdfNode {
    Blank(String),
    Iri(Iri),
    Literal {
        value: String,
        datatype: Iri,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },
}
