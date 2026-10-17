# AR-0031 bounded instruction probe — 2026-10-16

This record retains the initial tranche below; the subsequent nested-body
tranche at the end supersedes its blanket nested-fallback restriction.

Related: [AR-0031](../Architectural%20Reviews/AR-0031-compilation-scoped-processor-edition-profiles.md)
and [initial investigation](processor-edition-seam-investigation-2026-10-16.md).

## Scope

The first experiment is a `cfg(test)` instruction-level probe in
`compile/instruction_compiler/processor_edition_reference.rs`. The parent adds
only a conditional private module declaration. No production policy/helper,
public API, fixture or compiled artifact representation changes.

Explicit inputs select legacy versus modern processor edition and one of four
already-interpreted version witnesses (1, 2, 3, 4). This is **not** version parsing,
ancestry resolution, a complete compilation profile or module-graph propagation.
It recognizes only the existing legacy vocabulary plus `sequence` as a modern
witness and `foo` as an unknown witness. Other names and broader lowering are
explicitly unsupported by the probe, not claimed absent from the standard.

Known text/sequence instructions delegate to ordinary instruction compilation.
Unknown forward-compatible instructions delegate to the existing direct-fallback
lowerer. Fallback bodies are restricted to text and `xsl:text`: nested instruction
bodies are refused before delegation because their ordinary recursive compilation
would otherwise silently revert to declaration-derived policy. This restriction
is intentional experimental incompleteness, not selected semantics.

## Six focused checks

- Same immutable version-2.0 fixture: legacy selection lowers two direct fallback
  bodies in order, ignoring surrounding non-fallback text; modern selection
  rejects the unknown instruction. Production sequence compilation still rejects
  the original fixture. Lowered instructions/provenance equal the existing helper.
- Edition/version witness matrix: legacy non-1 witnesses and modern version-4
  witness admit deferred missing-fallback instructions; other combinations fail
  the probe's static classification. `FXAR0031` is not a production error choice.
- Modern-known `xsl:sequence` with fallback takes legacy fallback under legacy
  selection; modern selection preserves the ordinary compiler outcome instead.
- A simple `xsl:sequence select="'modern'"` exposed an existing capability refusal
  (`FXXP1003`). The final test explicitly preserves that modern Unsupported
  outcome. Legacy selection instead treats the instruction as unknown and lowers
  missing fallback to deferred failure. Recognition is not implementation success.
- Ordinary runtime execution of an injected, lowered missing-fallback instruction
  under a constant `if` succeeds when untaken and reports `XTDE1450` when reached.
  A normally compiled shell is used only to execute the probe body. This is not
  end-to-end compilation of the unknown-instruction stylesheet. The existing
  cross-layer test helper exposes failures as debug strings, so this runtime
  assertion does not prove structured diagnostic-field parity.
- Nested non-text fallback content is explicitly refused rather than leaking the
  production declaration heuristic into the experiment.

## Interpretation and next boundary

Validation: `scripts/verify.ps1` passes (1,982 core tests, 63 ignored;
native 22, WASM adapter 4, worker 18), as does
`cargo check -p fastxslt-wasm-workbench --target wasm32-unknown-unknown`.
Local Markdown links and `git diff --check` pass. Initial test/Clippy failures
were corrected before these gates: the modern sequence capability refusal is
now tested rather than incorrectly assuming that expression is implemented.

Explicit edition selection can route a bounded instruction through existing
lowering and execution without modifying production behavior or introducing an
interpreter. The experiment also demonstrates why recognized-but-unimplemented
instructions must remain distinct from edition-unknown instructions.

It does **not** establish a 1.0 processor profile, new OASIS passes, introspection
consistency, local-version scope, graph propagation, expression-error deferral,
profile isolation, compilation accounting or performance. No corpus replay is
credited to this probe. Last measured production counts remain 2,501 modern and
2,498 strict-preprocessing exact comparisons out of 3,173.

Next design the explicit immutable compilation-context propagation through shared
recursive sequence lowering, before removing the nested-body guard. Keep the
preprocessing field semantically separate; neither ambient/thread-local overrides
nor document rewriting are acceptable substitutes. This requires an ADR-0004
coupling review, not an automatic promotion of the tiny probe's witness enums.
Introspection and full module graphs remain later tranches. AR-0031 is not ready
for acceptance on this evidence alone.

