# Post-promotion bounded breadth workflow

Authority: maintainer direction, 2026-10-23, under
[ADR-0026](../../ADR/ADR-0026-evidence-gated-profile-promotion-and-frontier-progress.md).
Applies only to ordinary post-promotion breadth in this campaign. It does not
amend an ADR or establish a project-wide rollout.

## Boundary and autonomy

Historical control stop: maintainer approved the bounded B03-C01 shared UTF-16
repair, which completed before the separately scoped B03-S2 admission. Its
[closure](evidence/b03-utf16-ceiling-repair-2026-10-23.md) did not reset the batch
counter or change R0. Use the live register below for the current batch.

R0 is complete: the [promotion ledger](promotion-ledger.md) remains **9/9 closed**,
and its scope, obligations and 358/3,173 promotion baseline remain historical
truth. Ordinary breadth is not another promotion prerequisite or permission to
rewrite R0. ADR-0026 sections 4 and 6 distinguish case-driven breadth from
promotion and require bounded review intervals. This maintainer-approved
operating boundary applies those principles after promotion; no ADR conflict or
amendment is identified.

Proceed autonomously with a routine slice only when all conditions hold:

- It stays within accepted ADRs and existing semantic ownership.
- It introduces no architecture, authority, resource-access model, public API,
  ABI/wire contract, default behavior, supported-profile claim or promotion decision.
- It begins with a concrete unchanged corpus failure or a previously recorded
  selected-profile blocker, not a percentage target or speculative completeness.
- Before implementation, its narrow objective, admitted scope, explicit
  exclusions and stopping condition are recorded in the register below, with
  a linked evidence record where needed.
- It reuses shared compiler/runtime semantics, not a second evaluator,
  fixture-specific route or per-case profile selection.
- Focused correctness/control tests, required verification and the full conserved
  **3,173-case selected legacy / strict-preprocessing replay** follow the change.
- No stop condition below has been triggered.

**Do not ask for approval merely because another eligible bounded slice has
been identified.** Record it and proceed within the current batch. Maintainers
approve architectural direction and campaign boundaries, not each routine
implementation choice. This authorization changes no engine contract and grants
no automatic commit, push, release or default-change authority.

## Per-slice procedure

1. Record the unchanged case identity or named blocker and reproducible failure.
2. Declare the admitted semantic scope, exclusions and objective completion/stop
   criterion **before** implementation. Link the relevant accepted owners.
3. Implement only that scope. If adjacent features become convenient, leave them
   guarded; completing one slice does not extend it.
4. Run focused correctness/control regressions and required repository gates.
   Rust changes retain fmt, Clippy, workspace tests, docs/links, applicable
   conformance checks and the official WASM build requirements.
5. Replay all 3,173 cases under the fixed selected legacy / strict profile.
   Preserve prior reports and source/toolchain identity. Unsupported is valid.
6. Record exact movement, all disposition changes and regressions separately
   from ordinary declaration-driven / modern baseline movement. Replay that
   baseline for shared compiler/runtime changes; otherwise report its latest
   measured identity and mark any unmeasured movement unknown, never assumed zero.
7. Close the slice when its objective, verification and replay are complete.
   Increment the completed semantic-slice count once. Reassess the next failure
   within the same batch, unless a review trigger applies.

Do not weaken diagnostics, budgets, cancellation, authority, accounting,
artifact qualification, processor/preprocessing semantics, comparison rules or
corpus conservation. Never relabel wrong supported behavior as unsupported to
avoid a blocker. An exclusion or unknown result earns no exact-output credit.

## Batch persistence and early stops

Work in batches of **at most three completed semantic breadth slices**. A slice
is one semantic objective with its verification and replay—not a chat turn,
commit, test, process invocation or agent session. The register below is the
single authoritative post-promotion counter. Read it on resume; update it before
handoff or stopping. Context compaction, commits and resumed sessions never reset
it. Handoffs must link this file and carry the batch ID, completed count, active
slice and stop state. Do not relabel an unfinished objective to evade review.

Stop and return to the maintainer immediately if:

- a new architectural decision, ownership seam, standards-policy ambiguity or
  modification to an accepted ADR is needed;
