# AR-0019: XSLT 1.0 Compatibility Profile on the Modern Core

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-09-04 |
| Last reviewed | 2026-09-07 |
| Scope | Named XSLT 1.0 compatibility, backwards-compatible behavior, and shared modern execution |
| Trigger | A complete local legacy sweep found 366 initial definite unchanged passes and dominant gaps that largely overlap the XSLT 3.0 roadmap |
| Related ADRs | ADR-0002, ADR-0006, ADR-0007, ADR-0012, ADR-0013, ADR-0014 |
| Related reviews | AR-0001, AR-0004, AR-0008, AR-0011, AR-0014 |
| Related evidence | `docs/Evidence/oasis-xslt10-suite-candidate-review-2026-08-25.md`; `docs/Evidence/oasis-xslt10-initial-compatibility-measurement-2026-09-04.md`; `docs/Evidence/oasis-xslt10-static-computed-element-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-location-path-copy-of-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-source-node-kind-copy-of-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-static-element-namespace-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-descendant-match-pattern-repair-2026-09-04.md`; `docs/Evidence/oasis-xslt10-variable-copy-of-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-copy-of-path-union-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-named-processing-instruction-path-tranche-2026-09-04.md`; `docs/Evidence/oasis-xslt10-exact-rational-path-arithmetic-2026-09-05.md`; `docs/Evidence/oasis-xslt10-mixed-literal-path-arithmetic-2026-09-05.md`; `docs/Evidence/oasis-xslt10-sort-tranche-2026-09-06.md`; `docs/Evidence/oasis-xslt10-path-concat-tranche-2026-09-06.md`; `docs/Evidence/oasis-xslt10-literal-template-argument-tranche-2026-09-06.md`; `docs/Evidence/oasis-xslt10-empty-global-string-semantics-2026-09-06.md`; `docs/Evidence/oasis-xslt10-local-variable-sequence-semantics-2026-09-07.md`; `docs/Evidence/oasis-xslt10-attribute-predicate-and-variable-apply-tranche-2026-09-07.md`; `docs/Evidence/oasis-xslt10-sequence-focus-boolean-comparison-2026-09-07.md`; `docs/Evidence/oasis-xslt10-apply-templates-path-union-2026-09-07.md`; `docs/Evidence/oasis-xslt10-signed-modulo-boolean-conjunction-2026-09-07.md`; `docs/Evidence/oasis-xslt10-focus-value-equality-2026-09-07.md`; `docs/Evidence/oasis-xslt10-count-path-condition-2026-09-07.md`; `docs/Evidence/oasis-xslt10-relational-position-predicates-2026-09-07.md`; `docs/Evidence/oasis-xslt10-relative-last-and-chained-position-predicates-2026-09-07.md`; `docs/Evidence/oasis-xslt10-static-number-position-predicate-2026-09-07.md`; `docs/Evidence/oasis-xslt10-focus-relational-conjunction-2026-09-07.md`; `docs/Evidence/oasis-xslt10-position-then-name-predicate-2026-09-07.md` |

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
- [x] Implement a first shared `xsl:sort` slice for source-node
  `xsl:for-each` and `xsl:apply-templates`, raising the strict lower bound from
  808 to 857 while leaving 22 newly exposed mismatches and six later execution
  failures visibly uncredited.
- [x] Implement default single-level and bounded literal/focus-value
  `xsl:number` through shared XDM/focus/result paths, raising the strict lower
  bound from 857 to 863 while leaving one newly exposed mismatch uncredited.
- [x] Add bounded single-token decimal `xsl:number` formatting and charged
  context-item numeric conversion, raising the strict lower bound from 863 to
  871 while leaving two upstream doubt-annotated expectations uncredited.
- [x] Add charged single-level exact-name, any-element, and document `count` /
  `from` patterns, raising the strict lower bound from 871 to 880 with all nine
  newly initialized cases passing.
- [x] Add charged `level="any"` document-order accumulation and `from` reset,
  raising the strict lower bound from 880 to 887 while two newly exposed cases
  remain at the existing HTML-serialization boundary.
- [x] Add charged `level="multiple"` ancestor-lineage numbering with repeated
  decimal-token formatting, raising the strict lower bound from 887 to 893
  while one newly exposed later failure remains uncredited.
- [x] Add static unions of admitted `xsl:number` `count`/`from` atoms, raising
  the strict lower bound from 893 to 906 expected-result matches while the one
  newly visible mismatch remains explicitly uncredited with its upstream doubts
  metadata.
- [x] Add typed `node()`/`@*` number-pattern atoms and charged exact
  attribute-value predicates, raising the strict lower bound from 906 to 912
  with all six newly initialized cases matching exactly.
- [x] Add charged exact/modulo sibling-position predicates and one parent/child
  number-pattern relationship, raising the strict lower bound from 912 to 915
  with all three newly initialized cases matching exactly.
- [x] Compile Latin alphabetic/Roman and multi-token number-format plans plus
  static decimal grouping, raising the strict lower bound from 915 to 947 while
  one newly exposed later execution failure remains uncredited.
- [x] Compile XSLT 1.0 node-set string conversion as a distinct first-node
  location-path operation and preserve source comments through `xsl:copy`,
  raising the strict lower bound from 947 to 962 without weakening modern
  cardinality behavior.
- [x] Expand the typed `format-number()` evaluator through bounded source-free
  operands and static default-decimal pictures, raising the strict lower bound
  from 962 to 983 while preserving exercised invalid-picture errors.
- [x] Correct single-level `xsl:number` `from` boundary composition and isolate
  XPath 1.0 numeric-sort conversion, raising the exact lower bound from 983 to
  986; retain the source-copy serialization-only mismatch without credit.
- [x] Reuse variable EBV across conditions and value construction, isolate
  XPath 1.0 node-set/boolean comparison at compilation, and materialize a
  bounded source-derived temporary value, raising the exact lower bound from
  986 to 996 while retaining disputed temporary-tree expectations as visible
  mismatches.
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
- [x] Add typed principal-source `generate-id()` value and identity-equality
  operations, raising the lower bound to 608 while retaining modern
  zero-or-one cardinality rather than silently selecting legacy first-node
  conversion.
- [x] Compose `normalize-space()` with an admitted location path and the shared
  charged XDM string-value operation, raising the lower bound to 609 while
  keeping multi-node legacy conversion behind version-mode review.
