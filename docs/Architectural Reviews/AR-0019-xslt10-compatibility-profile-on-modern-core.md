# AR-0019: XSLT 1.0 Compatibility Profile on the Modern Core

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-09-04 |
| Last reviewed | 2026-09-04 |
| Scope | Named XSLT 1.0 compatibility, backwards-compatible behavior, and shared modern execution |
| Trigger | A complete local legacy sweep found 366 initial definite unchanged passes and dominant gaps that largely overlap the XSLT 3.0 roadmap |
| Related ADRs | ADR-0002, ADR-0006, ADR-0007, ADR-0012, ADR-0013, ADR-0014 |
| Related reviews | AR-0001, AR-0004, AR-0008, AR-0011, AR-0014 |
| Related evidence | `docs/Evidence/oasis-xslt10-suite-candidate-review-2026-08-25.md`; `docs/Evidence/oasis-xslt10-initial-compatibility-measurement-2026-09-04.md`; `docs/Evidence/oasis-xslt10-static-computed-element-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-location-path-copy-of-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-source-node-kind-copy-of-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-static-element-namespace-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-descendant-match-pattern-repair-2026-09-04.md`; `docs/Evidence/oasis-xslt10-variable-copy-of-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-copy-of-path-union-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-named-processing-instruction-path-tranche-2026-09-04.md` |

## Architectural question

Can FastXSLT complete and eventually advertise a named XSLT 1.0 compatibility
profile while retaining XSLT 3.0/XPath 3.1/XDM 3.1 as its semantic foundation
and using one compiler/runtime, with explicit compatibility behavior only where
the standards genuinely differ?

## Trigger and evidence

ADR-0007 rejected XSLT 1.0 as the engine's primary semantic foundation, not as
a future compatibility profile. It requires separate evidence rather than
inferring legacy support from shared syntax.

The hash-verified local OASIS Committee Draft 04 sweep now reaches all 3,173
catalog cases and proves 366 unchanged standard-operation XML results. The
largest first blockers are broader location paths, ordinary XSLT instructions,
top-level declarations, `xsl:copy-of`, match patterns, imports/includes,
namespace behavior, and serialization. Those capabilities are largely useful
to the modern profile too. A source-node-variable defect found by the suite was
already repaired in the shared runtime without adding a legacy execution path.

The same evidence also shows why the profile cannot be declared by arithmetic.
The archival suite has unresolved doubts, duplicate IDs, discretionary choices,
infrastructure gaps, redistribution constraints, incomplete error expectations,
and legacy output encodings. XSLT 1.0 also has behaviors whose relationship to
the modern standards needs explicit treatment, including backwards-compatible
conversions, recovery choices, result tree fragments, and serialization
details.

No evidence yet estimates how many successive blockers remain behind the first
failure in each case, proves the expected-error denominator, or demonstrates a
representative consumer requirement for a formal XSLT 1.0 label.

## Ownership and constraints

- XSLT 3.0, XPath 3.1, and XDM 3.1 remain the engine's semantic foundation under
  ADR-0007. The compatibility effort must not rebuild the core around a 1.0
  node-set or result-tree-fragment representation.
- Shared syntax and semantics should compile to the same typed plan and execute
  through the same runtime, work budgets, cancellation, diagnostics, resource
  authority, prepared input, and serialization boundaries.
- A version-dependent rule must be selected deliberately from stylesheet static
  context. Scattered runtime checks and a second evaluator are not acceptable
  substitutes for an owned compatibility model.
- Unsupported compatibility behavior must remain explicit. A plausible partial
  result is not a pass.
- The OASIS archive remains local-only and hash-verified. FastXSLT owns its
  runner, classifications, comparisons, and evidence, not the upstream bytes.
- Expected-error credit requires an owned phase/category expectation. Any
  failure is not automatically the expected failure.
- DTD/entity, encoding, URI authority, supplemental data, and live resource
  resolution remain governed by their existing security and ownership
  boundaries. Legacy corpus expectations do not grant ambient authority.
- A compatibility label and its exact scope require an accepted ADR and a
  conserved report. This incubating review creates neither.

## Alternatives

### A. Continue only the staged modern profile

The OASIS suite remains useful regression pressure but FastXSLT makes no named
XSLT 1.0 compatibility commitment. This minimizes version-specific machinery,
but leaves a common field workload and a relatively reachable product milestone
unclaimed.

### B. Add a named compatibility profile on the shared modern core

Implement shared language features once. Model only proven version-sensitive
differences in compile-time static context and typed plan selection. Preserve
one runtime wherever execution semantics agree. This is the leading experiment
because current evidence already produced shared-path gains and most dominant
frontiers are also modern-roadmap work.

