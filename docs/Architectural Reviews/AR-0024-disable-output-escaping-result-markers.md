# AR-0024: Disable-Output-Escaping Result Markers

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-09-30 |
| Last reviewed | 2026-09-30 |
| Scope | XSLT 1.0 `disable-output-escaping`, semantic result ownership, and serialization |
| Trigger | Thirty-two standard-operation OASIS XSLT 1.0 cases stop at the explicit `FXST1060` boundary, making historically awkward result behavior the nearest material compatibility frontier |
| Related ADRs | ADR-0002, ADR-0006, ADR-0007 |
| Related reviews | AR-0004, AR-0019 |
| Related evidence | [OASIS XSLT 1.0 disable-output-escaping reference marker](../Evidence/oasis-xslt10-disable-output-escaping-reference-marker-2026-09-30.md) |

## Architectural question

Can FastXSLT preserve the XSLT 1.0 disable-output-escaping intent as a bounded,
private serialization annotation on eligible text without turning serialized
markup into ordinary XDM structure, weakening normal escaping, or exposing the
annotation as a public result-tree contract?

## Trigger and evidence

The complete local OASIS sweep has 32 standard-operation cases whose first
failure is `disable-output-escaping="yes"`. They cover `xsl:text` and
`xsl:value-of`, XML/text/HTML output, temporary trees and copying, plus contexts
where the request has no effect such as attributes, comments, and processing
instructions.

FastXSLT currently rejects `yes` deliberately and admits `no` through the
ordinary escaped path. This prevents plausible but wrong output and preserves
the separation between semantic result construction and serialization. The
current 2,287 exact matches are 29 cases short of 73%, but that arithmetic is
only a discovery trigger. It is not evidence that all 32 cases should pass or
that the architectural boundary should be weakened.

## Ownership and constraints

- The result tree owns text values. Serialization owns whether an eligible
  text value is escaped when written to bytes or characters.
- A disable-output-escaping request is not parsed markup and must not create
  semantic result elements, attributes, namespaces, or node identities.
- Normal text remains escaped by default. The existing escaped path is the
  reference behavior and must not pay for or inherit the legacy rule when the
  marker is absent.
- The marker is private engine state. It is not part of XDM, semantic
  inspection, the public Rust facade, native handles, isolated wire framing, or
  the WASM contract.
- Attribute, comment, and processing-instruction construction consumes string
  values and discards the marker. Those contexts must retain their normal
  lexical validation and escaping rules.
- Temporary-tree string value, copying, and sequence construction need explicit
  tests. Marker preservation or loss must follow the selected XSLT 1.0
  compatibility rule rather than fall out accidentally from a Rust enum.
- Result-node count, result-text bytes, normalization, character maps, output
  encoding, cumulative serialized bytes, cancellation, and work accounting
  remain bounded and observable.
- Raw serialized text may intentionally be ill-formed. The host must not be
  told that successful serialization implies an XML-well-formed byte stream
  when this compatibility feature is active.
- Only XSLT 1.0 compatibility semantics may select the first experiment.
  Modern-version behavior requires independent standards evidence.

## Alternatives

### A. Retain explicit unsupported behavior

Keep `FXST1060` for `yes`. This preserves the cleanest result model and remains
honest, but leaves a normative XSLT 1.0 behavior and a concentrated corpus
frontier unresolved.

### B. Treat `yes` as `no`

This avoids representation work but silently returns escaped output where the
stylesheet requested otherwise. It can pass cases where the flag has no effect
while producing plausible wrong answers elsewhere. Rejected as a general rule.

### C. Private marked text in the result tree

Compile eligible XSLT 1.0 instructions to an explicit plan fact, construct a
private marked text node, and let serializers write its character data without
ordinary markup escaping. All string-consuming non-serialization contexts
discard the marker deliberately. This is the leading reference experiment.

The representation should keep marked and ordinary adjacent text distinct so
coalescing cannot spread the request across text that did not carry it.

### D. Serialize directly from the instruction executor

Bypass the semantic result and write raw bytes during execution. This couples
execution to output method/encoding/sinks, breaks result ownership, complicates
budgets and adapters, and creates a second serialization path. Rejected.

### E. Parse the authored text as markup

Convert disabled escaping content into result nodes. This invents semantics the
standard does not define, changes identity/namespaces, and creates injection and
authority confusion. Rejected.

## Findings and uncertainties

The corpus justifies a private experiment, not a public promise. A distinct
marked-text representation is the narrowest approach that preserves the
execution/serialization boundary and keeps ordinary escaping as an oracle.

The first tranche establishes the private plan fact, result marker, explicit
string-constructor marker loss, temporary-tree preservation by node identity,
and XML/HTML/text serializer behavior. The conserved sweep reaches 2,316 / 3,173
exact matches (73.00%); `FXST1060` is no longer an initialization frontier.
Remaining interaction work prevents promotion.

The review must still establish:

- exact `xsl:text` and `xsl:value-of` behavior for XML, HTML, and text output;
- deliberate marker loss in attribute, comment, and processing-instruction
  string construction;
- temporary-tree and `xsl:copy-of` behavior;
- adjacency/coalescing behavior between marked and ordinary text;
- interaction with character maps, normalization, output encodings, and result
  byte budgets; and
- whether a result carrying the marker needs a bounded internal inspection bit
  so adapters can avoid falsely describing it as guaranteed well-formed XML.

## Disposition

**Incubating.** Admit a private XSLT 1.0 marked-text reference experiment.
Ordinary escaped text remains the default and differential oracle. Do not
stabilize a public node kind, serializer switch, host capability, wire field, or
conformance statement. If the marker cannot remain contained to serialization
and explicit string-consuming boundaries, retain `FXST1060` instead.

## Required follow-up

- [ ] Inventory all 32 OASIS cases by constructor, output method, temporary-tree
  use, and no-effect context.
- [x] Add a private plan fact and result marker without changing ordinary text
  representation or coalescing behavior.
- [x] Prove marked XML/HTML text bypasses only ordinary markup escaping while
  text output remains byte-for-byte equivalent.
- [x] Prove attributes, comments, and processing instructions discard the
  marker and retain their normal validation/escaping.
- [x] Differential-test temporary-tree string value and copying behavior.
- [ ] Verify result/work/serialized-byte budgets, cancellation, output encoding,
  and character-map interactions.
- [x] Rerun the complete conserved OASIS denominator and record exact passes,
  later failures, mismatches, exclusions, and expected-error conservation.
- [ ] Decide whether the contained private behavior is mature enough for an ADR
  or should remain an explicit XSLT 1.0 profile exception.

## Reopening triggers

Move toward an ADR only when unchanged standards cases prove the marker can be
contained across result construction, temporary trees, serialization, and host
adapters. Reconsider or reject the experiment if it weakens ordinary escaping,
requires a public XDM node kind, bypasses result-byte accounting, or makes
well-formedness claims ambiguous without an owned host-visible contract.

## Review history

- 2026-09-30 -- Opened as Incubating from the 32-case OASIS frontier. Admitted
  only a private marked-text reference experiment and rejected silent ignore,
  direct executor serialization, and parsing authored text as markup.
- 2026-09-30 -- The first reference tranche retains a private plan/result marker,
  preserves it through invocation-owned temporary-tree copies, discards it in
  string constructors, and keeps ordinary escaping as the oracle. The complete
  sweep reaches 2,316 / 3,173 exact matches (73.00%); interaction and public
  well-formedness/accounting questions remain open.
