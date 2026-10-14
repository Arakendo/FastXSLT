# AR-0029: Stylesheet Preprocessing Edition and Legacy Whitespace

| Field | Value |
| --- | --- |
| Status | Accepted through ADR-0023; integration pending |
| Opened | 2026-10-07 |
| Last reviewed | 2026-10-08 |
| Scope | Stylesheet preprocessing, compatibility selection and corpus accounting |
| Trigger | One Microsoft mismatch and four Lotus expectations require different comment-separated whitespace behavior |
| Related ADRs | ADR-0001, ADR-0004, ADR-0007, ADR-0012, [ADR-0023](../ADR/ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md) |
| Related evidence | [Roadmap](../Plans/roadmap.md), [SDD](../Specifications/FastXSLT%20Software%20Design%20Document.md) |

## Architectural question

How should FastXSLT distinguish XSLT 1.0 stylesheet preprocessing from modern
stylesheet preprocessing, without equating local expression compatibility with
the standards edition of an entire stylesheet tree or selecting behavior to
match an individual archived fixture?

## Trigger and evidence

The verified OASIS CD04 baseline is 2,499 / 3,173 exact XML comparisons (78.76%).
`Microsoft/XSLTFunctions_TestIdFuncInComplexStruct#1` executes the expected ID
selections and template matches, but preserves whitespace-only stylesheet text
after comments. Those extra characters cause its exact comparison to fail.
This is not evidence of broken ID indexing or ID-rooted template matching.

The standards have a real preprocessing distinction:

