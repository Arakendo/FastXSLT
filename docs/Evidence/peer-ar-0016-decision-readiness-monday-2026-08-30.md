# Peer Review: AR-0016 Decision Readiness

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Reviewer | Monday |
| Subject | AR-0016 decision readiness after the visibility-view prototype |
| Outcome | Complete a bounded measurement matrix, reconcile stale review prose, and accept only the demonstrated strip-all visibility-view decision |

## Review assessment

The review found the semantic and lifecycle risks substantially retired for the
current surface. Differential verification already covered XPath navigation,
template selection, built-in rules, string values, copying, focus position and
size, concurrent strip/preserve execution, generation replacement, prepared
node identity, provenance, budgets, cancellation, and immutable prepared
storage.

The private invocation-owned visibility view was therefore considered the
leading implementation candidate, while the complete derived document remained
the safe semantic oracle. The preliminary timing and owned-capacity result was
large enough to justify completing decision-grade measurements rather than
reconsidering the representation.

## Promotion conditions

The review requested evidence sufficient to answer:

- one-shot construction-plus-execution cost against complete derivation;
- warm repeated invocation behavior and view latency distribution;
- peak and retained allocator-requested bytes attributable to each strategy;
- scaling across small, medium, large, deep, whitespace-heavy, and
  whitespace-light documents; and
- concurrent execution behavior without contention or representation leakage.

It also identified stale passages that still described the source-access
inventory, reference implementation, visibility prototype, and measurements as
future work. Those passages needed to distinguish an implemented leading
candidate from an accepted architectural decision before promotion.

## Recommended decision scope

The proposed accepted boundary was deliberately narrow:

> For the admitted `xsl:strip-space elements="*"` semantics, FastXSLT composes
> immutable prepared source XDM with an invocation-owned visibility view.
> Visible nodes retain prepared identity and provenance, and all
> source-semantic operations consume the effective view. The complete derived
> document remains a differential reference.

The review did not recommend admitting general whitespace-declaration matching,
`xsl:preserve-space`, import precedence, `xml:space`, schema-aware whitespace,
a public source-view abstraction, retained caches, streaming, or an unsafe
implementation.

## Disposition

The requested measurement matrix was completed in
`ar-0016-decision-measurement-matrix-2026-08-30.md`. The stale review prose was
reconciled, and ADR-0012 accepted the narrow invocation-owned strip-all view.
Broader whitespace semantics remain demand-gated.