- a correctness, security, authority or resource-control blocker appears;
- an unexplained corpus regression appears;
- work needs to expand beyond the declared slice, or would materially expand
  the selected profile beyond ordinary breadth;
- public/default/supported-profile/promotion behavior would change;
- two consecutive slices gain no corpus cases and close no named risk or blocker;
  carry this streak across batches and handoffs rather than resetting it silently;
- three semantic slices have completed in the current batch.

An interrupted or blocked slice remains visible and does not count as completed.
Record provisional blockers with witnesses, affected owner and why work must
stop; do not silently change R0's obligation denominator. If evidence invalidates
an existing closure, follow ADR-0026's reopening rule. A known defect cannot be
worked around by quietly choosing a new case.

## Review report

At the three-slice boundary (or earlier trigger), stop before starting another
slice and request maintainer review. Report:

- selected-profile exact movement from batch start to end, per-slice and net;
- ordinary production-baseline movement separately, with replay identities;
- cases and semantic families changed in each slice;
- regressions and disposition changes, including exclusions and unsupported cases;
- provisional architectural/correctness blockers, or explicitly none;
- focused/repository verification and WASM status, including failed or unrun gates;
- recommended next action and its scope/rationale.

Record the maintainer's disposition before beginning another batch. Continue,
defer or abandon may govern further breadth; a promotion or public change still
requires its separate accepted gates and explicit decision. Routine breadth does
not itself create a new promotion decision.

## Live batch register

- Batch **B07**, approved after B06; completed **3/3**. Stopped for [review](evidence/b07-review-2026-10-23.md).
- Start selected **513/3,173**, ordinary **2,504/3,173**. No blockers.
- Current selected **527/3,173 (+14)**, ordinary unchanged; all gates/WASM pass.
- No provisional blockers; no-gain/no-risk-closure streak 0. Review due now.
  No fourth slice before disposition. Carry B07 3/3 across resumes/handoffs.
  Final [identity](evidence/b07-s3-run-identity-2026-10-23.json).
- Fresh trace: `.workbench/b07-before.log`; unique-basename/span triage places
  value-of (512) ahead of for-each (128) and copy-of (32). Ambiguous paths and
  unusable spans excluded; counts are navigation hints, not gain estimates.

B07-S1 declared scope: `boolean01`, `boolean02`, `boolean04`, `boolean09` first
refuse value-of with true/false constants and Boolean composition. Admit bounded
trees of true()/false(), and/or, not()/boolean() and grouping only, through the
existing selected Boolean parser and shared IR/text result. Exclude source,
variable, comparison, numeric/string truth conversion and arbitrary calls.
Preserve parser bounds, controls and qualification. Stop after focused tests,
both full replays and repository/WASM gates.

B07-S2 scope before implementation: fresh first refusals `node01` (`./text()`)
and `AVTs__77531` (`./title`). Admit explicit `./` before already admitted
unprefixed child-name/text() value paths, selected legacy only. Exclude parent,
root, descendant, attribute, wildcard and predicate expansion. Shared lowering
retains legacy first-node conversion and effective navigation. Pin nested focus,
first-node selection, empty paths, views and controls. Stop after both full
replays and repository/WASM gates.

B07-S3 scope before implementation: fresh `select01` first refusal at 409..448
selects `/doc/a/b/@attr` inside iteration. Admit a final unprefixed named attribute
after nonempty unprefixed named child paths, optionally rooted or explicitly
relative. This is read-only path selection, not attribute construction or a new
owner. Reuse shared first-node string conversion, document order and effective
navigation. Exclude namespaces, wildcards, predicates, descendants and unions.
Pin nested focus/root distinction, first match, missing attributes, namespace
exclusion, escaping and controls. Stop after full replays/all gates, then review.

## Preserved B06 register


- Batch: **B06**, maintainer approved after B05 on 2026-10-23.
- Completed slices: **3/3**. Stopped for [B06 review](evidence/b06-review-2026-10-23.md).
- Starting selected **509/3,173**; ordinary **2,504/3,173**.
- No provisional blockers; no-gain/no-risk-closure streak 0.
- Current selected **513/3,173 (+4)**, ordinary unchanged; all gates/WASM passed.
- Review due now. No fourth slice before maintainer disposition; carry B06 3/3
  across resumes/handoffs. Final [identity](evidence/post-r0-numeric-copy-run-identity-2026-10-23.json).