## Subsequent tranche: explicit nested-fallback propagation

Two private callback seams now preserve existing ownership:
`compile_sequence_with_element_lowerer` owns ordinary child/text traversal and
lexical binding storage, while `compile_forward_fallbacks_with_body` owns direct
fallback selection/order, attribute checks and missing-fallback lowering.
Production wrappers supply the original lowering functions. No production
edition selector or compatibility rule changes. This is a narrow ADR-0004
responsibility extraction, not a new sequence compiler or runtime.

The test-only reference supplies an element callback which passes the same
explicit edition/version witnesses into every nested unknown-instruction
fallback. Shared traversal handles text and ignores comments/PIs as before.
General containers/local variables remain refused: their existing recursive
lowerers have not acquired explicit policy propagation. Local `xsl:version`
changes are refused rather than interpreted using a different owner. Probe
recursion has an explicit 16-instruction-depth guard; this is a test envelope,
not a selected production quota or complete compilation budget model.

The original nested-refusal test now proves nested fallback success for legacy
version 2 and modern version 4 witnesses, while modern version 2 still rejects
the unknown outer instruction. Three additional checks cover:

- A nested modern-known `xsl:sequence`: legacy takes its fallback; modern retains
  the ordinary compiler's failure code, category and original location. The
  fixture's invalid child structure can fail before expression capability checks;
  the probe must not relabel that failure as Unsupported or take legacy fallback.
- Explicit refusal of unpropagated `if` bodies, local-version changes and excessive
  nesting, instead of silently using production declaration-derived semantics.
- Concurrent legacy/modern probes over the same immutable document without
  ambient or thread-local edition state.

There are now nine focused checks. This still does not establish full compilation
graph propagation, local-version semantics, introspection, expression deferral,
or unchanged OASIS execution through an experimental processor preset.

Validation of this tranche: full `scripts/verify.ps1` passes with **1,985 core
tests**, 63 ignored, native 22, WASM adapter 4 and worker 18. The official WASM
target check, Markdown links and diff checks pass. Both complete production
OASIS replays preserve **2,501 modern / 2,498 strict-preprocessing exact**, 14 / 17
mismatches and 423 expected-error credits each. These replays check the shared
callback extraction for regression; they do not run the new probe as a processor
profile and earn no new corpus credit.

## Subsequent tranche: bounded introspection policy injection

A test-only hook in the existing static-introspection owner reuses its call,
string-literal and QName parsing, then passes the expanded request to an explicit
policy callback. Production `fold` and its declaration-derived version reporting
are unchanged. The hook is compiled only for tests and adds no public surface.

Three additional checks (12 focused checks total) establish:

- Hypothetical legacy/modern reporting witnesses can differ on the same immutable
  element, independently of its local `xsl:version` declaration.
- `xsl:sequence` is unavailable to the legacy witness, while modern recognition
  does not justify claiming availability: the probe leaves that answer unproved
  and preserves the real compiler's Unsupported result.
- Prefix aliases, default element namespaces and unprefixed function names reuse
  existing QName handling. Undeclared/malformed names, dynamic arguments, arity
  overloads and unexamined facilities are refused rather than assigned answers.

The callback's `None` is an experimental refusal, not XPath `false`. Its version
strings are hypothetical policy fixtures, **not** chosen processor reporting or
conformance claims. Only text and the zero-argument `true` function are positive
availability witnesses. This does not implement a capability catalog, integrate
introspection into the explicit-edition lowering path, or settle modern function
namespace/arity rules. The next integration must give recognition, capability and
reporting one explicit compilation owner without equating these concepts.

No new source file or folder was added: lexical mechanics remain with their
existing owner and the bounded policy remains in the test-only reference. There
is no production change in this tranche and no new corpus replay or credit.

Validation: all 12 focused checks pass; `scripts/verify.ps1` passes with 1,988
core tests (63 ignored), native 22, WASM adapter 4, and worker 18, plus formatting,
Clippy, documentation, Markdown links and corpus-integrity gates. An initial
Clippy identical-match-arms finding was fixed before the successful full run.

## Subsequent tranche: preprocessing and edition composition

Inspection of the module compiler confirms that its existing
`StylesheetCompilationContext` is preprocessing provenance: `qualify_module`
stamps an already compiled program, and callers must supply preprocessed trees.
It does not propagate processor policy into instruction compilation. Adding an
edition stamp alone would therefore not establish graph-wide edition semantics.