The risk is underestimating compatibility-specific conversion, recovery,
serialization, namespace, and result-tree-fragment behavior hidden behind the
current first failures.

### C. Build a separate XSLT 1.0 engine/backend

A separate evaluator could mirror legacy concepts directly, but duplicates
semantics, diagnostics, security accounting, optimization, and host parity. It
would violate the project's single-engine direction without evidence that the
shared core cannot express the required behavior.

### D. Treat every `version="1.0"` stylesheet as already supported

This would turn partial shared syntax into a misleading compatibility promise.
The current 366 definite passes out of 2,742 standard cases disprove that
interpretation.

## Findings and uncertainties

The measurement supports continued implementation of shared semantic features
with both OASIS and XSLT30/QT3 evidence. It does not yet show that completing the
legacy profile is a short path, only that it may be a useful path aligned with
the modern engine.

The key unknown is the compatibility delta after shared blockers shrink:

- which XSLT 1.0 conversion and effective-boolean-value rules differ observably
  from the modern typed evaluator;
- whether result tree fragment compatibility can be expressed as a bounded view
  over the existing temporary-tree model;
- which error recovery and discretionary choices FastXSLT should support or
  reject;
- how legacy namespace and serialization rules compose with the selected modern
  editions;
- whether a useful named profile can exclude DTDs, ambient I/O, extension
  functions, and uncommon encodings without misusing the term conformance; and
- whether consumer demand justifies the remaining compatibility-only work.

Implementation and corpus findings will be classified into five lanes:

1. **Shared modern primitives** -- XPath, construction, sorting, keys, numbering,
   patterns, imports, namespaces, and serialization capabilities that strengthen
   both the XSLT 1.0 checkpoint and the staged XSLT 3.0 profile.
2. **Cheap XSLT 1.0 completeness features** -- bounded work that closes a
   meaningful legacy denominator without distorting modern semantics or public
   architecture.
3. **Legacy compatibility semantics** -- conversion, result-tree-fragment,
   conflict, recovery, or other edition-sensitive behavior selected explicitly
   from stylesheet static context.
4. **Host-authorized capabilities** -- `document()`, URI/resource acquisition,
   and extension surfaces that must compose with sealed snapshots and explicit
   host authority rather than recreate ambient legacy behavior.
5. **Historically awkward result behavior** -- especially
   `disable-output-escaping` and legacy serialization details, which require an
   owned result/serialization contract and must not contaminate the clean
   semantic result model.

This classification is a routing rule, not a promise that every item in every
lane will be supported. It makes the compatibility campaign a formal checkpoint
on the modern path while preserving an explicit stop/review point for behavior
that does not belong in the shared engine.

## Disposition

**Under Review.** The roadmap now selects a named XSLT 1.0 compatibility
checkpoint as an intermediate implementation path toward broader XSLT 3.0
coverage. Work is routed through the five lanes above and the shared modern
compiler/runtime remains mandatory. This does not yet select the exact
advertised profile, claim conformance, admit a separate backend, or stabilize a
public version-mode contract.

## Required follow-up

- [x] Build a hash-verified local runner that conserves all 3,173 catalog cases.
- [x] Establish a strict lower-bound pass count without crediting arbitrary
  expected failures.
- [x] Prove at least one legacy-discovered defect can be repaired through the
  shared modern runtime.
- [x] Add a formal XSLT 1.0 compatibility checkpoint and subsequent modern
  expansion milestone to the project roadmap.
- [x] Implement the first shared construction tranche: static unprefixed and
  prefixed `xsl:element` QNames use the modern result-construction path, raising
  definite unchanged passes from 366 to 386 without a legacy backend.
- [x] Expand `xsl:copy-of` through the shared typed location-path evaluator and
  deep-copy path, raising definite unchanged passes from 386 to 395 while
  exposing later execution boundaries rather than crediting them.
- [x] Preserve source attributes, comments, and processing instructions through
  shared result construction, eliminating all 13 `FXRT1002` observations and
  raising definite unchanged passes from 395 to 403.
- [x] Compile literal `xsl:element` namespace URIs into retained namespace
  bindings and inherit the in-scope default namespace for unprefixed computed
  names, raising definite unchanged passes from 403 to 461 while keeping
  namespace AVTs explicit.
- [x] Repair admitted relative descendant match paths so `a//c` accepts a
  nonadjacent `a` ancestor, moving one known mismatch to pass.
- [x] Preserve included matched-template declaration order and stylesheet text
  runs separated by ignored comments, moving two known mismatches to pass and
  raising the strict lower bound to 464.
- [x] Compile static atomic `xsl:copy-of` values through the bounded shared
  result-text path, raising the strict lower bound to 468.
