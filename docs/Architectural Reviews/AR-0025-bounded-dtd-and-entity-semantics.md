# AR-0025: Bounded DTD and Entity Semantics

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-09-30 |
| Last reviewed | 2026-09-30 |
| Scope | XML DTD declarations, entity expansion, typed IDs, parser authority, and resource limits |
| Trigger | The 74% OASIS XSLT 1.0 checkpoint leaves 117 standard-operation cases at explicit source or stylesheet DTD denial, with related `id()` and `unparsed-entity-uri()` cases behind the same XML semantics |
| Related ADRs | ADR-0002, ADR-0006, ADR-0007 |
| Related reviews | AR-0008, AR-0014, AR-0019 |
| Related evidence | [Post-74% frontier and DTD inventory](../Evidence/oasis-xslt10-post-74-frontier-and-dtd-inventory-2026-09-30.md); [bounded internal-entity reference path](../Evidence/ar-0025-bounded-internal-entity-reference-path-2026-09-30.md); [bounded typed-ID reference path](../Evidence/ar-0025-bounded-typed-id-reference-path-2026-09-30.md); [bounded defaults and stylesheet path](../Evidence/ar-0025-bounded-default-attribute-and-stylesheet-path-2026-09-30.md); [redundant namespace-default experiment](../Evidence/ar-0025-redundant-namespace-default-experiment-2026-09-30.md); [single sealed external-subset experiment](../Evidence/ar-0025-single-sealed-external-subset-experiment-2026-09-30.md) |

## Architectural question

Can FastXSLT admit a bounded, authority-explicit subset of XML DTD and entity
semantics sufficient for standards behavior such as typed IDs and unparsed
entity metadata without granting ambient filesystem/network access, permitting
unbounded expansion, or making the current parser dependency the owner of XDM
semantics?

## Trigger and evidence

The conserved 3,173-case OASIS XSLT 1.0 denominator reached 2,348 exact XML
comparisons (74.00%). At that checkpoint, 87 standard source inputs and 30
stylesheet inputs (28 principal modules and two dependencies) stop at the
deliberate `dtd-forbidden` boundary. Related
standard cases exercise `id()` over DTD-typed attributes and
`unparsed-entity-uri()` over entity declarations, so merely ignoring the DTD
would initialize more documents while silently producing wrong XSLT results.

This is sufficient pressure to study the boundary, not to enable general DTD
processing. The production parser adapter has no ambient resolver and rejects
DTDs before XDM construction. The complete runner now separates syntactic
internal/external, declaration, entity-kind, default-attribute, and typed-ID
pressure for the exact direct frontier. Whether each declaration is observable
in its expected result, and whether external bytes are already sealed under a
usable logical identity, still requires case interpretation. The first private
reference path has bounded cycle, nesting, reference, replacement-byte,
cancellation, and work-accounting tests; broad memory-pressure evidence does
not yet exist.

## Ownership and constraints

- The host owns acquisition authority. A system/public identifier or URI-shaped
  entity name never authorizes filesystem or network access.
- The sealed resource snapshot owns all bytes available to parsing. Any
  external subset or entity admitted experimentally must resolve only through
  explicit logical identity and snapshot membership.
- The XML boundary owns declaration parsing, entity replacement, XML edition
  rules, expansion limits, and structured diagnostics.
- XDM owns node identity, document order, attribute type information needed by
  XPath/XSLT, and unparsed-entity metadata after parsing. Parser-native DTD
  objects must not escape into XDM or public APIs.
- XPath/XSLT own `id()` and `unparsed-entity-uri()` semantics. Those functions
  must consume explicit document metadata rather than reopen resources or query
  a live parser.
- Entity expansion must be bounded independently by declaration count, nesting
  depth, replacement characters/bytes, references, and total XML work. Input
  byte limits alone do not bound expanded content.
- Cycles, recursive parameter entities, malformed declarations, and denied or
  missing dependencies must produce structured outcomes without partial XDM.
- External general entities, external parameter entities, and external subsets
  remain denied unless their bytes and identities were explicitly admitted.
- No implementation may weaken ADR-0002's memory-resident execution rule or
  AR-0014's identity/authority separation.
- The existing DTD-denying parser path remains the safe reference and default
  until a later ADR accepts a supported profile.

## Alternatives

### A. Retain complete DTD denial

This preserves the simplest security and authority model and remains an honest
profile boundary. It leaves a material XSLT 1.0 compatibility cluster and XML
typed-ID/unparsed-entity semantics unsupported.

### B. Admit a bounded internal-subset reference implementation