B06-S3 scope before implementation: recorded unchanged `copy06` at 361..387
copies the numeric constant `32`. Reuse the already admitted selected argument
integer domain (signed i32, ASCII decimal digits, no leading plus) for constant
copying via shared static-atomic text construction. Pin zero/negative-zero,
leading zeroes, signed bounds, declared versions, text ownership and controls.
Exclude out-of-domain integers, decimal/exponent notation, arithmetic, variables
and general numeric functions. Stop after both full replays and all gates;
B06 then requires maintainer review, with no fourth slice.

B06-S2 scope before implementation: unchanged `copy29` has a second direct copy
selector `current()` after its now-admitted `.` selector. Admit only the complete
zero-argument call through shared `CopyOfCurrent`; exclude predicates, composed
paths, arguments and other uses of `current()`. Pin nested iteration entry focus,
root/current contrast, views and control/reuse behavior across declared versions.
Stop after focused tests, full selected/ordinary replays and repository/WASM gates.

B06-S1 scope before implementation: unchanged recorded `Lotus/copy_copy10#1`
refuses `docs//d` at 504..535. Admit exactly one interior `//` between nonempty
unprefixed named child paths, optionally rooted, using shared copy navigation.
Exclude additional descendant separators, wildcards, predicates, functions,
attributes and namespaces. Pin nested overlap/deduplication, document order,
root/relative focus, empty selection, views and controls. Stop after focused
tests, full selected/ordinary replays and repository/WASM verification.

## Preserved B05 register

- Batch: **B05**, approved after B04 on 2026-10-23.
- Completed semantic slices: **3/3**. State: **stopped for [B05 review](evidence/b05-review-2026-10-23.md)**.
- Starting selected **501/3,173**; ordinary **2,504/3,173**.
- Current selected **509/3,173 (+8)**; ordinary unchanged. All gates/WASM passed.
- No provisional blockers; no-gain/no-risk-closure streak 0. Review after three
  slices or an earlier trigger. Review is due now; no fourth slice before
  maintainer disposition. Carry B05 3/3 across resumes and handoffs.
  Final [source identity](evidence/post-r0-child-comment-copy-run-identity-2026-10-23.json).

B05-S1 preimplementation scope: unchanged recorded first refusals
`Lotus/conditional_conditional19#1` at 384..416 (`.//title`) and
`Lotus/position_position37#1` at 410..438 (`.//a`). Admit exactly `.//` followed
by one unprefixed name in selected iteration, through shared selection and body
lowering. Exclude descendant chains, wildcard/predicate forms, sorting and other
axes. Pin context-relative versus root behavior, ancestor exclusion, order,
position/size, empty selection, views and controls. Stop after focused checks,
full selected/ordinary replays and repository/WASM gates.

B05-S2 preimplementation scope: recorded unchanged `copy55` (709..734) and
`copy43` refuse direct `.` and `/` copy selectors. Admit these two complete node
selectors through shared copy instructions, preserving selected legacy recovery.
Exclude `current()`, extended self/parent paths, variables, namespace/attribute
selection and general expressions. Pin current versus root from nested focus,
document child copying, namespaces, whitespace views and control/reuse boundaries.
Stop after focused tests, both conserved replays and repository/WASM gates.

B05-S3 preimplementation scope: recorded `Lotus/copy_copy41#1` refusal at
435..472 selects `doc/comment()`. Admit child `comment()` alone or after named
child steps, optionally rooted, through shared copy location paths. Exclude
descendant comment tests, predicates, PI/text/node tests, mixed unions, variables
and general functions. Pin direct-only comment selection, root/relative focus,
source order, empty paths, strip/preserve views and exact/one-less controls with
cancellation/reuse. Stop after both conserved replays and repository/WASM gates;
B05 then requires review. Do not implement before S2 validation completes.

## Preserved B04 register

