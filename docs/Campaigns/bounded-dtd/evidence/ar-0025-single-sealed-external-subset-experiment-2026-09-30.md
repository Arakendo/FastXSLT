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
fallback, and network fallback remain unsupported. Repeated declarations keep
the first binding within a subset; internal declarations override external
declarations, matching XML declaration precedence without adding validation.

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
byte bound, reject `PUBLIC` and trailing identifier forms, preserve declaration
precedence, and accept processing instructions and comments in the document's
internal subset.

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

The same parser is now composed with the existing stylesheet dependency
resolver for a principal stylesheet. The external subset consumes one ordinary
resolution attempt, its independent byte ceiling, and the aggregate stylesheet
dependency-byte budget. A focused workbench test proves defaulted stylesheet
version semantics and retained DTD resource provenance. The reviewed
`stylesheet.dtd` family selects 12 catalog cases; 11 standard cases initialize,
execute, and compare exactly. Totals become 2,448 / 3,173 exact (77.15%), 2,640
initialized, 2,594 executed, 88 single-external-subset direct outcomes, and 14
explicit unsupported DTD outcomes.

The same principal-stylesheet mechanism was then replayed for the two
case-local copies of `htmllat1.dtd`. Each reviewed resource is 3,139 bytes and
contains the ISO Latin-1 internal general-character entity declarations used
by the unchanged `copy19`, `copy20`, and `string130` stylesheets. All three
cases initialize, execute, and compare exactly without any new acquisition
authority or parser grammar. Current totals are 2,451 / 3,173 exact (77.25%),
2,643 initialized, 2,597 executed, 91 single-external-subset direct outcomes,
and 11 explicit unsupported DTD outcomes.

Focused adversarial tests retain the same boundedness model. They reject
entity cycles spanning the external and internal subsets, apply one aggregate
declaration ceiling across both subsets, reject malformed external
declarations with the admitted DTD's logical identity and deterministic whole-
resource span, observe pre-existing XML cancellation, and enforce independent
external-byte and cumulative replacement ceilings. These tests establish
bounded failure behavior; they do not close the remaining declaration-level
external-offset provenance gap.

The loader experiment now applies one explicitly configured reviewed reference
to a matching included or imported stylesheet module as well as the principal
module. It does not discover arbitrary DTD names: an ASCII `SYSTEM` reference
must match the reviewed configuration, and the XML parser independently
validates the declaration after the DTD is resolved. A focused include test
proves success through the existing resolver and proves that an absent module
DTD returns the existing sealed-snapshot missing-resource diagnostic without
ambient fallback. The unchanged Microsoft `Import__84615` and
`Include__77751` cases then initialize and execute with `stylesheet1.dtd`, but
both expose later whitespace-result mismatches. Exact coverage remains 2,451 /
3,173 (77.25%); initialized cases rise to 2,645, executed cases to 2,599, and
the DTD frontier becomes 15 internal parses, 93 single-external parses, and
nine explicit unsupported outcomes.

The related function inventory separates parser admission from later language
surface. All ten directly DTD-dependent typed-ID source cases initialize and
execute through the bounded `id()` implementation; eight compare exactly and
two remain visible manual-comparator gaps. A lexical/catalog inventory finds 28
stylesheets calling `unparsed-entity-uri()`: 26 are expected-error cases and two
are standard cases (`expression_expression02` and `BVTs_bvt089`). Traces from
both standard cases and a representative expected-error case show the first
visible failure during expression compilation (`FXXP1001`), after XML parsing.
FastXSLT therefore does not infer support for notation/unparsed-entity metadata
from the external-subset parser experiment.

## Disposition

Retain the private experiment as evidence for AR-0025. It proves that a narrow
external subset can compose with sealed resource authority and bounded XML/XDM
construction. It does not yet justify a supported external-DTD profile or any
production API. Further expansion must be driven by inventoried semantics, not
the proximity of the 75% checkpoint.