Parse declarations contained in the already admitted document, expand only
bounded internal entities, and retain only the typed-ID and unparsed-entity
metadata required by the selected semantics. External identifiers remain
denied. This is the leading first experiment because it adds no acquisition
authority and gives limit/accounting behavior a complete safe oracle.

This slice may still be insufficient for cases whose declarations depend on an
external subset or parameter entities. Those cases must stay visibly
unsupported rather than silently behaving as if no DTD existed.

### C. Resolve DTD dependencies from the sealed snapshot

Compose XML reference resolution with AR-0014's logical-identity and authority
model. The host admits all bytes before parsing; the parser may request only a
bounded logical reference from that snapshot. This could support external
subsets without ambient I/O, but base identity, catalog mapping, recursion,
byte accounting, and denial precedence need executable evidence first.

### D. Require hosts to pre-expand entities

This avoids DTD parsing inside FastXSLT but loses original-byte provenance,
changes parser diagnostics, makes typed ID and unparsed-entity metadata
difficult to preserve, and shifts XML semantics inconsistently into adapters.
It is not a general solution.

### E. Enable the parser library's ordinary DTD/entity resolver

This would delegate authority and security policy to dependency defaults and
could introduce ambient I/O or unbounded expansion. Rejected.

## Findings and uncertainties

- DTD support is now a measured compatibility frontier rather than speculative
  completeness work.
- The 117 directly blocked standard cases are not equivalent to 117 expected
  passes; some will expose later XSLT, comparison, or archival-fixture issues.
- A bounded internal-subset experiment can preserve the current no-ambient-I/O
  rule and should precede any external dependency mechanism.
- Ignoring declarations is not acceptable because `id()`, entity replacement,
  defaulted attributes, and unparsed-entity functions can make the DTD
  semantically observable.
- The direct-frontier declaration constructs are now inventoried and the first
  two bounded slices have exact corpus outcomes. It remains unknown how much of
  the external-identifier frontier can be supported without parameter-entity,
  validation, or ambient-authority semantics.
- It is unknown whether a useful supported XML/XSLT profile can exclude
  external subsets while making a credible conformance statement.
- A first safe reference path now proves bounded character-data general
  entities in text and ordinary attributes without adding acquisition
  authority. A second slice syntax-validates non-validating internal `ELEMENT`
  declarations while rejecting declaration families with unimplemented
  observable semantics. The
  current quick-xml tokenizer also attempts namespace-declaration expansion
  before the adapter can apply its entity table, so entity-bearing namespace
  values remain outside this slice rather than receiving partial semantics.
- The exact direct frontier is now measurement-owned rather than inferred from
  the broader catalog inventory: 113 cases have internal subsets, 102 name
  external identifiers, 41 contain attribute-list declarations, 27 contain an
  explicit default candidate, 17 declare entities, 13 have internal general
  entities, five have external general entities, and 11 are typed-ID
  candidates. No direct case contains parameter-entity, notation, or unparsed-
  entity pressure. These overlapping syntactic candidates do not yet prove
  that every declaration affects the expected result.
- The bounded reference parser admits 15 of the 117 exact-frontier resources.
  The first two entity/declaration-only resources, all ten typed-ID source
  resources, two source-default resources, and one stylesheet resource are
  syntactically admitted. The lower bound is now 2,361 / 3,173: eight typed-ID
  cases, both source-default cases, and the stylesheet case compare exactly;
  two manual-comparator cases remain uncredited. The original 87-source/30-
  stylesheet denominator remains conserved.
- The remaining external-identifier-free stylesheet cases rely on
  fixed/defaulted namespace or version attributes. The stylesheet group must
  affect namespace resolution before ordinary start-event handling; it cannot
  be implemented honestly as post-XDM attribute injection at the current
  tokenizer seam. The private reference path now proves ordinary source
  defaults and a stylesheet whose relevant namespace/version attributes are
  authored explicitly.

## Focused namespace/version experiment

The next experiment asks:

> Can DTD-derived namespace/version defaults participate in stylesheet parsing
> with original-byte provenance, bounded accounting, and deterministic
> diagnostics, without replacing the XML boundary wholesale?

The first inventory narrows that question. All eight examined internal-only
Microsoft `22-8` stylesheets author the root `xmlns:xsl` binding. Their DTDs
repeat the same fixed binding on selected descendant XSLT elements; only
`Attributes__81545` and `Attributes__81551` omit the root `version="1.0"` that
their DTD supplies. None of these eight cases requires a previously unbound
prefix to be understood by the tokenizer.

The leading experiment is therefore smaller than byte-stream rewriting:

1. admit a DTD-derived namespace declaration only when the tokenizer's current
   in-scope binding already has the identical normalized value;