- [x] Copy atomic, source-node, and temporary-tree variables through their
  existing bounded shared representations, raising the strict lower bound to
  476 without adding a legacy result-tree-fragment backend.
- [x] Compile admitted `xsl:copy-of` path unions with charged evaluation,
  document-order normalization, and deduplication, raising the strict lower
  bound to 481 while retaining one later error and one mismatch visibly.
- [x] Retain and evaluate named processing-instruction node tests in the shared
  typed location-path model, raising the strict lower bound to 485.
- [x] Reuse the typed processing-instruction target for named template patterns,
  retain XPath 1.0 literal-star PI selection, and lower no-argument `name()` to
  the shared context-name operation, raising the strict lower bound to 514.
- [x] Lower `local-name()` and `namespace-uri()` context forms to typed shared
  expanded-name operations, raising the strict lower bound to 519 with no new
  mismatch or later failure.
- [x] Normalize context `string()` and `string(.)` to the existing charged `.`
  path and XDM string-value operation, raising the strict lower bound to 521.
- [x] Evaluate context `normalize-space()` over the complete charged XDM string
  value, preserving collapse state across text boundaries and raising the
  strict lower bound to 522.
- [x] Pass the existing invocation-local sequence focus into typed value
  evaluation for `position()` and `last()`, raising the strict lower bound to
  525 while preserving focusless `XPDY0002`.
- [x] Compose `name(..)` with the existing typed singleton parent path, raising
  the strict lower bound to 527 without selecting general multi-node legacy
  conversion behavior.
- [x] Count the complete context XDM string value for no-argument
  `string-length()`, raising the strict lower bound to 529 with Unicode and
  missing-focus behavior preserved.
- [x] Compare source-free XPath numeric literals through typed numeric rather
  than effective-boolean-value semantics, raising the strict lower bound to
  553 with all 24 newly initialized cases passing.
- [x] Compare homogeneous boolean and string literals through their typed
  equality semantics, raising the lower bound to 560 while leaving
  version-sensitive mixed-type coercion unsupported.
- [x] Fold bounded static `concat()` calls into the shared literal-result path,
  raising the lower bound to 567 and passing the unchanged 1,000-argument
  stress case without admitting dynamic concat semantics.
- [x] Fold two-literal `contains()`, `starts-with()`, `substring-before()`, and
  `substring-after()` through typed shared results, raising the lower bound to
  592 with all 25 newly initialized cases passing.
- [x] Fold three-literal `translate()` with codepoint, removal, duplicate, and
  non-recursive replacement semantics, raising the lower bound to 598 with all
  six newly initialized cases passing.
- [x] Fold literal `substring()` over finite positions with XPath rounding and
  Unicode-codepoint semantics, raising the lower bound to 606 with all eight
  newly initialized cases passing while leaving legacy NaN/infinity behavior
  behind the unresolved version-mode boundary.
- [x] Add work-charged inherited `xml:lang` matching shared by value and
  instruction-test expressions, raising the lower bound to 607 without
  admitting neighboring legacy mixed-type coercions.
- [ ] Split the dominant XPath and unsupported-instruction frontiers into
  actionable semantic families and compare them with the XSLT30/QT3 roadmap.
- [ ] Resolve or explicitly classify the 48 known executing comparison
  mismatches.
- [ ] Define expected-error and discretionary/doubts comparison rules.
  - [x] Report doubt-annotated mismatches separately: three of the current 30
    mismatches carry substantive doubts metadata; none is reclassified yet.
- [ ] Prototype at least one genuine version-dependent behavior through
  compile-time static context without a second runtime.
- [ ] Measure pass growth, regression risk, retained state, and hot-path cost as
  shared families land.
- [ ] Obtain consumer evidence before selecting the exact advertised profile or
  compatibility exclusions.
- [ ] If the profile is selected, accept an ADR defining version recognition,
  compatibility semantics, exclusions, diagnostics, and reporting language.

## Reopening triggers

Reopen a deferred or narrowed review if shared modern data-model choices prevent
an important legacy case, compatibility checks leak recurring branches into hot
execution, a representative consumer requires strict XSLT 1.0 behavior, or a
maintained redistributable legacy suite becomes available.

## Review history

- 2026-09-04 -- Opened as Incubating after the first complete local OASIS sweep
  established 366 definite unchanged XML passes and repaired one shared-runtime
  defect.
- 2026-09-04 -- Moved Under Review after the project selected XSLT 1.0 as a
  formal intermediate compatibility checkpoint on the way to broader XSLT 3.0
  coverage, without changing ADR-0007's modern semantic foundation.
