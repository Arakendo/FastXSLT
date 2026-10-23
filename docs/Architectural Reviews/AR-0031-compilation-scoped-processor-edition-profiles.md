# AR-0031: Compilation Scoped Processor Edition Profiles

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-10-16 |
| Last reviewed | 2026-10-16 |
| Scope | Processor edition, version interpretation, feature recognition and introspection |
| Trigger | OASIS version-2.0 fallback cases expect XSLT 1.0 processor behavior |
| Related ADRs | ADR-0004, ADR-0006, ADR-0007, ADR-0023, ADR-0024 |
| Related evidence | [Agent handoff and validated baseline](../Checkpoints/2026-10-16-vscode-agent-handoff.md) |

## Architectural question

How should a compilation-scoped processor-edition profile control declared
version interpretation, instruction recognition, forward-compatible processing,
availability introspection and processor-version reporting while preserving one
compiler/runtime and the modern semantic foundation?

Three concepts must remain distinct:

- The stylesheet declares a version, including locally scoped declarations.
- ADR-0023 selects the compilation graph's stylesheet preprocessing rules.
- A processor-edition profile would select the semantic processor against which
  declared versions and recognized facilities are interpreted.

The leading hypothesis is that compilation selects processor semantics and
those semantics interpret stylesheet versions, rather than stylesheet versions
selecting which processor FastXSLT pretends to be. This review does not yet accept
that model or a public selector.

## Trigger and evidence

The VS Code agent reported a clean stop at commit `187cc015`, with no engine or
fixture changes. Its `FXST1015` trace found no remaining matching cases. Other
compilation traces exposed a fallback cluster, including ordinary case
`Microsoft/Fallback__91818#1`, failing with `FXST1006`.

Local inspection confirms that `91818.xsl` declares version `2.0` and contains
unknown `xsl:foo` with a direct `xsl:fallback` child. The unchanged catalog cites
XSLT 1.0 and supplies an XML reference result. Neighboring cases exercise
multiple and nested fallbacks and lexical variables. They are not all success
cases: `91825`, for example, expects an execution error when fallback is outside
the unknown instruction. Enumerate catalog membership before predicting gains.

The distinction is standards-driven, not evidence of a broken reference:

