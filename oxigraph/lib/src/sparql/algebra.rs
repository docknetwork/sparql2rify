//! [SPARQL 1.1 Query Algebra](https://www.w3.org/TR/sparql11-query/#sparqlQuery) AST

use crate::model::*;
use crate::sparql::model::*;
use crate::sparql::parser::{parse_query, parse_update, ParseError};
use oxiri::Iri;
use rio_api::model as rio;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

/// A parsed [SPARQL query](https://www.w3.org/TR/sparql11-query/)
///
/// ```
/// use oxigraph::model::NamedNode;
/// use oxigraph::sparql::Query;
///
/// let query_str = "SELECT ?s ?p ?o WHERE { ?s ?p ?o . }";
/// let mut query = Query::parse(query_str, None)?;
///
/// assert_eq!(query.to_string(), query_str);
///
/// // We edit the query dataset specification
/// query.dataset_mut().set_default_graph(vec![NamedNode::new("http://example.com").unwrap().into()]);
/// assert_eq!(query.to_string(), "SELECT ?s ?p ?o FROM <http://example.com> WHERE { ?s ?p ?o . }");
/// # Result::Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct Query(pub QueryVariants);

impl Query {
    /// Parses a SPARQL query with an optional base IRI to resolve relative IRIs in the query
    pub fn parse(query: &str, base_iri: Option<&str>) -> Result<Self, ParseError> {
        parse_query(query, base_iri)
    }

    /// Returns [the query dataset specification](https://www.w3.org/TR/sparql11-query/#specifyingDataset)
    pub fn dataset(&self) -> &QueryDataset {
        match &self.0 {
            QueryVariants::Select { dataset, .. } => dataset,
            QueryVariants::Construct { dataset, .. } => dataset,
            QueryVariants::Describe { dataset, .. } => dataset,
            QueryVariants::Ask { dataset, .. } => dataset,
        }
    }

    /// Returns [the query dataset specification](https://www.w3.org/TR/sparql11-query/#specifyingDataset)
    pub fn dataset_mut(&mut self) -> &mut QueryDataset {
        match &mut self.0 {
            QueryVariants::Select { dataset, .. } => dataset,
            QueryVariants::Construct { dataset, .. } => dataset,
            QueryVariants::Describe { dataset, .. } => dataset,
            QueryVariants::Ask { dataset, .. } => dataset,
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Query {
    type Err = ParseError;

    fn from_str(query: &str) -> Result<Self, ParseError> {
        Self::parse(query, None)
    }
}

impl<'a> TryFrom<&'a str> for Query {
    type Error = ParseError;

    fn try_from(query: &str) -> Result<Self, ParseError> {
        Self::from_str(query)
    }
}

impl<'a> TryFrom<&'a String> for Query {
    type Error = ParseError;

    fn try_from(query: &String) -> Result<Self, ParseError> {
        Self::from_str(query)
    }
}

/// A parsed [SPARQL update](https://www.w3.org/TR/sparql11-update/)
///
/// ```
/// use oxigraph::sparql::Update;
///
/// let update_str = "CLEAR ALL ;";
/// let update = Update::parse(update_str, None)?;
///
/// assert_eq!(update.to_string().trim(), update_str);
/// # Result::Ok::<_, oxigraph::sparql::ParseError>(())
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct Update {
    pub(crate) base_iri: Option<Rc<Iri<String>>>,
    pub(crate) operations: Vec<GraphUpdateOperation>,
}

impl Update {
    /// Parses a SPARQL update with an optional base IRI to resolve relative IRIs in the query
    pub fn parse(update: &str, base_iri: Option<&str>) -> Result<Self, ParseError> {
        parse_update(update, base_iri)
    }
}

impl fmt::Display for Update {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(base_iri) = &self.base_iri {
            writeln!(f, "BASE <{}>", base_iri)?;
        }
        for update in &self.operations {
            writeln!(f, "{} ;", update)?;
        }
        Ok(())
    }
}

impl FromStr for Update {
    type Err = ParseError;

    fn from_str(update: &str) -> Result<Self, ParseError> {
        Self::parse(update, None)
    }
}

impl<'a> TryFrom<&'a str> for Update {
    type Error = ParseError;

    fn try_from(update: &str) -> Result<Self, ParseError> {
        Self::from_str(update)
    }
}

impl<'a> TryFrom<&'a String> for Update {
    type Error = ParseError;

    fn try_from(update: &String) -> Result<Self, ParseError> {
        Self::from_str(update)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum NamedNodeOrVariable {
    NamedNode(NamedNode),
    Variable(Variable),
}

impl fmt::Display for NamedNodeOrVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NamedNodeOrVariable::NamedNode(node) => node.fmt(f),
            NamedNodeOrVariable::Variable(var) => var.fmt(f),
        }
    }
}

impl From<NamedNode> for NamedNodeOrVariable {
    fn from(node: NamedNode) -> Self {
        NamedNodeOrVariable::NamedNode(node)
    }
}

impl From<NamedNodeRef<'_>> for NamedNodeOrVariable {
    fn from(node: NamedNodeRef<'_>) -> Self {
        NamedNodeOrVariable::NamedNode(node.into())
    }
}

impl From<Variable> for NamedNodeOrVariable {
    fn from(var: Variable) -> Self {
        NamedNodeOrVariable::Variable(var)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum TermOrVariable {
    Term(Term),
    Variable(Variable),
}

impl fmt::Display for TermOrVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TermOrVariable::Term(term) => term.fmt(f),
            TermOrVariable::Variable(var) => var.fmt(f),
        }
    }
}

impl From<NamedNode> for TermOrVariable {
    fn from(node: NamedNode) -> Self {
        TermOrVariable::Term(node.into())
    }
}

impl From<NamedNodeRef<'_>> for TermOrVariable {
    fn from(node: NamedNodeRef<'_>) -> Self {
        TermOrVariable::Term(node.into())
    }
}

impl From<BlankNode> for TermOrVariable {
    fn from(node: BlankNode) -> Self {
        TermOrVariable::Term(node.into())
    }
}

impl From<Literal> for TermOrVariable {
    fn from(literal: Literal) -> Self {
        TermOrVariable::Term(literal.into())
    }
}

impl From<Variable> for TermOrVariable {
    fn from(var: Variable) -> Self {
        TermOrVariable::Variable(var)
    }
}

impl From<Term> for TermOrVariable {
    fn from(term: Term) -> Self {
        TermOrVariable::Term(term)
    }
}

