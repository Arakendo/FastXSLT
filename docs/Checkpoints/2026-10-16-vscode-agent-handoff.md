# FastXSLT VS Code agent handoff — 2026-10-16

## Assignment

Continue autonomously on OASIS XSLT 1.0 breadth and the accepted AR-0030 /
ADR-0024 global-constructor integration. Work through coherent, tested slices
until an architectural decision, genuine blocker, or useful validated checkpoint
requires returning to the maintainer. Do not stop after each tiny edit merely to
ask whether to continue. Do not weaken semantics or controls to reach 80%.

The processor-edition seam was investigated on 2026-10-16. The catalog
inventory, implementation-owner map, standards matrix, and bounded private
experiment proposal are recorded in [processor-edition seam investigation](../Evidence/processor-edition-seam-investigation-2026-10-16.md).
The ownership direction is now accepted in
[ADR-0025](../ADR/ADR-0025-compilation-graph-scoped-processor-edition-policy.md),
closing AR-0031. Production behavior remains unchanged: implementation must
satisfy the ADR's gates for local scope, error timing, recognition/capability,
introspection, artifact qualification and profile-labelled corpus replay.
The test-only reference is not promoted by acceptance. No public selector,
default change or additional conformance credit is authorized.

A subsequent maintainer-authorized [bounded instruction probe](../Evidence/ar-0031-bounded-instruction-probe-2026-10-16.md)
adds test-only edition/version witnesses, now with 50 focused checks, nested
fallback propagation through shared callback seams, and a bounded introspection
policy callback reusing existing QName parsing. A test context composes the
existing preprocessing view with edition-aware nested lowering and preserves
preprocessing cancellation. Constant-test and bounded boolean-availability
`xsl:if` / `xsl:choose` now propagate edition through the shared conditional owner; unproved
availability is refused rather than borrowed from production folding. Other
expressions remain guarded.
Parameter-free template lowering now returns a complete ordinary template via
shared metadata/body assembly; parameter defaults are refused before re-entry.
A subsequent original-module entry point compiles exactly one parameter-free
root template via shared top-level routing/finalization, without a shell. It
admits only four exact version spellings. Other declarations and graph assembly
remain refused; this is not a general processor profile.
A separate supplied include-chain probe now carries one edition through at most
four already-parsed modules using existing include composition. Non-leaf modules
contain one include only; exact logical href matching is not resource resolution.
Mixed declarations, provenance and shared preprocessing controls are tested;
imports/general graphs and compiled-artifact profile reuse remain open.
Artifact tests now cover concurrent compilation/reuse after parsed modules are
dropped, failed recompilation and changed bytes under the same logical URI.
Existing mixed-preset linking rejects preprocessing-stamp mismatches; do not
mistake that for a processor-edition stamp or general reusable-module contract.
A private two-module qualified-leaf candidate now checks edition and preprocessing
independently, with cross-combinations used only as differential controls. It
accepts compiler-created leaves, not raw-program relabeling; the mechanism remains
experimental and is not a production artifact format or cache/reuse contract.
The qualified candidate now retains the original profile across intermediate
includes up to four modules, with transitive repeated-identity rejection and
differential program/preprocessing-work equality against the document-chain
reference. A single-import-only parent route now uses the existing import
composer, with nested precedence/provenance and preprocessing parity tests.
Competing rules, apply-imports and branching graphs remain outside the experiment.
Production semantics are
unchanged. General-container/local-version propagation, integrated capability
reporting and graph integration remain unproved; hypothetical version witnesses
are not conformance claims or selected production reporting. Do not mistake the
probe for a selected or complete processor profile.

Latest semantic audit corrects the modern no-fallback case to static refusal;
earlier shared legacy/modern deferral evidence is superseded. Both local-version
attribute forms are guarded pending edition-specific integration. The evidence
now includes a maintainer decision checkpoint: review the graph-policy ownership
direction rather than extending graph probes indefinitely. Local scope,
expression-error timing and typed truthful introspection remain explicit gates.