- [XSLT 1.0 section 2.5](https://www.w3.org/TR/xslt-10/#forwards)
  enables forward-compatible behavior for the specified non-1.0 version contexts;
  unknown instructions reached during execution use fallback. Untaken unknown
  instructions and selected expression errors have deferred-error requirements.
- [XSLT 3.0 sections 3.9–3.10](https://www.w3.org/TR/xslt-30/#forwards)
  distinguish compatibility with earlier versions from forward compatibility
  with versions later than 3.0. A version-2.0 stylesheet does not generally make
  an unknown XSLT instruction eligible for forward-compatible fallback.

Thus version `2.0` intentionally tests future-version handling when the selected
processor is 1.0; its meaning differs for a modern processor.

Current implementation points:

- `compile/golden_stylesheet_experiment.rs` contains
  `is_xslt10_forward_compatible_version`, currently testing `1.0 < v < 2.0`.
- `compile/instruction_compiler.rs` has separate legacy and modern compatibility
  helpers, including special handling within a stylesheet declaring `1.0`.
  Compatibility is not governed solely by the interval helper.
- Existing fallback compilation already delegates bodies to shared sequence
  compilation and represents a missing fallback as a deferred failure in its
  admitted legacy lane. The missing architectural owner is not a new interpreter.
- `uses_xslt10_compatibility` separately influences expressions and recovery;
  changing only fallback admission would not establish a coherent processor.

Latest validated counts remain modern **2,501 / 3,173**, strict-preprocessing
**2,498 / 3,173**, with 14 / 17 mismatches and 423 expected-error credits each.
The strict-preprocessing lane is **not** currently proof of complete XSLT 1.0
processor semantics. Opening this review earns no additional pass. No new
processor-profile experiment or full-suite replay was performed for this record.

## Ownership and constraints

- ADR-0007 remains the modern semantic foundation. A compatibility profile must
  use the same compiler, XDM and runtime, not introduce a second legacy engine.
- ADR-0023 remains preprocessing-only. Do not silently repurpose its selector as
  processor identity, or infer processor identity from principal/local versions.
- Candidate processor selection belongs to compilation and its complete module
  graph. Includes/imports inherit that selection while local declared versions
  retain the scope defined by the selected edition. Probe embedded and literal-
  result stylesheets separately.
- Implementation knowledge, edition-defined recognition and implemented
  capability are distinct. Recognizing a newer instruction in the shared compiler
  must not automatically expose it to the 1.0 profile. Conversely, an engine
  limitation must not masquerade as a standards-defined unknown instruction to
  justify fallback. Define this distinction explicitly before admission.
- Compiled artifacts and any future reuse keys must remain profile-confined.
  No source-dependent compiled state, cross-profile reuse or new cache is selected.
- All adapters project the same selected semantics. Host policy cannot enable
  ambient resource access, extension execution, or weaken controls through a profile.
- Preserve diagnostics, source locations, compile/execution error timing, work
  limits, cancellation and result ownership. ADR-0024 global initialization
  scheduling and ownership are not changed by this review.
- Keep upstream fixtures unchanged and all cases in conserved accounting.
  A profile-specific result cannot silently replace the modern or original oracle.

## Alternatives

### Retain current modern behavior and classify edition differences

Lowest implementation cost and preserves the current boundary. The fallback
cluster remains uncredited under modern semantics. This does not satisfy the
eventual goal of deliberately behaving as an XSLT 1.0 processor.

### Explicit processor edition with coherent presets — leading candidate

Keep processor semantics and preprocessing distinct internally. Investigate two
coherent combinations first:

| Candidate preset | Processor semantics | Preprocessing |
| --- | --- | --- |
| XSLT 1.0 compatibility | XSLT 1.0 | Strict 1.0 |
| Modern | Selected modern editions | Modern |

Cross-combinations may be useful private differential controls, but are not
proposed supported product modes. A preset must not imply full conformance before
its facilities are implemented and evidenced. Public names, defaults and API
placement remain deferred.

This provides an explicit owner but requires propagation through compilation,
recognition, expressions and introspection, with careful regression isolation.

### Broaden version heuristics or add a fallback-only toggle

Small apparent change, but risks mixing modern recognition with legacy fallback,
XPath behavior and reported capabilities. Not justified as a production fix;
even a test-only probe must be labeled incomplete and cannot establish a profile.

### Separate legacy compiler/runtime

Would duplicate semantics, diagnostics and resource controls. It conflicts with
the desired shared-engine direction and has no motivating evidence here.

## Findings and uncertainties

The cases establish a real processor-identity question distinct from whitespace
preprocessing and global construction. They do not establish the size of the
implementation, the number of newly exact cases or the complete profile contract.

Resolve these together rather than implementing only the first row:

| Surface | Required distinction or probe |
| --- | --- |
| Unknown instructions | Taken/untaken, missing/direct/multiple/nested fallback |
| Known newer instructions | Modern implementation knowledge must not leak into 1.0 recognition; probe `xsl:sequence` |
| Expressions | Deferred versus static errors, unknown functions, arity and XPath edition |
| Availability | `element-available()` and `function-available()` consistent with selected recognition/capability |
| Version reporting | `system-property('xsl:version')` consistent with the selected processor; no unsupported conformance claim |
| Version scope | Principal/local declarations, disabling compatibility and edition-specific version lexical rules |
| Module graph | Includes/imports with mixed declarations; embedded and simplified stylesheets |
| Preprocessing | Coherent presets and explicitly labeled cross-profile controls |
| Diagnostics | Invalid stylesheet versus engine-unsupported feature; exact failure origin and timing |

In particular, the 1.0 rule must be derived from its actual specification, not
approximated as `version > 1.0`. The existing helpers' parsing and boundary
assumptions need auditing as part of the experiment, not mechanical reuse.

## Disposition

**Proposed.** Open a separate processor-edition investigation. Do not broaden the
fallback interval, alter ADR-0023 or select a production/public profile by opening
this AR. The explicit compilation-scoped candidate is favored for investigation,
not accepted. Ordinary unrelated corpus work and AR-0030 integration may continue.

## Required follow-up

### Current decision checkpoint

The maintainer accepted [ADR-0025](../ADR/ADR-0025-compilation-graph-scoped-processor-edition-policy.md)
without modification on 2026-10-16. This closes the ownership review: one
immutable compilation-graph policy, distinct from preprocessing and local
declared versions, governs processor semantics through the shared compiler/runtime.
Recognition versus capability is normative. Production integration remains
subject to the ADR's executable gates; the experiment is not promoted.

The [semantic audit and decision checkpoint](../Campaigns/processor-edition-profile/evidence/ar-0031-bounded-instruction-probe-2026-10-16.md#semantic-audit-and-decision-checkpoint)
recommends a narrow ownership decision, not promotion of the prototype. Select
graph-scoped processor policy separately from preprocessing and local declared-
version context; require edition-aware recognition, error timing, introspection
and artifact qualification through the shared compiler/runtime.

The audit corrected the test-only modern missing-fallback path: unlike legacy
forward-compatible deferral, the modern witness now rejects it statically.
Fifty probes pass, but local-version semantics, general expression recovery,
typed truthful version reporting and the capability catalog remain implementation
gates. More graph variations are not needed before reviewing the direction.
ADR-0025 accepts the direction only; no production profile is promoted or
conformance credit earned.

### Experiment history

The [first bounded instruction probe](../Campaigns/processor-edition-profile/evidence/ar-0031-bounded-instruction-probe-2026-10-16.md)
now exercises explicit edition/version witnesses through existing lowering and
ordinary runtime controls. A subsequent private callback extraction propagates
the witnesses through nested fallback bodies; general containers and local-version
changes remain guarded. A modern-known `xsl:sequence` witness retains
its existing capability refusal rather than using fallback. This is narrower
than a complete profile experiment; the follow-up obligations below remain open.

The subsequent introspection probe reuses existing lexical/QName handling with
an explicit test-only policy callback. It separates unknown-to-edition from
unproved capability and keeps hypothetical version witnesses independent of local
declarations. Its 12 focused checks do not establish a capability catalog or
select truthful production version reporting; production folding is unchanged.

The composition tranche brings the focused total to 15: an explicit test context
applies the existing immutable preprocessing view before nested fallback lowering,
with coherent preset witnesses, orthogonality controls and preprocessing
cancellation. The current module context only stamps preprocessing provenance;
it does not carry edition policy through instruction lowering. Complete graph
propagation therefore remains open rather than being inferred from a new stamp.

The conditional prerequisite now uses shared `xsl:if` validation/construction with
explicit test/body callbacks. Eighteen focused checks include constant-condition
reachability on the ordinary runtime, nested policy propagation, diagnostic
parity and depth refusal. General expressions and other containers remain guarded;
this does not yet establish module-graph processor propagation.

At the next checkpoint conditional instruction validation/assembly is privately
extracted under ADR-0004 before expanding the probe. The bounded availability
callback now drives real conditional lowering/execution (21 focused checks),
with unproved modern availability refusing compilation rather than inheriting
production's false answer. General expressions and graph propagation stay open.

Choice-body propagation adds `when`/`otherwise` under the same bounded policy
(24 focused checks), preserving shared structural diagnostics and first-match
runtime behavior. Production false-branch pruning remains in its original
caller policy; the reference does not infer broader error suppression from this
slice. Complete module compilation and graph propagation are still unproved.

The parameter-free template-boundary probe now returns an ordinary complete
template using shared metadata/body assembly (27 focused checks). Parameter
defaults are explicitly refused before declaration-derived compilation can
re-enter. Runtime evidence still uses a compiled shell; this is not full module
compilation. The reference policy and invariant tests are separated under
ADR-0004 before further growth.

The next bounded entry compiles an original single-root-template module without
a shell or body patch (32 focused checks). Shared top-level routing/finalization
is retained; only four exact version spellings and a parameter-free root template
are admitted. Other declarations/routes and include/import graph propagation
remain refused. This is the first narrow whole-module evidence, not general
processor-profile integration or corpus credit.

A supplied include-chain reference now carries one edition through up to four
already-parsed modules (36 focused checks), using existing preprocessing and
include composition. Mixed declarations preserve leaf recognition/provenance;
shared preprocessing cancellation and exact-budget boundaries are tested. Exact
logical href matching is not a resolver. Imports, general declaration graphs,
local-version scope and reusable edition-qualified artifacts remain unproved.

Artifact isolation probes bring the total to 39: concurrent compilation/reuse,
parsed-module lifetime independence, failed recompilation and same-URI changed
bytes preserve existing programs. Mixed coherent presets fail the existing
preprocessing-stamp linker check. That does not establish independent processor-
edition identity; a separate qualified-artifact candidate remains unselected.

A subsequent two-module candidate retains edition and preprocessing separately
and admits only compiler-created leaves (42 focused checks). A full cross-product
control rejects mismatches even when preprocessing agrees, and legacy fallback
cannot be relabeled as modern. This mechanism is experimental, not selected;
it does not establish transitive reusable artifacts or public profile selection.

The next bounded candidate retains that qualification across intermediate
include assembly, capped at four modules (45 focused checks). Differential
program/work equality, late profile mismatch, deep repeated identity, fifth-module
refusal and final-link cancellation are pinned. General graph reuse, imports and
production profile selection remain unproved and unselected.

Qualified single-import edges now reuse existing import composition (48 focused
checks), preserving nested import precedence, mixed-version propagation, leaf
provenance and preprocessing differentials. Parents contain one dependency only;
competing rules, apply-imports and branching graphs remain unproved. No alternate
linker or production processor profile is selected.

- [ ] Inventory unchanged ordinary and expected-error cases separately; capture
  first failures, archive provenance, profile and exact commands.
- [ ] Audit version/compatibility, instruction recognition and introspection
  call sites; identify one private compilation owner and ADR-0004 extraction needs.
- [ ] Produce a standards matrix for 1.0 versus modern behavior across the
  surfaces above, including known newer instructions and unsupported known features.
- [ ] Design a private, bounded differential experiment through the shared
  compiler/runtime. No public profile types or production default change.
- [ ] Pin coherent preset selection and complete module-graph propagation;
  preserve local-version scope without allowing it to switch processor identity.
- [ ] Test fallback branch reachability, expression error timing, introspection,
  exact diagnostics, cancellation and bounded compilation/execution.
- [ ] Replay original OASIS expectations per profile and pinned modern cases;
  preserve the existing baselines and classify every regression independently.
- [ ] Evaluate compiled-artifact isolation and any measurable compile/warm-path
  cost before selecting implementation shape or making performance claims.
- [ ] Return findings for maintainer review and a separate ADR if justified.
  The delegated VS Code agent may report evidence but may not author or modify
  ARs/ADRs under its current handoff instructions.

## Reopening triggers

If deferred, reopen for a consumer requiring exact 1.0 processor behavior,
additional edition-sensitive corpus clusters, contradictory introspection or
new evidence that a coherent profile can share the engine without semantic drift.

## Review history

- 2026-10-16 — Opened at maintainer request after Luna's fallback-frontier
  escalation and local source/fixture review. Monday supports separating declared
  version, preprocessing and processor semantics, with coherent candidate presets.
  No ADR or production semantics changed.