impl From<NamedNodeOrVariable> for TermOrVariable {
    fn from(element: NamedNodeOrVariable) -> Self {
        match element {
            NamedNodeOrVariable::NamedNode(node) => TermOrVariable::Term(node.into()),
            NamedNodeOrVariable::Variable(var) => TermOrVariable::Variable(var),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct StaticBindings {
    variables: Vec<Variable>,
    values: Vec<Vec<Option<Term>>>,
}

impl StaticBindings {
    pub fn new(variables: Vec<Variable>, values: Vec<Vec<Option<Term>>>) -> Self {
        Self { variables, values }
    }

    pub fn variables(&self) -> &[Variable] {
        &*self.variables
    }

    pub fn variables_iter(&self) -> impl Iterator<Item = &Variable> {
        self.variables.iter()
    }

    pub fn values_iter(&self) -> impl Iterator<Item = &Vec<Option<Term>>> {
        self.values.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Default for StaticBindings {
    fn default() -> Self {
        Self {
            variables: Vec::default(),
            values: Vec::default(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct TriplePattern {
    pub subject: TermOrVariable,
    pub predicate: NamedNodeOrVariable,
    pub object: TermOrVariable,
}

impl TriplePattern {
    pub fn new(
        subject: impl Into<TermOrVariable>,
        predicate: impl Into<NamedNodeOrVariable>,
        object: impl Into<TermOrVariable>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }
    }
}

impl fmt::Display for TriplePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} .", self.subject, self.predicate, self.object)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct QuadPattern {
    pub subject: TermOrVariable,
    pub predicate: NamedNodeOrVariable,
    pub object: TermOrVariable,
    pub graph_name: Option<NamedNodeOrVariable>,
}

impl QuadPattern {
    pub fn new(
        subject: impl Into<TermOrVariable>,
        predicate: impl Into<NamedNodeOrVariable>,
        object: impl Into<TermOrVariable>,
        graph_name: Option<NamedNodeOrVariable>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            graph_name,
        }
    }
}

impl fmt::Display for QuadPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(graph_name) = &self.graph_name {
            write!(
                f,
                "GRAPH {} {{ {} {} {} }}",
                graph_name, self.subject, self.predicate, self.object
            )
        } else {
            write!(f, "{} {} {} .", self.subject, self.predicate, self.object)
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum PropertyPath {
    PredicatePath(NamedNode),
    InversePath(Box<PropertyPath>),
    SequencePath(Box<PropertyPath>, Box<PropertyPath>),
    AlternativePath(Box<PropertyPath>, Box<PropertyPath>),
    ZeroOrMorePath(Box<PropertyPath>),
    OneOrMorePath(Box<PropertyPath>),
    ZeroOrOnePath(Box<PropertyPath>),
    NegatedPropertySet(Vec<NamedNode>),
}

impl fmt::Display for PropertyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyPath::PredicatePath(p) => write!(f, "link({})", p),
            PropertyPath::InversePath(p) => write!(f, "inv({})", p),
            PropertyPath::AlternativePath(a, b) => write!(f, "alt({}, {})", a, b),
            PropertyPath::SequencePath(a, b) => write!(f, "seq({}, {})", a, b),
            PropertyPath::ZeroOrMorePath(p) => write!(f, "ZeroOrMorePath({})", p),
            PropertyPath::OneOrMorePath(p) => write!(f, "OneOrMorePath({})", p),
            PropertyPath::ZeroOrOnePath(p) => write!(f, "ZeroOrOnePath({})", p),
            PropertyPath::NegatedPropertySet(p) => write!(
                f,
                "NPS({{ {} }})",
                p.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

struct SparqlPropertyPath<'a>(&'a PropertyPath);

impl<'a> fmt::Display for SparqlPropertyPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            PropertyPath::PredicatePath(p) => write!(f, "{}", p),
            PropertyPath::InversePath(p) => write!(f, "^{}", SparqlPropertyPath(&*p)),
            PropertyPath::SequencePath(a, b) => write!(
                f,
                "({} / {})",
                SparqlPropertyPath(&*a),
                SparqlPropertyPath(&*b)
            ),
            PropertyPath::AlternativePath(a, b) => write!(
                f,
                "({} | {})",
                SparqlPropertyPath(&*a),
                SparqlPropertyPath(&*b)
            ),
            PropertyPath::ZeroOrMorePath(p) => write!(f, "{}*", SparqlPropertyPath(&*p)),
            PropertyPath::OneOrMorePath(p) => write!(f, "{}+", SparqlPropertyPath(&*p)),
            PropertyPath::ZeroOrOnePath(p) => write!(f, "{}?", SparqlPropertyPath(&*p)),
            PropertyPath::NegatedPropertySet(p) => write!(
                f,
                "!({})",
                p.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ),
        }
    }
}

impl From<NamedNode> for PropertyPath {
    fn from(p: NamedNode) -> Self {
        PropertyPath::PredicatePath(p)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct PathPattern {
    pub subject: TermOrVariable,
    pub path: PropertyPath,
    pub object: TermOrVariable,
}

impl fmt::Display for PathPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path({} {} {})", self.subject, self.path, self.object)
    }
}

impl PathPattern {
    pub fn new(
        subject: impl Into<TermOrVariable>,
        path: impl Into<PropertyPath>,
        object: impl Into<TermOrVariable>,
    ) -> Self {
        Self {
            subject: subject.into(),
            path: path.into(),
            object: object.into(),
        }
    }
}

struct SparqlPathPattern<'a>(&'a PathPattern);

impl<'a> fmt::Display for SparqlPathPattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} .",
            self.0.subject,
            SparqlPropertyPath(&self.0.path),
            self.0.object
        )
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum TripleOrPathPattern {
    Triple(TriplePattern),
    Path(PathPattern),
}

impl TripleOrPathPattern {
    pub(crate) fn subject(&self) -> &TermOrVariable {
        match self {
            TripleOrPathPattern::Triple(t) => &t.subject,
            TripleOrPathPattern::Path(t) => &t.subject,
        }
    }

    pub(crate) fn object(&self) -> &TermOrVariable {
        match self {
            TripleOrPathPattern::Triple(t) => &t.object,
            TripleOrPathPattern::Path(t) => &t.object,
        }
    }
}

impl<'a> fmt::Display for TripleOrPathPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TripleOrPathPattern::Triple(tp) => write!(f, "{}", tp),
            TripleOrPathPattern::Path(ppp) => write!(f, "{}", ppp),
        }
    }
}

impl From<TriplePattern> for TripleOrPathPattern {
    fn from(tp: TriplePattern) -> Self {
        TripleOrPathPattern::Triple(tp)
    }
}

impl From<PathPattern> for TripleOrPathPattern {
    fn from(ppp: PathPattern) -> Self {
        TripleOrPathPattern::Path(ppp)
    }
}

struct SparqlTripleOrPathPattern<'a>(&'a TripleOrPathPattern);