The bounded reference now composes an explicit edition and preprocessing choice
in a test-only compilation witness. Its coherent constructors pair legacy with
strict preprocessing and modern with modern preprocessing. Before lowering an
instruction it derives the existing bounded immutable stylesheet visibility view
over the stylesheet root. Nested fallback lowering consumes that effective view.
No production compilation-context or compiled-program fields were changed.

Three checks cover comment-separated whitespace in nested fallback bodies,
independence of recognition from preprocessing (including deliberately mismatched
private controls), and cancellation during preprocessing before a body is
returned. The source remains unchanged. The view's resource controls are reused;
this does not establish work charging throughout instruction lowering.

This is composition evidence, not module-graph integration. The probe still
receives already-interpreted version witnesses; embedded/simplified stylesheets,
local declaration interpretation, include/import propagation and compiled-artifact
profile checking remain open. The existing graph stamp must not be repurposed as
processor policy without threading that policy through the actual lowerers.

Validation: 15 focused probes pass. Full `scripts/verify.ps1` passes with 1,991
core tests (63 ignored), native 22, WASM adapter 4 and worker 18; the official
WASM target check also passes. Initial test-module import and enum-pattern
compile errors were corrected before validation. Production OASIS was not rerun:
this tranche changes test-only code and earns no corpus credit.

## Subsequent tranche: conditional recursive lowering

`compile_if_with_lowerers` now owns the existing attribute validation, required
test lookup, original location and `Instruction::If` construction. Its production
wrapper supplies the same boolean compiler and sequence compiler as before, in
the same test-before-body order. The bounded probe supplies a guarded constant
test compiler and recursively edition-aware body callback. Only `true()` and
`false()` are admitted; arbitrary expressions and other containers remain guarded.

Three more checks bring the focused total to 18. Shared conditional lowering is
differentially equal for ordinary text bodies, including exact missing-test and
unexpected-attribute failures. Actual lowered conditions then execute on the
ordinary runtime: reached missing fallback fails with `XTDE1450`, untaken fallback
does not, and a modern version-2 unknown instruction remains a compile error even
inside `false()`. Nested fallback/if/fallback preserves the selected edition and
the common depth guard. The runtime helper exposes debug-formatted errors, so
runtime assertions remain code-presence checks, not structured-diagnostic proof.

This is a prerequisite for graph integration, not graph integration itself.
Reusing module assembly before recursive bodies preserve policy would conceal
declaration-derived re-entry. Introspection callbacks are still independent
probes, not yet arbitrary condition folding under the experimental context.

ADR-0004 review: the parent is 3,464 lines with 21 direct child files. This
tranche retains the existing conditional owner and adds a narrow callback seam,
not another sibling file or expression implementation. Inputs are immutable
document/node and explicit lowerers; output remains the ordinary instruction or
compile failure. Production callbacks have no new host policy, mutable context,
resource access or runtime state. Broader conditional-family extraction is
deferred to the checkpoint before expanding beyond this constant-if probe;
do not accumulate per-container edition branches indefinitely in the parent.

Validation: all 18 focused probes pass. `scripts/verify.ps1` passes with 1,994
core tests (63 ignored), native 22, WASM adapter 4 and worker 18. The official
WASM check and diff check pass. Initial delimiter, raw-string lint and formatting
issues were corrected before the successful gates. Both unchanged production
OASIS replays preserve 2,501 modern / 2,498 strict-preprocessing exact results,
14 / 17 mismatches and 423 expected-error credits each. These are conservation
checks for the shared helper extraction, not experimental-profile corpus passes.

## Conditional-owner extraction checkpoint

The promised ADR-0004 review extracts conditional instruction validation and
assembly into private `conditional_instruction_compiler.rs` (190 lines). The
parent drops from 3,464 to 3,298 lines. This is distinct from the existing XPath
conditional-expression compiler: it owns `xsl:if` / `xsl:choose` structure,
attribute validation, test-before-body assembly and diagnostic origins, not
expression interpretation or processor policy. Existing boolean/sequence
compilers remain its collaborators; the parent's `compile_choose` wrapper
preserves the existing caller boundary. No new public types or runtime state.

