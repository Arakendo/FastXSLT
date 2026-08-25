# AR-0004: Structured Diagnostics and Boundary Error Identity

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Engine diagnostics, operation failures, and host translation |
| Trigger | M1 needs machine-readable negative results; the peer database review exposed the cost of string-shaped failures |
| Related ADRs | ADR-0001, ADR-0002 |
| Related evidence | `docs/Evidence/peer-database-documentation-review-2026-08-25.md`; future M1 negative cases and host consumers |

## Architectural question

What structured identities, categories, details, provenance, and compatibility
rules should FastXSLT expose for semantic diagnostics and operation failures,
and where should local engine errors be translated into that boundary contract?

## Trigger and evidence

The first vertical slice must distinguish invalid XML or stylesheet syntax,
unsupported language behavior, missing or denied resources, budget exhaustion,
host cancellation, external host failure, and internal engine failure. Human
messages alone are insufficient for applications, test harnesses, batch result
collection, logs, and a future ASP.NET adapter.

The SDD already requires structured, source-located diagnostics and separates a
reportable semantic outcome from failure to produce a trustworthy result. The
Tosumu peer demonstrates a useful shape: stable identity, a small policy
category, message, structured details, and preserved cause, with local errors
translated only at boundaries. That demonstrates consumer value, not the
correct FastXSLT vocabulary.

No FastXSLT parser, compiler, runtime, facade, or host adapter currently emits a
real failure. The standards profile and its normative error codes are unresolved.
There is no evidence for exact Rust types, category names, code prefixes,
serialization fields, or stability promises.

## Ownership and constraints

- The XML adapter owns parser-mechanical failures and source offsets it can
  establish; it does not own XPath or XSLT meaning.
- XPath and XSLT own standards-defined static and dynamic errors for the
  admitted profile.
- Resource and runtime policy own denied authority, missing admitted resources,
  cancellation, and exhausted budgets.
- Local modules may use focused private errors, preserving causes and context
  needed by the facade.
- The facade owns the host-neutral boundary report. Host adapters own
  translation to exceptions, statuses, logs, UI messages, or exit behavior.
- Standards identifiers and FastXSLT operational identifiers must remain
  distinguishable when their authority differs.
- Diagnostic details may expose source text, resource identity, or host
  provenance and require explicit disclosure and retention policy.

ADR-0002 forbids diagnostics from becoming tokens for reopening source files.
ADR-0001 discourages a shared error crate until independent dependency or
release pressure exists.

## Alternatives

### A. Display strings and local Rust errors only

Small initially, but hosts and harnesses must parse unstable text. Localization
or wording improvements become compatibility hazards.

### B. One public global error enum

Exhaustive matching is attractive, but it couples unrelated layers, makes every
new case a public change, and conflates reportable findings with operation
failure.

### C. Focused local errors translated into a small boundary report

Private layers keep role-specific errors. The facade exposes stable identity
where needed, a small policy category, message, structured details, locations,
related diagnostics, and a preserved cause that need not be serialized. This
supports multiple hosts without a repository-wide mega-enum.

### D. Versioned serialized envelope from the first slice

This could serve process and FFI boundaries, but choosing it before AR-0002
selects a host mechanism risks treating transport as the semantic model.

## Findings and uncertainties

- Machine-readable identity and category are required wherever callers make
  policy decisions; display text is not a control-flow contract.
- Reportable semantic findings and failure to produce a trustworthy result are
  distinct concepts.
- Local error ownership plus deliberate boundary translation is the strongest
  starting direction.
- Details, source locations, related locations, and causes need bounded
  retention and disclosure rules.
- A standards error code and a FastXSLT operational code have different owners.
- Batch-level and per-request failure semantics remain dependent on AR-0003.
- Exact code format, categories, Rust representation, serialization, forward
  compatibility, and host mapping lack executable evidence.

## Disposition

**Incubating.** M1 may introduce the smallest private structured report needed
for real cases, but must not publish an aspirational catalog or global error
hierarchy. Public identities and categories require an engine path, negative
fixtures, and consumer-shaped handling evidence.

## Required follow-up

- [ ] Exercise invalid XML, invalid stylesheet syntax, an unsupported feature,
  a missing resource, a denied resource, and an enforced limit or cancellation
  through the first vertical slice.
- [ ] Record which conditions are reportable outcomes and which prevent a
  trustworthy result.
- [ ] Map standards-defined errors separately from FastXSLT operational errors.
- [ ] Prototype focused local errors and one facade translation without adding
  a shared error crate.
- [ ] Prove callers never parse display strings and unknown future identities
  have a safe handling path.
- [ ] Define source/detail disclosure, retention bounds, cause behavior, and
  panic containment at public or FFI boundaries.
- [ ] Exercise the shape through an ASP.NET candidate or independent consumer.
- [ ] If evidence converges, propose an ADR defining ownership, minimum stable
  fields, compatibility, and documentation/code synchronization.

## Reopening triggers

After disposition, reopen or supersede this review when the standards profile
adds another error model, batch semantics require partial results, a host cannot
represent the selected shape safely, diagnostic retention becomes material, or
a serialized form needs versioning.

## Review history

- 2026-08-25 -- Opened as Incubating from deferred findings in the Tosumu
  documentation review.