- [x] Compose `local-name()` and `namespace-uri()` with admitted qualified and
  unqualified child paths, raising the lower bound to 613 while retaining the
  shared zero-or-one argument boundary.
- [x] Recognize the XPath-legal whitespace gap before the empty
  `string-length` argument list, raising the lower bound to 614 through the
  existing typed context operation.
- [x] Fold context-independent `floor()`, `ceiling()`, and `round()` through
  checked exact-rational semantics, raising the lower bound to 638 without
  admitting legacy conversion rules or adding runtime dispatch.
- [x] Compose the three integral functions with admitted typed paths, charged
  finite-decimal conversion, and modern zero-or-one cardinality, raising the
  lower bound to 647 without selecting legacy first-node conversion.
- [x] Compose `string()` with admitted static atoms and typed paths, raising the
  lower bound to 653 while retaining the multi-node legacy case as a visible
  modern `XPTY0004` boundary.
- [x] Compose `name()` with admitted typed paths, raising the lower bound to
  657 while retaining namespaced lexical-QName reconstruction as unsupported
  and the multi-node legacy case as a visible modern `XPTY0004` boundary.
- [x] Add expanded-name attribute steps to the qualified path model, raising
  the lower bound to 662 while retaining lexical-prefix reconstruction as an
  explicit boundary.
- [x] Add work-charged `preceding-sibling` and `preceding` axes with correct
  reverse-axis positional semantics, raising the lower bound to 686 through 24
  unchanged passes.
- [x] Add a work-charged `following` axis with context-descendant exclusion and
  forward positional semantics, raising the lower bound to 694 through eight
  unchanged passes.
- [x] Add explicit text, comment, and processing-instruction tests to the
  shared `following` axis, raising the lower bound to 697 while leaving two
  later whitespace-profile failures visible.
- [x] Add work-charged `ancestor` and `ancestor-or-self` axes and repair
  leading-`//` context expansion, raising the lower bound to 718 through 21
  unchanged passes while retaining one multi-node legacy conversion boundary.
- [x] Add explicit text, comment, and processing-instruction tests to the
  shared `self` axis, raising the lower bound to 721 through three unchanged
  passes.
- [x] Add explicit text, comment, and processing-instruction tests to the
  shared `preceding` axis, raising the lower bound to 724 through three
  unchanged composed-path passes.
- [x] Compile doubled AVT braces in static literal-result attributes through
  the ordinary retained text representation, raising the lower bound to 728
  through four unchanged passes while leaving dynamic AVTs unsupported.
- [x] Compose `.` as the shared abbreviated `self::node()` path step, raising
  the lower bound to 729 through unchanged `axes98` without a compatibility
  evaluator.
- [x] Recognize XPath whitespace around the shared `::` axis separator,
  raising the lower bound to 731 through two unchanged attribute-axis passes
  while retaining modern multi-node conversion behavior.
- [x] Canonicalize expanded-axis `attribute::*` and `child::*` match patterns
  to the existing typed wildcards, raising the lower bound to 734 through three
  unchanged passes without a runtime version branch.
- [x] Canonicalize `attribute::node()` to the same typed any-attribute pattern
  and node-test default priority, raising the lower bound to 737 through three
  unchanged passes including conflict-recovery pressure.
- [x] Compile the exact `{.}` AVT to a typed context string-value operation and
  charge source/temporary traversal, raising the lower bound to 738 while two
  line-ending comparison mismatches remain explicitly uncredited.
- [x] Expose checked source-free binary arithmetic only when its exact-rational
  result is integral, raising the lower bound to 747 through nine unchanged
  passes without runtime version dispatch.
- [x] Fold source-free finite `number()` conversion over decimal literals and
  quoted decimal lexical values, raising the lower bound to 749 through two
  unchanged passes without runtime version dispatch.
- [x] Add a typed, work-charged `number()` path operation plus shared
  empty/non-convertible `NaN` behavior, raising the lower bound to 753 through
  four unchanged passes while retaining modern zero-or-one cardinality.
- [x] Compile zero-argument `number()` to the same typed operation with an
  explicit context-item path, raising the lower bound to 756 through three
  unchanged catalog identities without a version branch.
- [x] Fold exact boolean-to-number equalities, raising the lower bound to 758
  through two unchanged cases without claiming general numeric comparison.
- [x] Fold exact valid constant short-circuit expressions, raising the lower
  bound to 760 through two unchanged cases without selecting legacy mixed-type
  coercion.
- [x] Preserve dynamic context for relative boolean name paths, raising the
  lower bound to 762 through two unchanged cases in the shared evaluator.
- [x] Prototype a typed compile-time XPath 1.0 compatibility mode through
  boolean-dominant mixed literal equality, raising the lower bound to 768 while
  the identical modern expressions remain rejected and runtime stays shared.
- [x] Extend that compile-time mode through constant number/string equality,
  raising the lower bound to 770 while retaining modern comparison behavior.
- [x] Route source-free literal `and`/`or` expressions into the shared typed
  evaluator, raising the lower bound to 773 with a modern lifecycle sentinel.
- [x] Compile XPath 1.0 literal division by zero to its non-finite value while
  retaining modern decimal errors, reaching 776 doubt-annotated expected-result
  matches without adding a runtime version branch.
- [x] Apply XPath 1.0 numeric conversion to ordered source-free string/number
  comparisons, reaching 777 through one doubt-annotated expected-result match
  while retaining the modern mixed-type boundary.
- [x] Compile bounded addition and multiplication between typed location paths
  with a static-context-selected first-node or zero-or-one policy, reaching 782
  through five doubt-annotated expected-result matches without a runtime
  version branch.
- [x] Extend the same typed plan through whitespace-delimited path subtraction
  and exact integral path division, reaching 787 through five more
  doubt-annotated matches while keeping hyphenated names and general numeric
  division explicit.
- [x] Retain one explicit unary-negation bit on each typed path operand,
  reaching 792 through five more doubt-annotated matches without confusing
  NCName hyphens or admitting general expression trees.
- [x] Add token-delimited checked path modulo, reaching 796 through four more
  doubt-annotated matches, then stop the bounded recognizer before the chained
  and nested arithmetic frontier.