- [XSLT 1.0 section 3.4](https://www.w3.org/TR/xslt.html#strip) applies whitespace
  stripping to text nodes in the constructed stylesheet tree. A whitespace-only
  node is stripped unless `xsl:text` or effective `xml:space` preserves it.
- [XSLT 3.0 section 4.3](https://www.w3.org/TR/xslt-30/#strip-stylesheet)
  removes comments and processing instructions and merges newly adjacent text
  before stripping whitespace-only text. That can preserve whitespace beside
  a non-whitespace text fragment. Modern preprocessing is not itself an
  expression-level backwards-compatibility switch.

The current private compiler implements the merged-run rule using
`text_run_contains_non_whitespace`. Its hand-authored comment-run regression
also applies that rule to a `version="1.0"` stylesheet. That historical test is
implementation evidence, not authority for the 1.0 standard.

### Withdrawn local-context candidate

A temporary candidate disabled merged-run preservation wherever the existing
`uses_xslt10_compatibility(document, parent)` helper selected 1.0 behavior.
Three focused tests passed: comment/PI boundaries and non-whitespace fragments;
explicit `xsl:text`, `xml:space` and local version context; and exact text-byte
budgets with cancellation. The complete repository gates also passed for that
candidate, with 1,660 core tests (60 ignored) and unchanged adapter test totals.

The full unchanged corpus replay nevertheless decreased exact comparisons from
2,499 to 2,496. It gained the Microsoft ID case but lost:

- `Lotus/axes_axes116#1`;
- `Lotus/axes_axes116#2` (the catalog's second scenario for the same resources);
- `Lotus/idkey_idkey22#1`; and
- `Lotus/whitespace_whitespace21#1`.

Initialized/executed totals remained 2,699 / 2,651; initialization/execution
failures remained 471 / 48; expected-error credits remained 423 / 431. XML
mismatches rose from 14 to 17. Comparator gaps remained four. The candidate
did not change archived bytes, comparison rules or the conserved denominator.

The candidate was withdrawn. Its use of local static version context would
silently settle a broader preprocessing boundary. No candidate engine change
or new focused test is retained; existing engine work is preserved. The
baseline must be replayed after withdrawal rather than assuming restoration.

The restoration replay confirms 2,499 exact comparisons, 2,699 initialized,
2,651 executed, 471 initialization failures, 48 execution failures, 14 XML
mismatches, four comparator gaps and 423 / 431 expected-error credits. Coverage
is unchanged; no candidate gain is credited independently of its losses.
The restored implementation passes all repository gates: 1,657 core tests
(60 ignored), 22 native tests (three ignored), four WASM adapter tests and 18
worker tests, strict Clippy, formatting, documentation, 1,983-file Markdown-link
verification, conformance inventories and the reviewed unsafe surface. The
official WASM target check also passes.

### Independent 1.0 observation

A read-only `System.Xml.Xsl.XslCompiledTransform` probe transforms
`<out>A<!-- ignored --> <z/></out>` to `<out>A<z /></out>`: the separate space
does not survive. For the unchanged Lotus `axes116`, `idkey22` and `whitespace21`
stylesheets, Microsoft's result element text also differs from the archived
`REF_OUT` text. `axes116` drops the initial whitespace-only fragment before
its comment; `whitespace21` drops the trailing whitespace-only fragment after
its second comment. The ID-case difference follows the same comment boundary.
Internal DTD parsing was enabled for the source probe and external XML
resolution was disabled. No source or expected fixture was edited.

This supports a genuine standards/archival-expectation distinction. It does
not prove every archived expectation is wrong, establish a Saxon destination
result, or authorize crediting a mismatching original case.

## Ownership and constraints

- FastXSLT owns stylesheet preprocessing and static semantics; XML adaptation
  preserves comments, PIs, text and original provenance rather than selecting
  an XSLT edition or erasing information globally.
- A standards edition/profile must be explicit at its actual owning boundary.
  Local `xsl:version` expression compatibility must not accidentally become
  a document-preprocessing selector.
- Include/import modules and embedded or literal-result stylesheets need a
  defined rule, not inference from the currently visited instruction.
- Prepared source whitespace views remain governed by ADR-0012. This review
  concerns stylesheet text, not mutable source XDM or parser-side stripping.
- One shared compiler and runtime remain; no legacy interpreter, hidden
  fixture-name switch, public parser type or second semantic engine is admitted.
- Preserve text provenance, diagnostics, `xsl:text`, effective `xml:space`,
  compile retention, budgets, cancellation and host-neutral adapter parity.
- Original OASIS exact comparison remains strict. A reviewed corrected or
  edition-specific oracle must be separately identified and cannot replace
  the original suite's exact numerator.

## Alternatives

1. Retain modern preprocessing for the current prototype, recording explicit
   1.0 compatibility gaps. This conserves the current engine but does not
   establish full XSLT 1.0 behavior merely because a stylesheet says 1.0.
2. Select a deliberate strict-1.0 versus modern preprocessing profile at a
   compilation boundary. Define module and embedded-stylesheet behavior before
   exposing any supported selector. Both feed the same compiler/runtime.
3. Select by principal module version, with a documented module-graph rule.
   This is smaller surface but can conflate a modern processor's compatibility
   mode with executing under the older standards edition.
4. Continue local instruction-version selection. The withdrawn experiment
   demonstrates the risk; this is not the nominated architecture.

Corpus-specific whitespace normalization or dispatch on fixture identity is
not an alternative. It would hide observable output differences.

## Findings and uncertainties

The distinction is reproducible and the local-context candidate is inadequate
as an implicit decision. The maintainer has selected explicit strict-1.0 and
modern preprocessing at the whole compilation-graph boundary. Includes/imports
inherit the profile; local version declarations do not switch it. Integration
must still prove embedded stylesheet ancestry and consistent static/temporary
constructor handling. Independent modern-processor comparison remains useful
evidence; existing compatibility helpers do not replace the explicit profile.

## Disposition

Accepted through
[ADR-0023](../ADR/ADR-0023-compilation-scoped-stylesheet-preprocessing-profiles.md).
Select explicit strict-1.0 and modern preprocessing once per compilation graph,
with one compiler/runtime and initially private selection. Retain the current
modern implementation until integration is verified. Acceptance does not claim
implemented strict support, publish a selector, change original corpus credit
or establish conformance. Ordinary breadth work remains independent.

## Required follow-up

### First implementation checkpoint (2026-10-08)

The private `compile/stylesheet_preprocessing.rs` owner now holds literal-text
run eligibility and effective `xml:space` lookup. Sequence compilation calls it
with an explicit modern token; existing constructor callers share the extracted
`xml:space` helper through the retained import seam. Strict text-node eligibility
is test-only until graph-scoped selection and constructor migration are complete.
This is not yet a canonical preprocessed tree or complete modern rule set.

Six focused controls pass: profile selection independent of local version,
comment/PI boundaries, element-delimited runs, inherited/reset `xml:space`,
explicit preservation and XML whitespace characters, original node/location
conservation, and invalid-space diagnostic provenance. The unchanged executable
modern comment-run regression also passes. These hand-authored controls earn no
corpus credit. Static/global constructor filtering still needs migration; no
strict OASIS or module-graph inheritance result is claimed at this checkpoint.

Workspace verification passes: 1,669 core tests (60 ignored), 22 native tests
(three ignored), four WASM adapter tests and 18 worker tests; formatting, strict
Clippy, documentation, 1,986-file local link verification, pinned conformance
source/inventory checks and the official WASM target check also pass.

The unchanged complete OASIS CD04 modern-baseline replay conserves 2,500 / 3,173
exact comparisons (78.79%), 2,701 initialized, 2,652 executed, 469 initialization
failures, 49 execution failures, 14 mismatches, four comparator gaps and 423 /
431 expected-error credits. This verifies extraction conservation, not strict-
profile conformance or new compatibility credit.

ADR-0004 review: the instruction compiler remains 3,353 lines and the composition
root 3,067; the new text-selection owner is 281 lines including its tests. The
compile directory has 18 direct files. Retain its flat named compiler subjects
at this checkpoint: the new module has one stylesheet-preprocessing responsibility
and depends only on XDM access and existing attribute/diagnostic helpers. It owns
no runtime, source view, host policy, public API or authority. Its extraction
reduces semantic responsibility in the pressured instruction compiler rather
than adding another local-version branch. Revisit directory grouping and parent
coupling when graph-profile plumbing and constructor migration become cohesive.

- [x] Select strict 1.0 preprocessing alongside modern preprocessing as an
  explicit compilation-graph profile, initially private.
- [ ] Compare the minimal probe under an independent modern processor and pin
  original versus edition-specific expectations separately.
- [x] Select one inherited profile for include/import, embedded and literal-
  result stylesheets, independent of local version compatibility.
- [x] Prove private test-gated graph selection and include/import, embedded and
  literal-result inheritance through the shared compiler/runtime.
- [ ] Complete canonical preprocessing/consumer integration before promoting
  strict selection beyond the private test-only graph reference.
- [ ] Audit shared sequence compilation and static/temporary constructors for
  consistent text handling and original diagnostic provenance.
- [ ] Retain exact `xsl:text`, `xml:space`, comment/PI, budget and cancellation
  controls under the chosen profile without changing source-view semantics.
- [ ] Review ADR-0004 compilation/test-owner pressure before integration; replay
  complete OASIS accounting and the modern corpus after each admitted slice.
- [x] Adopt the selected boundary through ADR-0023 and update the SDD.
- [ ] Review any eventual public profile/facade selection separately.

### Graph-reference implementation checkpoint (2026-10-08)

`StylesheetCompileLimits` now carries one private preprocessing token. The
test-only strict selection derives each loaded module's immutable relationship
view before `compile_loaded_graph`; principal, transitive include/import and
fragment-selected modules receive the same selection. Original node payloads,
document origin, source locations and enclosing XML ancestry remain retained.
The XDM filter owns only generic relationships, not stylesheet policy; source
preparation and source whitespace views are unchanged.

Common constructor selection now serves global text, global/local temporary
trees, static local text, comments and PIs as well as sequence text selection.
This exposed a hand-authored global-text regression that expected strict
stripping under the default modern compiler. Its expectation is now explicitly
modern and a paired strict/modern control preserves the distinction. No upstream
stylesheet, source or reference output was changed.

Sixteen focused controls pass, covering mixed local/principal versions,
transitive includes/imports, embedded ancestry, literal-result stylesheets,
global/local/static text constructors, comments/PIs, `xsl:text`, inherited and
reset `xml:space`, original identity/provenance, and exact/one-less XDM work
budgets plus cancellation. Failure never mutates the input document. These are
private integration controls, not corpus conformance credit.

The measurement script accepts `-PreprocessingProfile modern` or
`-PreprocessingProfile strict-xslt10`, selects once for the whole replay, emits
the profile in the report, and restores its harness environment afterward.
The engine does not inspect that environment. The script's default remains
modern during this test-only integration checkpoint.

The complete unchanged CD04 replays report:

| Observation | Modern baseline | Strict graph experiment |
| --- | ---: | ---: |
| Catalog denominator | 3,173 | 3,173 |
| Exact XML comparisons | 2,500 (78.79%) | 2,497 (78.70%) |
| Initialized / executed | 2,701 / 2,652 | 2,701 / 2,652 |
| Initialization / execution failures | 469 / 49 | 469 / 49 |
| XML mismatches / comparator gaps | 14 / 4 | 17 / 4 |
| Expected-error credit / denominator | 423 / 431 | 423 / 431 |

Strict fixes `Microsoft/XSLTFunctions_TestIdFuncInComplexStruct#1`; it loses the
four Lotus comparisons recorded in the withdrawn experiment. Those original
mismatches remain counted. The numerator is not combined across profiles, and
the independent 1.0 observation remains separate evidence rather than a
replacement oracle.

Checkpoint validation: `scripts/verify.ps1` passes, including the reviewed unsafe
surface, formatting, strict Clippy, 1,679 core tests (60 ignored), 22 native tests
(three ignored), four WASM adapter tests, 18 worker tests, documentation and
conformance provenance/inventories. Both complete profile replays pass their
conserved-ledger checks. The official WASM target also builds; this does not
expose strict profile selection through its adapter.

Remaining gates are explicit: audit residual optimized constructor and
parameter paths, finish the canonical modern instruction-specific preprocessing
rules, integrate real compilation control into graph preprocessing (the graph
reference at this checkpoint supplied an unbounded control, while its derived-view
controls test bounds directly), and validate compiled-profile confinement before
production promotion. Private precompiled-module composition must not become a
way to mix profiles. No public Rust/.NET/WASM selector or protocol field is added.

ADR-0004 continuation: instruction/compiler composition units remain 3,353 /
3,065 lines. The preprocessing owner is 446 lines including tests; graph
orchestration is 1,055, its named child test owner 132, and generic XDM storage
1,391. Retain these named seams at this checkpoint. Graph orchestration calls
the compiler-owned policy; XDM has no upward stylesheet dependency. The runtime
directory's 54 direct files remain density pressure, but this test family is
grouped under `resource_compiler/` rather than adding another unrelated flat
sibling. Revisit extraction at canonical-stage promotion, not by manufacturing
generic provider or host-policy abstractions.

### Controlled graph and shortcut audit checkpoint (2026-10-08)

Strict graph preprocessing now accepts a supplied `InvocationControl`, shared
across all loaded modules. The test-only corpus workbench supplies its XDM work
ceiling instead of manufacturing an unbounded preprocessing control. Compilation
preprocessing and source preparation have independent phase budgets. Graph
loading and lowering are not metered by this control; existing XML/dependency
ceilings remain in place. This closes the graph-reference control plumbing gap,
not the broader bounded-compilation promotion gate.

Twenty focused controls pass. The four additions prove aggregate exact/one-less
graph budgets, cancellation swept across charge calls in principal and included
modules, real workbench budget projection, and preserved local constructor text.
Control failures identify the original failing module-root span and have no
transform request identity. The span identifies the preprocessing phase's
module, not the precise allocation or charge site. A partially processed graph
publishes no compiled program; subsequent compilation against the unchanged
sealed snapshot succeeds.

The residual audit found local `value-of` and `for-each` text-tree shortcuts
whose structural child filters could discard `xml:space="preserve"` text.
Eligibility now uses common constructor selection, with preserved text falling
through to complete shared sequence compilation. Static element-only local
constructor eligibility uses that same selection. Structural validation filters
have not been globally replaced; typed constructors and parameter consumers
still need their own audit. No public selector, second runtime, fixture change
or resource authority is introduced.

ADR-0004 review: the graph composition owner is 1,072 lines, its named profile
test child 292, and the instruction compiler remains 3,353. The refactor reduces
the resolver/compiler call from seven scalar arguments to a limits bundle and
phase control; tests remain under the existing named `resource_compiler/`
subject. Keep these seams for this checkpoint, with the already recorded
canonical-stage decomposition review still required before promotion.

The complete unchanged CD04 replays remain modern 2,500 / 3,173 (78.79%)
and strict experiment 2,497 / 3,173 (78.70%). Both retain 2,701 initialized,
2,652 executed, 469 initialization failures, 49 execution failures, four
comparator gaps and 423 / 431 expected-error credits; modern/strict mismatches
remain 14 / 17. No constructor control earns archival corpus credit.
`scripts/verify.ps1` passes: formatting, strict Clippy, 1,683 core tests (60
ignored), 22 native tests (three ignored), four WASM adapter tests, 18 worker
tests, documentation, local Markdown links and pinned conformance checks.
The official `wasm32-unknown-unknown` target check and `git diff --check` pass.

### Global and parameter consumer checkpoint (2026-10-08)

The residual consumer audit migrates global temporary text parts and
named-template, conditional and parentless-constructor eligibility to common
constructor selection. Literal `xsl:text` branch inspection ignores comments/PIs
through the shared text compiler instead of reading an unfiltered string value.
Named and matched template-parameter defaults now inspect the same selected
children. Their text-only defaults concatenate those fragments: reading the
original subtree string value after filtering could otherwise restore whitespace
that preprocessing had removed.

Two additional paired-profile controls prove preserved global dynamic text and
named/matched parameter defaults, including comment/PI-separated text,
strip-eligible whitespace-only defaults and `xml:space="preserve"`. Twenty-two
focused controls pass. The complete unchanged CD04 replays remain modern
2,500 / 3,173 (78.79%) and strict experiment 2,497 / 3,173 (78.70%), with all
other counters conserved against the preceding checkpoint. No archived output
or denominator was edited, and these hand-authored controls earn no corpus pass.

Typed-constructor eligibility and structural validation remain explicit audit
work. Canonical modern instruction-specific stripping/merged-text consumption
and compiled-profile confinement are still promotion gates. The composition
root is 3,070 lines and the named graph-profile test child 307; retain the
existing semantic seams under ADR-0004 for this bounded consumer migration.
No new flat file, generic provider, cross-layer profile dependency or public
selector is introduced.

Checkpoint validation passes: `scripts/verify.ps1`, including 1,685 core tests
(60 ignored), 22 native tests (three ignored), four WASM adapter tests and 18
worker tests, plus formatting, strict Clippy, documentation, Markdown links and
pinned conformance checks. The official WASM target and `git diff --check` also
pass. Strict remains a private test-only selection.

### Typed-range eligibility checkpoint (2026-10-09)

The admitted typed integer-range constructor used structural child filters at
the variable, `xsl:for-each` and literal-wrapper levels. Those filters could
discard preserved text before shortcut eligibility was established. All three
now consume common constructor selection. Selected text outside the shortcut's
proven shape remains explicitly unsupported (`FXST1017`), rather than being
silently erased or classified as invalid XML/XSLT. Even where wider typed
semantics could accept a shape, this private shortcut does not claim them.

Two additional paired-profile controls cover preserved text independently at
all three levels, original failure provenance/category and continued execution
of strip-eligible formatting with comments/PIs. Twenty-four focused controls
pass. This is a semantic admission correction, not new conformance credit.
Structural validation filters elsewhere were not indiscriminately replaced.
Canonical modern instruction-specific preprocessing and compiled-profile
confinement remain open, and strict selection stays test-only.

ADR-0004 continuation: the instruction compiler remains 3,353 lines and the
named graph-profile test child is 343. Retain the existing constructor and
profile-test subjects for this three-filter correction; it creates no new
module, crate, public type or cross-layer dependency. Canonical-stage promotion
still requires the previously recorded decomposition review.

Both unchanged complete CD04 replays conserve the preceding ledger: modern
2,500 / 3,173 (78.79%), strict experiment 2,497 / 3,173 (78.70%), with 2,701
initialized, 2,652 executed, 469 initialization failures, 49 execution failures,
four comparator gaps and 423 / 431 expected-error credits. Modern/strict
mismatches remain 14 / 17. `scripts/verify.ps1` passes, including 1,687 core
tests (60 ignored), 22 native tests (three ignored), four WASM adapter tests,
18 worker tests, formatting, strict Clippy, documentation, local links and
pinned conformance checks. The official WASM target also builds.

### Modern visibility reference checkpoint (2026-10-09)

The stylesheet-owned visibility reference now accepts an explicit profile;
the existing strict wrapper retains its semantics and charge behavior. A
test-only modern candidate implements the forced whitespace-removal rules of
[XSLT 3.0 section 4.3](https://www.w3.org/TR/xslt-30/#strip-stylesheet): the
specified 20 parents, whitespace before param/sort/context-item/on-completion
and after catch. These rules override preserved space only for whitespace-only
merged runs; non-whitespace runs and `xsl:text` content survive. Expanded names,
not local-name spelling alone, govern instruction recognition.

Three new controls test the complete parent list, comment/PI-separated sibling
runs, literal-name non-matches, retained provenance/storage, exact/one-less work
and cancellation/recovery. Twenty-seven focused controls pass. Conservative
node work is charged before modern run and neighboring-node scans. This is a
safe reference, not a measured optimization or a promise of complete compiler
resource accounting.

Original fragments remain stored; merged-run eligibility does not physically
replace nodes or invent provenance. Canonical merged-text lowering, empty-element
static validation (`XTSE0260`), production graph integration and compiled-profile
confinement remain explicit gates. No generic XDM stylesheet policy, public
selector, source stripping or authority is added. The modern candidate is not
used by the corpus replay or production compilation yet.

ADR-0004 review: the named preprocessing owner is 678 lines including its
controls. Keep the rules and their differential tests together for this bounded
reference; no new flat file or cross-layer dependency is warranted. The
previously recorded parent/compiler decomposition review still applies before
canonical-stage promotion.

Both complete unchanged CD04 replays remain modern 2,500 / 3,173 (78.79%)
and strict experiment 2,497 / 3,173 (78.70%), with 14 / 17 mismatches and
423 expected-error credits each. `scripts/verify.ps1` passes, including 1,690
core tests (60 ignored), 22 native tests (three ignored), four WASM adapter
tests, 18 worker tests, formatting, strict Clippy, documentation, local links
and pinned conformance checks. The official WASM target and diff checks pass.

### Modern graph-reference checkpoint (2026-10-10)

A separate test-only opt-in now routes the modern visibility reference through
the loaded stylesheet graph before lowering. The existing graph pass accepts
one profile, shared across principal and dependency modules. Strict selection
clears the candidate-modern opt-in; no local-version inference, adapter field,
environment selector or public API is added. Existing production and corpus
modern compilation do not opt in.

The new end-to-end control includes a version-1.0 dependency from a version-3.0
principal. Under inherited preserved space, the modern candidate removes the
comment-separated whitespace before `xsl:param`, while its parameter-default
text survives. The test also observes the unchanged default path's extra
leading whitespace separately, rather than relabeling that path as corrected.
Exact/one-less aggregate budgets preserve the included module's failure
provenance. Twenty-eight focused controls pass.

This establishes graph plumbing and a concrete lowering effect, not canonical
text merging, empty-element validation, compiled-artifact profile confinement
or production promotion. The candidate still retains original text fragments.
ADR-0004 review: graph orchestration is 1,086 lines and its named profile test
child 405; retain this private composition/test seam for the bounded extension.
The full canonical-stage decomposition review remains required.

Validation: all local verification gates pass, including 1,691 core tests
(60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and 18 worker
tests, strict Clippy, documentation, local links and pinned conformance checks.
The official `wasm32-unknown-unknown` build check and diff checks also pass.
The unchanged original OASIS lanes replay at 2,500/3,173 modern exact matches
and 2,497/3,173 strict exact matches; neither replay enables the new modern
graph-reference candidate. No corpus credit is added for this checkpoint.

### Modern empty-content validation checkpoint (2026-10-10)

The test-only modern graph reference now applies the `XTSE0260` rule after its
comment/PI and whitespace eligibility decisions. The reviewed unconditionally
empty subset is `include`, `import`, `output`, `strip-space`,
`preserve-space`, `decimal-format`, `namespace-alias`, `output-character`,
`mode`, `context-item`, `copy-of` and `number`, qualified by the XSLT namespace.
Conditional content models remain instruction-validator responsibilities; this
table is not claimed as a complete language content-model inventory.
In particular, modern `xsl:key` permits a sequence constructor when `use` is
absent; it is deliberately excluded and a reference control pins that boundary.
The engine's existing narrower key lowering does not redefine the modern
content model.

Whitespace removed by preprocessing does not cause the error. Retained
non-whitespace text, child elements and preserved whitespace do, at the original
owning element location. Modern forced stripping still takes precedence, so
`xsl:mode` formatting removed by that rule remains acceptable. Same-spelled
literal/foreign elements do not acquire XSLT validation. Strict preprocessing
does not inherit this modern static-validation pass.

Thirty-one focused controls pass. An included version-1.0 module under the
modern candidate reports the exact original `xsl:output` location and no
transform request identity. Exact failure-work capacity reaches `XTSE0260`;
one-less capacity fails earlier with `FXCT0002`, and injected cancellation
before validation reports `FXCT0001`. Failed preprocessing leaves the sealed
snapshot reusable by the separate unchanged compilation lane. The cancellation
fault counts charge calls, independently of aggregate charged work units.

ADR-0004 review: keep the bounded rules with their stylesheet-preprocessing
owner and the end-to-end proof in the existing graph-profile test child. No
new sibling file, XDM policy field, public selector or second compiler is added.
Canonical merged-text lowering and compiled-profile confinement remain open.

### Template-argument consumer audit (2026-10-10)

The residual audit found `compile_content_argument_value` still selecting
`meaningful_children` and reading the unfiltered subtree string value for its
text-only branch. A paired-profile regression first failed under the unchanged
modern lane: a strip-eligible whitespace-only argument produced two spaces
instead of empty text. The compiler now uses shared constructor selection and
assembles only selected fragments. Preserved whitespace around a dynamic
`xsl:value-of` is visible to eligibility and uses the complete shared sequence
fallback rather than being discarded by the specialized constructor.

This is shared compiler behavior, not activation of the candidate modern graph
pass. Profile selection stays compilation-scoped and private. The focused
family now contains 32 controls; original corpus expectations remain unchanged.
ADR-0004: retain the existing template-invocation compiler's argument-value
responsibility for this bounded filter/read correction; it acquires no runtime
state, acquisition authority, public API or new cross-layer policy dependency.

Final checkpoint validation: 32 focused controls and all repository gates pass,
including 1,695 core tests (60 ignored), 22 native tests (3 ignored), 4 WASM
adapter tests, 18 worker tests, formatting, strict Clippy, documentation, local
Markdown links and pinned conformance checks. The official WASM target and diff
checks pass. Full original OASIS replays remain 2,500/3,173 modern and
2,497/3,173 strict; the template-argument correction earns no additional
archival exact pass. The modern graph candidate remains test-only and is not
enabled by either of these corpus replays.

ADR-0004 size checkpoint: the preprocessing owner is 816 physical lines,
template-invocation compilation 1,678 and the named graph-profile test child
489. Retain their existing private responsibilities for these bounded repairs;
review canonical-stage ownership/decomposition before the larger merging and
profile-confinement integration.

### Completed-artifact confinement checkpoint (2026-10-10)

Completed resource-graph programs now carry private compilation provenance:
modern baseline, strict reference or modern reference. Separating the two
modern stages prevents the candidate's partial preprocessing implementation
from being treated as interchangeable with the unchanged baseline. The stamp
is stylesheet-derived static metadata, independent of local expression/version
compatibility; clones and equality retain it. It is not a public selector,
semantic inspection field, cache key guarantee or ABI/wire representation.

The existing retained-program assembly paths reject foreign stamps with
`FXST1058 / unsupported` at the original dependency declaration: single/multiple
includes, single/multiple imports, and mixed import/include composition. Tests
put both foreign reference stages at each position, retain baseline reuse as
the success control, and prove equal declared version or otherwise equal modern
template payload does not erase provenance. A strict selection also clears a
previous candidate-modern opt-in.

There is no compiled-program cache to partition. Fresh module lowering in a
resource graph still creates local baseline-labelled intermediate programs;
the graph's selected preprocessing has already been applied to its immutable
views, and only the fully assembled/finalized artifact is stamped as that graph
reference. Intermediate-module context threading and canonical-stage promotion
remain to be audited. This checkpoint guards reuse of completed artifacts; it
does not claim those remaining integration gates are finished.

### Canonical text-run reference checkpoint (2026-10-10)

A private test-only child-content reference applies the selected visibility
profile, then coalesces surviving adjacent text into logical runs. Runs retain
ordered borrowed fragments with each original node, value and source location.
The reference does not copy text payload, replace XDM node identities, mutate
the source, fabricate a joined span, or store invocation state in compiled or
prepared artifacts. Element boundaries end runs. Strict stripping occurs before
coalescing; modern merged-run eligibility and forced stripping occur before
retention. A run is not claimed as a new externally observable XDM text node.

The focused family now has 38 controls. New controls pin strict/modern fragment
differences, `xsl:text`, Unicode/NBSP, source-payload and location borrowing,
forced-stripped whitespace not returning, exact/one-less charged work, partial
retention failure and cancellation during coalescing after visibility succeeds.
Both failure paths leave the original document reusable. Charges cover traversal
and retained logical slots before allocation; they are not allocator/RSS byte
accounting. The reference is not yet consumed by graph lowering or the original
corpus lanes, and earns no additional conformance credit.

ADR-0004 review: keep module-reuse validation within the existing private module
composition owner (1,729 lines), and declarative compilation provenance within
the static semantic-program model (2,275 lines). The latter remains a cohesive
type model, not an execution or host-policy owner; its small marker does not
justify splitting unrelated declarations during this checkpoint. Canonical
content has a named private child under the preprocessing subject (272 lines),
rather than another sibling in the dense compilation directory. The graph
composition owner is 1,098 lines and its named profile controls 621. No new
public layer, crate, generic provider or upward XDM-to-compiler dependency is
introduced. Revisit ownership before canonical graph promotion.

Validation: 38 focused controls and all repository gates pass, including 1,701
core tests (60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and
18 worker tests, formatting, strict Clippy, docs, links and pinned conformance
checks. The official `wasm32-unknown-unknown` build and diff checks pass. Full
original OASIS replays conserve 2,500/3,173 modern exact matches and
2,497/3,173 strict exact matches, with 423/431 expected-error credits in each
lane. Neither replay uses the candidate-modern or canonical-content reference.
No new archival pass is credited.

### Canonical position-provenance checkpoint (2026-10-10)

The private canonical reference now resolves decoded logical-run positions to
an original fragment, retaining its original node/location and a fragment-local
decoded byte offset. UTF-8 interior bytes and out-of-range offsets are rejected.
At a boundary the following nonempty fragment owns the position; EOF belongs to
the final fragment. No concatenation or replacement source span is required.

Entity-reference and CDATA controls explicitly preserve the distinction between
decoded offsets and raw XML byte coordinates. Adding a decoded offset to an
original span would be unsound, so this reference supplies a fragment anchor,
not a fabricated exact raw-byte diagnostic. A finer map would require evidence
from XML adaptation; it is not inferred from the fragment's string length.

Four additional controls bring the preprocessing family to 42. They cover
multibyte characters, boundaries/EOF, very large invalid offsets, profile-specific
stripping, original location borrowing, entity/CDATA anchors, exact/one-less
lookup work, mid-lookup cancellation and reuse. Lookup charges each fragment
visit and retains no new buffer. Production and corpus selection are unchanged;
no archival pass is credited by these hand-authored controls.

The next consumer seam is literal versus expanded text lowering. Literal text
may retain fragment-wise emission and provenance, but whole-run expression
recognition must not independently parse comment-separated fragments or pretend
that their source spans are contiguous. Canonical graph lowering and explicit
intermediate-module profile context remain open before promotion.

Validation: all 42 focused controls and repository gates pass, including 1,705
core tests (60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and
18 worker tests. Formatting, strict Clippy, documentation, local links, pinned
conformance checks, the official WASM target check and diff checks pass. The
canonical module is 461 lines including its eight controls, within its existing
named preprocessing responsibility. No original corpus replay is needed to credit
this test-only change: the last measured modern/strict exact numerators remain
2,500 and 2,497 respectively, and neither lane uses this reference.

### Canonical sequence-lowering reference checkpoint (2026-10-10)

The canonical-content reference now has a first vertical lowering consumer.
Shared baseline element dispatch was separated into one private function; the
canonical traversal calls that same owner rather than copying instruction,
extension/fallback, local-binding or forward-compatible failure logic. Ordinary
literal runs emit the original fragment instructions, retaining each original
location and the same runtime instruction/text charges as baseline lowering.
No merged text identity or fabricated span is introduced.

Nine controls bring the focused family to 51. Differential checks compare
complete instruction values under both profiles for literal/Unicode content,
excluded elements, initial/local bindings, recursive literal and conditional
constructors, fallback handling and exact original compiler failures. A compiled
artifact executes through the shared runtime after the original stylesheet and
effective view are dropped. Exact/one-less preprocessing work, cancellation and
subsequent reuse remain conserved. The preprocessing control meters visibility
and canonical traversal, not general lowering CPU or instruction allocation.

Split expanded-text runs return `FXST1066 / unsupported` in the private reference
instead of being treated as separate text-value templates. Single-fragment text
continues through the existing compiler. General whole-run expression parsing
and its provenance integration are not selected by this checkpoint. Recursive
constructors currently use baseline traversal of the same selected view; the
new reference is local sequence integration, not complete graph promotion.
Neither original corpus lane opts into it, and the new controls earn no pass.

ADR-0004 review: the instruction compiler remains 3,361 lines and owns the common
instruction-dispatch semantics; its newly factored dispatch does not add another
semantic owner. The private vertical reference is a named 418-line child with
nine controls, borrowing preprocessing content and calling parent dispatch in
one direction. The instruction-compiler directory now has 20 direct files, all
named instruction/expression subjects. For this bounded checkpoint retain that
layout rather than relocate unrelated compiler families while changing traversal.
Before extending the reference to recursive graph lowering, review a cohesive
sequence-construction subgroup and intermediate profile-context ownership; no
generic helper bucket, public provider or new crate is justified here.

Validation: all 51 focused controls and repository gates pass, including 1,714
core tests (60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and
18 worker tests. Formatting, strict Clippy, documentation, local links, pinned
conformance checks, the official WASM target and diff checks pass. Fresh complete
original OASIS replays conserve modern 2,500/3,173 with 14 mismatches and strict
2,497/3,173 with 17 mismatches; both retain 423/431 expected-error credits. Neither
replay selects the canonical sequence reference. Production default behavior and
archived expectations are unchanged.

### Include-graph compilation context checkpoint (2026-10-10)

A private compiler-owned context now carries the selected preprocessing identity
into each module of an already-preprocessed include-only reference graph. The
principal program is qualified before retained includes merge, so the existing
`FXST1058 / unsupported` check compares actual graph identities rather than
baseline-labelled intermediates. Supplied foreign artifacts are never relabelled
to make them fit. Existing baseline include entry points use the same assembly
owner with baseline context; no second include composer is introduced.

The context is compilation provenance, not a view validator or whitespace
algorithm. The graph owner must apply the selected profile first. It does not
infer version, acquire resources, expose host policy, create a cache or add a
public selector. Principal leaves keep full principal validation; dependency
leaves retain the existing unlinked contract until their parent composes them.

Six new controls bring the focused family to 57. They compare whole compiled
programs and intermediate stamps for nested and branching includes under all
three stages; inject both foreign stages at each of two include positions;
preserve original compiler-failure provenance and reuse; and distinguish missing
principal references from dependency references legitimately supplied by a parent.
An import edge defers the whole shape without mutating effective documents.
Import and mixed graphs still use the original assembler and final reference
stamp; this checkpoint does not claim their intermediate context is integrated.
Canonical recursive/text lowering remains separately pending.

ADR-0004 review: context is a small scalar compilation concern in the existing
854-line preprocessing owner, and retained include assembly stays in its
1,747-line composition owner. The resource adapter is 1,110 lines and retains
orchestration; its new named 350-line graph-context child owns only the bounded
test-reference traversal and six controls. The resource-compiler subject has two
direct files. No runtime state, generic document provider, alternate instruction
compiler, public contract or crate split is introduced.

Validation: the focused family is 57 controls; all repository gates pass,
including 1,720 core tests (60 ignored), 22 native tests (3 ignored), 4 WASM
adapter tests and 18 worker tests. Formatting, strict Clippy, documentation,
local links, pinned conformance checks, the official WASM target and diff checks
pass. Fresh complete original OASIS replays conserve modern 2,500/3,173 with
14 mismatches and strict 2,497/3,173 with 17 mismatches, each with 423/431
expected-error credits. Strict include-only corpus graphs now use explicit
module context; unchanged exact accounting proves conservation, not new credit.
Neither lane selects recursive canonical lowering. No production default,
archival expectation or public contract changed.

### Import/mixed compilation context checkpoint (2026-10-10)

The explicit context now reaches intermediate programs of admitted import and
principal import/include reference trees as well as include-only graphs.
Context-qualified retained composition calls the existing precedence, output,
alias and conflict owners; newly compiled programs are qualified before merge,
while foreign retained artifacts remain rejected at the dependency declaration.
Principal import-order and root dependency-count checks are conserved. Nested
mixed shapes outside the existing slice defer to the original classification
path without publishing a partially composed graph or mutating effective trees.

The first full strict replay exposed one regression: routing flat principal
imports through generic retained-program assembly bypassed document-aware
namespace-alias precedence and made
`Microsoft/Namespace-alias_NSAlias_In_Import#1` disagree with its original result.
The generic path was not retained for that shape. Flat principal imports now
use the existing document-aware compiler with the same explicit context;
principal aliases and visibility declarations still participate before imported
module lowering. Context qualification does not reconstruct already-lowered
namespace semantics. A focused conflicting-alias control pins complete compiled
program parity under all three preprocessing stages.

Five additional controls bring the focused family to 62. Import/mixed programs
and intermediate stamps are compared against the baseline assembly; foreign
artifacts are injected at every retained position; original import-order and
dependency-count failures and unsupported nested mixed shapes are conserved.
The earlier include-to-import deferral control now proves admitted composition
and immutable graph reuse. Recursive canonical lowering and whole-run expanded
text remain separate promotion gates. Production defaults stay unchanged.

ADR-0004 review: composition remains in its named 1,811-line owner rather than
duplicating precedence semantics. The private graph-context child is 657 lines,
including 11 controls, and owns reference traversal rather than instruction
lowering or resource acquisition. The 854-line preprocessing owner retains the
scalar context definition; no public selector, provider, host policy or crate
boundary is introduced.

Validation: all 62 focused controls and repository gates pass, including 1,725
core tests (60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and
18 worker tests. Formatting, strict Clippy, documentation, local links, pinned
conformance checks, the official WASM target and diff checks pass. Fresh full
original OASIS replays restore and conserve modern 2,500/3,173 with 14 mismatches
and strict 2,497/3,173 with 17 mismatches; each retains 423/431 expected-error
credits. Neither lane selects recursive canonical lowering, and no archived
expectation or production default changed. These controls earn no new corpus
credit.

### Whole-run expanded-text reference checkpoint (2026-10-10)

The canonical sequence reference now lowers split text for the existing narrow
whole-expression `expand-text` form through the shared expression compiler.
Baseline literal lowering was factored into node selection and value lowering;
the baseline recognizer and successful instruction semantics are unchanged.
The reference joins only surviving fragments after edition selection and only
when no fragment is excluded. General text-value-template syntax remains outside
this tranche; unsupported and partially excluded expanded runs retain explicit
`FXST1066` rather than interpreting fragments independently.

Copied decoded bytes are charged conservatively to the private preprocessing
XDM work domain before buffer growth. This is not runtime result-text charging,
allocator-exact accounting or a new public compilation budget. The logical
expression uses the first original fragment as its diagnostic anchor; no new
XDM identity, merged source span or raw XML offset is manufactured. Shared
expression failures remain unchanged apart from selecting that original anchor.

Four additional controls bring the focused family to 66: comment/PI/CDATA split
expression parity, shared failure provenance, exact/one-less copy budgets and
cancellation after partial copying with immutable reuse, and real runtime
execution after original fragment-owner release. The successful typed expression
uses modern local expression semantics under both preprocessing profiles, proving
that edition selection does not choose its local expression version. Recursive
constructors still use baseline traversal of the selected view; neither corpus
lane nor production defaults select this local canonical reference.

ADR-0004 review: the shared instruction owner is 3,373 lines and retains the
same dispatch/expression responsibilities; its change is a 12-line net factoring
of existing literal-value lowering. The named private canonical sequence child
is 665 lines with 13 controls and still calls that owner in one direction.
Retain the current 20-file instruction subject layout at this bounded checkpoint;
recursive integration must revisit the previously identified sequence-construction
subgroup rather than add a second constructor compiler or generic context.

Validation: all 66 focused controls and repository gates pass, including 1,729
core tests (60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and
18 worker tests. Formatting, strict Clippy, documentation, local links, pinned
conformance checks, the official WASM target and diff checks pass. Fresh complete
original replays remain modern 2,500/3,173 with 14 mismatches and strict
2,497/3,173 with 17 mismatches, each with 423/431 expected-error credits.
This private reference earns no corpus credit and changes no archived output.

### Recursive literal-constructor reference checkpoint (2026-10-10)

Canonical traversal now recurses through nested literal-result element bodies.
One profile-selected visibility view is created at entry and reused at every
literal depth; recursive calls coalesce its surviving children rather than
reselecting an edition or creating another view. The existing literal owner
still validates control/extension attributes and namespace exclusions, builds
static/dynamic attributes and returns computed-attribute body exclusions.
Only body traversal changes in the private reference.

The shared element dispatcher accepts a private literal-compilation function;
XSLT instruction dispatch, lexical variable updates, extension fallbacks and
forward-compatible failures remain in their existing owner. Production supplies
its original literal compiler. The canonical reference supplies the same static
literal construction followed by canonical body traversal. This is a statically
dispatched private hook, not a public provider, second instruction compiler or
new runtime backend. XSLT instruction bodies, attribute-content compilation and
static/temporary constructor paths remain on baseline selected-view traversal;
this checkpoint does not claim full recursive integration.

Four additional controls bring the focused family to 70. Complete instruction
parity covers literal and computed attributes, exclusions, inherited/reset space
and explicit text. Failure controls preserve shared validation order and original
nested locations. Exact/one-less aggregate traversal work, cancellation after
entering a nested body and successful immutable view reuse are tested. A split
expanded expression nested two literal levels executes through the ordinary
runtime after all original stylesheet/fragment owners are dropped, under both
preprocessing profiles. No corpus lane opts into this reference.

ADR-0004 review: the instruction owner is 3,406 lines, with its existing static
literal construction separated from body traversal and dispatch still shared.
The canonical sequence child is 877 lines including 17 controls; it owns only
reference traversal, invoking existing static/dispatch owners in one direction.
The 20-file directory was revisited for a sequence-construction subgroup. Retain
the named local reference for this literal-only checkpoint: moving unrelated
instruction/expression families would not clarify this two-owner seam. Before
extending the hook through general XSLT instruction bodies, review a cohesive
sequence-construction owner rather than grow a broad mutable compiler context.
There is no added production dispatch allocation, public surface or crate split;
no build-time or execution-speed improvement is claimed.

Validation: all 70 focused controls and repository gates pass, including 1,733
core tests (60 ignored), 22 native tests (3 ignored), 4 WASM adapter tests and
18 worker tests. Formatting, strict Clippy, documentation, local links, pinned
conformance checks, the official WASM target and diff checks pass. Complete
original OASIS replays conserve modern 2,500/3,173 with 14 mismatches and strict
2,497/3,173 with 17 mismatches, each with 423/431 expected-error credits.
Neither profile selects this canonical reference in corpus measurement; no
archived expectation, production default or exact credit changes.

## Reopening triggers

Independent standards evidence falsifies the selected edition distinction;
integration exposes an incompatible graph/ancestry ownership requirement; or a
named consumer requires public profile selection beyond the private decision.

## Review history

- 2026-10-07 -- Opened as Incubating after a candidate gained one case but lost
  four. Candidate withdrawn; original accounting and modern behavior conserved.
- 2026-10-08 -- Revisited for a maintainer decision; accepted through ADR-0023.
  Explicit strict/modern compilation-graph profiles selected. Implementation and
  corpus replay remain pending; historical evidence above is unchanged.
- 2026-10-08 -- Private strict graph reference and shared constructor controls
  implemented; complete modern/strict replays conserve separate 2,500 / 2,497
  exact numerators. Production promotion remains gated as recorded above.
- 2026-10-08 -- Supplied aggregate preprocessing controls, phase provenance
  and local constructor shortcut eligibility verified; both corpus numerators
  conserved. Canonical preprocessing and profile confinement remain open.
- 2026-10-09 -- Typed-range constructor eligibility now retains selected text
  in admission decisions; explicit unsupported controls and both complete
  corpus replays pass. Canonical preprocessing/profile confinement remain open.
- 2026-10-10 -- Modern graph-reference empty-content checks and shared
  template-argument text selection verified. All gates and both original corpus
  lanes pass; canonical merging and compiled-profile confinement remain open.
- 2026-10-10 -- Completed-artifact stamps and retained-program reuse checks
  verified; borrowed canonical text-run reference established. All gates and
  original corpus lanes pass. Canonical graph lowering and intermediate-module
  context integration remain pending before promotion.
- 2026-10-10 -- Canonical position lookup and first shared sequence-lowering
  consumer verified. Original fragment instructions/provenance survive owner
  release; split expanded text stays explicitly unsupported in the reference.
  All gates and both full original corpus replays pass without numerator drift.
  Recursive canonical lowering and intermediate graph context remain pending.
- 2026-10-10 -- Explicit private module context integrated for reference
  include-only trees, with branching, retained-artifact rejection and original
  failure-provenance controls. All gates and both original corpus replays pass.
  Import/mixed intermediate context and recursive canonical lowering remain open.
- 2026-10-10 -- Import and principal mixed reference contexts integrated with
  existing graph gates. Full replay caught a flat-import namespace-alias seam;
  document-aware child compilation was preserved and a regression control added.
  Recursive canonical lowering remains open; no production promotion is implied.
- 2026-10-10 -- Narrow whole-run expanded-text reference uses shared expression
  lowering with original-fragment anchors, pre-copy work charging, cancellation
  and runtime owner-release controls. Recursive canonical integration remains open.
- 2026-10-10 -- Nested literal-result bodies now recurse canonically through
  shared static construction and element dispatch. Attribute exclusions,
  failure provenance, aggregate traversal controls and owner-release execution
  are pinned; XSLT/static/temporary recursive integration remains open.
