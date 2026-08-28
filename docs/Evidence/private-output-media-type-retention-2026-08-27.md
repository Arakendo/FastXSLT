# Private Output Media-Type Retention

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Standards surface | `xsl:output/@media-type` |
| Boundary | Private compiled stylesheet and bounded semantic inspection |
| Outcome | Literal media type retained as owned static serialization metadata |

## Executed slice

The private stylesheet compiler now admits the unqualified `media-type`
attribute on `xsl:output` and retains its string value in immutable
stylesheet-derived `OutputSettings`. Absence remains distinct from an explicitly
supplied value. The bounded semantic inspection projection includes the value
and charges its retained text against the inspection text budget.

A focused in-memory stylesheet proves
`application/x-fastxslt-test+xml` survives compilation. The reviewed `hello`
golden stylesheet now declares `application/xml`; its existing end-to-end
transform still produces the same expected XML bytes, and the inspection test
proves the metadata survives without exposing the compiler or XDM
representation.

## Claim boundary

This is static metadata retention, not a general serialization-conformance
claim. FastXSLT still supports only its narrow XML serialization path, and its
experimental transform calls return text rather than a stable result object
carrying media type. The value does not select an output method, authorize an
external destination, or trigger I/O.

The attribute also occurs in the deferred Tokimu/Web3D problem space, but this
checkpoint did not acquire or execute Web3D revision `35289`, inventory that
stylesheet, or move CR-0001's first unsupported frontier. It therefore provides
ordinary standards-driven implementation progress only, not consumer fidelity
evidence.
