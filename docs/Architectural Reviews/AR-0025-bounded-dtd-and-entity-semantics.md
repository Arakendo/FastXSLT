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
| Related evidence | Complete local OASIS XSLT 1.0 sweep at commit `d9e12c9f` and the 2026-09-30 post-checkpoint frontier sample |

## Architectural question

Can FastXSLT admit a bounded, authority-explicit subset of XML DTD and entity
semantics sufficient for standards behavior such as typed IDs and unparsed
entity metadata without granting ambient filesystem/network access, permitting
unbounded expansion, or making the current parser dependency the owner of XDM
semantics?

## Trigger and evidence

The conserved 3,173-case OASIS XSLT 1.0 denominator reached 2,348 exact XML
comparisons (74.00%). At that checkpoint, 87 standard source documents and 30
standard stylesheets stop at the deliberate `dtd-forbidden` boundary. Related
standard cases exercise `id()` over DTD-typed attributes and
`unparsed-entity-uri()` over entity declarations, so merely ignoring the DTD
would initialize more documents while silently producing wrong XSLT results.

This is sufficient pressure to study the boundary, not to enable general DTD
processing. The current parser adapter has no ambient resolver and rejects DTDs
before XDM construction. No complete inventory yet separates internal-only
declarations, sealed external dependencies, parameter entities, validation
requirements, entity-expansion shape, or cases that depend on typed metadata.
No adversarial expansion or resource-accounting evidence exists.

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
- It is unknown which declaration constructs dominate the corpus, how much
  exact coverage each bounded slice unlocks, and whether the current parser can
  remain the tokenization mechanism without taking ownership of policy.
- It is unknown whether a useful supported XML/XSLT profile can exclude
  external subsets while making a credible conformance statement.

## Disposition

**Incubating.** Preserve DTD denial as the default and admit only a private,
safe, bounded internal-subset experiment after case inventory. Do not enable a
dependency resolver, ambient filesystem/network access, validation claims,
public DTD types, or a conformance statement. External DTD/resource support
requires separate sealed-snapshot evidence and a later decision.

## Required follow-up

- [ ] Inventory the 117 directly blocked standard cases by internal/external
  subset, declaration type, entity kind, typed-ID dependency, defaulted
  attribute dependency, and expected output method.
- [ ] Identify related `id()` and `unparsed-entity-uri()` cases whose first
  visible failure occurs after parsing.
- [ ] Build a safe internal-subset reference parser with explicit declaration,
  nesting, reference, replacement-size, and total-work limits.
- [ ] Prove that DTD denial remains the default and that no URI or system/public
  identifier causes ambient acquisition.
- [ ] Preserve original-byte provenance and deterministic diagnostics across
  entity replacement and declaration failures.
- [ ] Add typed-ID and unparsed-entity metadata to XDM only through a reviewed
  private representation; keep parser-native types contained.
- [ ] Run entity-expansion, cycle, malformed-declaration, cancellation, and
  memory-pressure adversarial tests.
- [ ] Rerun the complete conserved OASIS denominator and report exact passes,
  later failures, mismatches, and exclusions without treating initialization as
  conformance.
- [ ] Decide whether an internal-only profile is sufficient or whether sealed
  external-resource resolution deserves a follow-up experiment and ADR.

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