**Architecture review authority stays with the maintainer and this conversation.**
The VS Code agent must not create, draft, amend, accept, supersede or close ARs
or ADRs. Read existing records as constraints. If work exposes an architectural
question or requires changing an accepted boundary, stop and bring the concrete
problem, reproducer, evidence and alternatives back to the maintainer here.
Do not implement a proposed architectural choice while waiting for disposition.
Record routine progress in the roadmap, evidence or checkpoint instead; changes
to AR/ADR records must be handled here.

The immediate next task is **classify remaining compilation blockers**, not
speculatively admit every currently guarded runtime instruction. The latest
complete OASIS replay reported no `FXRT1004` execution frontier, despite many
remaining constructor guards. Establish a real failing consumer before choosing
the next integration slice.

## Workspace and preservation

Commit update: after this handoff was drafted, the maintainer requested that all
pending work be committed before transfer. The implementation and this checkpoint
are included in that commit. The uncommitted inventory and HEAD below describe
the pre-commit checkpoint; use `git log -1` and `git status --short` for the actual
starting state. This one-time commit request does not authorize subsequent agent
commits or pushes.

- Work only in `F:\LocalSource\FastXSLT`; ClassLibrary is unrelated.
- Current HEAD when this checkpoint was written: `aca19c22` —
  `Advance stylesheet preprocessing and global demand reference evidence`.
- The working tree contains substantial **uncommitted tracked and untracked
  implementation, tests and documentation** from the preceding sessions. These
  are the current working implementation, not disposable scratch files.
- In particular, ADR-0024, the production resolver/initializer/constructor files,
  constructor tests, compiler admission and reference files are still untracked.
  Do not reset, clean, stash away, or overwrite this work to reproduce HEAD.
- This handoff does not authorize commit or push. Preserve unrelated edits and
  report the resulting working-tree state at handback.
- Check `git status --short` before beginning. Do not run competing agents or
  mutating builds against this checkout concurrently.

## Read first

1. Root `AGENTS.md` and any more-specific instructions.
2. [ADR-0024: invocation-owned global resolution](../ADR/ADR-0024-invocation-owned-global-resolution-with-shared-execution.md).
3. [AR-0030](../Architectural%20Reviews/AR-0030-invocation-owned-global-constructor-evaluation.md),
   especially the latest production evidence and outstanding integration checklist.
4. [Roadmap](../Plans/roadmap.md), current critical path.
5. [Software Design Document](../Specifications/FastXSLT%20Software%20Design%20Document.md),
   global initialization and shared-constructor contracts.
6. Relevant accepted ownership records before touching their boundaries:
   ADR-0004 decomposition, ADR-0012 effective whitespace views, ADR-0014/0017
   lexical frames, ADR-0022 retained node owners, ADR-0023 preprocessing profiles.

The roadmap and AR retain historical observations. Some earlier paragraphs still
say production rejects `variable40`; **later production evidence supersedes
that observation**. Do not mistake reference-only milestones for production
support or rewrite historical evidence as though it had always been current.

## Validated baseline

Last implementation validation completed on 2026-10-15:

| Measurement | Result |
| --- | --- |
| Core tests, all features | 1,976 passed; 63 ignored |
| Native workbench tests | 22 passed; 3 ignored |
| WASM adapter tests | 4 passed |
| Worker tests | 18 passed |
| Shared-constructor focused tests | 97 passed |
| OASIS modern profile | 2,501 / 3,173 exact (78.82%); 14 mismatches |
| OASIS strict preprocessing profile | 2,498 / 3,173 exact; 17 mismatches |
| Expected-error credits | 423 in each profile |

Full repository verification, official WASM target check, local Markdown links
(1,991 files at that point), and `git diff --check` passed. This handoff itself is
documentation-only; those engine gates are the prior validated baseline, not a
claim of a new full run on October 16.

Modern 80% needs **38 more exact cases** (2,539 total). Only unchanged
`Lotus/variable_variable40#1` earned a new corpus pass during this production
integration. Subsequent reader/instruction slices increased supported behavior
and tests but left both corpus numerators unchanged. Preserve original expected
outputs, denominators, profile distinctions and expected-error classification.