impl<'a> fmt::Display for SparqlTripleOrPathPattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TripleOrPathPattern::Triple(tp) => write!(f, "{}", tp),
            TripleOrPathPattern::Path(ppp) => write!(f, "{}", SparqlPathPattern(ppp)),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Expression {
    NamedNode(NamedNode),
    Literal(Literal),
    Variable(Variable),
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    Greater(Box<Expression>, Box<Expression>),
    GreaterOrEq(Box<Expression>, Box<Expression>),
    Lower(Box<Expression>, Box<Expression>),
    LowerOrEq(Box<Expression>, Box<Expression>),
    In(Box<Expression>, Vec<Expression>),
    NotIn(Box<Expression>, Vec<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    UnaryPlus(Box<Expression>),
    UnaryMinus(Box<Expression>),
    UnaryNot(Box<Expression>),
    FunctionCall(Function, Vec<Expression>),
    Exists(Box<GraphPattern>),
    Bound(Variable),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::NamedNode(node) => node.fmt(f),
            Expression::Literal(l) => l.fmt(f),
            Expression::Variable(var) => var.fmt(f),
            Expression::Or(a, b) => write!(f, "({} || {})", a, b),
            Expression::And(a, b) => write!(f, "({} && {})", a, b),
            Expression::Equal(a, b) => write!(f, "({} = {})", a, b),
            Expression::NotEqual(a, b) => write!(f, "({} != {})", a, b),
            Expression::Greater(a, b) => write!(f, "({} > {})", a, b),
            Expression::GreaterOrEq(a, b) => write!(f, "({} >= {})", a, b),
            Expression::Lower(a, b) => write!(f, "({} < {})", a, b),
            Expression::LowerOrEq(a, b) => write!(f, "({} <= {})", a, b),
            Expression::In(a, b) => write!(
                f,
                "({} IN ({}))",
                a,
                b.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expression::NotIn(a, b) => write!(
                f,
                "({} NOT IN ({}))",
                a,
                b.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expression::Add(a, b) => write!(f, "{} + {}", a, b),
            Expression::Sub(a, b) => write!(f, "{} - {}", a, b),
            Expression::Mul(a, b) => write!(f, "{} * {}", a, b),
            Expression::Div(a, b) => write!(f, "{} / {}", a, b),
            Expression::UnaryPlus(e) => write!(f, "+{}", e),
            Expression::UnaryMinus(e) => write!(f, "-{}", e),
            Expression::UnaryNot(e) => write!(f, "!{}", e),
            Expression::FunctionCall(function, parameters) => {
                write!(f, "{}(", function)?;
                let mut cont = false;
                for p in parameters {
                    if cont {
                        write!(f, ", ")?;
                    }
                    p.fmt(f)?;
                    cont = true;
                }
                write!(f, ")")
            }
            Expression::Exists(p) => write!(f, "EXISTS {{ {} }}", p),
            Expression::Bound(v) => write!(f, "BOUND({})", v),
        }
    }
}

impl From<NamedNode> for Expression {
    fn from(p: NamedNode) -> Self {
        Expression::NamedNode(p)
    }
}

impl From<Literal> for Expression {
    fn from(p: Literal) -> Self {
        Expression::Literal(p)
    }
}

impl From<Variable> for Expression {
    fn from(v: Variable) -> Self {
        Expression::Variable(v)
    }
}

struct SparqlExpression<'a>(&'a Expression);