2. retain that redundant local declaration in XDM namespace metadata so the
   derived tree does not erase where the declaration applies;
3. inject ordinary `version` defaults through the existing bounded default-
   attribute path; and
4. continue to reject any declaration needed to bind or rebind a name before
   tokenization.

This semantic-equivalence experiment is now executable. It does not implement
general namespace defaulting. The complete cases remain blocked earlier by the
shared source's external `plants.dtd` identifier, so the OASIS numerator does
not change. A provenance-preserving pre-tokenization layer remains a candidate
only if a real admitted case requires a missing or different binding before
QName resolution.

The implementation subsequently narrowed one more distinction: an unused
prefixed declaration may be retained as metadata when the prefix is not yet in
scope, because it cannot change tokenization of any QName in that start event.
An element or attribute that actually uses an unbound prefix still fails in the
tokenizer, while default-namespace establishment and changed existing bindings
remain unsupported.

Abort this experiment rather than widening it if it requires rewriting admitted
bytes, duplicating XML well-formedness parsing, exposing quick-xml internals,
mapping broad synthetic spans back to the original resource, or accepting a
binding whose equivalence cannot be established before XDM construction. A
parser-boundary replacement remains a separate architectural decision and is
not justified by a round coverage threshold.

## Focused sealed external-subset experiment

The next experiment asks a deliberately smaller question than general external
DTD support:

> Can one document resolve exactly one quoted `SYSTEM` external subset from an
> already sealed snapshot, under independent byte, declaration, expansion, XML
> work, and resolution-attempt bounds, without ambient acquisition?

The private path now proves that shape. The host-side measurement adapter
explicitly admits the sibling `plants.dtd` bytes and logical identity before
sealing. Preparation resolves the relative reference through a one-attempt
snapshot resolver; missing membership produces the existing structured
missing-resource outcome. The XML boundary accepts only the supplied quoted
`SYSTEM` reference, verifies it against the document declaration, attributes
external declaration failures to the external logical resource, merges the
bounded declaration tables, and constructs no partial XDM on failure.

The experiment deliberately excludes `PUBLIC` identifiers, recursive external
subsets, parameter entities, external general entities, validation, catalogs,
live callbacks, and filesystem/network fallback. Cross-subset declaration
shadowing also remains explicitly unsupported rather than approximating XML
precedence rules.

The first complete hash-verified OASIS replay opted in only the eight named
Microsoft `Attributes__81543` through `81551` standard cases (with absent
catalog ordinals excluded); all eight initialized, executed, and compared
exactly. A second replay generalized only the already-reviewed family: a
principal source named `Plants.xml` (case-insensitive), the exact quoted sibling
reference `SYSTEM "plants.dtd"`, a present case-local file no larger than 64
KiB, and declarations accepted by the same bounded grammar. This admits 101
catalog cases, including expected-error cases, without general external-subset
discovery.

The same mechanism also admits the 120-byte case-local `t04.dtd` used by
`Lotus/idkey_idkey04`; it contains only element declarations and one typed `ID`
attribute. Seventy-seven standard cases now complete the single-external-subset
parse. The strict lower bound is 2,437 / 3,173 (76.80%): 76 exact gains over the
internal-only baseline, with one additional newly executing case exposed as a
visible mismatch. The conserved 117-case direct frontier is now 15 internal-
subset parses, 77 single sealed external-subset parses, and 25 explicit
unsupported outcomes.

## Disposition

**Incubating.** Preserve DTD denial as the default and admit only private, safe,
bounded internal-subset and single sealed-external-subset experiments after
case inventory. Do not enable live resolution, ambient filesystem/network
access, validation claims, public DTD types, or a conformance statement. A
supported external-resource profile requires a later decision.

## Required follow-up

- [x] Add a reproducible catalog inventory for standard principal source and
  stylesheet inputs containing DTDs.
- [x] Inventory the 117 directly blocked standard cases by internal/external
  subset, declaration type, entity kind, typed-ID dependency, defaulted
  attribute dependency, and expected output method.
- [ ] Identify related `id()` and `unparsed-entity-uri()` cases whose first
  visible failure occurs after parsing.
- [x] Build a safe internal-subset reference parser with explicit declaration,
  nesting, reference, replacement-size, and total-work limits.
- [x] Prove that DTD denial remains the default and that no URI or system/public
  identifier causes ambient acquisition.
- [ ] Preserve precise original-byte provenance across entity replacement and
  declaration failures. The single-external-subset experiment currently
  preserves the owning logical resource and a deterministic whole-resource
  span, not declaration-level external offsets.