Directory review: 22 direct files remain a flat, named set of instruction and
expression owners. A single conditional-family file does not yet justify a
new directory containing only that file. Retain this layout at this checkpoint;
revisit grouping if multiple conditional-specific owners emerge. No generic
helper bucket or unrelated code movement is introduced. All 18 AR-0031 probes
pass immediately after the behavior-preserving extraction, before expression
admission is widened.

Full `scripts/verify.ps1` also passes at the extraction checkpoint: 1,994 core
tests plus the existing adapter/worker suites, docs, links and corpus-integrity
gates. A documentation backtick lint was corrected before this successful run.

## Availability-driven conditional execution

The existing bounded introspection policy now supplies boolean conditions to
the experimental `xsl:if` lowerer, not just standalone assertions. The policy
function moved out of the nested test module into the same test-only reference;
no duplicated availability table or production introspection change was added.

Three new probes (21 total) check actual ordinary-runtime execution for available
text/true witnesses and legacy-unavailable sequence, refusal of unproved modern
sequence availability, and policy propagation through nested fallback/conditional
bodies. The modern refusal is particularly important: production currently folds
that availability query to false, but the experiment must not borrow that answer
when its selected capability has not been established. Unsupported retains the
original conditional location and no compiled body is returned.

String-valued version reporting is still excluded from condition folding: the
hypothetical reporting witnesses do not authorize general effective-boolean or
comparison semantics. This remains a narrow availability slice, not full XPath
edition handling, local-version interpretation or module-graph integration.

Validation: all 21 focused probes and full `scripts/verify.ps1` pass, with 1,997
core tests (63 ignored), native 22, WASM adapter 4 and worker 18. Official WASM
and diff checks pass. The first runtime-output assertion omitted the ordinary
XML declaration; its expected string was corrected without changing execution.
Both unchanged OASIS replays retain 2,501 modern / 2,498 strict-preprocessing
exact, 14 / 17 mismatches and 423 expected-error credits each. No new corpus
credit or profile acceptance is inferred.

## Choice-body propagation

The private conditional owner now exposes a shared `compile_choose_with_lowerers`
seam. Structure validation, attribute checks, original locations and ordered
branch assembly remain shared. Production supplies its existing expression
compiler and false-branch pruning behavior; the experimental caller compiles all
bounded branch bodies under its explicit edition/version witnesses. The latter
does not claim general forward-compatible expression-error suppression or pruning.

The probe's condition helper is shared by `if` and `choose`, admitting only the
existing constants and bounded boolean-availability witnesses. Body and condition
entry points refuse local-version overrides rather than silently consulting a
different owner. `when` and `otherwise` cannot reset the processor selection.

Three additional tests bring the total to 24. They cover differential instruction
and structured compile-failure equality for ordinary/malformed choices; first-
matching-branch runtime behavior with untaken versus reached missing fallback;
modern version-2 static rejection despite an earlier true branch; availability
conditions and nested fallback in `otherwise`; and local-version refusal. Runtime
error assertions still use the existing string-returning test helper and do not
constitute structured runtime-diagnostic parity.

ADR-0004: the conditional owner remains cohesive at 215 lines; the test-only
reference is 907 lines, mostly evidence fixtures. No new file or production
policy owner was introduced in this tranche. Before substantial further growth,
split reference policy/entry points from invariant-grouped tests rather than
letting this become another oversized compiler unit. Whole-module compilation,
lexical variables, local-version interpretation and graph propagation remain open.

Validation: all 24 focused probes pass; full `scripts/verify.ps1` passes with
2,000 core tests (63 ignored), native 22, WASM adapter 4 and worker 18. Official
WASM and diff checks pass. Both unchanged OASIS replays preserve 2,501 modern /
2,498 strict-preprocessing exact comparisons, 14 / 17 mismatches and 423
expected-error credits each. No new corpus credit is attributed to the probe.

## Template-boundary reference and test ownership

The 907-line reference was split before expansion: policy/lowering remains in
`processor_edition_reference.rs`, while its invariant tests live in the private
`processor_edition_reference/tests.rs` child. All 24 probes passed after this
mechanical move. This adds a named reference-test subdirectory without increasing
the flat instruction-compiler sibling count or widening visibility.

The shared template compiler now takes a private body-lowering callback. Its
production wrapper supplies the original sequence compiler; attribute checks,
parameter handling and template location remain in their existing owner.
The experimental caller refuses all template parameters before that shared
compiler can evaluate defaults. A parameter-free template body is lowered under
the explicit edition witness, producing a complete ordinary `Template`, not
merely an instruction vector patched into an existing template body.