- Batch: **B04**, authorized by maintainer approval of B03 on 2026-10-23.
- Completed semantic slices: **3/3**. State: **stopped for [B04 review](evidence/b04-review-2026-10-23.md)**.
- Starting selected: **492/3,173**; current **501/3,173 (+9)**; ordinary: **2,504/3,173 unchanged**.
- [S1 evidence](evidence/b04-s1-final-wildcard-copy-2026-10-23.md) and
  [run identity](evidence/post-r0-final-wildcard-copy-run-identity-2026-10-23.json).
- [B03 review](evidence/b03-review-2026-10-23.md) approved; C01 remains closed.
- No provisional blockers. Consecutive no-gain/no-risk-closure slices: **0**.
- Review due **now**. No fourth slice before maintainer disposition.
- Final [run identity](evidence/post-r0-descendant-copy-run-identity-2026-10-23.json).
  S2 +7 and S3 +1; all gates/WASM passed, no new blockers. Carry B04 3/3 on resume.

B04-S1 scope before implementation: recorded unchanged first refusals
`Lotus/namespace_namespace138#1` (641..670, `doc/*`) and
`Microsoft/Output__84374#1` (164..202, `/xslTutorial/*`). Admit a final child-element
wildcard after one or more unprefixed named child steps, optionally rooted,
using shared copy selection and legacy recovery. Exclude intermediate wildcards,
bare `/*`, descendants, predicates, attributes, namespaces, variables and unions.
Pin focus/root distinction, order, empty selection, namespaces, effective whitespace,
controls and guarded neighbors. Stop after focused tests and conserved full
selected/ordinary replays plus repository/WASM verification.

S1 closed: `namespace138` exact; `Output__84374` executes but has an unusable
archival reference and earns no credit. No regression/blocker. All gates/WASM
passed. On resume carry **B04 1/3**, not a new batch. Reassess the recorded
named-path/final-wildcard iteration frontier (e.g. `Output__84616`) before
declaring S2; this note is not permission to broaden S1 or skip its scope record.

B04-S2 preimplementation scope: recorded `Microsoft/Output__84616#1` refusal
at 232..281, `for-each select="SALAD/SALAD_INGREDIENTS/*"`. Admit a final child
wildcard after named child steps, optionally rooted, through shared iteration.
Exclude intermediate wildcards, bare `/*`, predicates, sorting and other axes.
Pin element-only order, focus position/size/restoration, namespaced children,
effective whitespace, empty paths and controls. Stop after focused checks,
both full conserved replays and repository/WASM gates. Starting selected 493.

B04-S3 preimplementation scope: recorded unchanged `Microsoft/Whitespaces__91453`
through `91456` first refuse `copy-of select="//span"`; their stylesheets also use
`//div/p`. Admit leading `//` plus nonempty unprefixed named child steps through
shared copy location-path compilation. Exclude relative/internal descendant
abbreviations, wildcards, predicates, namespaces, attributes, variables and unions.
Pin rooted selection from nested focus, order/duplicates, empty selection,
namespace exclusion, whitespace views, budgets and cancellation/reuse. Stop after
both conserved replays and repository/WASM verification, then request B04 review.

## Preserved B03 register

- Batch: **B03**, authorized by maintainer approval of B02 on 2026-10-23.
- Completed semantic slices: **3/3**.
- State: **stopped for [B03 review](evidence/b03-review-2026-10-23.md); no fourth slice**.
- [B02 review](evidence/b02-review-2026-10-23.md): approved; continue under the same boundaries.
- [Prior batch review](evidence/b01-review-2026-10-23.md): approved; continue under the same boundaries.
- Current measured selected exact: **492/3,173 (+5)**; ordinary **2,504/3,173 (unchanged)**.
- Starting selected baseline: **487/3,173 exact**, selected legacy / strict,
  [B02 final report](evidence/post-r0-root-iteration-selected-report-2026-10-23.txt).
- Starting ordinary baseline: **2,504/3,173 exact**, declaration-driven / modern,
  [independent report](evidence/post-r0-copy-baseline-report-2026-10-23.txt).
- Source/toolchain identity: [latest run manifest](evidence/post-r0-boolean-copy-run-identity-2026-10-23.json).
- Consecutive completed no-gain/no-risk-closure slices: **0**.
- Provisional candidate blocker: **B03-C01**, UTF-16 result ceiling checked after
  intermediate construction/encoding; [evidence](evidence/b03-utf16-ceiling-blocker-2026-10-23.md).
  Maintainer authorized repair; [closed with unchanged replays](evidence/b03-utf16-ceiling-repair-2026-10-23.md).
  R0's closed denominator is unchanged.
