# AR-0025 Single Sealed External-Subset Experiment

## Question

Can FastXSLT resolve exactly one external DTD subset from an already sealed
resource snapshot without granting ambient acquisition authority or implying
general validating-XML support?

## Admitted shape

The private experiment admits only this shape:

- one quoted `SYSTEM` identifier on the document type declaration;
- one host-admitted external-subset resource whose logical identity is resolved
  relative to the source document identity;
- one snapshot-resolution attempt;
- a 64 KiB external-subset byte ceiling in the OASIS measurement;
- aggregate declaration, nesting, reference, replacement-byte, XML-event, XDM-
  node, cancellation, and work budgets; and
- the previously bounded non-validating `ELEMENT` and `ATTLIST` grammar plus
  internal general entities.

`PUBLIC` identifiers, parameter entities, external general entities, recursive
external subsets, validation, catalogs, live resolver callbacks, filesystem
fallback, and network fallback remain unsupported. Cross-subset declaration
shadowing remains unsupported rather than approximating declaration precedence.

The production parser and ordinary workbench constructors continue to deny all
DTDs. The experiment is reachable only through test-private preparation and
workbench paths.

## Authority and failure evidence

The host-side adapter reads and admits the DTD bytes before sealing the
snapshot. Preparation uses the existing snapshot resolver to resolve the
relative identifier. Removing the admitted DTD produces the structured
`FXRS0002` missing-resource outcome naming the resolved logical identity; no
filesystem or network acquisition is attempted.

The XML boundary independently checks that the supplied reference matches the
document declaration. External byte-limit and declaration failures name the
external logical resource. The current deterministic external diagnostic span
covers that complete resource; declaration-level external offsets remain open
work and are not claimed.

A focused transform proves that an external fixed attribute is incorporated
before XDM construction and is visible to XPath. Focused parser tests prove the
byte bound, reject `PUBLIC` and trailing identifier forms, reject cross-subset
attribute shadowing, and accept processing instructions and comments in the
document's internal subset.

## Real corpus result

The complete hash-verified OASIS XSLT 1.0 runner opts in only these previously
inventoried standard cases:

- `Attributes__81543`
- `Attributes__81544`
- `Attributes__81545`
- `Attributes__81546`
- `Attributes__81547`
- `Attributes__81548`
- `Attributes__81550`
- `Attributes__81551`

Their shared `Plants.xml` document declares the sibling `plants.dtd`. That DTD
contains non-validating element declarations and one fixed prefixed namespace
attribute; it contains no parameter entity or external general entity. The
measurement adapter explicitly admits those bytes under the resolved sibling
identity. No catalog-wide external-subset discovery is enabled.

All eight cases initialize, execute, and compare exactly. The conserved result
changes as follows:

| Measurement | Before | After |
| --- | ---: | ---: |
| Exact OASIS comparisons | 2,361 | 2,369 |
| Exact percentage of 3,173 | 74.41% | 74.66% |
| Initialized cases | 2,552 | 2,560 |
| Successfully executed cases | 2,506 | 2,514 |
| Internal-subset reference outcomes | 15 | 15 |
| Single-external-subset reference outcomes | 0 | 8 |
| Explicit unsupported DTD outcomes | 102 | 94 |

The complete direct DTD frontier remains 117 cases. The eight exact gains are
not extrapolated to the other external-identifier cases.

## Disposition

Retain the private experiment as evidence for AR-0025. It proves that a narrow
external subset can compose with sealed resource authority and bounded XML/XDM
construction. It does not yet justify a supported external-DTD profile or any
production API. Further expansion must be driven by inventoried semantics, not
the proximity of the 75% checkpoint.
