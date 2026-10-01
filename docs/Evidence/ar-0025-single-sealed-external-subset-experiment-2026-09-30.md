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

The first complete hash-verified OASIS XSLT 1.0 replay opted in only these
previously inventoried standard cases:

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

All eight cases initialized, executed, and compared exactly. That pilot moved
the strict lower bound to 2,369 / 3,173 (74.66%).

The second replay generalized the host-side admission rule only across the same
reviewed resource family. A case is eligible when:

- its principal source is named `Plants.xml`, case-insensitively;
- its admitted bytes contain the exact quoted sibling reference
  `SYSTEM "plants.dtd"`;
- a case-local `plants.dtd` exists and is at most 64 KiB; and
- the external declarations pass the same bounded engine grammar.

The four available physical DTDs contain only non-validating element
declarations and one fixed prefixed namespace attribute. Three are byte-
identical; the fourth differs only in that fixed namespace value. None contains
a parameter entity or external general entity. The measurement adapter admits
only the case-local bytes under the resolved sibling logical identity. It does
not discover arbitrary external references.

This rule selects 101 catalog cases, including expected-error cases. Seventy-
six standard direct-frontier cases complete the external parse. Relative to the
internal-only baseline, 75 become exact, one newly executing case
(`Completeness__84358`) becomes a visible mismatch, and the rest retain later
or expected-error dispositions. The conserved result is:

| Measurement | Before | After |
| --- | ---: | ---: |
| Exact OASIS comparisons | 2,361 | 2,436 |
| Exact percentage of 3,173 | 74.41% | 76.77% |
| Initialized cases | 2,552 | 2,628 |
| Successfully executed cases | 2,506 | 2,582 |
| Internal-subset reference outcomes | 15 | 15 |
| Single-external-subset reference outcomes | 0 | 76 |
| Explicit unsupported DTD outcomes | 102 | 26 |

The complete direct DTD frontier remains 117 cases. No outcome is extrapolated
to a different DTD filename, declaration grammar, resolution topology, or
authority model.

A subsequent source-side inventory found one additional resource with the same
bounded shape. `Lotus/idkey_idkey04` names the 120-byte sibling `t04.dtd`, whose
only semantic metadata beyond element declarations is one `ID` attribute. The
same one-attempt snapshot path admits it, and the unchanged case compares
exactly through the existing XDM typed-ID index and XSLT 1.0 `id()` evaluator.
The resulting totals are 2,437 / 3,173 exact (76.80%), 2,629 initialized, 2,583
executed, 77 single-external-subset direct outcomes, and 25 explicit
unsupported DTD outcomes. Broader source candidates using parameter entities,
external general entities, notation/unparsed-entity metadata, or markup-
producing replacement text remain unsupported.

## Disposition

Retain the private experiment as evidence for AR-0025. It proves that a narrow
external subset can compose with sealed resource authority and bounded XML/XDM
construction. It does not yet justify a supported external-DTD profile or any
production API. Further expansion must be driven by inventoried semantics, not
the proximity of the 75% checkpoint.
