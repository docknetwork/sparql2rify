use crate::types;
use crate::RdfNode;
use oxigraph::model::{Literal, LiteralContent, Term};
use oxigraph::sparql::algebra::{
    NamedNodeOrVariable, TermOrVariable, TripleOrPathPattern, TriplePattern,
};

/// try to represent a basic graph pattern as triples only. If the pattern contains path items
/// return Err
pub fn as_triples(bgp: &[TripleOrPathPattern]) -> Result<Vec<TriplePattern>, types::InvalidRule> {
    bgp.iter()
        .map(|trpl| match trpl {
            TripleOrPathPattern::Triple(tp @ TriplePattern { .. }) => Ok(tp.clone()),
            TripleOrPathPattern::Path(_) => Err(types::InvalidRule::IllegalPathPattern),
        })
        .collect()
}

/// convert an oxigraph basic graph pattern to a graph usable in as a rify `if_all` or `then` clause
pub fn to_rify_pattern(bgp: &[TriplePattern]) -> Vec<rify::Claim<rify::Entity<String, RdfNode>>> {
    bgp.iter().map(to_rify_triple).collect()
}

fn to_rify_triple(trpl: &TriplePattern) -> rify::Claim<rify::Entity<String, RdfNode>> {
    let TriplePattern {
        subject,
        predicate,
        object,
    } = trpl;
    [
        tov_to_rify_entity(subject),
        nnov_to_rify_entity(predicate),
        tov_to_rify_entity(object),
    ]
}

fn tov_to_rify_entity(patt: &TermOrVariable) -> rify::Entity<String, types::RdfNode> {
    match patt {
        TermOrVariable::Term(t) => rify::Entity::Bound(t.clone().into()),
        TermOrVariable::Variable(v) => rify::Entity::Unbound(v.name.clone()),
    }
}

fn nnov_to_rify_entity(patt: &NamedNodeOrVariable) -> rify::Entity<String, types::RdfNode> {
    match patt {
        NamedNodeOrVariable::NamedNode(nn) => {
            rify::Entity::Bound(types::RdfNode::Iri(nn.iri.clone()))
        }
        NamedNodeOrVariable::Variable(v) => rify::Entity::Unbound(v.name.clone()),
    }
}

impl From<Term> for RdfNode {
    fn from(t: Term) -> Self {
        match t {
            Term::NamedNode(iri) => Self::Iri(iri.iri),
            Term::BlankNode(bn) => Self::Blank(bn.as_str().to_string()),
            Term::Literal(Literal {
                0: LiteralContent::String(value),
            }) => Self::Literal {
                value,
                datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                language: None,
            },
            Term::Literal(Literal {
                0: LiteralContent::LanguageTaggedString { value, language },
            }) => Self::Literal {
                value,
                datatype: "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString".to_string(),
                language: Some(language),
            },
            Term::Literal(Literal {
                0: LiteralContent::TypedLiteral { value, datatype },
            }) => Self::Literal {
                value,
                datatype: datatype.iri,
                language: None,
            },
        }
    }
}

impl From<rify::InvalidRule<String>> for types::InvalidRule {
    fn from(ir: rify::InvalidRule<String>) -> Self {
        match ir {
            rify::InvalidRule::UnboundImplied(name) => Self::UnboundImplied { name },
        }
    }
}