- [x] Replace the exact binary form with an owned recursive path-only operator
  tree, reaching 798 through two repeated-division matches while exposing one
  exact-decimal runtime boundary without credit.
- [x] Retain exact-rational intermediate values through that shared operator
  tree and recognize an operator keyword after a closed operand, reaching 800
  through the exposed decimal multiplication and parenthesized-division cases
  without a runtime version branch.
- [x] Add exact literal leaves and unary grouped negation to the shared tree,
  reaching 808 through eight mixed path/literal cases while leaving two newly
  exposed execution boundaries uncredited and preserving the existing
  compile-time XSLT 1.0 non-finite rule.
- [ ] Split the dominant XPath and unsupported-instruction frontiers into
  actionable semantic families and compare them with the XSLT30/QT3 roadmap.
- [ ] Resolve or explicitly classify the 50 known executing comparison
  mismatches.
- [ ] Define expected-error and discretionary/doubts comparison rules.
  - [x] Report doubt-annotated mismatches separately: four of the current 50
    mismatches carry substantive doubts metadata; none is reclassified yet.
- [x] Prototype at least one genuine version-dependent behavior through
  compile-time static context without a second runtime.
- [x] Preserve empty local bindings as strings, iterate source-node-valued local
  variables through the shared runtime frame, and select XSLT 1.0 first-node
  variable conversion at compile time, raising the lower bound from 1,050 to
  1,061 while retaining one newly exposed whitespace mismatch uncredited.
- [x] Lower abbreviated attribute presence and literal-equality predicates to
  the shared charged attribute-axis operation, then apply source-node variables
  without temporary-tree conversion, raising the lower bound from 1,061 to
  1,067 while retaining one newly exposed serialization mismatch uncredited.
- [x] Route instruction-local `position() != last()` through the existing
  dynamic sequence focus with located focusless failure and work accounting,
  raising the lower bound from 1,067 to 1,070 through three exact cases.
- [x] Reuse charged path-union normalization for `xsl:apply-templates`, raising
  the lower bound from 1,070 to 1,093 while retaining eight later execution
  failures and two later comparison mismatches visibly and uncredited.
- [x] Route source-attribute `xsl:copy` through the existing pending-attribute
  owner, raising the lower bound from 1,093 to 1,094 and exposing seven genuine
  `XTDE0410` placement errors instead of an unsupported-node-kind boundary.
- [x] Retain source element and attribute lexical prefixes separately from
  expanded-name identity and use them for `name()`, eliminating all seven
  `FXRT1008` observations and raising the lower bound from 1,094 to 1,101.
- [x] Reuse that lexical context-name operation as a typed `xsl:sort` key,
  then add controlled context string-length, `count(path)`, and compile-selected
  XSLT 1.0 `number(path)` keys, raising the lower bound from 1,101 to 1,105
  without admitting general dynamic sort expressions.
- [x] Preserve ordered path-predicate semantics for the bounded
  attribute-filter-then-position form, raising the lower bound from 1,105 to
  1,108 without admitting reverse-order or general predicate chains.
- [x] Lower explicit `position() = N` path predicates to the existing typed
  positional operation, raising the lower bound from 1,108 to 1,118 without
  admitting general predicate comparisons.
- [x] Recognize an unprefixed NCName predicate as the existing named child-axis
  test, raising the lower bound from 1,118 to 1,120 while leaving newly exposed
  later boundaries visible and uncredited.
- [x] Preserve direct reverse-axis versus parenthesized filter predicate order
  for one bounded reverse-axis step, raising the lower bound from 1,120 to 1,123
  without admitting general filter expressions.
- [x] Reuse explicit QName resolution for simple ordinary value paths, raising
  the lower bound from 1,123 to 1,125 without admitting namespace wildcards,
  qualified functions, or a second path evaluator.
- [x] Route simple explicit QName `xsl:for-each` and `xsl:apply-templates`
  selections through that same namespace-aware path owner, raising the lower
  bound from 1,125 to 1,127 without a second apply-selection evaluator.
- [x] Route simple explicit QName `xsl:sort` keys through that same private
  typed path owner, raising the lower bound from 1,127 to 1,130 while retaining
  one newly exposed result mismatch visibly and uncredited.
- [x] Recognize the namespace name fixed for the reserved `xml` prefix in
  instruction expression resolution, raising the lower bound from 1,130 to
  1,131 without requiring a redundant namespace declaration.
- [x] Compose literal `lang()` path predicates and bounded top-level `and`
  conjunctions through the shared charged context-language operation, raising
  the lower bound from 1,131 to 1,136 while preserving the next
  qualified-predicate boundary visibly.
- [x] Preserve the original step focus for bounded node-test and
  `position() = N` conjunctions, raising the lower bound from 1,136 to 1,140
  without rewriting them as semantically different chained predicates.
- [x] Compose a typed missing-attribute predicate with the symmetric
  `last()=position()` spelling, raising the lower bound from 1,140 to 1,142
  without admitting general negation.
- [x] Reuse checked exact arithmetic for signed-modulo comparisons inside the
  existing source-free boolean tree, raising the lower bound from 1,142 to
  1,143 without adding a compatibility-only evaluator.
- [x] Compare `position()` or `last()` with a static nonnegative integer or
  each other in ordinary value expressions and instruction conditions through
  the existing sequence focus, raising the lower bound from 1,143 to 1,150
  while preserving located focusless `XPDY0002`.
- [x] Compare `count(path)` with a static nonnegative integer in instruction
  conditions through the shared controlled path evaluator, raising the lower
  bound from 1,150 to 1,153 without a compatibility-only sequence evaluator.
- [x] Evaluate static-integer relational position predicates against each
  path step's typed focus, raising the lower bound from 1,153 to 1,158 while
  keeping general positional match-pattern semantics explicitly unsupported.
- [x] Evaluate checked `last()-N` and chained positional predicates in lexical
  order with a recomputed focus, raising the lower bound from 1,158 to 1,161
  while keeping non-simple multi-step match patterns explicitly unsupported.
- [x] Reuse checked source-free `number()` conversion for an exact integral
  path position, raising the lower bound from 1,161 to 1,162 without adding a
  compatibility-only numeric evaluator.
- [x] Compose focus-relative relational comparisons through the instruction
  boolean tree with short-circuit `and`, raising the lower bound from 1,162 to
  1,163 without adding a compatibility-only boolean evaluator.
