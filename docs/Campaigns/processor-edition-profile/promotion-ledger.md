# ADR-0025 processor-profile promotion ledger

- Revision: R0 proposed 2026-10-22; **approved by maintainer 2026-10-23**
- Governance: [ADR-0026](../../ADR/ADR-0026-evidence-gated-profile-promotion-and-frontier-progress.md)
- Semantic authority: [ADR-0025](../../ADR/ADR-0025-compilation-graph-scoped-processor-edition-policy.md), [ADR-0023](../../ADR/ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md)
- Authoritative mandatory denominator: **9 obligations**
- Obligations: **9/9 closed — Promote**, maintainer disposition 2026-10-23
- Review counter: **3/3 semantic tranches completed in the second approved interval; review complete**
- Meaning: closure accounting, not a remaining-effort percentage

## Approved admission scope

Promote an **incomplete selected XSLT 1.0 processor / strict-1.0 preprocessing
profile** through a crate-private, non-test shared compiler path, callable by the
existing OASIS measurement adapter. Keep the current declaration-driven baseline
unchanged. Do not promote the separate selected-modern experiment, publish an
API selector, change adapter defaults or claim complete 1.0 conformance.

The semantic feature ceiling is the bounded selected legacy integration already
present at this checkpoint, not everything the ordinary compiler can lower:

- ordinary selected matched/named template and fallback dispatch;
- the already-admitted output declarations, literal results, text/messages,
  conditional bodies, child-path iteration, focus values and copy;
- existing bounded local/parameter/global bindings and value-expression families;
- existing static attribute sets, element construction, match predicates, keys
  and whitespace declarations;
- existing selected include/import composition and sealed document-global reads.

The exact initial admission guards are in `matched_integration.rs` and its
`conditional_integration`, `template_arguments`, `keys` and `module_graph`
children. Their tests and the [integration evidence](evidence/adr-0025-version-spelling-checkpoint-2026-10-17.md)
define the current partial behavior, not broad support inferred from the family
names above. Audit snapshot: base commit `1ccb058f71976fa9845578da9f8d53fa695238bc`
plus pending workspace changes; `matched_integration.rs` SHA-256
`EF9B3FE92C6CC14FE708A8D95455436E711C28A509D0960AEA000F2E3A1CCF0B`.
This digest identifies an inspection snapshot, not a reproducible release build.
The final report must identify the complete candidate source and toolchain.

This ceiling does **not** grandfather incorrect behavior or turn probe-only
limits into product semantics. Repair policy propagation, error classification,
controls and artifact qualification within the scope. Unsupported recognized
features remain unsupported; syntax acceptance in the baseline is not authority
to expand this profile. If migration reveals a needed scope change, return it
for review rather than automatically enlarging this list.

## Explicitly deferred feature breadth

Do not make these prerequisites merely to improve the private numerator:

- `whitespace23` normalized-path computed attributes/AVTs and `whitespace35`
  document-selection/dynamic-name composition;
- general AVTs, arbitrary XPath grammar, broader sorting/modes and constructors;
- new DTD/entity authority, namespace-valued variables, cross-document ordering,
  extension functions, additional XML encoding or serialization families;
- new public facade/ABI/WASM selectors, performance optimization, complete
  selected-modern support or historical review retrofits.

Recognizing and explicitly refusing excluded behavior is required. An excluded
feature does not excuse a broken admitted combination, false introspection or
an invalid artifact stamp. Shared production fixes remain a separate lane.

## Fixed obligations

IDs map one-to-one to ADR-0025's nine implementation gate categories. Existing
evidence is partial unless explicitly stated; no row is closed by test count.

