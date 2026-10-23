# Processor-edition profile campaign

Status: **R0 promotion scope proposed; awaiting maintainer approval**.

Objective: move the bounded selected XSLT 1.0 processor / strict preprocessing
slice into a crate-private non-test path and measure it against the full OASIS
denominator. Keep the declaration-driven baseline unchanged. No public selector,
default change or complete conformance claim is part of this campaign.

## Current work

- [Promotion ledger](promotion-ledger.md) is the single owner of proposed scope,
  obligations, provisional issues, two-track progress and the next review boundary.
- R0 proposes nine obligations; none is closed. The mandatory denominator is
  not approved yet. The folder move does not approve or change that scope.
- Next action: review R0, including Q01's proposed disposition under P06, before
  starting the proposed qualification, error/introspection and full-replay tranches.

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