- [x] Preserve a bounded position-then-lexical-name predicate chain, raising
  the lower bound from 1,163 to 1,164 without reordering its focus semantics.
- [x] Reuse typed sequence focus for the exact `ceiling(last() div 2)`
  midpoint, raising the lower bound from 1,164 to 1,165 without a general
  compatibility-only arithmetic evaluator.
- [x] Match simple named elements at exact or static-before source-sibling
  positions, raising the lower bound from 1,165 to 1,169 while preserving
  source position across sorted application order.
- [x] Preserve an empty level-any number list rather than formatting it as
  numeric zero, raising the lower bound from 1,169 to 1,170.
- [x] Apply XSLT 1.0 numeric variables to a single child-step position, raising
  the lower bound from 1,170 to 1,173 while retaining an explicit guard for
  multi-step predicate focus.
- [x] Reuse typed global/path and atomic-variable machinery for source-dependent
  count, static local values, ignored undeclared named-call arguments, and
  existential XSLT 1.0 node-set/string comparison, raising the lower bound from
  1,173 to 1,178.
- [x] Reuse typed atomic frames for static string locals, static-function
  parameter defaults, and string-compatible variable comparison, raising the
  lower bound from 1,178 to 1,183 without admitting general node-set pair
  comparison.
- [x] Preserve existential source-path equality in typed template arguments
  and reuse the computed-attribute compiler for nested `xsl:attribute`, raising
  the lower bound from 1,183 to 1,184.
- [x] Compose charged `sum(path)` through template arguments and computed
  attributes, admit unqualified source-attribute AVTs, and preserve
  invocation-local atomic aliases, raising the lower bound from 1,184 to 1,186.
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

- 2026-09-08 -- Literal result attributes now admit one bounded
  `concat('literal',$variable)` expression. The unchanged Lotus `impincl24`
  case raises the strict lower bound from 1,208 to 1,209 and proves global
  atomic-variable visibility through the admitted `apply-imports` path without
  changing any mismatch or execution-failure count. General AVT concatenation
  remains explicit.
- 2026-09-08 -- Literal result attributes now admit one bounded
  `starts-with(@value,@prefix)` expression between static text fragments. The
  unchanged Lotus `attribvaltemplate11` case raises the strict lower bound from
  1,207 to 1,208 and removes one `FXST1031` initialization frontier without
  changing any mismatch or execution-failure count. General AVT boolean
  expressions remain explicit.
- 2026-09-08 -- Literal result attributes now admit one bounded
  `concat(@left,@right)` expression between static text fragments. The
  unchanged Lotus `attribvaltemplate10` case raises the strict lower bound from
  1,206 to 1,207 and removes one `FXST1031` initialization frontier without
  changing any mismatch or execution-failure count. General AVT expressions
  remain explicit.
- 2026-09-08 -- Literal result attributes now compose static text with one
  checked integer offset over an unqualified source attribute. The unchanged
  Lotus `attribvaltemplate06` case raises the strict lower bound from 1,205 to
  1,206 and removes one `FXST1031` initialization frontier without changing any
  mismatch or execution-failure count. General AVT arithmetic remains explicit.
- 2026-09-08 -- The bounded legacy HTML serializer now admits exactly one
  unnamespaced HTML root with one `href`-bearing anchor and text-only content.
  The newly executable Lotus mixed-path AVT case raises the strict lower bound
  from 1,204 to 1,205 and reduces `FXSR1001` execution failures by one without
  changing any other disposition. General HTML result shapes remain explicit.
- 2026-09-08 -- Literal result attributes now retain one typed source location
  path between static text fragments and evaluate it through the shared
  controlled path owner with XSLT 1.0 first-node string conversion. Two
  unchanged Microsoft AVT cases raise the strict lower bound from 1,202 to
  1,204 without adding a comparison mismatch. One Lotus case reaches the
  independent bounded HTML-serialization frontier and remains uncredited;
  multiple expressions and the general AVT grammar remain unsupported.
- 2026-09-08 -- Elements in a stylesheet-declared prefixed or `#default`
  extension namespace now stop at explicit unsupported `FXST1059` rather than
  becoming literal result elements. Two unchanged Microsoft expected-error
  cases leave the unexpected-success class, reducing it from 7 to 5 without
  changing the 1,202 exact matches. Extension invocation, fallback, and host
  authority remain unselected.
- 2026-09-08 -- Stylesheet-root validation now rejects forbidden `mode` and
  malformed or unbound `extension-element-prefixes` tokens. Two unchanged
  Microsoft expected-error cases move from unexpected success to observed
  initialization failure, reducing that unresolved class from 9 to 7 without
  changing the 1,202 exact matches. Extension execution and authority remain
  outside this lexical validation tranche.
- 2026-09-08 -- Principal/module/simplified stylesheet versions now require a
  positive decimal lexical, and an explicit mode on a named-only template is
  invalid. Three unchanged Microsoft expected-error cases move from unexpected
  success to observed initialization failure, reducing that unresolved class
  from 12 to 9 without changing the 1,202 exact matches. Unknown positive
  versions retain the existing forwards-compatible boundary.
- 2026-09-08 -- Static `xsl:element` construction now rejects the reserved
  XMLNS namespace URI as invalid `XTDE0835`. Two unchanged Microsoft expected-
  error cases move from unexpected success to observed initialization failure,
  reducing that unresolved class from 14 to 12 without changing the 1,202 exact
  result matches. Dynamic namespace AVTs and broader reserved-name validation
  remain unadmitted.
- 2026-09-08 -- Typed `xsl:sort` keys now admit bounded unions of supported
  source location paths, normalize the node-set by document order and identity,
  and apply XSLT 1.0 first-node string conversion. The unchanged
  `Lotus/sort_sort26#1` case raises the lower bound from 1,201 to 1,202. Three
  additional cases initialize and execute: one matches and two remain visible
  comparison mismatches, raising that frontier from 79 to 81; execution
  failures remain 163. Arbitrary union operands and temporary-tree unions remain
  unadmitted.
- 2026-09-08 -- Exact XSLT 1.0 `count(current())` now observes the required
  singleton outer source focus, and `[count(current())]` lowers to the existing
  numeric position-one predicate without entering the general XPath grammar.
  The unchanged `Lotus/select_select86#1` case raises the lower bound from
  1,200 to 1,201; mismatches remain 79 and execution failures remain 163.
  General current-node arithmetic, comparison, and navigation remain
  unadmitted.
