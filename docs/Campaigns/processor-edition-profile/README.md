# Processor-edition profile campaign

Status: **Complete — R0 measured-profile promotion approved 2026-10-23; 9/9 obligations closed**.

Objective: move the bounded selected XSLT 1.0 processor / strict preprocessing
slice into a crate-private non-test path and measure it against the full OASIS
denominator. Keep the declaration-driven baseline unchanged. No public selector,
default change or complete conformance claim is part of this campaign.

## Current work

- [Promotion ledger](promotion-ledger.md) is the single owner of approved scope,
  obligations, provisional issues, two-track progress and the next review boundary.
- R0 has nine fixed obligations; P01–P09 are closed for the bounded profile.
  Maintainer disposition: Promote. Q01 remains within P06; no provisional blocker remains.
- [Current migration/review report](evidence/r0-non-test-migration-review-2026-10-23.md):
  explicit non-test entry, entry/semantic audits, controls/cost, and both full
  corpus replays completed. Selected exact remains 358/3,173; baseline remains
  2,504/3,173. The incomplete crate-private profile is now promoted; no public,
  default or conformance change was made.
- First interval history: [qualification audit and Q01 repair](evidence/r0-p06-qualification-audit-2026-10-23.md),
  [expression/introspection audit](evidence/r0-p04-p05-boundary-audit-2026-10-23.md),
  and [full selected-profile replay](evidence/r0-p08-full-selected-measurement-2026-10-23.md).
- The second interval and promotion review are complete. Preserve the
  [selected report](evidence/r0-migrated-selected-report-2026-10-23.txt) and
  [run identity](evidence/r0-migrated-run-identity-2026-10-23.json) as the R0
  promotion baseline. Original references remain available for differential checks.
- Next: resume case-driven OASIS breadth with explicit profile-labelled replay.
  Deferred features remain deferred; promotion does not expand R0 or choose a
  public selector. Record future gains separately rather than overwriting R0.
- [First P01 dependency checkpoint](evidence/r0-p01-strict-component-2026-10-23.md):
  strict preprocessing builds outside tests; full selected report unchanged.

## Authority

- [ADR-0025: processor-edition policy](../../ADR/ADR-0025-compilation-graph-scoped-processor-edition-policy.md)
- [ADR-0023: preprocessing profiles](../../ADR/ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md)
- [ADR-0026: evidence-gated promotion](../../ADR/ADR-0026-evidence-gated-profile-promotion-and-frontier-progress.md)

## Evidence and handoffs

- [Existing integration history](evidence/adr-0025-version-spelling-checkpoint-2026-10-17.md)
- [Latest qualification safeguard](evidence/adr-0025-version-spelling-checkpoint-2026-10-17.md#2026-10-22-promotion-audit-and-qualification-safeguard)
- [Project roadmap](../../Plans/roadmap.md)

The [evidence index](evidence/README.md) includes the migrated historical records.
Put subsequent dedicated campaign observations in `evidence/` and handoffs in
`checkpoints/`, creating the latter when needed, and link them here and from the
relevant ledger row. Do not append unrelated feature campaigns to this folder.
