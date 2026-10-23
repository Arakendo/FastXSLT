# Historical evidence campaign migration

Date: 2026-10-22. Scope: documentation organization only.

The former flat Evidence directory contained 1,045 records and its README.
The migration moved 982 records into 16 campaign collections and retained 63
shared design/foundational records in Evidence. No historical record was dropped.
New README indexes are navigation aids, not additional evidence observations.

## Organization

Named architectural experiments follow their topic (for example bounded DTD,
prepared-document capacity and processor-edition policy). Corpus records follow
their suite campaign, then a primary subject such as bindings, paths, templates
or serialization. Subjects are navigation categories, not claims of exclusive
ownership: a record may cover several mechanisms and retains its existing links.
Shared design and early lifecycle evidence remain separately indexed.

Numbered ARs and ADRs stay in their central collections. Their references were
updated where destinations moved; their decisions were not changed. Existing
dirty implementation work and the proposed promotion-ledger approval status
are independent of this migration. Historical collections do not acquire the
ADR-0026 pilot governance retroactively.

## Preservation and verification

The [manifest](evidence-migration-2026-10-22.json) records each source/destination
and SHA-256 before and after the move. The source hashes describe the working
files at migration time, including pending edits, not necessarily HEAD.

- 982 unique source/destination pairs; every destination exists and matches its
  recorded post-migration hash, and none of the old source paths remains.
- All 11 raw JSON measurement records have identical before/after hashes.
- Markdown relative links were rebased from their original document location;
  repository-root path references were updated. Record names, dates, headings,
  observations, limitations and corpus accounting were retained.
- `scripts/check-markdown-links.ps1`: passed, 2,060 Markdown files checked.
- `git diff --check`: passed.

No engine validation was required for this documentation relocation. Prior
implementation verification is not relabeled as a run for this migration.
