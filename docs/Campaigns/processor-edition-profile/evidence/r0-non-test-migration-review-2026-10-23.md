# R0 non-test migration and second-interval review

**Subsequent disposition, 2026-10-23:** the maintainer approved **Promote** and
closed P09 (R0 9/9). See the [authoritative ledger](../promotion-ledger.md#final-promotion-disposition--2026-10-23).
The review-time observations and request below are preserved as recorded; the
migrated selected report and run manifest are now the R0 promotion baseline.
No source, raw report, default, public selector or feature ceiling changes as a
result of recording this disposition.

Date: 2026-10-23. The maintainer approved this continuation after the first
three-tranche review. Scope remains R0: the frozen incomplete legacy slice,
not the baseline compiler's entire language surface.

## Outcome

All three continuation objectives have executable evidence: non-test migration,
entry/semantic qualification, and control/cost/full-corpus verification. **P01–P08
are closed for this bounded candidate; P09 awaits maintainer promotion review.**
This is 8/9 obligation closure, not 8/9 of XSLT 1.0, effort, or release readiness.
No promotion, default, public selector or conformance claim has been made.

The explicit entry is `runtime::resource_compiler::compile_selected_legacy_resource`.
It consumes the existing sealed resolver and graph limits, loads the ordinary
bounded graph, and routes only to selected lowering. Strict preprocessing and
qualification are supplied by that lowering; unsupported shapes do not retry
through the baseline. The OASIS measurement adapter calls this same entry.

Non-test compilation now includes the bounded selected legacy lowerers, required
shared helpers, strict content filtering and legacy artifact stamps. Modern
edition selection, original instruction/root probes, dependency-shell probes and
their test fixtures remain test-only. Existing adapter defaults are unchanged.
The earlier strict-component transitional dead-code annotations were removed.

The historical `processor_edition_reference` module path remains private.
Production imports are explicit; test fixture imports remain test-gated. There
is no new crate, evaluator, mutable policy, cache, acquisition authority or unsafe
surface. Existing keys, arguments, conditions and graph child modules retain
their ownership seams; this migration does not merge them into another compiler.

## Three-tranche account and finite closure map

| Tranche / obligation | Closure evidence on migrated candidate |
| --- | --- |
| 1: P01 non-test entry | Ordinary workbench and official WASM builds compile the selected resource entry and its lowering dependencies. Frozen admission predicates remain intact. Direct-entry tests prove legacy behavior, strict qualification, source-owner release and no baseline fallback. The retained independent root-template reference compares success, dormant/reached failure and fallback execution against migrated lowering. Strict preprocessing retains its separate differential oracle. |
| 2: P02 version and entry scope | Existing numeric/local-version, conditional/message/computed-container, output and mixed-module witnesses replay against the migrated lowerers. `body_excluding`/child lowering retain `scoped_version`; declaration validators keep their guarded options. The explicit entry tests cover `stylesheet`/`transform`, declared versions 1/2/3, mixed-version include and the excluded simplified/embedded shapes. No local declaration selects the processor. |
| 2: P03 recognition | Known-newer `xsl:sequence` still selects legacy fallback through an actual include graph; original ForwardComp replay and recognition/capability refusal controls pass. Modern missing-fallback controls remain separate test-only witnesses. No backlog facility becomes unknown merely because it is unimplemented. |
| 2: P04 expression timing | The [finite owner map](r0-p04-p05-boundary-audit-2026-10-23.md#p04-finite-owner-map) is unchanged. Its migrated helpers and existing lexical/short-circuit tests pass; direct sealed-entry controls preserve reached versus dormant property-arity failure and static unbound names. No broad compiler-error catch or grammar expansion was introduced. |
| 2: P05 introspection | The six-owner, two-version matrix is now repeated through the sealed resource entry: 12 admitted controls and 36 explicit refusals. QName-aliased numeric processor reporting remains `1`, independent of declared version; availability and vendor queries stay Unsupported, not invented false/empty results. No general availability catalog or conformance claim is added. |
| 2: P06 qualification | The [qualification inventory](r0-p06-qualification-audit-2026-10-23.md#qualification-route-inventory) survives migration. The only new non-test context constructor supplies the coherent legacy/strict pair. Baseline context guards and checked mergers remain intact. Qualification/isolation/replacement/lifetime tests and the direct mixed-version include test pass. No artifact is baseline-lowered then selected-stamped. |
| 3: P07 controls and cost | New direct-entry control sweeps compare every insufficient XDM budget with the exact required boundary, check `FXCT0002 / Limit`, active charged cancellation `FXCT0001 / Cancelled`, original provenance, retained old artifact and recovery. Existing execution controls and sealed-resource tests pass. Compile and warm costs are separately reported below. |
| 3: P08 complete replay | Both selected/strict and ordinary baseline/modern replay all 3,173 unchanged cases. Source, overlay, toolchain, corpus and report identities are captured; original expectations are unchanged. Exact matches do not move; refusal/error-credit accounting is corrected as described below. |
| P09 pending review | Final repository verification, official WASM and pinned conformance checks pass. Maintainer disposition is still required before measured-profile admission; this report does not perform that disposition. |

The older evidence statements that selected lowering/context construction was
test-only remain historical. This report updates current ownership without
rewriting those observations. No new mandatory obligation or deferred family
was added.

## Two refusal defects repaired, without adding feature support

1. A simplified/literal-result stylesheet carrying `xsl:version` reached the
   standard-root validator and was classified Invalid. A failing direct-entry
   test established this. The selected graph now refuses that unsupported entry
   explicitly before lowering; arbitrary non-stylesheet XML remains distinct.
2. The single-dependency shortcut bypassed the embedded-child entry check and
   attempted to lower the enclosing document root. A failing child-fragment test
   established Invalid instead of Unsupported. It now validates the child's
   selected root before the shortcut, preserving child-resource provenance.

Principal fragments are already refused by the resolver as bases, before XML
admission; that failure has no fabricated stylesheet location. Dependency
fragments can be resolved by the shared loader, but the selected profile still
refuses embedded roots. These are P01/P02 repairs, not a new obligation or support
for simplified/embedded stylesheet execution. No unresolved provisional blocker
was found in this interval.

## Conserved corpus results

- [Selected legacy / strict report](r0-migrated-selected-report-2026-10-23.txt):
  **358/3,173 exact**, two mismatches. All terminal totals remain conserved.
- [Declaration-driven baseline / modern report](r0-migrated-baseline-report-2026-10-23.txt):
  **2,504/3,173 exact (78.92%)**, 16 mismatches. This is a fresh replay, not a
  relabeling of the historical production observation.
- Selected initialization failures remain 2,785: 122 Invalid, 2,649 Unsupported,
  14 missing-resource failures. Another three missing catalog inputs occur before
  initialization. See the full report for all exclusions and terminal counts.
- The simplified-entry repair reclassifies 25 initialization failures from
  Invalid to Unsupported. Ten were expected-error catalog entries: selected
  expected-error observations therefore change **125 → 115**, and nonsemantic
  uncredited failures **301 → 311**. No unsupported outcome earns an error credit.
- The 431 expected-error cases are now 115 semantic Invalid observations
  (105 initialization + 10 execution), 311 nonsemantic/uncredited failures,
  three unavailable inputs and two unusable archival error expectations.
  These are not exact-code/error-phase conformance credits or output passes.

The original two selected output mismatches remain unchanged and uncredited;
the [before-migration report](r0-p08-full-selected-measurement-2026-10-23.md)
identifies them. This review does not declare their archived expectations wrong.
They do not represent a newly introduced migration regression. Completing R0
does not bring the selected profile to 78.92% or earn additional production cases.

## Bounded cost observation

[Raw samples](r0-migrated-cost-report-2026-10-23.txt) include two runs, five
alternating-order windows per lane/fixture, 100 compile warmups, 2,000 measured
compilations and 5,000 warm executions per window. XML parsing/acquisition are
outside timing. Compile includes strict preprocessing, lowering and artifact
drop; warm includes shared execution **and serialization**, not evaluator-only
time. The comparison is the retained root-template reference, not another engine
or the full production compiler. These short micro-windows are not a publication
benchmark or ASP.NET throughput claim.

| Fixture | Reference compile ns, run 1 / 2 | Candidate compile ns, run 1 / 2 | Reference warm ns, run 1 / 2 | Candidate warm ns, run 1 / 2 |
| --- | ---: | ---: | ---: | ---: |
| Text | 1,558 / 1,563 | 1,750 / 1,777 | 640 / 693 | 643 / 669 |
| Fallback | 2,135 / 2,110 | 2,418 / 2,340 | 642 / 837 | 639 / 806 |
| Conditional | 3,071 / 3,169 | 3,092 / 3,110 | 687 / 723 | 688 / 724 |

Values are within-run medians. The text/fallback compilation increase is about
0.19–0.28 microseconds, consistent with the candidate's broader matched-module
declaration/admission checks; this is an explanation from the changed work, not
a profiler attribution. Conditional compile results straddle parity. Warm costs
have no consistent penalty in these fixtures and show visible cross-run noise;
no universal speedup or no-regression guarantee follows. Compile XDM charges are
identical between lanes: 18 for text, 27 for fallback and conditional. Nothing
was optimized or had its resource charges relaxed to improve these numbers.

## Validation and identity

- **2,300 workspace tests passed** (2,256 core + 22 native + 4 WASM + 18 worker).
- **230 policy/reference probes** and **24 selected replay/control tests** passed.
- fmt, Clippy, documentation, local links, unsafe-surface and pinned corpus checks
  passed; official `wasm32-unknown-unknown` build passed.
- Both full corpus runs and both release cost runs passed. Cache hard-link
  fallback-to-copy warnings were environmental, not failed gates.
- [Run/source manifest](r0-migrated-run-identity-2026-10-23.json) captures 437
  source/overlay identities, Rust 1.95.0 details, unchanged corpus digest and
  output hashes. Candidate is the dirty tree over `b4c92004`, not that clean commit.

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
cargo test -p fastxslt --all-features non_test_selected_entry
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10
./scripts/measure-oasis-xslt10.ps1
cargo test -p fastxslt --all-features --release r0_selected_migration_cost_probe -- --ignored --nocapture
```

## Review request

The second interval is **3/3 complete**. Review the bounded candidate for
measured-profile promotion under R0, or request a specific remaining witness.
Promotion would admit only the frozen incomplete crate-private profile, with
explicit refusals and the measured report. It would not select public APIs,
adapter defaults, a complete edition claim, or broader feature migration.