impl<'a> fmt::Display for SparqlExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Expression::NamedNode(node) => node.fmt(f),
            Expression::Literal(l) => l.fmt(f),
            Expression::Variable(var) => var.fmt(f),
            Expression::Or(a, b) => write!(
                f,
                "({} || {})",
                SparqlExpression(&*a),
                SparqlExpression(&*b)
            ),
            Expression::And(a, b) => write!(
                f,
                "({} && {})",
                SparqlExpression(&*a),
                SparqlExpression(&*b)
            ),
            Expression::Equal(a, b) => {
                write!(f, "({} = {})", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::NotEqual(a, b) => write!(
                f,
                "({} != {})",
                SparqlExpression(&*a),
                SparqlExpression(&*b)
            ),
            Expression::Greater(a, b) => {
                write!(f, "({} > {})", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::GreaterOrEq(a, b) => write!(
                f,
                "({} >= {})",
                SparqlExpression(&*a),
                SparqlExpression(&*b)
            ),
            Expression::Lower(a, b) => {
                write!(f, "({} < {})", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::LowerOrEq(a, b) => write!(
                f,
                "({} <= {})",
                SparqlExpression(&*a),
                SparqlExpression(&*b)
            ),
            Expression::In(a, b) => write!(
                f,
                "({} IN ({}))",
                a,
                b.iter()
                    .map(|v| SparqlExpression(v).to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expression::NotIn(a, b) => write!(
                f,
                "({} NOT IN ({}))",
                a,
                b.iter()
                    .map(|v| SparqlExpression(v).to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expression::Add(a, b) => {
                write!(f, "{} + {}", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::Sub(a, b) => {
                write!(f, "{} - {}", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::Mul(a, b) => {
                write!(f, "{} * {}", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::Div(a, b) => {
                write!(f, "{} / {}", SparqlExpression(&*a), SparqlExpression(&*b))
            }
            Expression::UnaryPlus(e) => write!(f, "+{}", SparqlExpression(&*e)),
            Expression::UnaryMinus(e) => write!(f, "-{}", SparqlExpression(&*e)),
            Expression::UnaryNot(e) => match e.as_ref() {
                Expression::Exists(p) => write!(f, "NOT EXISTS {{ {} }}", SparqlGraphPattern(&*p)),
                e => write!(f, "!{}", e),
            },
            Expression::FunctionCall(function, parameters) => {
                write!(f, "{}(", function)?;
                let mut cont = false;
                for p in parameters {
                    if cont {
                        write!(f, ", ")?;
                    }
                    SparqlExpression(&*p).fmt(f)?;
                    cont = true;
                }
                write!(f, ")")
            }
            Expression::Bound(v) => write!(f, "BOUND({})", v),
            Expression::Exists(p) => write!(f, "EXISTS {{ {} }}", SparqlGraphPattern(&*p)),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Function {
    Str,
    Lang,
    LangMatches,
    Datatype,
    IRI,
    BNode,
    Rand,
    Abs,
    Ceil,
    Floor,
    Round,
    Concat,
    SubStr,
    StrLen,
    Replace,
    UCase,
    LCase,
    EncodeForURI,
    Contains,
    StrStarts,
    StrEnds,
    StrBefore,
    StrAfter,
    Year,
    Month,
    Day,
    Hours,
    Minutes,
    Seconds,
    Timezone,
    Tz,
    Now,
    UUID,
    StrUUID,
    MD5,
    SHA1,
    SHA256,
    SHA384,
    SHA512,
    Coalesce,
    If,
    StrLang,
    StrDT,
    SameTerm,
    IsIRI,
    IsBlank,
    IsLiteral,
    IsNumeric,
    Regex,
    Custom(NamedNode),
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Str => write!(f, "STR"),
            Function::Lang => write!(f, "LANG"),
            Function::LangMatches => write!(f, "LANGMATCHES"),
            Function::Datatype => write!(f, "DATATYPE"),
            Function::IRI => write!(f, "IRI"),
            Function::BNode => write!(f, "BNODE"),
            Function::Rand => write!(f, "RAND"),
            Function::Abs => write!(f, "ABS"),
            Function::Ceil => write!(f, "CEIL"),
            Function::Floor => write!(f, "FLOOR"),
            Function::Round => write!(f, "ROUND"),
            Function::Concat => write!(f, "CONCAT"),
            Function::SubStr => write!(f, "SUBSTR"),
            Function::StrLen => write!(f, "STRLEN"),
            Function::Replace => write!(f, "REPLACE"),
            Function::UCase => write!(f, "UCASE"),
            Function::LCase => write!(f, "LCASE"),
            Function::EncodeForURI => write!(f, "ENCODE_FOR_URI"),
            Function::Contains => write!(f, "CONTAINS"),
            Function::StrStarts => write!(f, "STRSTATS"),
            Function::StrEnds => write!(f, "STRENDS"),
            Function::StrBefore => write!(f, "STRBEFORE"),
            Function::StrAfter => write!(f, "STRAFTER"),
            Function::Year => write!(f, "YEAR"),
            Function::Month => write!(f, "MONTH"),
            Function::Day => write!(f, "DAY"),
            Function::Hours => write!(f, "HOURS"),
            Function::Minutes => write!(f, "MINUTES"),
            Function::Seconds => write!(f, "SECONDS"),
            Function::Timezone => write!(f, "TIMEZONE"),
            Function::Tz => write!(f, "TZ"),
            Function::Now => write!(f, "NOW"),
            Function::UUID => write!(f, "UUID"),
            Function::StrUUID => write!(f, "STRUUID"),
            Function::MD5 => write!(f, "MD5"),
            Function::SHA1 => write!(f, "SHA1"),
            Function::SHA256 => write!(f, "SHA256"),
            Function::SHA384 => write!(f, "SHA384"),
            Function::SHA512 => write!(f, "SHA512"),
            Function::Coalesce => write!(f, "COALESCE"),
            Function::If => write!(f, "IF"),
            Function::StrLang => write!(f, "STRLANG"),
            Function::StrDT => write!(f, "STRDT"),
            Function::SameTerm => write!(f, "sameTerm"),
            Function::IsIRI => write!(f, "isIRI"),
            Function::IsBlank => write!(f, "isBLANK"),
            Function::IsLiteral => write!(f, "isLITERAL"),
            Function::IsNumeric => write!(f, "isNUMERIC"),
            Function::Regex => write!(f, "REGEX"),
            Function::Custom(iri) => iri.fmt(f),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum GraphPattern {
    BGP(Vec<TripleOrPathPattern>),
    Join(Box<GraphPattern>, Box<GraphPattern>),
    LeftJoin(Box<GraphPattern>, Box<GraphPattern>, Option<Expression>),
    Filter(Expression, Box<GraphPattern>),
    Union(Box<GraphPattern>, Box<GraphPattern>),
    Graph(NamedNodeOrVariable, Box<GraphPattern>),
    Extend(Box<GraphPattern>, Variable, Expression),
    Minus(Box<GraphPattern>, Box<GraphPattern>),
    Service(NamedNodeOrVariable, Box<GraphPattern>, bool),
    AggregateJoin(GroupPattern, Vec<(Aggregation, Variable)>),
    Data(StaticBindings),
    OrderBy(Box<GraphPattern>, Vec<OrderComparator>),
    Project(Box<GraphPattern>, Vec<Variable>),
    Distinct(Box<GraphPattern>),
    Reduced(Box<GraphPattern>),
    Slice(Box<GraphPattern>, usize, Option<usize>),
}

impl fmt::Display for GraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphPattern::BGP(p) => write!(
                f,
                "BGP({})",
                p.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            GraphPattern::Join(a, b) => write!(f, "Join({}, {})", a, b),
            GraphPattern::LeftJoin(a, b, e) => {
                if let Some(e) = e {
                    write!(f, "LeftJoin({}, {}, {})", a, b, e)
                } else {
                    write!(f, "LeftJoin({}, {})", a, b)
                }
            }
            GraphPattern::Filter(e, p) => write!(f, "Filter({}, {})", e, p),
            GraphPattern::Union(a, b) => write!(f, "Union({}, {})", a, b),
            GraphPattern::Graph(g, p) => write!(f, "Graph({}, {})", g, p),
            GraphPattern::Extend(p, v, e) => write!(f, "Extend({}), {}, {})", p, v, e),
            GraphPattern::Minus(a, b) => write!(f, "Minus({}, {})", a, b),
            GraphPattern::Service(n, p, s) => write!(f, "Service({}, {}, {})", n, p, s),
            GraphPattern::AggregateJoin(g, a) => write!(
                f,
                "AggregateJoin({}, {})",
                g,
                a.iter()
                    .map(|(a, v)| format!("{}: {}", v, a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            GraphPattern::Data(bs) => {
                let variables = bs.variables();
                write!(f, "{{ ")?;
                for values in bs.values_iter() {
                    write!(f, "{{")?;
                    for i in 0..values.len() {
                        if let Some(ref val) = values[i] {
                            write!(f, " {} \u{2192} {} ", variables[i], val)?;
                        }
                    }
                    write!(f, "}}")?;
                }
                write!(f, "}}")
            }
            GraphPattern::OrderBy(l, o) => write!(
                f,
                "OrderBy({}, ({}))",
                l,
                o.iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            GraphPattern::Project(l, pv) => write!(
                f,
                "Project({}, ({}))",
                l,
                pv.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            GraphPattern::Distinct(l) => write!(f, "Distinct({})", l),
            GraphPattern::Reduced(l) => write!(f, "Reduce({})", l),
            GraphPattern::Slice(l, start, length) => write!(
                f,
                "Slice({}, {}, {})",
                l,
                start,
                length
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| '?'.to_string())
            ),
        }
    }
}

impl Default for GraphPattern {
    fn default() -> Self {
        GraphPattern::BGP(Vec::default())
    }
}

impl From<TripleOrPathPattern> for GraphPattern {
    fn from(p: TripleOrPathPattern) -> Self {
        GraphPattern::BGP(vec![p])
    }
}

impl GraphPattern {
    pub fn visible_variables(&self) -> BTreeSet<&Variable> {
        let mut vars = BTreeSet::default();
        self.add_visible_variables(&mut vars);
        vars
    }

    fn add_visible_variables<'a>(&'a self, vars: &mut BTreeSet<&'a Variable>) {
        match self {
            GraphPattern::BGP(p) => {
                for pattern in p {
                    match pattern {
                        TripleOrPathPattern::Triple(tp) => {
                            if let TermOrVariable::Variable(ref s) = tp.subject {
                                vars.insert(s);
                            }
                            if let NamedNodeOrVariable::Variable(ref p) = tp.predicate {
                                vars.insert(p);
                            }
                            if let TermOrVariable::Variable(ref o) = tp.object {
                                vars.insert(o);
                            }
                        }
                        TripleOrPathPattern::Path(ppp) => {
                            if let TermOrVariable::Variable(ref s) = ppp.subject {
                                vars.insert(s);
                            }
                            if let TermOrVariable::Variable(ref o) = ppp.object {
                                vars.insert(o);
                            }
                        }
                    }
                }
            }
            GraphPattern::Join(a, b) => {
                a.add_visible_variables(vars);
                b.add_visible_variables(vars);
            }
            GraphPattern::LeftJoin(a, b, _) => {
                a.add_visible_variables(vars);
                b.add_visible_variables(vars);
            }
            GraphPattern::Filter(_, p) => p.add_visible_variables(vars),
            GraphPattern::Union(a, b) => {
                a.add_visible_variables(vars);
                b.add_visible_variables(vars);
            }
            GraphPattern::Graph(g, p) => {
                if let NamedNodeOrVariable::Variable(ref g) = g {
                    vars.insert(g);
                }
                p.add_visible_variables(vars);
            }
            GraphPattern::Extend(p, v, _) => {
                vars.insert(v);
                p.add_visible_variables(vars);
            }
            GraphPattern::Minus(a, _) => a.add_visible_variables(vars),
            GraphPattern::Service(_, p, _) => p.add_visible_variables(vars),
            GraphPattern::AggregateJoin(_, a) => {
                for (_, v) in a {
                    vars.insert(v);
                }
            }
            GraphPattern::Data(b) => vars.extend(b.variables_iter()),
            GraphPattern::OrderBy(l, _) => l.add_visible_variables(vars),
            GraphPattern::Project(_, pv) => vars.extend(pv.iter()),
            GraphPattern::Distinct(l) => l.add_visible_variables(vars),
            GraphPattern::Reduced(l) => l.add_visible_variables(vars),
            GraphPattern::Slice(l, _, _) => l.add_visible_variables(vars),
        }
    }
}

struct SparqlGraphPattern<'a>(&'a GraphPattern);

impl<'a> fmt::Display for SparqlGraphPattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            GraphPattern::BGP(p) => {
                for pattern in p {
                    write!(f, "{}", SparqlTripleOrPathPattern(pattern))?
                }
                Ok(())
            }
            GraphPattern::Join(a, b) => write!(
                f,
                "{{ {} }} {{ {} }}",
                SparqlGraphPattern(&*a),
                SparqlGraphPattern(&*b)
            ),
            GraphPattern::LeftJoin(a, b, e) => {
                if let Some(e) = e {
                    write!(
                        f,
                        "{} OPTIONAL {{ {} FILTER({}) }}",
                        SparqlGraphPattern(&*a),
                        SparqlGraphPattern(&*b),
                        SparqlExpression(e)
                    )
                } else {
                    write!(
                        f,
                        "{} OPTIONAL {{ {} }}",
                        SparqlGraphPattern(&*a),
                        SparqlGraphPattern(&*b)
                    )
                }
            }
            GraphPattern::Filter(e, p) => write!(
                f,
                "{} FILTER({})",
                SparqlGraphPattern(&*p),
                SparqlExpression(e)
            ),
            GraphPattern::Union(a, b) => write!(
                f,
                "{{ {} }} UNION {{ {} }}",
                SparqlGraphPattern(&*a),
                SparqlGraphPattern(&*b),
            ),
            GraphPattern::Graph(g, p) => {
                write!(f, "GRAPH {} {{ {} }}", g, SparqlGraphPattern(&*p),)
            }
            GraphPattern::Extend(p, v, e) => write!(
                f,
                "{} BIND({} AS {})",
                SparqlGraphPattern(&*p),
                SparqlExpression(e),
                v
            ),
            GraphPattern::Minus(a, b) => write!(
                f,
                "{} MINUS {{ {} }}",
                SparqlGraphPattern(&*a),
                SparqlGraphPattern(&*b)
            ),
            GraphPattern::Service(n, p, s) => {
                if *s {
                    write!(f, "SERVICE SILENT {} {{ {} }}", n, SparqlGraphPattern(&*p))
                } else {
                    write!(f, "SERVICE {} {{ {} }}", n, SparqlGraphPattern(&*p))
                }
            }
            GraphPattern::Data(bs) => {
                if bs.is_empty() {
                    Ok(())
                } else {
                    write!(f, "VALUES ( ")?;
                    for var in bs.variables() {
                        write!(f, "{} ", var)?;
                    }
                    write!(f, ") {{ ")?;
                    for values in bs.values_iter() {
                        write!(f, "( ")?;
                        for val in values {
                            match val {
                                Some(val) => write!(f, "{} ", val),
                                None => write!(f, "UNDEF "),
                            }?;
                        }
                        write!(f, ") ")?;
                    }
                    write!(f, " }}")
                }
            }
            GraphPattern::AggregateJoin(GroupPattern(group, p), agg) => write!(
                f,
                "{{ SELECT {} WHERE {{ {} }} GROUP BY {} }}",
                agg.iter()
                    .map(|(a, v)| format!("({} AS {})", SparqlAggregation(a), v))
                    .chain(group.iter().map(|e| e.to_string()))
                    .collect::<Vec<String>>()
                    .join(" "),
                SparqlGraphPattern(&*p),
                group
                    .iter()
                    .map(|e| format!("({})", e.to_string()))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            p => write!(
                f,
                "{{ {} }}",
                SparqlGraphRootPattern {
                    algebra: p,
                    dataset: &QueryDataset::default()
                }
            ),
        }
    }
}

struct SparqlGraphRootPattern<'a> {
    algebra: &'a GraphPattern,
    dataset: &'a QueryDataset,
}

impl<'a> fmt::Display for SparqlGraphRootPattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut distinct = false;
        let mut reduced = false;
        let mut order = None;
        let mut start = 0;
        let mut length = None;
        let mut project: &[Variable] = &[];

        let mut child = self.algebra;
        loop {
            match child {
                GraphPattern::OrderBy(l, o) => {
                    order = Some(o);
                    child = &*l;
                }
                GraphPattern::Project(l, pv) if project.is_empty() => {
                    project = pv;
                    child = &*l;
                }
                GraphPattern::Distinct(l) => {
                    distinct = true;
                    child = &*l;
                }
                GraphPattern::Reduced(l) => {
                    reduced = true;
                    child = &*l;
                }
                GraphPattern::Slice(l, s, len) => {
                    start = *s;
                    length = *len;
                    child = l;
                }
                p => {
                    write!(f, "SELECT ")?;
                    if distinct {
                        write!(f, "DISTINCT ")?;
                    }
                    if reduced {
                        write!(f, "REDUCED ")?;
                    }
                    write!(
                        f,
                        "{}{} WHERE {{ {} }}",
                        build_sparql_select_arguments(project),
                        self.dataset,
                        SparqlGraphPattern(p)
                    )?;
                    if let Some(order) = order {
                        write!(
                            f,
                            " ORDER BY {}",
                            order
                                .iter()
                                .map(|c| SparqlOrderComparator(c).to_string())
                                .collect::<Vec<String>>()
                                .join(" ")
                        )?;
                    }
                    if start > 0 {
                        write!(f, " OFFSET {}", start)?;
                    }
                    if let Some(length) = length {
                        write!(f, " LIMIT {}", length)?;
                    }
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct GroupPattern(pub Vec<Variable>, pub Box<GraphPattern>);

impl fmt::Display for GroupPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Group(({}), {})",
            self.0
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.1
        )
    }
}

fn build_sparql_select_arguments(args: &[Variable]) -> String {
    if args.is_empty() {
        "*".to_owned()
    } else {
        args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Aggregation {
    Count(Option<Box<Expression>>, bool),
    Sum(Box<Expression>, bool),
    Min(Box<Expression>, bool),
    Max(Box<Expression>, bool),
    Avg(Box<Expression>, bool),
    Sample(Box<Expression>, bool),
    GroupConcat(Box<Expression>, bool, Option<String>),
}

impl fmt::Display for Aggregation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Aggregation::Count(e, distinct) => {
                if *distinct {
                    if let Some(ex) = e {
                        write!(f, "COUNT(DISTINCT {})", ex)
                    } else {
                        write!(f, "COUNT(DISTINCT *)")
                    }
                } else if let Some(ex) = e {
                    write!(f, "COUNT({})", ex)
                } else {
                    write!(f, "COUNT(*)")
                }
            }
            Aggregation::Sum(e, distinct) => {
                if *distinct {
                    write!(f, "Aggregation(Distinct({}), Sum, {{}})", e)
                } else {
                    write!(f, "Aggregation({}, Sum, {{}})", e)
                }
            }
            Aggregation::Min(e, distinct) => {
                if *distinct {
                    write!(f, "Aggregation(Distinct({}), Min, {{}})", e)
                } else {
                    write!(f, "Aggregation({}, Min, {{}})", e)
                }
            }
            Aggregation::Max(e, distinct) => {
                if *distinct {
                    write!(f, "Aggregation(Distinct({}), Max, {{}})", e)
                } else {
                    write!(f, "Aggregation({}, Max, {{}})", e)
                }
            }
            Aggregation::Avg(e, distinct) => {
                if *distinct {
                    write!(f, "Aggregation(Distinct({}), Avg, {{}})", e)
                } else {
                    write!(f, "Aggregation({}, Avg, {{}})", e)
                }
            }
            Aggregation::Sample(e, distinct) => {
                if *distinct {
                    write!(f, "Aggregation(Distinct({}), Sum, {{}})", e)
                } else {
                    write!(f, "Aggregation({}, Sample, {{}})", e)
                }
            }
            Aggregation::GroupConcat(e, distinct, sep) => {
                if *distinct {
                    if let Some(s) = sep {
                        write!(
                            f,
                            "Aggregation(Distinct({}), GroupConcat, {{\"separator\" \u{2192} {}}})",
                            e,
                            fmt_str(s)
                        )
                    } else {
                        write!(f, "Aggregation(Distinct({}), GroupConcat, {{}})", e)
                    }
                } else if let Some(s) = sep {
                    write!(
                        f,
                        "Aggregation({}, GroupConcat, {{\"separator\" \u{2192} {}}})",
                        e,
                        fmt_str(s)
                    )
                } else {
                    write!(f, "Aggregation(Distinct({}), GroupConcat, {{}})", e)
                }
            }
        }
    }
}

struct SparqlAggregation<'a>(&'a Aggregation);

impl<'a> fmt::Display for SparqlAggregation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Aggregation::Count(e, distinct) => {
                if *distinct {
                    if let Some(e) = e {
                        write!(f, "COUNT(DISTINCT {})", SparqlExpression(e))
                    } else {
                        write!(f, "COUNT(DISTINCT *)")
                    }
                } else if let Some(e) = e {
                    write!(f, "COUNT({})", SparqlExpression(e))
                } else {
                    write!(f, "COUNT(*)")
                }
            }
            Aggregation::Sum(e, distinct) => {
                if *distinct {
                    write!(f, "SUM(DISTINCT {})", SparqlExpression(e))
                } else {
                    write!(f, "SUM({})", SparqlExpression(e))
                }
            }
            Aggregation::Min(e, distinct) => {
                if *distinct {
                    write!(f, "MIN(DISTINCT {})", SparqlExpression(e))
                } else {
                    write!(f, "MIN({})", SparqlExpression(e))
                }
            }
            Aggregation::Max(e, distinct) => {
                if *distinct {
                    write!(f, "MAX(DISTINCT {})", SparqlExpression(e))
                } else {
                    write!(f, "MAX({})", SparqlExpression(e))
                }
            }
            Aggregation::Avg(e, distinct) => {
                if *distinct {
                    write!(f, "AVG(DISTINCT {})", SparqlExpression(e))
                } else {
                    write!(f, "AVG({})", SparqlExpression(e))
                }
            }
            Aggregation::Sample(e, distinct) => {
                if *distinct {
                    write!(f, "SAMPLE(DISTINCT {})", SparqlExpression(e))
                } else {
                    write!(f, "SAMPLE({})", SparqlExpression(e))
                }
            }
            Aggregation::GroupConcat(e, distinct, sep) => {
                if *distinct {
                    if let Some(sep) = sep {
                        write!(
                            f,
                            "GROUP_CONCAT(DISTINCT {}; SEPARATOR = {})",
                            SparqlExpression(e),
                            fmt_str(sep)
                        )
                    } else {
                        write!(f, "GROUP_CONCAT(DISTINCT {})", SparqlExpression(e))
                    }
                } else if let Some(sep) = sep {
                    write!(
                        f,
                        "GROUP_CONCAT({}; SEPARATOR = {})",
                        SparqlExpression(e),
                        fmt_str(sep)
                    )
                } else {
                    write!(f, "GROUP_CONCAT({})", SparqlExpression(e))
                }
            }
        }
    }
}

fn fmt_str(value: &str) -> rio::Literal<'_> {
    rio::Literal::Simple { value }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum OrderComparator {
    Asc(Expression),
    Desc(Expression),
}

impl fmt::Display for OrderComparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderComparator::Asc(e) => write!(f, "ASC({})", e),
            OrderComparator::Desc(e) => write!(f, "DESC({})", e),
        }
    }
}

impl From<Expression> for OrderComparator {
    fn from(e: Expression) -> Self {
        OrderComparator::Asc(e)
    }
}

struct SparqlOrderComparator<'a>(&'a OrderComparator);

impl<'a> fmt::Display for SparqlOrderComparator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            OrderComparator::Asc(e) => write!(f, "ASC({})", SparqlExpression(e)),
            OrderComparator::Desc(e) => write!(f, "DESC({})", SparqlExpression(e)),
        }
    }
}

