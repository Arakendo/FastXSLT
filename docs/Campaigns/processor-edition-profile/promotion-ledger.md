# ADR-0025 processor-profile promotion ledger

- Revision: R0 proposal, 2026-10-22; **awaiting maintainer scope approval**
- Governance: [ADR-0026](../../ADR/ADR-0026-evidence-gated-profile-promotion-and-frontier-progress.md)
- Semantic authority: [ADR-0025](../../ADR/ADR-0025-compilation-graph-scoped-processor-edition-policy.md), [ADR-0023](../../ADR/ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md)
- Authoritative mandatory denominator: **not yet approved**
- Proposed obligations: **9; 0 closed, 7 in progress, 2 open**
- Meaning: closure accounting, not a remaining-effort percentage

## Proposed admission scope

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

## Proposed finite obligations

IDs map one-to-one to ADR-0025's nine implementation gate categories. Existing
evidence is partial unless explicitly stated; no row is closed by test count.

| ID / status | Owner and existing evidence | Exit criterion and next action |
| --- | --- | --- |
| P01 — in progress | Selected module/sequence compiler, preprocessing owner; private lowering uses shared execution, but remains under `cfg(test)`. | A single crate-private selected entry lowers every admitted path with explicit policy and strict preprocessing, refuses other entry shapes without baseline fallback, and builds outside tests. Audit callbacks/defaults/constructors; migrate the frozen slice, not adjacent features. Retain the reference for differential execution. |
| P02 — in progress | `legacy_numeric_version`, `scoped_version`, canonical preprocessing tests; root and literal-result version evidence exists. | Pin legal/invalid local-version forms and scope for every admitted declaration/instruction, including inherited mixed-version modules. Embedded/literal-result stylesheet entry shapes must either use the same policy correctly or explicitly refuse before lowering. No version spelling may select a processor. Produce a finite entry/scope matrix from existing paths, then fill only missing cells. |
| P03 — in progress | Selected recognition/fallback tests and complete ForwardComp replay (13 exact + 5 expected errors). | Recognition stays independent from capability on all admitted routes: known-newer, standards-unknown, and recognized-but-unsupported examples retain their distinct dispositions. Dormant/reached fallback behavior and original provenance survive the non-test migration. Reuse the existing matrix and audit new entry wiring; no feature catalog expansion. |
| P04 — in progress | `boolean_call_timing`, conditional integration and variable-binding controls establish selected expression families, not arbitrary XPath. | For each admitted expression-bearing owner (values, tests, admitted bindings/arguments and AVTs), show static versus reached/deferred failure and short-circuit behavior where applicable; unbound names and unsupported capability must not be broadly caught/deferred. Map current admission predicates to this finite matrix, reject remaining forms explicitly. |
| P05 — in progress | Selected numeric processor-version leaf exists; general capability/availability reconciliation is not established. | One explicit account reconciles recognition, admitted capability and availability/reporting for this incomplete profile. Supported introspection must have correct types/QName handling and truthful results; unsupported introspection must refuse explicitly rather than invent false answers. Audit the reachable reporting calls and pin the required admitted cases. If truthful reporting requires a policy decision, return that issue; do not invent a conformance claim. |
| P06 — in progress | Shared linker checks stamps; selected include/import and lifetime/isolation tests exist. Baseline context guard is the current repair candidate. | All admitted graph entry and composition paths preserve processor/preprocessing qualification and reject incompatible artifacts; none lowers under baseline then claims selected semantics. Close arbitrary test-shell dependencies or refuse those paths before admission, preserve host graph bounds, mixed-version behavior, failed recompilation, changed-byte identity and lifetime isolation. Audit all context/stamp writes and finalization paths; rerun selected graph tests. |
| P07 — in progress | Selected control/cleanup regressions compare shared execution charges; sealed-resource ownership remains unchanged. | Exact/one-less budgets, active cancellation, failure cleanup and no new authority hold across the migrated path. Measure compile and warm execution separately against the retained reference on a fixed workload, with methodology and regressions explained. No optimization campaign is required. Finish the control-coverage audit and bounded measurement. |
| P08 — open | Complete private impincl 29/29 and whitespace 18/22 reports exist; no full selected-processor OASIS report exists. | Full 3,173-case report uses one fixed selected processor/strict preprocessing pair and conserves every disposition, including exclusions, unsupported and harness gaps. Bind corpus/engine/harness/profile/overlay/target identities; retain baseline and modern regressions separately. Add explicit selected routing to the existing harness without fixture/profile switching, then replay before and after migration. |
| P09 — open | Repository and WASM gates pass at preceding checkpoints; current repair verification is separate from final admission. | Final candidate passes fmt, Clippy, workspace tests, docs/links, pinned corpus checks and official WASM build, with no undeclared fixture/API/default changes. Maintainer reviews the report, obligation evidence and provisional list before admission. Run final gates on the exact candidate; no automatic promotion. |