Three new tests (27 total) prove differential template/metadata equality on an
ordinary multi-instruction body, execution of a full legacy-fallback template,
modern rejection at the original unknown instruction, and refusal of both
select-based and constructor-based parameter defaults. The runtime test still
installs the resulting template into an ordinary compiled shell program: this is
template-boundary integration, **not whole-module compilation or module linking**.

ADR-0004 review: the reference owner is now 261 lines and its tests 728 lines.
The 3,128-line stylesheet compiler retains template declaration/parameter
validation; only a private callback seam was added. Moving that whole family
while its profile propagation is unresolved would mix broader decomposition
with semantic integration. Retain at this checkpoint and revisit before
admitting parameter defaults or changing top-level template assembly. No public
API, production edition policy, resolver authority or runtime owner changed.

Validation: 27 focused probes pass. Full `scripts/verify.ps1` passes with 2,003
core tests (63 ignored), native 22, WASM adapter 4 and worker 18. Official WASM
and diff checks pass. Both unchanged OASIS replays retain 2,501 modern / 2,498
strict-preprocessing exact, 14 / 17 mismatches and 423 expected-error credits
each. The reference split and template callback earn no new corpus credit.

## Bounded original-module compilation

The reference now has an original-document module entry point. It admits exactly
one parameter-free `match="/"` template, no other declarations or template
attributes, and only the exact version spellings `1.0`, `2.0`, `3.0`, `4.0`.
Those are bounded lexical witnesses, not general edition-specific numeric parsing.
Preprocessing is selected independently of the declaration from the explicit
edition. The existing immutable visibility view runs before lowering.

Shared top-level compilation and root-template routing now accept a private
root-template lowerer. Production supplies the unchanged `compile_template`.
The reference supplies its edition-aware parameter-free template lowerer and
uses existing finalization routines and preprocessing qualification. It returns
a complete `StylesheetProgram` from the original stylesheet, with no fabricated
shell, XML rewriting, skipped template declaration or post-compilation body patch.
Other template-routing paths are not made edition-aware by this callback and
remain excluded by the reference's preflight checks.

Four new tests bring the focused total to 31: original-module fallback execution
across the four declaration witnesses and both editions; full-program equality
against ordinary compilation of the same effective preprocessing view; explicit
refusal of includes, globals, non-root/named templates and parameters; and
preprocessing cancellation returning no program. Full-program comparison includes
the preprocessing stamp rather than discarding it to manufacture equality.

This establishes a narrow whole-module slice, not an include/import graph or a
general processor profile. Compilation work accounting beyond preprocessing,
general/local version parsing, globals, parameters, matched/named templates and
profile-confined graph assembly remain open. No OASIS pass is credited from
hand-authored fixtures.

An additional guard regression brings the focused total to 32. Probe admission
uses an exact attribute whitelist, not the compatibility-aware production
attribute validator: that validator may legitimately ignore attributes which
must nevertheless be refused by this narrow experiment. Across declarations
1.0, 2.0 and 4.0 and both edition witnesses, mode, priority, name, extra/foreign
attributes, default-mode and xml:space cannot bypass the bounded module route.
This is probe eligibility, not a new standards restriction.

ADR-0004: the stylesheet compiler is 3,143 lines and the reference owner 350.
The existing declaration router still owns fast-path eligibility and dispatch;
the top-level owner still owns declaration validation/composition. The callback
does not transfer those responsibilities or introduce adapter/runtime state.
Retain these narrow private seams; review broader ownership before admitting
other template routes rather than duplicating top-level compilation.

Validation: all 32 focused probes and `scripts/verify.ps1` pass: 2,008 core
tests (63 ignored), native 22, WASM adapter 4 and worker 18. The official
`wasm32-unknown-unknown` adapter check and `git diff --check` also pass.
Unchanged OASIS replays retain 2,501 modern / 2,498 strict-preprocessing XML
exact matches, 14 / 17 mismatches and 423 expected-error credits each. These
are existing preprocessing lanes, not claims for the experimental processor
edition. No new corpus credit or production profile is selected.

## Supplied include-chain propagation