/// A SPARQL query [dataset specification](https://www.w3.org/TR/sparql11-query/#specifyingDataset)
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct QueryDataset {
    pub default: Option<Vec<GraphName>>,
    pub named: Option<Vec<NamedOrBlankNode>>,
}

impl Default for QueryDataset {
    fn default() -> Self {
        Self {
            default: Some(vec![GraphName::DefaultGraph]),
            named: None,
        }
    }
}

impl QueryDataset {
    /// Checks if this dataset specification is the default one
    /// (i.e. the default graph is the store default graph and all the store named graphs are available)
    ///
    /// ```
    /// use oxigraph::sparql::Query;
    ///
    /// assert!(Query::parse("SELECT ?s ?p ?o WHERE { ?s ?p ?o . }", None)?.dataset().is_default_dataset());
    /// assert!(!Query::parse("SELECT ?s ?p ?o FROM <http://example.com> WHERE { ?s ?p ?o . }", None)?.dataset().is_default_dataset());
    ///
    /// # Result::Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_default_dataset(&self) -> bool {
        self.default
            .as_ref()
            .map_or(false, |t| t == &[GraphName::DefaultGraph])
            && self.named.is_none()
    }

    /// Returns the list of the store graphs that are available to the query as the default graph or `None` if the union of all graphs is used as the default graph
    /// This list is by default only the store default graph
    pub fn default_graph_graphs(&self) -> Option<&[GraphName]> {
        self.default.as_deref()
    }

    /// Sets if the default graph for the query should be the union of all the graphs in the queried store
    pub fn set_default_graph_as_union(&mut self) {
        self.default = None;
    }

    /// Sets the list of graphs the query should consider as being part of the default graph.
    ///
    /// By default only the store default graph is considered.
    /// ```
    /// use oxigraph::model::NamedNode;
    /// use oxigraph::sparql::Query;
    ///
    /// let mut query = Query::parse("SELECT ?s ?p ?o WHERE { ?s ?p ?o . }", None)?;
    /// query.dataset_mut().set_default_graph(vec![NamedNode::new("http://example.com")?.into()]);
    /// assert_eq!(query.to_string(), "SELECT ?s ?p ?o FROM <http://example.com> WHERE { ?s ?p ?o . }");
    ///
    /// # Result::Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_default_graph(&mut self, graphs: Vec<GraphName>) {
        self.default = Some(graphs)
    }

    /// Returns the list of the available named graphs for the query or `None` if all graphs are available
    pub fn available_named_graphs(&self) -> Option<&[NamedOrBlankNode]> {
        self.named.as_deref()
    }

    /// Sets the list of allowed named graphs in the query.
    ///
    /// ```
    /// use oxigraph::model::NamedNode;
    /// use oxigraph::sparql::Query;
    ///
    /// let mut query = Query::parse("SELECT ?s ?p ?o WHERE { ?s ?p ?o . }", None)?;
    /// query.dataset_mut().set_available_named_graphs(vec![NamedNode::new("http://example.com")?.into()]);
    /// assert_eq!(query.to_string(), "SELECT ?s ?p ?o FROM NAMED <http://example.com> WHERE { ?s ?p ?o . }");
    ///
    /// # Result::Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_available_named_graphs(&mut self, named_graphs: Vec<NamedOrBlankNode>) {
        self.named = Some(named_graphs);
    }
}