- 2026-09-08 -- XSLT 1.0 value compilation now admits one exact standalone
  `[current()]` path predicate, including the bounded parenthesized first-node
  form, without widening the general XPath parser. The unchanged
  `Lotus/select_select85#1` case raises the lower bound from 1,199 to 1,200;
  mismatches remain 79 and execution failures remain 163. General `current()`
  expressions and modern XPath admission remain unchanged.
- 2026-09-08 -- Predicate-bearing `xsl:apply-templates` paths can now navigate
  from each node in an invocation-owned source-node variable, preserve
  per-root predicate focus, and normalize the combined result by document
  order and identity. The unchanged `Lotus/select_select80#1` case raises the
  lower bound from 1,198 to 1,199; mismatches remain 79 and execution failures
  remain 163. Temporary-tree predicates and cross-document ordering remain
  unadmitted.
- 2026-09-08 -- An exact `current()` template argument now retains caller
  source-node identity in the invocation-local typed frame; XSLT 1.0
  `normalize-space($variable)` and computed-attribute `count($variable)` reuse
  that sequence while source-attribute computed values use the callee focus.
  The unchanged `Lotus/select_select79#1` case raises the lower bound from
  1,197 to 1,198; mismatches remain 79. General `current()` placement and
  arbitrary computed-attribute expressions remain unadmitted.
- 2026-09-08 -- Local variables can now retain a variable-only union of
  invocation-owned source nodes through the existing copy-on-write frame;
  runtime binding normalizes document order and identity, and XSLT 1.0
  `count($variable)` observes the typed sequence. The unchanged
  `Lotus/select_select72#1` case raises the lower bound from 1,196 to 1,197;
  mismatches remain 79 and execution failures remain 162. Atomic,
  temporary-tree, arbitrary-expression, and cross-document unions remain
  unadmitted.
- 2026-09-08 -- `xsl:apply-templates` can now combine one distinct
  invocation-owned source-node variable with typed location paths, normalize
  the union by source document order and node identity, and establish focus
  only after normalization. The unchanged `Lotus/select_select65#1` case
  raises the lower bound from 1,195 to 1,196; mismatches remain 79 and
  execution failures remain 162. General sequence and temporary-tree unions
  remain unadmitted.
- 2026-09-08 -- XSLT 1.0 value expressions can now navigate the shared typed
  relative path from an invocation-owned source-node variable, normalize the
  combined result, and apply first-node string conversion. The unchanged
  `Lotus/select_select77#1` case raises the lower bound from 1,194 to 1,195;
  mismatches remain 79 and execution failures remain 162. Atomic and temporary
  values are not silently treated as source nodes.
- 2026-09-08 -- Standalone exact decimal literals now reuse the checked numeric
  plan instead of falling through to location-path parsing. The unchanged
  `Lotus/math_math105#1` case raises the lower bound from 1,193 to 1,194;
  mismatches remain 79 and execution failures remain 162. Bare paths and bare
  variables retain their existing typed owners.
- 2026-09-08 -- Directional, operator-aware subtraction recognition now
  distinguishes omitted-whitespace arithmetic from hyphens in valid XML names.
  Four unchanged OASIS cases move directly from initialization failure to exact
  output, raising the lower bound from 1,189 to 1,193 while mismatches remain 79
  and execution failures remain 162. A broader whitespace rule was rejected
  after the full corpus exposed two regressions.
- 2026-09-08 -- The checked exact-rational numeric tree now admits
  unprefixed variable leaves only under XSLT 1.0 static context and resolves
  them through the existing invocation-local value owner. Unchanged
  `Lotus/math_math08#1`, `math_math97#1`, and `math_math100#1` move directly
  from initialization failure to exact output. The lower bound is 1,189;
  mismatches remain 79 and execution failures remain 162.
- 2026-09-07 -- Charged `sum(path)` template arguments and computed-attribute
  concatenation, unqualified source-attribute AVTs, and invocation-local
  atomic aliases move unchanged `Lotus/variable_variable62#1` and
  `Lotus/variable_variable44#1` to exact output. The lower bound is 1,186;
  mismatches remain 79, while one additional initialized case reaches a later
  visible runtime failure and leaves execution failures at 162.
- 2026-09-07 -- Existential source-path equality as a typed template argument
  and nested computed-attribute compilation move unchanged
  `Lotus/variable_variable60#1` to exact output. The lower bound is 1,184 with
  mismatches and execution failures unchanged.
- 2026-09-07 -- Static string locals, a two-literal string-function template
  parameter default, and string-compatible variable comparison move five
  unchanged cases to exact output. The lower bound is 1,183 with mismatches and
  execution failures unchanged.
- 2026-09-07 -- Global `count(path)`, static local boolean/integer values,
  undeclared named-call argument handling, and exact node-set/string equality
  semantics move five unchanged variable cases to exact output. The lower
  bound is 1,178 with mismatches and execution failures unchanged.
- 2026-09-07 -- Numeric variables now supply positions for a bounded
  single-child-step value path. Three unchanged cases raise the lower bound to
  1,173; descendant and other multi-step forms remain explicitly unsupported
  rather than approximating per-step predicate focus.
- 2026-09-07 -- Level-any numbering now distinguishes an empty number list
  from numeric zero while retaining format punctuation. Unchanged
  `Microsoft/Number__84694#1` raises the lower bound to 1,170 and removes one
  comparison mismatch.
- 2026-09-07 -- Exact and static-before named-sibling match patterns now use
  source-tree focus independently from application order. Four unchanged cases
  raise the lower bound to 1,169 with no new mismatch or execution failure.
- 2026-09-07 -- The typed focus operand now retains the exact rounded-up
  `ceiling(last() div 2)` midpoint. Unchanged `position27` raises the lower
  bound to 1,165 without changing mismatch or execution-failure counts.
- 2026-09-07 -- A typed path step can now apply positional filters before one
  trailing lexical `name()` equality. Unchanged `position82` raises the strict
  lower bound to 1,164 without changing mismatch or execution-failure counts.