- [x] Inventory all eight internal-only Microsoft `22-8` stylesheet shapes and
  test the redundant-namespace/default-version semantics with exact frontier
  accounting and explicit rejection for any non-equivalent binding. Their
  shared external-subset source keeps the complete cases unsupported.
- [x] Add typed-ID metadata to XDM only through a reviewed private
  representation; keep parser-native types contained. Unparsed-entity metadata
  remains a separate incomplete slice.
- [ ] Run entity-expansion, cycle, malformed-declaration, cancellation, and
  memory-pressure adversarial tests.
- [x] Rerun the complete conserved OASIS denominator and report exact passes,
  later failures, mismatches, and exclusions without treating initialization as
  conformance.
- [x] Test one bounded sealed external-subset resolution attempt against a real
  corpus family without ambient acquisition or broader external DTD semantics.
- [ ] Decide whether the demonstrated single-external-subset profile deserves
  a supported ADR or should remain a private compatibility experiment.

## Reopening triggers

Move toward an ADR only when the internal reference path has bounded adversarial
evidence, exact corpus outcomes, deterministic accounting, and no ambient
authority. Reject or retain full denial if useful semantics require implicit
I/O, unbounded expansion, parser-owned public types, or unacceptable prepared
memory amplification.

## Review history

- 2026-09-30 -- Opened as Incubating from the post-74% OASIS frontier. DTD
  denial remains the default; only a bounded internal-subset reference
  experiment is admitted after inventory.
- 2026-09-30 -- Added the first private bounded internal general-entity
  reference path. It handles character-data replacement in text and ordinary
  attributes, charges declaration/expansion work, and rejects external
  identifiers and unimplemented declaration semantics. Production entry points
  retain complete DTD denial.
- 2026-09-30 -- Added direct-frontier accounting to the complete OASIS runner.
  All 117 standard DTD denials now contribute reproducible role, declaration,
  entity-kind, typed-ID/default candidate, and expected-comparator counts.
- 2026-09-30 -- Added bounded grammar validation for non-validating internal
  `ELEMENT` declarations and a measurement-only prepared-source route. Two
  unchanged standard cases become exact; 115 cases remain explicitly
  unsupported, and production DTD denial is unchanged.
- 2026-09-30 -- Added private non-defaulting `CDATA`/`ID`/`IDREF` declaration
  metadata, a per-document XDM ID index, and charged XSLT 1.0 `id()` semantics.
  All ten typed-ID source cases initialize and execute, eight compare exactly,
  and two manual-comparator cases remain uncredited. The strict lower bound is
  2,358 / 3,173 (74.31%); 105 direct DTD-frontier cases remain explicitly
  unsupported.
- 2026-09-30 -- Added bounded literal/fixed attribute defaults and enabled the
  same private parser for explicitly selected stylesheet compilation. Two
  source-default cases and one stylesheet-entity case become exact. The strict
  lower bound reaches 2,361 / 3,173 (74.41%); the conserved direct frontier is
  15 parsed and 102 explicitly unsupported. DTD-derived namespace declarations
  remain rejected at the pre-tokenization boundary.
- 2026-09-30 -- Narrowed the apparent namespace seam without rewriting input
  bytes. The private parser may retain a DTD-derived namespace declaration only
  when an identical binding is already in scope, and ordinary defaulted
  stylesheet version attributes compile through the existing bounded path.
  Missing or changed bindings remain rejected. The motivating `22-8` cases
  still stop at their shared source's external `plants.dtd` identifier, so the
  strict lower bound remains 2,361 / 3,173 (74.41%).
- 2026-09-30 -- Added one private, bounded, sealed-snapshot `SYSTEM` external-
  subset experiment. Exactly eight pre-inventoried Microsoft `22-8` cases opt
  in; all eight compare exactly. The strict lower bound reaches 2,369 / 3,173
  (74.66%), with 15 internal parses, eight single-external parses, and 94
  explicit unsupported outcomes across the conserved 117-case DTD frontier.
- 2026-09-30 -- Replayed the same mechanism across the reviewed case-local
  `plants.dtd` family without widening the parser grammar or authority model.
  The strict lower bound reaches 2,436 / 3,173 (76.77%); the DTD frontier is 15
  internal parses, 76 single-external parses, and 26 explicit unsupported
  outcomes. One newly executing case remains a visible mismatch.
- 2026-09-30 -- Admitted the separate 120-byte `t04.dtd` typed-ID source through
  the same one-attempt sealed path. `Lotus/idkey_idkey04` compares exactly, the
  lower bound reaches 2,437 / 3,173 (76.80%), and 25 direct DTD-frontier cases
  remain explicitly unsupported.