| ID / status | Owner and existing evidence | Exit criterion and next action |
| --- | --- | --- |
| P01 — closed | Selected module/sequence compiler, preprocessing owner; bounded legacy lowering and its explicit resource entry now build outside tests; modern selection remains test-only. | A single crate-private selected entry lowers every admitted path with explicit policy and strict preprocessing, refuses other entry shapes without baseline fallback, and builds outside tests. Audit callbacks/defaults/constructors; migrate the frozen slice, not adjacent features. Retain the reference for differential execution. |
| P02 — closed | `legacy_numeric_version`, `scoped_version`, canonical preprocessing tests; root and literal-result version evidence exists. | Pin legal/invalid local-version forms and scope for every admitted declaration/instruction, including inherited mixed-version modules. Embedded/literal-result stylesheet entry shapes must either use the same policy correctly or explicitly refuse before lowering. No version spelling may select a processor. Produce a finite entry/scope matrix from existing paths, then fill only missing cells. |
| P03 — closed | Selected recognition/fallback tests and complete ForwardComp replay (13 exact + 5 expected errors). | Recognition stays independent from capability on all admitted routes: known-newer, standards-unknown, and recognized-but-unsupported examples retain their distinct dispositions. Dormant/reached fallback behavior and original provenance survive the non-test migration. Reuse the existing matrix and audit new entry wiring; no feature catalog expansion. |
| P04 — closed | `boolean_call_timing`, conditional integration and variable-binding controls establish selected expression families, not arbitrary XPath. | For each admitted expression-bearing owner (values, tests, admitted bindings/arguments and AVTs), show static versus reached/deferred failure and short-circuit behavior where applicable; unbound names and unsupported capability must not be broadly caught/deferred. Map current admission predicates to this finite matrix, reject remaining forms explicitly. |
| P05 — closed | Selected numeric processor-version leaf exists; general capability/availability reconciliation is not established. | One explicit account reconciles recognition, admitted capability and availability/reporting for this incomplete profile. Supported introspection must have correct types/QName handling and truthful results; unsupported introspection must refuse explicitly rather than invent false answers. Audit the reachable reporting calls and pin the required admitted cases. If truthful reporting requires a policy decision, return that issue; do not invent a conformance claim. |
| P06 — closed | Shared linker checks stamps; selected include/import and lifetime/isolation tests exist. Baseline context guards and the migrated selected entry have been revalidated. | All admitted graph entry and composition paths preserve processor/preprocessing qualification and reject incompatible artifacts; none lowers under baseline then claims selected semantics. Close arbitrary test-shell dependencies or refuse those paths before admission, preserve host graph bounds, mixed-version behavior, failed recompilation, changed-byte identity and lifetime isolation. Audit all context/stamp writes and finalization paths; rerun selected graph tests. |
| P07 — closed | Selected control/cleanup regressions compare shared execution charges; sealed-resource ownership remains unchanged. | Exact/one-less budgets, active cancellation, failure cleanup and no new authority hold across the migrated path. Measure compile and warm execution separately against the retained reference on a fixed workload, with methodology and regressions explained. No optimization campaign is required. Finish the control-coverage audit and bounded measurement. |
| P08 — closed | [Before/after migration reports](evidence/r0-non-test-migration-review-2026-10-23.md): 358/3,173 selected exact, 2,504/3,173 baseline exact; conserved dispositions and source/toolchain identities recorded. | Repeat the full conserved report after non-test migration under the same fixed profile, preserving exclusions, unsupported and harness gaps. Retain baseline and modern regressions separately; no fixture/profile switching. |
| P09 — closed | Final candidate gates and WASM pass; maintainer explicitly approved measured-profile promotion on 2026-10-23. | Final candidate passes fmt, Clippy, workspace tests, docs/links, pinned corpus checks and official WASM build, with no undeclared fixture/API/default changes. Maintainer reviews the report, obligation evidence and provisional list before admission. Run final gates on the exact candidate; no automatic promotion. |