- 2026-09-07 -- Instruction conditions now compose checked focus-relative
  comparisons through short-circuit `and`. Unchanged `position41` raises the
  strict lower bound to 1,163 without changing mismatch or execution-failure
  counts.
- 2026-09-07 -- Static integral `number()` conversion now composes with the
  typed position-predicate owner. Unchanged `position67` raises the strict
  lower bound to 1,162 without changing mismatch or execution-failure counts.
- 2026-09-07 -- Three relative-last and chained-position cases raise the
  strict lower bound to 1,161. Positional filters now recompute their focus in
  lexical order, while the multi-step match-pattern guard expands to cover the
  newly admitted non-simple form.
- 2026-09-07 -- Five relational position-predicate cases raise the strict
  lower bound to 1,158. A focused compiler guard keeps an independently exposed
  multi-step match-pattern semantic mismatch unsupported and uncredited.
- 2026-09-07 -- Typed count-path equality now reuses the shared controlled
  location-path evaluator in instruction conditions. Three unchanged position
  cases raise the strict lower bound to 1,153 without changing mismatch or
  execution-failure counts.
- 2026-09-07 -- Typed focus equality reuses the existing sequence focus across
  value expressions and instruction conditions. Seven unchanged cases raise
  the strict lower bound to 1,150 without changing mismatch or execution-
  failure counts; two additional position cases advance to distinct later
  boundaries and remain uncredited.
- 2026-09-07 -- Exact signed-modulo comparisons now compose through the
  existing source-free boolean tree. Unchanged `math83` raises the strict lower
  bound to 1,143 without changing mismatch or execution-failure counts.
- 2026-09-07 -- A typed missing-attribute predicate and the symmetric
  `last()=position()` spelling raise the strict lower bound to 1,142 across two
  unchanged position cases; broader negation remains unsupported.
- 2026-09-07 -- Bounded node-test and `position() = N` conjunctions now retain
  the original named-candidate focus. Four unchanged position cases raise the
  strict lower bound to 1,140 without changing mismatch or execution-failure
  counts.
- 2026-09-07 -- Literal `lang()` path predicates now reuse the existing
  context-language evaluator and compose with already-supported atoms through
  a bounded top-level `and`. Five unchanged cases raise the strict lower bound
  to 1,136; `expression06` advances to a distinct qualified-attribute predicate
  boundary and remains uncredited.
- 2026-09-07 -- Instruction expression namespace resolution now recognizes the
  reserved `xml` prefix implicitly. Unchanged `attribset20` raises the strict
  lower bound to 1,131 without changing mismatch or execution-failure counts.
- 2026-09-07 -- Qualified `xsl:sort` child/attribute keys now reuse the shared
  namespace-aware path owner. Three unchanged exact cases raise the strict
  lower bound to 1,130; one newly executing whitespace/result mismatch remains
  visible and uncredited.
- 2026-09-07 -- The shared apply-selection compiler now resolves simple
  explicit QName paths for both `xsl:for-each` and `xsl:apply-templates`. Two
  unchanged exact cases raise the strict lower bound to 1,127 while mismatch
  and execution-failure counts remain unchanged.
- 2026-09-07 -- Ordinary value selection now reuses the existing qualified
  child/attribute path owner for simple explicit QName paths. Two exact cases
  raise the strict lower bound to 1,125 without changing mismatch or execution
  failure counts.
- 2026-09-07 -- A bounded parenthesized reverse-axis filter now normalizes its
  candidate sequence before applying predicates, while direct reverse-axis
  predicates retain proximity order. Three exact cases raise the strict lower
  bound to 1,123.
- 2026-09-07 -- An unprefixed NCName path predicate now reuses the existing
  bounded named-child test. Two unchanged exact cases raise the strict lower
  bound to 1,120; cases reaching later unsupported or invalid boundaries remain
  uncredited.
- 2026-09-07 -- Explicit `position() = N` path predicates now lower to the
  existing typed positional selection. Ten unchanged exact cases raise the
  strict lower bound to 1,118 while non-equality and dynamic comparisons remain
  unsupported.
- 2026-09-07 -- Typed path steps now retain and evaluate one bounded attribute
  predicate before one bounded positional predicate. Three unchanged exact
  cases raise the strict lower bound to 1,108; reverse predicate order and the
  related parenthesized filter-expression case remain explicitly unsupported.
- 2026-09-07 -- The existing sort owner now admits charged typed context-name,
  context-string-length, `count(path)`, and XSLT 1.0 `number(path)` keys. Four
  exact cases raise the strict lower bound to 1,105 without changing mismatch or
  execution-failure counts.
- 2026-09-07 -- Prepared XDM now retains source attribute lexical prefixes and
  both context/path `name()` operations use retained element or attribute
  spelling without reconstructing namespace prefixes. Seven exact cases remove
  the `FXRT1008` frontier and raise the strict lower bound to 1,101.
- 2026-09-07 -- Source-attribute `xsl:copy` now reuses pending-attribute
  construction and its existing duplicate/late-attribute owner. One exact case
  raises the strict lower bound to 1,094; seven other cases reach explicit
  `XTDE0410` placement errors and remain uncredited.
- 2026-09-07 -- Top-level apply-selection unions now reuse the controlled path
  evaluator, source identity normalization, document order, duplicate removal,
  and sequence focus. Twenty-three exact cases raise the strict lower bound to
  1,093; eight later failures and two mismatches remain uncredited.
- 2026-09-07 -- Instruction-local `position() != last()` now consumes the
  existing sequence focus, charges the comparison, and retains located
  `XPDY0002` behavior without a general comparison parser. Three exact cases
  raise the strict lower bound to 1,070.
- 2026-09-07 -- Abbreviated attribute presence, static string equality, and
  direct source-node-variable template application add six exact results and
  reduce the dominant generic path frontier from 155 to 137 observations; the
  strict lower bound reaches 1,067.
- 2026-09-07 -- Childless local bindings now retain empty-string identity and
  variable-only `xsl:for-each` resolves source-node or temporary-tree value
  kinds through the shared runtime. Twelve cases advance past execution, eleven
  match exactly, and the strict lower bound reaches 1,061.
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
- 2026-09-04 -- Typed principal-source node identity raised the lower bound
  from 607 to 608 through unchanged `idkey06`; multi-node legacy conversion
  remains behind the unresolved version-mode boundary.
