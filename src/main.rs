mod convert;
mod types;
mod util;

use crate::convert::{as_triples, to_rify_pattern};
use crate::types::{InvalidRule, RdfNode};
use oxigraph::model::GraphName;
use oxigraph::sparql::algebra::{GraphPattern, Query, QueryDataset, QueryVariants};
use rify::Rule;
use std::borrow::Borrow;
use std::error::Error;
use std::io::{stdin, stdout, Read};
use std::process::exit;

fn main() {
    handle_args();

    let res = || -> Result<(), Box<dyn Error>> {
        let mut stin = String::new();
        stdin().read_to_string(&mut stin)?;
        let q = Query::parse(&stin, None)?;
        let rules = sparql2rify(q)?;
        serde_json::to_writer_pretty(stdout(), &rules)?;
        println!();
        Ok(())
    }();

    if let Err(e) = res {
        eprintln!("{}", e);
        exit(1);
    }
}

fn handle_args() {
    match std::env::args().nth(1).as_deref() {
        None => {}
        Some("--help") | Some("-h") => {
            eprintln!("sparql2rify - Convert a SPARQL CONSTRUCT clause to a rify rule.");
            eprintln!("USE: cat input.sparql | sparql2rify > output.json");
            exit(0);
        }
        _ => {
            eprintln!("Invalid argument, try --help.");
            exit(2);
        }
    }
}

fn sparql2rify(sparql: Query) -> Result<Rule<String, RdfNode>, InvalidRule> {
    let (construct, dataset, algebra, base_iri) = match sparql.0 {
        QueryVariants::Construct {
            construct,
            dataset,
            algebra,
            base_iri,
        } => (construct, dataset, algebra, base_iri),
        _ => return Err(InvalidRule::MustBeConstruct),
    };

    if (QueryDataset {
        default: Some(vec![GraphName::DefaultGraph]),
        named: None,
    } != dataset)
    {
        return Err(InvalidRule::IllegalFrom);
    }

    if base_iri.is_some() {
        return Err(InvalidRule::IllegalBaseIri);
    }

    let (project, _vars) = match algebra.borrow() {
        GraphPattern::Project(patt, vars) => (patt, vars),
        _ => return Err(InvalidRule::MustBeBasicGraphPattern),
    };
    let bgp = match &**project {
        GraphPattern::BGP(bgp) => bgp,
        _ => return Err(InvalidRule::MustBeBasicGraphPattern),
    };

    // graph pattern must not contain path patterns
    let bgp = as_triples(&bgp)?;

    let mut if_all = to_rify_pattern(&bgp);
    let mut then = to_rify_pattern(&construct);

    // blank nodes in `then` are a footgun so they are not allowed
    for ent in then.iter().flatten() {
        if let Some(name) = util::as_blank(ent) {
            return Err(InvalidRule::BlankNodeImplied {
                name: name.to_string(),
            });
        }
    }

    util::unbind_blanks(&mut if_all, &mut then)?;

    Rule::create(if_all, then).map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RdfNode::Iri;
    use rify::Entity::{Bound, Unbound};

    #[test]
    fn simple_rule() {
        let sparql = "CONSTRUCT { ?s ?p ?o . }  WHERE { ?s ?p ?o . }"
            .parse()
            .unwrap();
        let r = sparql2rify(dbg!(sparql)).unwrap();
        assert_eq!(
            r,
            rify::Rule::create(
                vec![[unbd("s"), unbd("p"), unbd("o")]],
                vec![[unbd("s"), unbd("p"), unbd("o")]]
            )
            .unwrap()
        );
    }

    #[test]
    fn reified_claim() {
        let sparql = "
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            
            CONSTRUCT {
                ?s ?p ?o .
            } WHERE {
                ?a rdf:subject ?s ;
                   rdf:predicate ?p ;
                   rdf:object ?o .
            }
        "
        .parse();
        let res = sparql2rify(sparql.unwrap()).unwrap();
        assert_eq!(
            res,
            rify::Rule::create(
                vec![
                    [unbd("a"), rdf("subject"), unbd("s")],
                    [unbd("a"), rdf("predicate"), unbd("p")],
                    [unbd("a"), rdf("object"), unbd("o")]
                ],
                vec![[unbd("s"), unbd("p"), unbd("o")]]
            )
            .unwrap()
        );
    }

    #[test]
    fn anonymous_blanknode() {
        let sparql = "
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            
            CONSTRUCT { } WHERE {
                [] rdf:subject [] .
            }
        "
        .parse();
        sparql2rify(sparql.unwrap()).unwrap();
    }

    #[test]
    fn errs() {
        use InvalidRule::*;
        let cases: &[(_, &[_])] = &[
            (MustBeConstruct, &["SELECT ?a ?b ?c WHERE { ?s ?p ?o . }"]),
            (IllegalFrom, &[]),
            (IllegalBaseIri, &[]),
            (
                MustBeBasicGraphPattern,
                &[
                    "CONSTRUCT {} WHERE { {} UNION  {} . }",
                    "CONSTRUCT {} WHERE { GRAPH <http://example.com> {} . }",
                ],
            ),
            (IllegalPathPattern, &[]),
            (
                UnboundImplied {
                    name: "a".to_string(),
                },
                &["CONSTRUCT { ?a ?b ?c . } WHERE {}"],
            ),
            (
                NameCollision {
                    name: "a".to_string(),
                },
                &["CONSTRUCT {  } WHERE { _:a ?a <http://example.com> . }"],
            ),
        ];
        for (err, queries) in cases {
            for query in *queries {
                assert_eq!(err, &sparql2rify(query.parse().unwrap()).unwrap_err());
            }
        }
    }

    #[test]
    fn more_errs() {
        let query = "CONSTRUCT { ?a ?b [] . } WHERE {}";
        let err = sparql2rify(query.parse().unwrap()).unwrap_err();
        match err {
            InvalidRule::BlankNodeImplied { .. } => {}
            _ => {
                dbg!(err);
                panic!();
            }
        }
    }

    fn rdf(suffix: &str) -> rify::Entity<String, RdfNode> {
        Bound(Iri(format!(
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#{}",
            suffix
        )))
    }

    fn unbd(name: &str) -> rify::Entity<String, RdfNode> {
        Unbound(name.to_string())
    }
}