P01–P07 already have implementation evidence, but their **whole-scope exit
criteria are not yet demonstrated**. Keeping them open avoids converting isolated
successes into a false `7/9 complete` claim. Each row may gain a finite closure
checklist during the audit; changes in meaning or scope require review.

## Provisional issue requiring disposition

**Q01 — baseline lowering with selected qualification.** Shared context-aware
compilation could stamp baseline-lowered principals as selected processors.
The pending safeguard rejects those contexts and preserves selected composition
through policy-aware lowering or strictly validated empty dependency shells.
Recommendation: classify Q01 **within P06**, not as a tenth obligation. The guard
does not by itself close P06. Until reviewed it remains separately visible;
approval of R0 can explicitly include this disposition.

The [qualification checkpoint](evidence/adr-0025-version-spelling-checkpoint-2026-10-17.md#2026-10-22-promotion-audit-and-qualification-safeguard)
now passes full repository verification, 227 processor-policy probes, 24 selected
replay/control tests and the official WASM build. The safety tranche is complete
for review; no broader obligation closure or promotion is implied.

## Two-track checkpoint

| Track | Current observation | Delta in this qualification tranche |
| --- | --- | --- |
| Production, declaration-driven baseline / modern preprocessing | 2,504/3,173 exact (78.92%); prior full replay retained | 0 credited; no full replay run for this repair |
| Declaration-driven baseline / strict preprocessing | 2,501/3,173 exact; prior full replay retained | 0 credited; not selected processor semantics |
| Selected legacy / strict preprocessing, private | impincl 29/29 exact; whitespace 18 exact + 2 mismatches + 2 unsupported = 22 | No new corpus credit expected from qualification safeguard |
| Promotion | Proposed 0/9 closed; authoritative ledger pending approval | Q01 identified and repaired provisionally; P06 not closed |

Baseline provenance remains the [2026-10-18 choose-attribute checkpoint](../oasis-xslt10/evidence/construction/oasis-xslt10-prefixed-computed-element-attribute-set-2026-09-28.md#2026-10-18-choose-attribute-consistency)
and [campaign evidence](../oasis-xslt10/evidence/documents-and-resources/oasis-xslt10-literal-document-path-binding-2026-10-17.md).
These historical observations are not relabeled as runs of today's dirty tree.

Corpus: OASIS `XSLT-testsuite-04.ZIP`, SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`,
3,173 catalog cases. The [measurement script](../../../scripts/measure-oasis-xslt10.ps1)
currently selects **preprocessing only**, not the selected processor; invoking
`-PreprocessingProfile strict-xslt10` does not satisfy P08.

## Proposed work/review boundary

R0 is an initial review request, not an already approved campaign revision.
Do not start deferred feature breadth while awaiting review. Completing Q01
verification is the already-started safety tranche; record it separately from
the post-approval counter rather than retroactively consuming three reviews.

If approved, the next review is after these three bounded tranches, or the
ADR-0026 early triggers:

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

## Reproduction commands

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
cargo test -p fastxslt --all-features preprocessing_context
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

These commands verify current evidence; they do not yet provide P08's missing
selected full-suite route. P07's measurement command and P08's selected full
replay command must be recorded when implemented, not invented here.

## Revision/disposition log

- 2026-10-22 — R0 proposed. Nine rows trace the accepted gate categories;
  no authoritative count or scope approval inferred. Request approval of the
  bounded legacy-only scope, Q01 under P06, and the three-tranche boundary.