- 2026-09-04 -- Path-dependent `normalize-space()` raised the lower bound from
  608 to 609 through unchanged `string10`; the modern zero-or-one argument
  boundary remains explicit.
- 2026-09-04 -- Path-dependent expanded-name functions raised the lower bound
  from 609 to 613; one newly initialized multi-node case remains a visible
  `XPTY0004` rather than silently taking the first node.
- 2026-09-04 -- Whitespace-tolerant zero-argument function recognition raised
  the lower bound from 613 to 614 through unchanged `select20` without changing
  `string-length` semantics.
- 2026-09-04 -- Checked constant integral functions raised the lower bound from
  614 to 638; all 24 newly initialized direct-result and equality cases pass
  without adding a mismatch or later failure.
- 2026-09-04 -- Typed integral-function paths raised the lower bound from 638
  to 647; all nine newly initialized cases pass, while a first-party boundary
  case preserves multi-node `XPTY0004`.
- 2026-09-04 -- Static-atom and typed-path `string()` raised the lower bound
  from 647 to 653. Seven cases initialize: six pass and the multi-node legacy
  case reports the retained modern cardinality error.
- 2026-09-04 -- Typed `name()` paths raised the lower bound from 653 to 657.
  Eleven cases initialize: four pass, three reach existing XML-comparator gaps,
  and four retain explicit cardinality, lexical-prefix, or serialization
  boundaries without adding a mismatch.
- 2026-09-04 -- Expanded-name attribute path steps raised the lower bound from
  657 to 662. Five cases pass and one reaches the retained lexical-prefix
  boundary without adding a mismatch or comparator gap.
- 2026-09-04 -- Shared `preceding-sibling` and `preceding` path axes raised the
  lower bound from 662 to 686. All 24 newly initialized cases pass; reverse-axis
  positions are applied before ordinary document-order normalization.
- 2026-09-04 -- The shared `following` path axis raised the lower bound from
  686 to 694. All eight newly initialized cases pass without changing any
  mismatch, comparator-gap, runtime-failure, or unexpected-success count.
- 2026-09-04 -- Explicit non-element kind tests on `following` raised the lower
  bound from 694 to 697. Three cases pass; two more reach the existing
  `xsl:strip-space`/`xml:space` runtime boundary and remain uncredited.
- 2026-09-04 -- Shared `ancestor` and `ancestor-or-self` axes plus correct
  leading-`//` context expansion raised the lower bound from 697 to 718. All 21
  newly executing cases pass; one additional initialized case stops at the
  explicit modern multi-node conversion boundary.
- 2026-09-04 -- Explicit non-element kind tests on `self` raised the lower
  bound from 718 to 721. All three newly initialized cases pass without changing
  any later-failure or wrong-answer category.
- 2026-09-04 -- Explicit non-element kind tests on `preceding` raised the lower
  bound from 721 to 724. All three newly initialized composed-axis cases pass
  without changing any later-failure or wrong-answer category.
- 2026-09-04 -- Static doubled-brace AVT escaping raised the lower bound from
  724 to 728. Four newly executing identities pass; three other cases cross
  compilation and retain visible runtime failures rather than receiving credit.
- 2026-09-04 -- The shared path grammar now composes `.` as abbreviated
  `self::node()`, raising the lower bound from 728 to 729 through unchanged
  `axes98` without changing any later-failure or wrong-answer category.
- 2026-09-04 -- XPath whitespace around the `::` axis separator raised the
  lower bound from 729 to 731. Two cases pass; a third retains the visible
  modern multi-node conversion boundary rather than taking the legacy first
  node.
- 2026-09-04 -- Expanded-axis wildcard patterns raised the lower bound from 731
  to 734. All three newly initialized cases pass through the existing wildcard
  pattern operations and default priority.
- 2026-09-04 -- Expanded `attribute::node()` patterns raised the lower bound
  from 734 to 737. All three newly initialized cases pass while preserving the
  existing attribute-wildcard priority and tied-rule recovery behavior.
- 2026-09-04 -- The exact `{.}` AVT now uses one typed, work-charged context
  string-value operation across source and temporary trees. One of three newly
  executed cases passes; two line-ending mismatches remain visible and receive
  no compatibility credit, raising the lower bound from 737 to 738.
- 2026-09-04 -- Checked source-free binary arithmetic with an exact integral
  result raised the lower bound from 738 to 747. All nine newly initialized
  cases pass without changing a mismatch or later-failure category.
- 2026-09-04 -- Static finite `number()` conversion raised the lower bound from
  747 to 749. Both newly initialized cases pass without changing a mismatch or
  later-failure category; dynamic and special-value conversion remain outside
  the slice.
- 2026-09-04 -- Typed `number()` paths plus shared empty/non-convertible `NaN`
  behavior raised the lower bound from 749 to 753. All four newly initialized
  cases pass without changing a mismatch or later-failure category; multi-node
  conversion retains `XPTY0004`.
- 2026-09-04 -- Zero-argument `number()` raised the lower bound from 753 to 756
  through three unchanged identities by compiling its implicit context item to
  the same typed path operation.
- 2026-09-04 -- Exact boolean-to-number equality folding raised the lower bound
  from 756 to 758 through two unchanged cases.
- 2026-09-04 -- Exact valid constant short-circuit folding raised the lower
  bound from 758 to 760 through two unchanged cases while leaving mixed-type
  XPath 1.0 coercion behind the explicit version boundary.
- 2026-09-04 -- Relative boolean name-path evaluation raised the lower bound
  from 760 to 762 through two unchanged cases and repaired the shared runtime's
  accidental document-node rebinding.
- 2026-09-04 -- The first typed compile-time compatibility-mode experiment
  applied XPath 1.0 boolean-dominant mixed literal equality only to
  `version="1.0"` stylesheets. Six unchanged cases raised the lower bound from
  762 to 768; the paired modern stylesheet remains rejected and the executable
  plan contains no compatibility branch.
- 2026-09-05 -- Constant number/string equality reused the same compile-selected
  compatibility boundary and raised the lower bound from 768 to 770. Both
  leading-zero operand orders pass; the modern stylesheet remains rejected.
- 2026-09-05 -- Shared source-free literal boolean composition raised the lower
  bound from 770 to 773. A version 3.0 lifecycle sentinel executes the same
  typed evaluator, so this is not a legacy-only runtime path.