- Review due: **now, three completed semantic slices**.

B03-S3 preimplementation declaration: fresh full frontier replay reproduces
`Lotus/copy_copy12#1` refusing `<xsl:copy-of select="true()"/>` at 373..403.
Admit exact zero-argument `true()` and `false()` calls (outer whitespace allowed)
through the shared static atomic-copy compiler/runtime. Exclude other calls,
argument expressions, numeric copying, variables, dynamic boolean expressions,
new function introspection and any new evaluation path. Pin selected versions
1/2/3, output/control bounds, cancellation/reuse and adjacent refusals. Stop
after focused verification and both conserved replays; then B03 requires review.

B03-S2 preimplementation declaration: `Lotus/output_output80#1`, declaration
span 307..338, refuses exact `UTF-16`. Admit only that case-insensitive encoding
label through existing shared byte output. Exclude aliases, other encodings,
string-returning API changes, output-version/standalone expansion and defaults.
Pin selected versions 1/2/3, shared exact/one-less limits and cancellation/reuse;
replay all 3,173 selected cases and ordinary baseline. Stop at verified label
admission and classify downstream failures without widening scope.

B03-S1 preimplementation declaration: fresh frontier and byte-span mapping show
`Lotus/copy_copy19#1`, `copy22` and `numberformat06` refusing their ISO-8859-1
output declaration. Admit only the case-insensitive exact `ISO-8859-1` label in
selected output validation, retaining shared output compilation and byte
serialization. No other encoding, alias, default or string-return API change.
Pin actual Latin-1 bytes, XML character references, unrepresentable literal/text
failure, versions 1/2/3, unsupported encodings, exact/one-less output ceiling,
cancellation/reuse. Stop after focused tests, full selected/baseline replay and
repository/WASM gates. Start **487 / 2,504**. Subsequently
[closed](evidence/b03-s1-latin1-output-2026-10-23.md): 491 selected (+4),
ordinary unchanged; all gates/WASM pass, no blocker.

## Preserved B02 register

| Slice / state | Named blocker | Preimplementation scope and exclusions | Movement / verification |
| --- | --- | --- | --- |
| B02-S1 — complete | Initial `copy07` attribution corrected: its span is an attribute template. Preimplementation replay actually identifies wildcard iteration in `node07` (604..629) and `Elements__89065` (188..214); [evidence and correction](evidence/b02-s1-wildcard-iteration-2026-10-23.md) | Admit standalone child-element wildcard iteration through shared selection and selected body lowering. Pin order, namespaces, non-element exclusion, focus restoration, empty selection, views and controls. Exclude wildcard path steps, predicates, sorting, attributes/namespace axes and adjacent instructions. Stop after focused tests, full selected/baseline replay and repository/WASM gates. | 482 → 483 (+1); ordinary 2,504 unchanged; gates/WASM pass; no blocker |

### B02-S2 and S3 declarations and closure

B02-S2 closed: [quoted string copying](evidence/b02-s2-string-copy-2026-10-23.md),
483 → 486 exact (+3), ordinary unchanged, all gates/WASM passed, no blocker.

**B02-S3 preimplementation scope:** unchanged `Microsoft/AVTs__77574#1` refuses
`for-each select="/bookstore"` at 197..231 in `XSLT17007.xsl` (direct source check).
Admit one leading slash plus nonempty ASCII named child steps for selected
iteration through shared path selection and selected body lowering. Exclude `/`
alone, wildcard/predicate/qualified steps, mixed descendant paths, sorting,
resource functions and new body expressions. Pin absolute versus relative
selection from non-root focus, order, empty results, focus restoration, views
and controls. Stop after focused tests, full replays, repository/WASM gates;
then B02 review is mandatory. Start **486 / 2,504**. Subsequently
[closed](evidence/b02-s3-rooted-iteration-2026-10-23.md): selected 487 (+1),
ordinary unchanged; all gates/WASM pass; no blocker.