The next test-only reference admits at most four already-parsed documents in a
caller-supplied chain. Each non-leaf module contains exactly one `xsl:include`;
the leaf uses the existing parameter-free root-template probe. Exact logical
href equality checks the supplied edge. This is not URI resolution, snapshot
admission, file acquisition or a replacement graph resolver. Repeated resource
identities, missing/extra dependencies, imports, additional declarations and
compatibility-ignored attributes remain refused.

One explicit edition argument flows through every recursive call. Each module
retains its own declared version witness; none chooses processor identity.
Every module receives the same coherent preprocessing preset and shared control.
The existing include-program composition function performs merging and checks
preprocessing provenance. Non-leaf preflight permits no executable body to enter
its production lowerer. The probe accepts documents, not previously compiled
artifacts: it does not establish an edition-qualified reusable artifact format.

Four tests bring the focused set to 36. A three-module mixed-version matrix
varies principal and leaf declarations under both editions, proving leaf fallback
recognition and original leaf diagnostic/template provenance. Preprocessing
differentials preserve strict `A` versus modern `A ` across the include edge.
Refusal tests pin unsupported graph shapes. Cancellation is swept into dependency
preprocessing; a shared budget at principal-only and total-minus-one work fails,
while the exact total succeeds. No partial program is returned on those failures.
These are preprocessing charge boundaries, not complete compilation accounting.

ADR-0004: the 312-line private include-chain owner keeps bounded graph orchestration
and its four tests out of the instruction reference. It delegates composition
without modifying the production module compiler or extending its responsibilities.
The experiment directory now has three directly related files; no new crate,
public API, authority, or production policy is introduced.

Still open: ordinary mixed declarations, named/matched templates, imports and
precedence, resource-resolution integration, local-version scope, expression
errors, complete compile budgets and profile-qualified compiled-artifact reuse.
This establishes only a supplied transitive include-chain witness, not general
module-graph propagation or additional conformance credit.

Validation: `scripts/verify.ps1` passes with 2,012 core tests (63 ignored),
native 22, WASM adapter 4 and worker 18; all 36 processor-edition probes are
included. The official WASM adapter build check and diff whitespace check pass.
This tranche changes only the test-only reference and records; OASIS was not
replayed again. The previous 2,501 / 2,498 preprocessing-lane observations remain
the last measured baselines, not experimental-edition results.

## Compiled artifact lifetime and isolation

Three additional tests bring the reference set to 39. Both coherent edition
witnesses compile the same supplied include graph concurrently. Parsed modules
and preprocessing views are then dropped; four scoped execution threads reuse
the two programs 32 times each against a separate source, preserving strict
`A` versus modern `A ` output. This is deterministic isolation/lifetime evidence,
not a throughput benchmark.

A second test retains a successfully compiled legacy fallback program while a
modern compilation of the same documents fails at the leaf. It then reparses
different stylesheet bytes under the same logical leaf URI and compiles a new
modern program. The old program remains equal to its saved value; after dropping
all stylesheet documents, alternating old/new execution preserves their distinct
results. Neither failed compilation nor resource-name reuse mutates prior state.

The existing include linker rejects both directions of mixed coherent-preset
artifacts with `FXST1058`, classified unsupported at the principal include
location. **This is a preprocessing-stamp check, not processor-edition identity.**
Because the current coherent presets have different preprocessing stamps, that
check alone cannot prove safe arbitrary edition-qualified reuse. The bounded
chain still accepts documents and recompiles under one edition, not caller-
supplied artifacts. No processor stamp, cache key or reusable module API has been
selected; explicitly testing a separate edition-qualified candidate remains open.

ADR-0004: artifact lifetime and reuse tests have their own private subject file
under the include-chain experiment, rather than expanding instruction lowering.
Only test code changes in this tranche; production compiler/linker/runtime
ownership and all public contracts remain unchanged.

Validation: full `scripts/verify.ps1` passes with 2,015 core tests (63 ignored),
native 22, WASM adapter 4 and worker 18. All 39 processor-edition probes pass;
official WASM and diff checks pass. No OASIS replay or new conformance credit is
claimed for this test-only isolation tranche.

## Independently qualified leaf candidate