- 2026-09-05 -- Compile-selected XPath 1.0 non-finite literal division raised
  the measured expected-result matches from 773 to 776. All three promoted
  cases carry suite doubts metadata; modern decimal division by zero remains
  rejected.
- 2026-09-05 -- Compile-selected XPath 1.0 ordered literal conversion raised
  the measured expected-result matches from 776 to 777. The promoted case is
  doubt-annotated, and modern mixed-type comparison remains rejected.
- 2026-09-05 -- A shared typed binary-numeric path plan raised the measured
  expected-result matches from 777 to 782 through five doubt-annotated
  addition/multiplication cases. XSLT 1.0 first-node and modern zero-or-one
  cardinality are selected at compilation; runtime remains shared.
- 2026-09-05 -- Whitespace-delimited subtraction and exact path division raised
  the measured expected-result matches from 782 to 787 through five more
  doubt-annotated cases. The bounded parser distinguishes hyphenated names and
  retains fractional and zero division as explicit unsupported boundaries.
- 2026-09-05 -- Explicit unary signs on typed path operands raised the measured
  expected-result matches from 787 to 792 through five more doubt-annotated
  cases. Signs are retained in the compiled plan and runtime remains shared.
- 2026-09-05 -- Checked path modulo raised the measured expected-result matches
  from 792 to 796 through four doubt-annotated cases. The next arithmetic cases
  require a real expression tree, so the bounded recognizer stops here.
- 2026-09-05 -- The path plan became a recursive precedence-preserving operator
  tree and raised expected-result matches from 796 to 798 through two
  doubt-annotated repeated-division cases. One additional case now reaches the
  explicit non-integer lexical boundary and remains uncredited.
- 2026-09-05 -- Exact-rational path operands/intermediates and token-correct
  recognition of `div` after a closed parenthesized operand raised
  expected-result matches from 798 to 800 through two doubt-annotated cases.
  The same typed runtime preserves modern cardinality and explicit
  non-terminating, modulo, zero-divisor, and overflow boundaries.
- 2026-09-05 -- Exact literal leaves, unary grouped negation, and
  token-correct grouped/numeric subtraction raised expected-result matches
  from 800 to 808 through eight doubt-annotated cases. Two additional cases
  reach later execution boundaries and remain uncredited; compiler dispatch
  preserves the existing XSLT 1.0 `0 div 0` rule and runtime stays shared.
- 2026-09-06 -- Shared variable EBV, a compile-selected XPath 1.0
  node-set/boolean/string/number comparison plan, and bounded source-derived
  temporary values raised expected-result matches from 986 to 996. Two doubt-annotated
  temporary-tree boolean expectations remain visible and uncredited rather
  than changing the standards-correct conversion to fit archival output.
- 2026-09-06 -- Compile-selected XPath 1.0 variable `string()` and `number()`
  conversion reused the shared atomic, source-node, and temporary-tree owners
  and raised expected-result matches from 996 to 1,002. Initialization and
  execution each gained six cases without adding a mismatch or execution
  failure; modern typed behavior remains unchanged.
- 2026-09-06 -- Namespace-aware path-plus-literal plans for `contains()`,
  `starts-with()`, `substring-before()`, and `substring-after()` reused the
  shared controlled path/string-value runtime and raised expected-result
  matches from 1,002 to 1,021. All 19 newly executed cases agree exactly; no
  mismatch or execution-failure count changed, and modern cardinality remains
  untouched.
- 2026-09-06 -- A compatibility-only `sum(path)` plan reused namespace-aware
  controlled navigation and per-node string-value conversion, raising
  expected-result matches from 1,021 to 1,026 through five exact cases. Empty
  node sets, invalid numerics, and work accounting remain explicit; no
  mismatch or execution-failure count changed.
- 2026-09-06 -- A compatibility-only path-based `substring()` plan reused the
  controlled path evaluator and the existing Unicode/codepoint XPath-rounding
  helper, raising expected-result matches from 1,026 to 1,033 through seven
  exact cases without adding a mismatch or execution failure.
- 2026-09-06 -- A compatibility-only path-based `translate()` plan reused the
  controlled path evaluator and shared Unicode/codepoint translation helper,
  raising expected-result matches from 1,033 to 1,041 through eight exact
  cases without adding a mismatch or execution failure.
- 2026-09-06 -- A bounded compatibility-only `concat()` plan combined static
  string/integer operands, variables, and namespace-aware location paths
  through the shared XSLT 1.0 variable conversion, first-node string-value, and
  result-text owners, raising expected-result matches from 1,041 to 1,044
  through three exact cases. A fourth case now reaches the existing
  HTML-serialization boundary and remains uncredited; variable support is
  focused-oracle groundwork with no additional corpus credit, and no comparison
  mismatch was added.
- 2026-09-06 -- Quoted `xsl:with-param/@select` values now reuse the existing
  owned atomic template-argument path, raising expected-result matches from
  1,044 to 1,047 through three exact cases. The third case exposed and drove a
  shared text-run repair: excluded leading parameters remain sequence
  boundaries during whitespace classification. No mismatch or execution
  failure was added. The same bounded compiler/runtime owner now also carries
  boolean literals and the template-call focus expressions `position()` and
  `last()`; focused tests establish those semantics, but this measurement
  assigns them no additional corpus credit.
- 2026-09-06 -- A childless untyped global variable or parameter now retains
  the standard empty-string value rather than an empty temporary document.
  Three unchanged cases move to exact results, including two repaired boolean
  mismatches and one formerly unbound AVT parameter, raising the measured
  expected-result count from 1,047 to 1,050 while reducing comparison
  mismatches from 77 to 75 and execution failures from 174 to 173.
- 2026-09-06 -- The accumulated compatibility runtime crossed ADR-0004's
  2,000-line review threshold. XSLT 1.0 conversions and bounded path functions
  moved into a private 266-line typed module, reducing the parent value
  evaluator from 2,017 to 1,764 lines without moving compiler, XDM, result,
  host-policy, or public-API ownership.
- 2026-09-04 -- The exploratory report identified the then-current two
  mismatches with substantive doubts metadata separately from the other 27;
  the later path-union tranche adds `copy_copy09` as a third doubt-annotated
  mismatch without changing any case disposition.