impl fmt::Display for QueryDataset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO: does not encode everything
        if let Some(graphs) = &self.default {
            for g in graphs {
                if !g.is_default_graph() {
                    write!(f, " FROM {}", g)?;
                }
            }
        }
        if let Some(graphs) = &self.named {
            for g in graphs {
                write!(f, " FROM NAMED {}", g)?;
            }
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum QueryVariants {
    Select {
        dataset: QueryDataset,
        algebra: Rc<GraphPattern>,
        base_iri: Option<Rc<Iri<String>>>,
    },
    Construct {
        construct: Rc<Vec<TriplePattern>>,
        dataset: QueryDataset,
        algebra: Rc<GraphPattern>,
        base_iri: Option<Rc<Iri<String>>>,
    },
    Describe {
        dataset: QueryDataset,
        algebra: Rc<GraphPattern>,
        base_iri: Option<Rc<Iri<String>>>,
    },
    Ask {
        dataset: QueryDataset,
        algebra: Rc<GraphPattern>,
        base_iri: Option<Rc<Iri<String>>>,
    },
}

impl fmt::Display for QueryVariants {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryVariants::Select {
                dataset,
                algebra,
                base_iri,
            } => {
                if let Some(base_iri) = base_iri {
                    writeln!(f, "BASE <{}>", base_iri)?;
                }
                write!(f, "{}", SparqlGraphRootPattern { algebra, dataset })
            }
            QueryVariants::Construct {
                construct,
                dataset,
                algebra,
                base_iri,
            } => {
                if let Some(base_iri) = base_iri {
                    writeln!(f, "BASE <{}>", base_iri)?;
                }
                write!(f, "CONSTRUCT {{ ")?;
                for triple in construct.iter() {
                    write!(f, "{} ", triple)?;
                }
                write!(
                    f,
                    "}}{} WHERE {{ {} }}",
                    dataset,
                    SparqlGraphRootPattern {
                        algebra,
                        dataset: &QueryDataset::default()
                    }
                )
            }
            QueryVariants::Describe {
                dataset,
                algebra,
                base_iri,
            } => {
                if let Some(base_iri) = base_iri {
                    writeln!(f, "BASE <{}>", base_iri.as_str())?;
                }
                write!(
                    f,
                    "DESCRIBE *{} WHERE {{ {} }}",
                    dataset,
                    SparqlGraphRootPattern {
                        algebra,
                        dataset: &QueryDataset::default()
                    }
                )
            }
            QueryVariants::Ask {
                dataset,
                algebra,
                base_iri,
            } => {
                if let Some(base_iri) = base_iri {
                    writeln!(f, "BASE <{}>", base_iri)?;
                }
                write!(
                    f,
                    "ASK{} WHERE {{ {} }}",
                    dataset,
                    SparqlGraphRootPattern {
                        algebra,
                        dataset: &QueryDataset::default()
                    }
                )
            }
        }
    }
}