A test-only `CompiledLeaf` candidate retains the processor-edition witness and
preprocessing profile separately from its ordinary compiled program. Construction
performs bounded compilation under those inputs; there is no raw-program wrapping
or relabel method. Linking consumes one leaf, checks both dimensions before
preprocessing/merging the parent, and delegates assembly to the existing linker.
The result is an ordinary final program, not another reusable qualified leaf.
This is deliberately a two-module experiment, not a cache or transitive reuse API.

Three tests bring the focused set to 42. The 4-by-4 profile matrix admits exactly
matching pairs and rejects mismatches at the original include location with
test-only `FXAR0031`/unsupported, before parent XDM preprocessing charges. This
includes equal preprocessing with different processor editions and equal editions
with different preprocessing. Cross-combinations are differential controls only;
they are not proposed supported profiles.

A legacy fallback stylesheet compiled with modern preprocessing cannot be
reinterpreted as a modern processor artifact: modern direct compilation refuses
the unknown instruction, and the qualification gate refuses the legacy artifact.
Matching legacy linking still executes after the parsed leaf has been dropped.
Wrong resource identity, repeated identity, extra parent declarations and link
preprocessing cancellation remain refused without returning a partial program;
fresh compilation/linking succeeds afterward.

ADR-0004: the 223-line candidate and its tests have a named private owner beneath
the include experiment. Existing exact-edge preflight is shared with the supplied
chain; no second include merger or production profile flag is introduced. The
original coherent-preset module entry delegates to the same bounded lowerer;
explicit preprocessing selection exists only for these test controls.

This demonstrates a possible qualification mechanism, not its acceptance.
General artifact reuse, semantic-policy versioning, imports, local version scope,
complete compile accounting and performance remain open. Production behavior and
corpus credit do not change.

Validation: all 42 focused probes pass within `scripts/verify.ps1`: 2,018 core
tests (63 ignored), native 22, WASM adapter 4 and worker 18. Formatting, Clippy,
documentation, links and corpus integrity gates pass, as do the official WASM
adapter and diff checks. OASIS was not replayed for this test-only candidate;
no experimental-edition conformance result is claimed.

## Bounded transitive qualification

The candidate now retains qualification across intermediate include composition,
renamed `CompiledModule` to distinguish compiled leaf creation from composition.
The original profile is carried forward, not replaced by the requested profile.
A bounded list of participating logical resource identities rejects re-entry
of any earlier module; the total chain is capped at four modules. Leaf creation
still requires compilation; final ordinary programs cannot be wrapped back into
qualified artifacts. This is not a cache, graph resolver or public module API.

Three further tests bring the focused set to 45. A four-module chain under both
coherent presets produces a program equal to the existing document-chain
reference and consumes equal XDM preprocessing work. Additional tests reject an
edition change at the final hop despite identical preprocessing, a repeated
deep resource identity, and a fifth module. Cancellation after earlier modules
have compiled and their source documents have been dropped returns no final
program. These checks do not establish allocator-exact retention accounting or
complete compilation budgets.

The qualification owner remains a private test-only subject; production module
composition is unchanged. Imports, branching graphs, local version scope,
broader instruction/expression propagation and semantic-policy revision identity
remain unproved. The four-module bound is an experiment limit, not host policy.

Validation: all 45 focused probes pass within full `scripts/verify.ps1`, with
2,021 core tests (63 ignored), native 22, WASM adapter 4 and worker 18. Official
WASM and diff checks pass. The qualification owner remains cohesive at 372 lines;
no production source-unit responsibility changed. OASIS was not replayed for
this test-only tranche and no new conformance credit is claimed.

## Qualified single-import edges

The qualified candidate now permits a parent containing exactly one import,
using the existing single-import program composer. Include and import preflight
share the same exact attribute, logical href and one-declaration checks, but
the supplied-document include-only reference remains include-only. Both routes
retain the same qualification, repeated-identity and four-module bounds. No
production compiler code changes in this tranche.

Three tests bring the focused set to 48. Two import edges followed by an include
retain the leaf's compiled fallback, source provenance and processor edition
across differently declared parent versions. The ordinary root template becomes
a matched document rule at import precedence -2 via the existing composer;
the outer include preserves that precedence and runtime output. A single-import
comparison proves full-program and preprocessing-work equality against direct
existing composition under both coherent presets, including strict `A` versus
modern `A ` text behavior.

An equal-preprocessing/different-edition import fails qualification at the
original import location. Wrong edge kind, extra declarations, ignored-attribute
attempts and preprocessing cancellation remain rejected; fresh linking succeeds
after cancellation. Competing parent/imported rules, `apply-imports`, branching
graphs and general template dispatch are explicitly not admitted by this probe.