- 2026-09-04 -- The first shared construction tranche admitted static
  `xsl:element` QNames and raised the strict lower bound from 366 to 386 passes;
  namespace attributes, dynamic names, and attribute sets remain explicit.
- 2026-09-04 -- Shared node-path `xsl:copy-of` raised the lower bound to 395 and
  moved 95 cases past initialization; 81 of those expose later execution
  boundaries and remain visibly uncredited.
- 2026-09-04 -- Source node-kind copying eliminated all 13 `FXRT1002`
  observations, producing eight more definite passes and preserving five later
  mismatch/error obligations.
- 2026-09-04 -- Static computed-element namespace semantics raised the strict
  lower bound to 461. Reviewing its first nine mismatches found and repaired
  default-namespace inheritance, leaving no net mismatch increase.
- 2026-09-04 -- Descendant match-pattern repair moved unchanged
  `conflictres15` from mismatch to pass and raised the lower bound to 462.
- 2026-09-04 -- Include declaration-position preservation and comment-separated
  text-run handling moved `impincl06` and `whitespace21` from mismatch to pass,
  raising the lower bound to 464.
- 2026-09-04 -- Static string, integer, and boolean `xsl:copy-of` values raised
  the lower bound from 464 to 468 without exposing another mismatch.
- 2026-09-04 -- Variable-valued `xsl:copy-of` reused the shared atomic,
  source-node, and temporary-tree representations and raised the lower bound
  from 468 to 476; all eight newly initialized cases reached definite passes.
- 2026-09-04 -- Bounded `xsl:copy-of` path unions raised the lower bound from
  476 to 481. One newly initialized case remains a later execution failure and
  `copy_copy09` remains an explicitly uncredited, doubt-annotated mismatch.
- 2026-09-04 -- Named processing-instruction tests became a typed shared path
  step and raised the lower bound from 481 to 485; all four newly initialized
  cases reached definite passes.
- 2026-09-04 -- Named PI template patterns, literal-star PI selection, and the
  no-argument `name()` spelling raised the lower bound from 485 to 514. The
  wider frontier also exposes 18 additional mismatches, ten later execution
  failures, and one expected-error unexpected success; all remain uncredited.
- 2026-09-04 -- Typed context `local-name()` and `namespace-uri()` operations
  raised the lower bound from 514 to 519; all five newly initialized cases pass.
- 2026-09-04 -- Context `string()` spellings reused the existing `.` path and
  raised the lower bound from 519 to 521; both newly initialized cases pass.
- 2026-09-04 -- Context `normalize-space()` became a charged streaming
  string-value operation and raised the lower bound from 521 to 522 without a
  new mismatch or later failure.
- 2026-09-04 -- Typed `position()` and `last()` value operations reused the
  runtime's existing sequence focus and raised the lower bound from 522 to 525;
  two additional initialized cases stop at visible later execution boundaries.
- 2026-09-04 -- Typed singleton-parent `name(..)` composition raised the lower
  bound from 525 to 527; both newly initialized cases pass.
- 2026-09-04 -- Typed context `string-length()` raised the lower bound from 527
  to 529; both newly initialized cases pass without a new later boundary.
- 2026-09-04 -- Typed source-free numeric literal comparisons raised the lower
  bound from 529 to 553; all 24 newly initialized cases reach definite passes
  without a new mismatch or runtime failure.
- 2026-09-04 -- Homogeneous boolean and string literal equality raised the
  lower bound from 553 to 560; all seven newly initialized cases pass while
  mixed-type legacy coercion remains explicit.
- 2026-09-04 -- Bounded static `concat()` folding raised the lower bound from
  560 to 567; all seven newly initialized cases pass, including the unchanged
  1,000-argument stress case.
- 2026-09-04 -- Typed static binary string-function folding raised the lower
  bound from 567 to 592; all 25 newly initialized cases pass without a new
  mismatch or runtime failure.
- 2026-09-04 -- Codepoint-correct static `translate()` folding raised the lower
  bound from 592 to 598; all six newly initialized cases pass without a new
  mismatch or runtime failure.
- 2026-09-04 -- Finite-literal static `substring()` folding raised the lower
  bound from 598 to 606; all eight newly initialized cases pass without a new
  mismatch or runtime failure, while version-sensitive NaN/infinity behavior
  remains unsupported.
- 2026-09-04 -- Literal `lang()` with inherited `xml:lang` semantics raised the
  lower bound from 606 to 607; value and instruction-test compilation share one
  work-charged evaluator.
- 2026-09-04 -- The exploratory report identified the then-current two
  mismatches with substantive doubts metadata separately from the other 27;
  the later path-union tranche adds `copy_copy09` as a third doubt-annotated
  mismatch without changing any case disposition.