## Implementation map

Paths below are relative to `crates/fastxslt/src/` (runtime and compile are **not**
nested under `xslt/`).

- `compile/global_constructor_admission.rs`: shared fallback after specialized
  initializer refusal; program-wide parameter/match eligibility checks.
- `compile/xslt10_global_constructor_compiler.rs` and
  `compile/golden_stylesheet_experiment.rs`: specialized paths must decline
  noneligible expressions to shared compilation, not misclassify them as invalid.
- `runtime/runtime_context/global_initialization.rs`: initializer materialization,
  transient owned values, publication and `GlobalInitializerReads` interface.
- `runtime/runtime_context/global_resolution.rs`: one invocation owner, one
  completed store, recursive resolution, active stack, retained failure, readers.
- `runtime/runtime_context/global_constructor_execution.rs`: fresh global entry
  inputs and instruction eligibility; delegates to ordinary `execute_sequence`.
- `runtime/runtime_context/global_constructor_tests.rs` and its subject modules:
  production integration tests. Latest modules are `comment_pi.rs` and
  `source_copy.rs`; shared test stylesheet/run helpers currently live in
  `attribute_focus.rs`.
- `runtime/value_evaluator/xslt10_compatibility.rs` and child comparison modules:
  connected legacy string, boolean and typed comparison readers.
- `runtime/golden_runtime_experiment.rs`: ordinary execution and copy consumers.
- `runtime/runtime_context/global_demand_reference/`: retained test-only oracle,
  not a license to claim equivalent production support.

### Selected semantics — do not reopen accidentally

- One invocation-owned resolver and completed-value store. Initial scheduling
  remains **eager**; recursive reads do not select lazy omission of unused globals.
- Global constructors execute via the ordinary sequence/template executor.
  Specialized initializers are optimizations with guarded fallback, not another
  global evaluator.
- Legacy global entry gets a fresh principal effective-document root and singleton
  focus, with no inherited caller locals, current rule, or source-specific caches.
- Resolve before borrowing completed values. Never retain a mutable map entry or
  completed-store borrow across recursive execution.
- Active ordinals are bounded to 128 levels; failures clear active state, retain
  one primary failure, and cannot publish partial values or silently restart.
  Unwinding poisons the owner rather than pretending initialization can resume.
- Preserve effective source owners, temporary lifetimes, COW frames, source
  locations, cancellation and attributable work/retention charges.
- No public API/ABI, new resource authority, second execution backend, cross-
  invocation cache, or parallel global evaluation is selected by this work.

### Current bounded admission

The authoritative list is the constructor guard, not this abbreviated inventory:

- Text, messages, deferred failures and static local atomics.
- Connected legacy global string operations, effective booleans, literal/pair
  comparisons and their admitted value-of forms; preserve typed/node-set behavior.
- Source-controlled and admitted global-controlled `if`/`choose` conditions.
- Parameter-free named/matched dispatch and unsorted principal-source path iteration.
- Literal elements with admitted static, exact-variable and focus-based attributes.
- Comments and static-target PIs, including shared-executor dynamic bodies.
- Principal-source current-node, child-element and location-path copies, using
  the ordinary effective-source copier and result-owned payload.

Still guarded: module graphs with shared constructors, template parameters,
unproved match patterns/readers, modern global contexts, computed attributes and
attribute sets, dynamic PI targets, sorted/variable-selected iteration,
variable/document/ancestor-specialized copies and other unproved operations.
Some of these work in the private reference; that does not remove production gates.
Production bounded declaration/read-edge diagnostic context and cost/host
regression obligations are still incomplete. AR-0030 is **not closed**.

## Next work sequence

1. Inspect current status and reproduce a full modern frontier with detailed
   compile failures. Start with:

   ```powershell
   ./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1015
   ```

   `TraceFrontier` is a substring filter on the frontier label. Detailed rows
   include `expected-error=true/false`; separate ordinary cases from expected
   errors. If this yields no useful candidates, inspect the full report's ranked
   standard-operation initialization frontier and trace a relevant label instead.