/// The [graph update operations](https://www.w3.org/TR/sparql11-update/#formalModelGraphUpdate)
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum GraphUpdateOperation {
    /// [insert data](https://www.w3.org/TR/sparql11-update/#def_insertdataoperation)
    InsertData { data: Vec<QuadPattern> },
    /// [delete data](https://www.w3.org/TR/sparql11-update/#def_deletedataoperation)
    DeleteData { data: Vec<QuadPattern> },
    /// [delete insert](https://www.w3.org/TR/sparql11-update/#def_deleteinsertoperation)
    DeleteInsert {
        delete: Vec<QuadPattern>,
        insert: Vec<QuadPattern>,
        using: QueryDataset,
        algebra: GraphPattern,
    },
    /// [load](https://www.w3.org/TR/sparql11-update/#def_loadoperation)
    Load {
        silent: bool,
        from: NamedNode,
        to: Option<NamedNode>,
    },
    /// [clear](https://www.w3.org/TR/sparql11-update/#def_clearoperation)
    Clear { silent: bool, graph: GraphTarget },
    /// [create](https://www.w3.org/TR/sparql11-update/#def_createoperation)
    Create { silent: bool, graph: NamedNode },
    /// [drop](https://www.w3.org/TR/sparql11-update/#def_dropoperation)
    Drop { silent: bool, graph: GraphTarget },
}

