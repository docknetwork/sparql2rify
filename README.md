# Converts a SPARQL CONSTRUCT query into a lower level logical rule usable by rify.

# Usage

```bash
cat input.sparql | sparql2rify > output.json
```

# Examples

Input:

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

CONSTRUCT {
    ?s ?p ?o .
} WHERE {
    ?a rdf:subject ?s ;
       rdf:predicate ?p ;
       rdf:object ?o .
}
```

Output:

```json
{
  "if_all": [
    [
      {"Unbound": "a"},
      {"Bound": {"Iri": "http://www.w3.org/1999/02/22-rdf-syntax-ns#subject"}},
      {"Unbound": "s"}
    ],
    [
      {"Unbound": "a"},
      {"Bound": {"Iri": "http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate"}},
      {"Unbound": "p"}
    ],
    [
      {"Unbound": "a"},
      {"Bound": {"Iri": "http://www.w3.org/1999/02/22-rdf-syntax-ns#object"}},
      {"Unbound": "o"}
    ]
  ],
  "then": [
    [
      {"Unbound": "s"},
      {"Unbound": "p"},
      {"Unbound": "o"}
    ]
  ]
}
```

When it is safe to do so, blank nodes in the query are interpreted as unbound variables.

Input:

```sparql
CONSTRUCT {} WHERE {
  <http://example.com> <http://example.com> [] .
  <http://example.com> <http://example.com> _:a . 
}
```

Output:

```json
{
  "if_all": [
    [
      {"Bound": {"Iri": "http://example.com"}},
	  {"Bound": {"Iri": "http://example.com"}},
      {"Unbound": "c80d4fddd1806c129d681ace73f7fe9b"}
    ],
	[
      {"Bound": {"Iri": "http://example.com"}},
	  {"Bound": {"Iri": "http://example.com"}},
      {"Unbound": "a"}
    ]
  ],
  "then": []
}
```

Blank nodes are not allowed in the output. This input will be refused:

```sparql
CONSTRUCT {
  <http://example.com> <http://example.com> _:a .
} WHERE {
  <http://example.com> <http://example.com> _:a . 
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
