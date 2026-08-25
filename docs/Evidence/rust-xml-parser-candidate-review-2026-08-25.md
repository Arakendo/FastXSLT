# Rust XML Parser Candidate Review

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Scope | XML parser mechanics for the first private transform slice |
| Environment | Windows, Rust workspace MSRV 1.85 |
| Candidates | `quick-xml` 0.40.1, `xmlparser` 0.13.6, `roxmltree` 0.21.1 |
| Informs | AR-0008 and M1 Slice 2 |

## Method

Candidate metadata was obtained with `cargo info` from crates.io. Published
source and API documentation were inspected from the downloaded crates and the
upstream repositories. Only `quick-xml` was added, at an exact version and as a
development dependency. A test-only adapter exercised it from an in-memory byte
slice; no candidate type entered the public FastXSLT API.

Sources:

- [`quick-xml` 0.40.1 documentation](https://docs.rs/quick-xml/0.40.1/quick_xml/)
  and [repository](https://github.com/tafia/quick-xml);
- [`xmlparser` 0.13.6 documentation](https://docs.rs/xmlparser/0.13.6/xmlparser/)
  and [repository](https://github.com/RazrFalcon/xmlparser);
- [`roxmltree` 0.21.1 documentation](https://docs.rs/roxmltree/0.21.1/roxmltree/)
  and [repository](https://github.com/RazrFalcon/roxmltree).

This was a boundary and behavior review, not a throughput benchmark, security
audit, XML conformance result, or dependency admission decision.

## Metadata observations

| Candidate | License | Declared Rust version | Physical model | Default dependency pressure |
| --- | --- | ---: | --- | --- |
| `quick-xml` 0.40.1 | MIT | 1.79 | Pull events over caller bytes | `memchr` |
| `xmlparser` 0.13.6 | MIT/Apache-2.0 | Not declared in crates.io metadata | Zero-allocation tokenizer | None |
| `roxmltree` 0.21.1 | MIT OR Apache-2.0 | 1.60 | Borrowing read-only tree | `memchr` |

All three license expressions are compatible candidates for an MIT library,
subject to preserving required notices and completing the normal dependency
review before production admission. `quick-xml` 0.40.1 fits FastXSLT's Rust
1.85 floor. The later `quick-xml` 0.41 release declared Rust 1.86 in its
published source, so this experiment deliberately did not use a floating
version.

A source scan of `quick-xml` 0.40.1 found the word `unsafe` only in comments
describing optimizations that were not used. This observation is not a
transitive unsafe-code audit: `memchr` and all future feature-enabled
dependencies remain separate review surfaces under ADR-0003.

## Candidate findings

### `quick-xml`

The slice reader consumes `&[u8]` without a read buffer, produces pull events,
tracks byte positions, checks matching end tags by default, can validate XML
comments, reports DTDs as events, checks raw duplicate attributes, and resolves
element and attribute namespaces through `NsReader`. Default namespaces are
correctly absent from unprefixed attributes.

These mechanics align with a private adapter that copies semantic data into a
FastXSLT-owned XDM. The dependency need not own nodes, lifetimes, diagnostics,
resource identity, host authority, or the public API.

The crate does not by itself establish the whole XML contract. The experiment
had to add:

- one-root and outside-root document checks;
- rejection of every DTD before any entity or external identifier can be used;
- rejection of unknown general entities in content and attributes;
- unknown-prefix and duplicate expanded-attribute checks;
- explicit event and nesting-depth limits; and
- logical resource identity around byte-offset diagnostics.

Name-character validation, namespace-constraint conformance, XML declaration
placement/version rules, supported input encodings, exact line/column mapping,
text coalescing, and complete XDM node construction remain unresolved. In
particular, the reader API is optimized parsing machinery rather than proof of
XML 1.0 plus Namespaces conformance.

### `xmlparser`

`xmlparser` is a small, zero-allocation tokenizer with token spans and no
dependencies. Its own documentation explicitly leaves tree validation to the
caller and notes that mismatched nesting and duplicate attributes are not
reported. That would make FastXSLT responsible for more basic well-formedness
mechanics in the first slice. It remains useful as a comparison point if
`quick-xml` validation or allocation behavior becomes unsuitable.

### `roxmltree`

`roxmltree` supplies a convenient read-only tree with namespace resolution and
source positions. Its tree borrows the source text and performs more XML
normalization than a tokenizer. Adopting its nodes directly would couple XDM
identity and lifetimes to a dependency tree, encourage a permanently
materialized random-access assumption, and obscure FastXSLT's ownership under
AR-0007. It could serve as a test oracle for selected mechanics, but it is not
the preferred engine representation.

## Private experiment results

Ten focused tests now pass across the resource and XML experiments; five cover
the XML adapter. The XML cases demonstrate:

- namespace expansion for elements and attributes, including default-namespace
  behavior;
- root byte span and logical resource provenance;
- rejection of mismatched tags, multiple roots, unknown prefixes, duplicate
  expanded attributes, DTDs, and unknown entities;
- retention pressure for comments and processing instructions; and
- event-count and nesting-depth failures with structured private identities.

The experiment reads only bytes supplied by the caller. A malicious external
identifier is rejected as a DTD event; no filesystem or network resolver is
present and no ambient access is attempted.

## Result

`quick-xml` 0.40.1 is the leading mechanics candidate for the private vertical
slice. This is not production admission or an accepted architectural decision.
Keep the exact dev-only pin until AR-0008's missing conformance, encoding,
diagnostic, dependency, and performance evidence is gathered. Promote it to a
normal private dependency only when the first owned XDM construction path needs
it and the acceptance gates pass.