impl fmt::Display for GraphUpdateOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphUpdateOperation::InsertData { data } => {
                writeln!(f, "INSERT DATA {{")?;
                for quad in data {
                    writeln!(f, "\t{}", quad)?;
                }
                write!(f, "}}")
            }
            GraphUpdateOperation::DeleteData { data } => {
                writeln!(f, "DELETE DATA {{")?;
                for quad in data {
                    writeln!(f, "\t{}", quad)?;
                }
                write!(f, "}}")
            }
            GraphUpdateOperation::DeleteInsert {
                delete,
                insert,
                using,
                algebra,
            } => {
                if !delete.is_empty() {
                    writeln!(f, "DELETE {{")?;
                    for quad in delete {
                        writeln!(f, "\t{}", quad)?;
                    }
                    writeln!(f, "}}")?;
                }
                if !insert.is_empty() {
                    writeln!(f, "INSERT {{")?;
                    for quad in insert {
                        writeln!(f, "\t{}", quad)?;
                    }
                    writeln!(f, "}}")?;
                }
                if let Some(using_default) = using.default_graph_graphs() {
                    for g in using_default {
                        if !g.is_default_graph() {
                            writeln!(f, "USING {}", g)?;
                        }
                    }
                }
                if let Some(using_named) = using.available_named_graphs() {
                    for g in using_named {
                        writeln!(f, "USING NAMED {}", g)?;
                    }
                }
                write!(
                    f,
                    "WHERE {{ {} }}",
                    SparqlGraphRootPattern {
                        algebra,
                        dataset: &QueryDataset::default()
                    }
                )
            }
            GraphUpdateOperation::Load { silent, from, to } => {
                write!(f, "LOAD ")?;
                if *silent {
                    write!(f, "SILENT ")?;
                }
                write!(f, "{}", from)?;
                if let Some(to) = to {
                    write!(f, " INTO GRAPH {}", to)?;
                }
                Ok(())
            }
            GraphUpdateOperation::Clear { silent, graph } => {
                write!(f, "CLEAR ")?;
                if *silent {
                    write!(f, "SILENT ")?;
                }
                write!(f, "{}", graph)
            }
            GraphUpdateOperation::Create { silent, graph } => {
                write!(f, "CREATE ")?;
                if *silent {
                    write!(f, "SILENT ")?;
                }
                write!(f, "GRAPH {}", graph)
            }
            GraphUpdateOperation::Drop { silent, graph } => {
                write!(f, "DROP ")?;
                if *silent {
                    write!(f, "SILENT ")?;
                }
                write!(f, "{}", graph)
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum GraphTarget {
    NamedNode(NamedNode),
    DefaultGraph,
    NamedGraphs,
    AllGraphs,
}

impl fmt::Display for GraphTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamedNode(node) => write!(f, "GRAPH {}", node),
            Self::DefaultGraph => write!(f, "DEFAULT"),
            Self::NamedGraphs => write!(f, "NAMED"),
            Self::AllGraphs => write!(f, "ALL"),
        }
    }
}

impl From<NamedNode> for GraphTarget {
    fn from(node: NamedNode) -> Self {
        Self::NamedNode(node)
    }
}

impl From<NamedNodeRef<'_>> for GraphTarget {
    fn from(node: NamedNodeRef<'_>) -> Self {
        Self::NamedNode(node.into())
    }
}

impl From<NamedOrDefaultGraphTarget> for GraphTarget {
    fn from(graph: NamedOrDefaultGraphTarget) -> Self {
        match graph {
            NamedOrDefaultGraphTarget::NamedNode(node) => Self::NamedNode(node),
            NamedOrDefaultGraphTarget::DefaultGraph => Self::DefaultGraph,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum NamedOrDefaultGraphTarget {
    NamedNode(NamedNode),
    DefaultGraph,
}

impl fmt::Display for NamedOrDefaultGraphTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamedNode(node) => write!(f, "GRAPH {}", node),
            Self::DefaultGraph => write!(f, "DEFAULT"),
        }
    }
}

impl From<NamedNode> for NamedOrDefaultGraphTarget {
    fn from(node: NamedNode) -> Self {
        Self::NamedNode(node)
    }
}

impl From<NamedNodeRef<'_>> for NamedOrDefaultGraphTarget {
    fn from(node: NamedNodeRef<'_>) -> Self {
        Self::NamedNode(node.into())
    }
}

impl From<NamedOrDefaultGraphTarget> for Option<NamedNodeOrVariable> {
    fn from(graph: NamedOrDefaultGraphTarget) -> Self {
        match graph {
            NamedOrDefaultGraphTarget::NamedNode(node) => Some(node.into()),
            NamedOrDefaultGraphTarget::DefaultGraph => None,
        }
    }
}
