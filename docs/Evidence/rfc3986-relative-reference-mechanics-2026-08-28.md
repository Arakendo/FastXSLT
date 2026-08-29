# RFC 3986 Relative-Reference Mechanics

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Boundary | Private sealed-snapshot resolver |
| Candidate | `iri-string` 0.7.14, exact pinned version |
| License | MIT OR Apache-2.0 |
| Rust floor | 1.60 declared; FastXSLT floor remains 1.85 |
| Active dependency graph | No transitive dependency under the selected default `std` feature |
| Outcome | Sibling/parent IRI resolution and fragment-free acquisition identity execute without ambient access |

## Standards fit

XSLT 3.0 section 5.8 requires relative external-resource references to be
resolved against the containing element's base URI according to RFC 3986 after
the specified escaping treatment. Functions and Operators 3.1 likewise defines
`fn:resolve-uri` through RFC 3986 adapted for IRI characters. Resolution is a
syntactic operation; it does not dereference the resulting identity.

The initially considered `url` crate was rejected for this owner because it
implements the WHATWG URL Standard. Its `join` operation is useful for web URLs
but does not establish the RFC 3986/3987 and XSLT behavior required here.

`iri-string` provides strict RFC 3986 URI and RFC 3987 IRI reference types and a
relative-reference resolver. FastXSLT calls
`ensure_rfc3986_normalizable()` before accepting its output, so exceptional
inputs that would need the crate's WHATWG serialization fallback become an
explicit resolution failure rather than silently changing algorithms.

Primary references reviewed:

- <https://www.w3.org/TR/xslt-30/#uri-references>
- <https://www.w3.org/TR/xpath-functions-31/#func-resolve-uri>
- <https://www.w3.org/TR/xmlbase/#resolution>
- <https://docs.rs/iri-string/0.7.14/iri_string/>
- <https://github.com/lo48576/iri-string>

## Dependency review

The exact `iri-string` 0.7.14 manifest declares `MIT OR Apache-2.0`, Rust 1.60,
and optional `memchr`/`serde` dependencies. `cargo tree --features workbench`
shows no transitive dependency activated for `iri-string` in FastXSLT's chosen
configuration. The exact crate and checksum are retained in `Cargo.lock`; it is
enabled only for the private `workbench` feature and tests.

A source scan found dependency-owned `unsafe` regions, primarily validated
string-slice representation casts and internal typed-component operations,
including resolver internals. This is third-party dependency unsafe, not a
first-party ADR-0003 exception. The project calls only validated safe APIs and
does not expose the dependency's types. A future production promotion still
requires vulnerability/maintenance review and continued differential standards
evidence; passing these tests does not prove the dependency's unsafe invariants.

## Executed mechanics

The private resolver now accepts a supplied absolute IRI base and an IRI
reference, charges one resolution attempt, resolves syntactically, and looks up
only the resulting identity in the already sealed snapshot. Tests establish:

- `include-0401a.xsl` resolves beside
  `https://example.invalid/styles/include-0401.xsl`;
- `../shared/module.xsl` removes one path segment according to the resolver;
- `#embedded` acquires bytes under the fragment-free base identity and returns
  `embedded` separately;
- relative/invalid bases and invalid IRI references are rejected;
- results that cannot be serialized under pure RFC 3986 resolution are
  rejected; and
- URL-shaped identities never authorize network access or fallback outside the
  snapshot.

Denied-versus-missing precedence and the fixed attempt budget remain identical
to exact qualified lookup.

## Claim boundary

This is a private mechanics experiment, not complete XSLT URI or IRI
conformance. It does not yet derive an element's base URI from document identity
and `xml:base`, perform XSLT's escaping step, assemble stylesheet modules,
interpret a fragment, apply catalogs, expose a resolver API, or authorize live
acquisition. It selects no cache identity and admits no cross-generation
sharing. The later
[include-0401 execution](xslt30-include-0401-sealed-module-execution-2026-08-28.md)
uses this exact resolver path without broadening the mechanics experiment into
general module or resolver conformance.