2. Inspect actual unchanged stylesheet/source resources for a named case. Decide
   whether the first blocker is constructor admission, unsupported expression
   compilation, module composition, or an unrelated semantic subsystem. Record
   that distinction; do not infer it from the variable's name or error code alone.
3. Choose the smallest reusable slice within accepted decisions. If it involves
   another reader, connect it to the same owner and prove typed value semantics,
   local shadowing, dependency/cycle behavior and failure provenance before
   removing its guard. If unrelated, ordinary corpus breadth may proceed without
   pretending it completes AR-0030.
4. Add focused tests and unchanged corpus evidence. Test ordinary/shared parity,
   effective views, result lifetime, exact/one-less controls, cancellation and
   no partial publication as relevant. Keep explicit unsupported tests for
   adjacent unproved combinations.
5. Run the gates below, update evidence/checkpoint/roadmap with observed results,
   and update the SDD only to describe implementation within already accepted
   decisions. Do not edit ARs or ADRs. Continue
   to another justified slice if safe. Report failed experiments honestly and
   remove rejected candidates without disturbing inherited changes.

Do not repeat the recent pattern of expanding runtime guards merely because a
path looks easy: neither comments/PIs nor source copies changed OASIS counts.
They are valid integration work, but the next coverage step needs actual pressure.

## Validation and practical notes

```powershell
cargo test -p fastxslt --all-features shared_constructor
./scripts/verify.ps1
cargo check -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
./scripts/measure-oasis-xslt10.ps1
./scripts/measure-oasis-xslt10.ps1 -PreprocessingProfile strict-xslt10
./scripts/check-markdown-links.ps1
git diff --check
```

- `verify.ps1` includes formatting, Clippy with warnings denied, workspace tests,
  docs, links, unsafe-surface checks and corpus inventories. Do not suppress gates.
- Corpus script validates the pinned archive hash and 3,173-case catalog. Local
  archive/extracted fixtures are under `.workbench/oasis-xslt10`; do not edit them
  or upstream vendor fixtures. Use `-TraceCase` when a named case needs detail.
- Prefer sequential Cargo runs against one target directory to avoid lock waits
  and misleading contention. Do not kill an unidentified process holding a lock.
- Windows incremental-cache hard-link warnings have been benign; they are not
  semantic failures. Investigate real compile/test failures separately.
- Use `--all-features` for normal focused tests. A manual `variable40` reference
  replay recently gained its missing `workbench` feature gate; default-feature
  focused test compilation was then checked successfully as well.
- `cancelling_on_charge(domain, n)` counts matching **charge calls**, not units.
  A threshold of one requires a second matching charge call. Measure initialization
  alone via `with_completed(..., |_, _| Ok(()))` when deriving exact limits;
  converting a completed value to string afterward adds separate charges.
- Result-text-byte accounting is not total allocator/RSS accounting. Do not
  describe passing a text budget as proving all memory is bounded by that number.
- On Windows use `rg directory -g pattern`, not a wildcard in the directory path.
- Follow ADR-0004 for file/function/folder pressure. Extract along a named
  responsibility, not merely to satisfy a line-count limit.

## Stop and hand back when

- A change needs a new decision on ownership, authority, scheduling, public
  contracts, semantic edition, diagnostic attachment or another unresolved AR.
  Describe the concrete case and alternatives and return to the maintainer here;
  do not create or modify an AR/ADR, or silently choose policy.
- Standards and archival expectations conflict. Keep the original comparator
  result visible; do not alter fixtures or reclassify failure as success.
- Verification reveals a regression that cannot be safely resolved within the
  selected slice, required resources are unavailable, or concurrent edits conflict.
- A substantial coherent tranche is validated and the next step requires a
  materially different seam. Update this handoff or add a dated successor.

At handback report: changed behavior/files, exact commands and outcomes, both
corpus profiles, remaining blockers/decision requests, and uncommitted status.
No percentage gain, performance gain, AR closeout or successful gate should be
claimed without corresponding evidence. No maintainer approval is currently
needed simply to investigate the next in-scope compilation blocker.