ADR-0004: the 174-line import test subject is separate from qualification and
instruction lowering. Precedence rebasing and merge semantics remain owned by
the existing module compiler; the candidate adds no alternate linker, authority,
public interface or selected production policy.

Validation: all 48 focused probes pass within full `scripts/verify.ps1`, with
2,024 core tests (63 ignored), native 22, WASM adapter 4 and worker 18. Official
WASM and diff checks pass. No OASIS replay or conformance gain is claimed for
this test-only import tranche.

## Semantic audit and decision checkpoint

Normative recheck on 2026-10-16 found a correction to the earlier reference:
modern unknown instructions without direct fallback must fail statically, even
under future-version processing. Legacy deferral must not be reused there.
The modern probe now rejects that shape before the shared fallback lowerer;
existing conditional/choice tests were corrected, not counted as new passes.
One original-module test pins dormant/reached legacy versus modern timing.
See [XSLT 3.0 section 3.10](https://www.w3.org/TR/xslt-30/#forwards).

Local-version scope is also edition-specific. XSLT 1.0 describes module version
and literal-result `xsl:version` scope; modern standard attributes also include
unprefixed `version` on XSLT elements. The probe now refuses both lexical forms
on its instruction paths, with a regression test, rather than letting the shared
attribute validator ignore an unimplemented local scope. This is conservative
probe eligibility, not implemented local-version semantics. See
[XSLT 1.0 section 2.5](https://www.w3.org/TR/xslt-10/#forwards) and
[XSLT 3.0 section 3.4](https://www.w3.org/TR/xslt-30/#standard-attributes).

XSLT 1.0 forward-compatible expression recovery distinguishes syntax errors,
unknown unprefixed functions and call errors from their actual evaluation.
Processor-version reporting describes the processor, not the stylesheet's
declaration. These requirements need integration beyond current literal/boolean
probe witnesses. [XSLT 1.0 sections 2.5 and 12.4](https://www.w3.org/TR/xslt-10/#misc-func)

### Implementation-owner findings

- Production sequence compilation currently catches selected expression error
  codes under declaration-derived compatibility helpers. That is not yet an
  edition-aware expression policy, and catching an entire instruction is not
  proof of function-call-level error timing. Keep expression integration open.
- The introspection probe has only string/boolean witnesses. Its `"1"` and
  `"3.0"` outputs are not proof of edition-correct atomic typing or truthful
  supported-processor reporting. Availability outside explicitly proved entries
  remains refused, not silently false.
- The earlier modern no-fallback timing observations are superseded by this
  correction. They demonstrated shared execution machinery, not modern semantic
  parity. Production behavior and original OASIS observations were not altered.
- Fifty focused probes now cover the bounded experiment. General local scope,
  expression recovery and a coherent capability/reporting catalog are still
  implementation obligations, not completed coverage.

### Recommendation for the maintainer

The ownership direction is ready for a narrow architectural decision; further
include/import variations are not prerequisites. Select one immutable graph-
scoped processor policy, separate from preprocessing and scoped declared-version
context. The policy must own recognition and error-timing decisions as well as
introspection meaning, using shared lowering/runtime. Preserve both processor
and preprocessing qualification when composing compiled artifacts.

Do not accept the experimental Rust wrapper, diagnostic code, version strings,
four-module limit or lexical witnesses as product contracts. Accepting the
direction would not promote the prototype or close its implementation gates.
Before production integration: implement edition-specific local scope and
lexical interpretation; pin dormant/reached expression errors; reconcile
recognition with actual availability and typed truthful reporting; replay the
unchanged fallback corpus and both full baselines; measure compile/warm costs.
No general error-code catch or partial capability table may substitute for those
gates. Public selectors, defaults and conformance claims remain deferred.

Validation: all 50 focused probes pass. Full `scripts/verify.ps1` passes,
including 2,026 core tests (63 ignored), native 22, WASM adapter 4 and worker 18,
formatting, Clippy, documentation, Markdown links and conformance checks.
The official `wasm32-unknown-unknown` adapter build check and `git diff --check`
also pass. No OASIS replay or new conformance credit is claimed; this tranche
changes the test-only reference and records the decision checkpoint.