P01–P08 closure is scoped to the frozen R0 slice and supported by the
[second-interval closure map](evidence/r0-non-test-migration-review-2026-10-23.md#three-tranche-account-and-finite-closure-map),
not test-count thresholds. P09's implementation gates and maintainer admission
review are complete. These closures do not authorize adjacent
features, a public profile or defaults. Original exit criteria remain above;
the closure map records how the migrated candidate satisfies them.

## Q01 disposition

**Q01 — baseline lowering with selected qualification.** Shared context-aware
compilation could stamp baseline-lowered principals as selected processors.
The pending safeguard rejects those contexts and preserves selected composition
through policy-aware lowering or strictly validated empty dependency shells.
Maintainer disposition, 2026-10-23: Q01 is **within P06**, not a tenth obligation.
The guard does not by itself close P06. Retain Q01's evidence visibly under that
obligation; there are no separately pending provisional blockers at approval.

The [qualification checkpoint](evidence/adr-0025-version-spelling-checkpoint-2026-10-17.md#2026-10-22-promotion-audit-and-qualification-safeguard)
now passes full repository verification, 227 processor-policy probes, 24 selected
replay/control tests and the official WASM build. The safety tranche is complete
for review; no broader obligation closure or promotion is implied.

The [R0 P06 audit](evidence/r0-p06-qualification-audit-2026-10-23.md) completed
tranche 1 on 2026-10-23. It found and guarded two residual Q01 paths in the old
test-only global-constructor include/import helpers. The pre-repair regression
failed; post-repair repository verification, 228 policy probes, 24 selected
replay/control tests and WASM pass. Current route inspection is complete; P06
remains in progress pending qualification validation of the P01 non-test entry.
No new obligation or scope was added.

Tranche 2 is now complete: the [P04/P05 boundary audit](evidence/r0-p04-p05-boundary-audit-2026-10-23.md)
maps existing expression owners and reporting/refusal behavior. A six-owner,
two-version matrix adds 12 positive controls and 36 unsupported-introspection
checks without new grammar. All verification gates and WASM pass. P04/P05
remain in progress until the non-test entry preserves and revalidates those
boundaries.

Tranche 3 completed the [full selected-profile measurement](evidence/r0-p08-full-selected-measurement-2026-10-23.md):
358 exact, two mismatches, and every terminal disposition conserved across 3,173
cases. Expected-error observations remain separate. P08 moves to in progress;
the after-migration replay is still required. No new provisional blocker is
asserted by this measurement. The two mismatches remain visible for review, not
silently excused as oracle errors. The required review then approved continuation
within unchanged R0; see the second-interval disposition below.

## Two-track checkpoint

| Track | Current observation | Delta across this review interval |
| --- | --- | --- |
| Production, declaration-driven baseline / modern preprocessing | Fresh 2026-10-23 replay: 2,504/3,173 exact (78.92%) | 0 exact delta after migration |
| Declaration-driven baseline / strict preprocessing | 2,501/3,173 exact; prior full replay retained | 0 credited; not selected processor semantics |
| Selected legacy / strict preprocessing, promoted crate-private measured profile | R0 promotion baseline: 358/3,173 exact (11.28%), 2 mismatches | 0 exact delta; 25 Invalid refusals corrected to Unsupported; expected-error observations 125 → 115 |
| Promotion | 9/9 closed — Promote | Frozen measured profile admitted; no default, public API or conformance change |

Baseline provenance remains the [2026-10-18 choose-attribute checkpoint](../oasis-xslt10/evidence/construction/oasis-xslt10-prefixed-computed-element-attribute-set-2026-09-28.md#2026-10-18-choose-attribute-consistency)
and [campaign evidence](../oasis-xslt10/evidence/documents-and-resources/oasis-xslt10-literal-document-path-binding-2026-10-17.md).
Those historical observations remain intact. The fresh selected and baseline
runs are separately identified in the [migration report](evidence/r0-non-test-migration-review-2026-10-23.md).

Corpus: OASIS `XSLT-testsuite-04.ZIP`, SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`,
3,173 catalog cases. The [measurement script](../../../scripts/measure-oasis-xslt10.ps1)
now explicitly selects processor and preprocessing separately. Use both
`-ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10` for P08;
preprocessing alone still selects the declaration-driven baseline.

## Approved work/review boundary

R0 was approved on 2026-10-23. Prefer closure evidence over improving the private
numerator. Unsupported families are valid measurement outcomes, not invitations
to expand the semantic ceiling. Q01 verification preceding approval does not
consume a post-approval tranche. No promotion, defaults, API exposure or
conformance claim is authorized.

The review boundary has been reached after these three bounded tranches:

1. **P06 qualification audit:** enumerate all context-taking compile/link routes,
   close Q01's validation, and record remaining graph-path refusals or defects.
2. **P04/P05 closure audit:** map admitted expression owners and introspection to
   existing tests, add only missing obligation witnesses or scoped repairs, and
   return any semantic-policy issue. Do not admit new grammar to satisfy a test.
3. **P08 full private measurement:** implement explicit selected-profile harness
   routing and conserve the full denominator, leaving guarded cases unsupported.
   Stop at a real harness/ownership blocker rather than filling feature families.

These are objectives, not guarantees that three tranches finish the campaign.
The review chooses promote/continue/defer/abandon from actual evidence. P01/P02
and P07/P09 remain mandatory; no row is waived by this ordering.

Maintainer disposition after the first review: **Continue within unchanged R0**,
not promote. The approved second interval was: (1) P01 non-test migration with explicit
refusals and retained reference; (2) P02–P06 entry/graph and semantic parity
validation; (3) P07 controls/cost measurement plus P08 post-migration replay and
P09 verification, then return for review even if whole obligations remain open.
No numerator-driven grammar expansion is included.

That interval and its maintainer review are now complete: **Promote, 9/9 closed**.
The maintainer accepted the [final candidate report](evidence/r0-non-test-migration-review-2026-10-23.md).
The two selected output mismatches remain visible; no corrected oracle or
fixture exception was introduced. No unresolved provisional blocker is recorded.

### Final promotion disposition — 2026-10-23

FastXSLT admits the measured, incomplete, crate-private selected XSLT 1.0
processor / strict-1.0 preprocessing profile through the non-test shared compiler
path. P09 is closed by explicit maintainer approval, not by this document alone.
Preserve the [selected report](evidence/r0-migrated-selected-report-2026-10-23.txt)
and [source/run manifest](evidence/r0-migrated-run-identity-2026-10-23.json)
unchanged as the **R0 promotion baseline**: 358/3,173 exact, with all other
dispositions retained. The independently replayed ordinary baseline remains
2,504/3,173 exact (78.92%); neither numerator replaces or combines with the other.

This closes R0, not XSLT 1.0 implementation. It changes no default or public,
native, WASM or .NET selector and makes no conformance claim. It does not expand
the frozen semantic ceiling or imply 358 is the eventual profile ceiling.
Future breadth must identify its case, scope and separate replay; it must not
overwrite this baseline or silently enlarge this closed ledger. The ADR-0026
pilot's wider process rollout remains a separate maintainer decision.

## Reproduction commands

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
cargo test -p fastxslt --all-features preprocessing_context
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10
cargo test -p fastxslt --all-features --release r0_selected_migration_cost_probe -- --ignored --nocapture
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

These commands verify the current migrated candidate and reproduce P08/P07.
The retained before-migration report is historical; do not overwrite its identity.

## Revision/disposition log

- 2026-10-22 — R0 proposed. Nine rows trace the accepted gate categories;
  no authoritative count or scope approval inferred. Request approval of the
  bounded legacy-only scope, Q01 under P06, and the three-tranche boundary.
- 2026-10-23 — Maintainer approved R0 unchanged: nine mandatory obligations,
  Q01 within P06, closed semantic ceiling and the listed three-tranche review.
  Implementation/measurement only; prefer closure evidence over private gains.
- 2026-10-23 — Tranche 1/3: qualification audit and residual Q01 repair completed.
  0/9 whole obligations closed; no corpus delta; no pending new provisional issue.
- 2026-10-23 — Tranche 2/3: expression/introspection boundary audit completed.
  0/9 whole obligations closed; no corpus delta or new provisional issue.
  Existing boundaries have a finite migration checklist; no grammar was widened.
- 2026-10-23 — Tranche 3/3: full selected-profile report and conservation complete.
  P08 in progress; 0/9 closed. All verification gates and WASM passed. Review due
  before further migration; no promotion or production coverage change.
- 2026-10-23 — Maintainer approved continuation through the proposed second
  interval without widening R0. Counter resets to 0/3; preprocessing extraction
  is a component of P01, not a separately completed semantic tranche.
- 2026-10-23 — [P01 strict preprocessing dependency](evidence/r0-p01-strict-component-2026-10-23.md)
  builds outside tests and matches the retained reference. Full selected report
  unchanged; 2,292 workspace tests and 229 policy probes pass. P01 and second
  interval remain in progress (0/3); non-test graph wiring is next.
- 2026-10-23 — Second interval 3/3 complete: non-test entry and frozen lowering
  migrated; P02–P06 entry/qualification/timing/refusal audits passed; P07 controls
  and cost comparison, P08 before/after reports and P09 technical gates passed.
  Two entry-shape classification defects repaired within P01/P02. P01–P08 closed;
  P09 awaiting maintainer admission review. Scope and denominator unchanged.
- 2026-10-23 — Maintainer disposition: **Promote the bounded R0 measured profile**.
  P09 closed; **9/9 obligations closed**. No unresolved provisional blockers.
  Preserved the migrated selected report as the R0 promotion baseline. No feature
  expansion, default/API change or conformance claim authorized.
