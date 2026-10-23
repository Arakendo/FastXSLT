# Campaigns

A campaign groups the paperwork for a bounded blocker or delivery objective:
scope, promotion ledger, evidence, review dispositions and handoff checkpoints.
Create one when work needs sustained tracking across tranches, not for every
ordinary fix. Campaigns do not introduce another architectural approval layer.

## Current campaigns

| Campaign | Status | Next boundary |
| --- | --- | --- |
| [Processor-edition profile](processor-edition-profile/README.md) | Promotion scope proposed; R0 awaiting approval | Review the finite ledger before further feature expansion |

## Historical campaign collections

These collections organize existing evidence; they do not reopen or close work.
Authoritative dispositions remain in the central AR/ADR records linked by the
evidence. Corpus counts and observations retain their original profiles/dates.

| Collection | Records migrated |
| --- | ---: |
| [OASIS XSLT 1.0](oasis-xslt10/README.md) | 620 |
| [XSLT 3.0 corpus](xslt30-corpus/README.md) | 195 |
| [QT3 corpus](qt3-corpus/README.md) | 55 |
| [Prepared-document capacity](prepared-document-capacity/README.md) | 28 |
| [ASP.NET hosting](aspnet-hosting/README.md) | 24 |
| [Native registry retention](native-registry-retention/README.md) | 14 |
| [Performance optimization](performance-optimization/README.md) | 10 |
| [Preparation pipeline](preparation-pipeline/README.md) | 8 |
| [Isolated transport batching](isolated-transport-batching/README.md) | 6 |
| [Whitespace visibility](whitespace-visibility/README.md) | 5 |
| [Bounded DTD](bounded-dtd/README.md) | 5 |
| [Adversarial remediation](adversarial-remediation/README.md) | 4 |
| [Finite dispatch](finite-dispatch/README.md) | 2 |
| [Competitive benchmarks](competitive-benchmarks/README.md) | 2 |
| [WASM target](wasm-target/README.md) | 1 |

Three further records moved into the active processor-edition campaign above:
**982 records migrated in total**. The
[migration manifest](evidence-migration-2026-10-22.json) records old/new paths
and before/after SHA-256 values; the [migration report](evidence-migration-2026-10-22.md)
records preservation checks. Markdown changes rebase references; raw JSON
measurements retain identical bytes. Shared foundational/design evidence stays
in the [Evidence index](../Evidence/README.md).

## Folder convention

Use a stable topic slug under `docs/Campaigns/`. Start small:

```text
<topic>/
  README.md              objective, status, scope and current next step
  promotion-ledger.md    when a finite admission ledger is needed
  evidence/              dated observations and verification records, as needed
  checkpoints/           dated handoffs or resumable checkpoints, as needed
```

Create supporting directories only when they have documents to hold. Additional
plans or review notes belong with the campaign when they serve that objective.
Keep one authoritative ledger rather than copying it into the roadmap, a handoff
and an evidence report. Each supporting record identifies its scope, date and
observed revision; the campaign index links to the current records.

ADRs, proposed ADRs and Architectural Reviews stay in their existing central
numbered collections. Link to them; do not move or duplicate them. A campaign
plan remains execution guidance and an evidence record remains an observation:
neither becomes an architectural contract by living in this folder. Scope
approval and promotion still follow the applicable accepted decisions.

Keep the roadmap concise and link to the campaign. New campaign-specific
evidence and handoffs go here; shared/cross-campaign evidence can remain in
`docs/Evidence/`. Historical records can move into a coherent campaign with
repository-wide link updates and a migration manifest; preserve their outcomes,
dates and limitations. Large corpus collections use subject-based evidence
subfolders rather than recreating one oversized directory. This organization
does not retrofit ADR-0026's pilot governance onto historical campaigns.

At closure, mark the campaign complete, deferred or abandoned, record the
disposition and remaining work in its README, and preserve its ledger/evidence
at the same paths. Update this index; do not delete the paper trail.
