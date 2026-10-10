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
