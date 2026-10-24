# Processor-edition profile campaign

Status: **R0 complete (9/9); B07 3/3 complete — stopped for review**.

R0 moved the bounded selected XSLT 1.0 processor / strict preprocessing slice
into a crate-private non-test path. Current work grows ordinary breadth through
accepted shared semantics, with full OASIS replay and separate ordinary-baseline
accounting. No public selector, default change or conformance claim is authorized.

## Current work

- [B07 review](evidence/b07-review-2026-10-23.md): fresh ranking chose value
  expressions. Constant Booleans, explicit relative paths and final attribute
  reads add **14 selected exact (513 → 527)**; ordinary **2,504 unchanged**.
  All gates/WASM pass; no new blockers. Follow the [live register](breadth-workflow.md).

- [B06 review](evidence/b06-review-2026-10-23.md): interior descendant copying,
  direct `current()` copying and bounded integer constants yield **509 → 513
  selected (+4)**; ordinary **2,504 unchanged**. All gates/WASM pass. No fourth
  slice before review. Carry the [live counter](breadth-workflow.md).

- B05: relative descendant iteration closes a named blocker without exact gain;
  direct current/root copying adds seven exact cases; child-comment copying adds
  one. All verification/WASM passes. [B05 review](evidence/b05-review-2026-10-23.md):
  **501 → 509 selected; 2,504 ordinary unchanged**. See the
  [live register](breadth-workflow.md). No fourth slice before review.

- [B04 review](evidence/b04-review-2026-10-23.md): selected **492 → 501 (+9)**,
  ordinary **2,504 unchanged**. All verification/WASM passes; no provisional
  blockers. No fourth slice starts before disposition.

- [B04-S2](evidence/b04-s2-final-wildcard-iteration-2026-10-23.md): final-wildcard
  iteration adds **7 exact**, reaching **500/3,173** selected; ordinary remains
  2,504. All gates/WASM pass. [S3](evidence/b04-s3-root-descendant-copy-2026-10-23.md)
  adds one exact case, reaching 501. The review boundary is reached.

- [B04-S1](evidence/b04-s1-final-wildcard-copy-2026-10-23.md): selected **493/3,173
  (+1)**, ordinary **2,504 unchanged**. Final wildcard copy paths admitted; all
  gates/WASM pass, no blocker. Next action is frontier reassessment and a bounded
  S2 declaration, without another routine approval request.

- [B03 review](evidence/b03-review-2026-10-23.md): selected **487 → 492 (+5)**,
  ordinary **2,504 unchanged**. C01 repaired; all gates/WASM pass. S2 removes the
  UTF-16 label blocker without exact gain; S3 makes `copy12` exact. No fourth
  slice started before review. Maintainer subsequently approved B04; its live
  scope/counter is in the [workflow](breadth-workflow.md).

- [B03-C01 UTF-16 ceiling audit](evidence/b03-utf16-ceiling-blocker-2026-10-23.md):
  repaired after maintainer approval; [verification](evidence/b03-utf16-ceiling-repair-2026-10-23.md)
  preserves both corpus reports. S2 separately admits the UTF-16 label through
  shared byte serialization. R0's denominator is unchanged.

- [B03-S1 ISO-8859-1 output](evidence/b03-s1-latin1-output-2026-10-23.md)
  is complete: selected **487 → 491 (+4)**, ordinary unchanged; gates/WASM pass.
  The next-candidate audit triggered the resource-control stop above.

- [B02 review](evidence/b02-review-2026-10-23.md): selected **482 → 487 (+5)**,
  ordinary **2,504 unchanged**, all gates/WASM pass, no provisional blocker.
  Maintainer approved B03. Fresh reassessment selected bounded ISO-8859-1 output
  admission first; see the live register for scope and stop criteria.

- [B02-S1 wildcard iteration](evidence/b02-s1-wildcard-iteration-2026-10-23.md)
  is complete: selected **483/3,173 (+1)**, ordinary unchanged. Gates/WASM pass.
  [S2 string copy](evidence/b02-s2-string-copy-2026-10-23.md) adds 3 exact;
  [S3 rooted iteration](evidence/b02-s3-rooted-iteration-2026-10-23.md) adds 1.
  The three-slice boundary is now reached.

- [B01 review](evidence/b01-review-2026-10-23.md): selected **449 → 482 exact
  (+33)**; ordinary **2,504 unchanged**. Repository/WASM gates pass, no provisional
  blocker. Maintainer approved continuation into B02 under the same boundaries.

- [Breadth workflow and live register](breadth-workflow.md) govern current work:
  declare each narrow slice, proceed without individual approval when eligible,
  and return after at most three completed semantic slices or an earlier trigger.
  Carry the register across commits, compaction, handoffs and resumed sessions.
- [Promotion ledger](promotion-ledger.md) remains the authoritative closed R0
  scope and obligation record; it is not the current breadth counter.
- R0 has nine fixed obligations; P01–P09 are closed for the bounded profile.
  Maintainer disposition: Promote. Q01 remains within P06; no provisional blocker remains.
- [Current migration/review report](evidence/r0-non-test-migration-review-2026-10-23.md):
  explicit non-test entry, entry/semantic audits, controls/cost, and both full
  corpus replays completed. R0 selected exact was 358/3,173; baseline was
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
- The maintainer approved the [post-R0 comment slice](evidence/post-r0-selected-comments-2026-10-23.md),
  now complete: selected exact **358 → 393/3,173**, ordinary baseline unchanged
  at **2,504/3,173**. `message11` passes unchanged. This is separate post-R0
  breadth; the closed ledger and promotion baseline retain their original scope.
  Its declared scope is complete; later slices are recorded independently.
- The next approved [named element-copy path slice](evidence/post-r0-selected-copy-paths-2026-10-23.md)
  is complete: selected exact **393 → 449/3,173**, ordinary baseline unchanged.
  `CopyOfFromDocumentRoot` passes unchanged. Both earlier checkpoints remain.
- [B01-S1 static-target processing instructions](evidence/b01-s1-selected-processing-instructions-2026-10-23.md)
  is complete: selected exact **449 → 466/3,173 (+17)**, ordinary unchanged.
  Verification and WASM pass.
- [B01-S2 wildcard copy](evidence/b01-s2-selected-wildcard-copy-2026-10-23.md)
  adds 3 exact cases; [B01-S3 named descendant iteration](evidence/b01-s3-selected-descendant-iteration-2026-10-23.md)
  adds 13. Both slices are complete and their batch review was approved.
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