Preserved B02-S2 declaration (recorded before code): unchanged
`Lotus/copy_copy05#1`, `copy-of select="'test'"`. Admit only complete XPath 1.0
quoted string literals (no embedded matching delimiter), using shared static
copy lowering and runtime. Exclude numeric/boolean/arithmetic/variable values,
modern doubled-quote syntax, malformed expressions and other copy selectors.
Pin both quote styles, empty/Unicode/escaped text, declared versions, controls,
provenance and excluded syntax. Stop after full selected/baseline replay and
repository/WASM gates. Start **483 / 2,504**; subsequently closed as linked above.

## Preserved B01 register

The comment (+35) and named copy-path (+56) slices were completed before this
workflow direction and had individual maintainer approvals. They remain in
their original evidence records; they are not retroactively charged to B01.
This explicit initial transition is not permission to reset later batches.

Before coding, add a row with ID `B01-S1` (then S2/S3 as work proceeds):

| Slice / state | Named case or blocker and evidence | Admitted scope / exclusions / stop criterion | Selected delta / ordinary delta | Dispositions, regressions, controls and gates |
| --- | --- | --- | --- | --- |
| B01-S1 — complete | Unchanged `Lotus/message_message12#1`, selected `FXAR0031` at PI constructor; [closure evidence](evidence/b01-s1-selected-processing-instructions-2026-10-23.md) | Admit `xsl:processing-instruction` with a static ASCII NCName target other than case-insensitive `xml`, `name` attribute only, selected-policy body and shared legacy runtime (ADR-0025/0023). Exclude dynamic/invalid/qualified targets, new expression grammar, extra attributes and adjacent constructors. Stop after focused controls, full selected/baseline replay and repository/WASM gates. | 449 → 466 (+17) / 2,504 unchanged | +18 successful executions, +1 layout exclusion, +2 separate expected-error credits; no new mismatch/execution failure; focused, repository, policy/replay and WASM gates pass; no blocker |

| B01-S2 — complete | Unchanged `Lotus/copy_copy04#1`, unsupported `FXAR0031` at `copy-of select="*"`; [closure](evidence/b01-s2-selected-wildcard-copy-2026-10-23.md) | Admit only the standalone child-element wildcard for selected `xsl:copy-of`, including surrounding whitespace, through shared `CopyOfChildElements`. Exclude wildcard path steps, attributes, namespace nodes, predicates, descendant/context/parent selectors and other instructions. Stop after focus/order/subtree/view/control tests, full selected/baseline replay and repository/WASM gates. | 466 → 469 (+3) / 2,504 unchanged | +3 successes; other disposition counts unchanged; all gates/WASM pass; no blocker |

| B01-S3 — complete | Unchanged `Lotus/dflt_dflt04#1`, `FXAR0031` at `for-each select="//center"` (425..457); [closure](evidence/b01-s3-selected-descendant-iteration-2026-10-23.md) | Admit exactly `//` followed by one ASCII NCName for selected `for-each`, through shared selection parsing/evaluation and selected body lowering. Pin document-root selection from non-root focus, order, position/size restoration, whitespace views, controls and reuse. Exclude relative descendants, extra path steps, wildcard/qualified names, predicates, sorting, variables, resource access and broader expressions. Stop after full selected/baseline replay and repository/WASM gates, then request B01 review. | 469 → 482 (+13) / 2,504 unchanged | +13 successes, +1 explained expected runtime error; mismatch/exclusion counts unchanged; all gates/WASM pass; witness reaches separate expression guard; no blocker |

Keep declarations and completed results in the same row or linked evidence;
do not erase the preimplementation scope. Detailed experiments belong in
`evidence/`, not an expanding second promotion ledger. Archive completed batch
rows here or in a linked record when a reviewed next batch is opened.

## Workflow history

- 2026-10-23 — Maintainer approved B01 review and continuation. B01 remains
  3/3 complete (+33 exact); B02 starts at 0/3, selected 482 / ordinary 2,504.

- 2026-10-23 — Maintainer authorized autonomous ordinary breadth within accepted
  ownership, bounded predeclared slices, full selected replay and at most three
  semantic slices per review. B01 starts at 0/3 from 449 selected / 2,504 ordinary.
  R0 and prior slice evidence preserved; ADR-0026 unchanged.
