# AR-0019: XSLT 1.0 Compatibility Profile on the Modern Core

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-09-04 |
| Last reviewed | 2026-09-09 |
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

- 2026-09-13 -- Literal `contains()` over the context-node string value now
  extends the typed node-string match family with shared source/temporary
  semantics. One more case executes and matches exactly; the generic
  unsupported match-pattern frontier falls from 18 to 17 and exact results
  rise from 1,368 to 1,369.
  See [text contains match evidence](../Evidence/oasis-xslt10-text-contains-match-pattern-2026-09-13.md).
- 2026-09-13 -- One exact named-sibling position now composes with one exact
  attribute-value predicate without erasing predicate order. Source and
  temporary trees distinguish position-then-attribute from attribute-then-
  position. Three more cases execute and match exactly; the generic unsupported
  match-pattern frontier falls from 21 to 18 and exact results rise from 1,365
  to 1,368.
  See [ordered position/attribute match evidence](../Evidence/oasis-xslt10-ordered-position-attribute-match-patterns-2026-09-13.md).
- 2026-09-13 -- The exact `@*[name()='NCName']` match predicate now retains its
  lexical-name semantics and `0.5` predicate priority instead of collapsing to
  an exact expanded-name pattern. One more case executes and matches exactly;
  the generic unsupported match-pattern frontier falls from 22 to 21 and exact
  results rise from 1,364 to 1,365.
  See [attribute-name match evidence](../Evidence/oasis-xslt10-attribute-name-match-predicate-2026-09-13.md).
- 2026-09-13 -- Bounded namespace-aware attribute patterns now resolve exact
  names, namespace wildcards, presence predicates, and qualified-element
  attribute-value predicates against stylesheet namespace context. Two more
  cases match exactly; a third advances to a visible import/result-namespace
  mismatch and remains uncredited. The generic unsupported match-pattern
  frontier falls from 25 to 22 and exact results rise from 1,362 to 1,364.
  See [namespace-aware attribute match evidence](../Evidence/oasis-xslt10-namespace-aware-attribute-match-patterns-2026-09-13.md).
- 2026-09-13 -- Exact integer equality against a wildcard element string value
  or unqualified attribute now uses explicit XPath numeric conversion shared by
  source and temporary trees. One more case executes and matches exactly. A
  second identity reveals a later broad composed predicate and remains visibly
  unsupported; decimal/exponent, relational, arithmetic, variable, namespace,
  and general predicate forms remain outside the slice. Exact results rise from
  1,361 to 1,362.
  See [wildcard numeric match evidence](../Evidence/oasis-xslt10-wildcard-numeric-match-predicates-2026-09-13.md).
- 2026-09-13 -- Wildcard-element exact attribute-value predicates now use a
  charged typed matcher, while `node()` attribute-presence predicates normalize
  to the equivalent wildcard-element presence matcher. Source and temporary
  trees share both operations. Two more cases execute and both match exactly;
  namespace-qualified, numeric/relational, broader node-test, variable,
  function, and general boolean predicates remain outside the slice. Exact
  results rise from 1,359 to 1,361.
  See [generalized attribute node-test evidence](../Evidence/oasis-xslt10-generalized-attribute-match-node-tests-2026-09-13.md).
- 2026-09-13 -- Chained and conjunctive pairs of exact unqualified attribute-
  value match predicates now normalize to one typed, charged attribute scan
  shared by source and temporary trees. Three more cases execute and all match
  exactly; larger predicate sets, namespaces, inequality, numeric/relational
  comparison, variables, functions, and general boolean composition remain
  outside the slice. Exact results rise from 1,356 to 1,359.
  See [two-attribute-value match evidence](../Evidence/oasis-xslt10-two-attribute-value-match-patterns-2026-09-13.md).
- 2026-09-13 -- The typed context-node string matcher now admits exact
  inequality, negated exact equality, and two-literal equality-or while
  preserving source/temporary parity and bounded charging. Three more cases
  execute and all match exactly; general boolean trees, numeric or relational
  comparison, functions, variables, and multiple predicates remain outside the
  slice. Exact results rise from 1,353 to 1,356.
  See [composed node-string match evidence](../Evidence/oasis-xslt10-composed-node-string-match-predicates-2026-09-13.md).
- 2026-09-13 -- Exact context-node string equality now composes with bounded
  unqualified element, text, comment, and processing-instruction match tests
  through one charged typed matcher shared by source and temporary trees. Five
  more cases execute and all match exactly; numeric, function, composed,
  qualified, and multiple predicates remain outside the slice. Exact results
  rise from 1,348 to 1,353.
  See [node string-value match evidence](../Evidence/oasis-xslt10-node-string-value-match-patterns-2026-09-13.md).
- 2026-09-13 -- The exact `*[@name]` wildcard-element attribute-presence
  pattern now uses a dedicated charged typed matcher with source and temporary-
  tree parity. Two more cases execute and both match exactly; value comparisons,
  namespace wildcards, composed predicates, and general boolean predicates
  remain outside the slice. Exact results rise from 1,346 to 1,348.
  See [wildcard attribute-presence evidence](../Evidence/oasis-xslt10-wildcard-attribute-presence-patterns-2026-09-13.md).
- 2026-09-13 -- The exact `//name[true()]` pattern now normalizes at compile
  time to the existing typed leading-descendant path while retaining path
  priority and charged document-rooted membership. Two more cases execute and
  both match exactly; no general function-predicate matcher was admitted. Exact
  results rise from 1,344 to 1,346.
  See [static-true descendant match evidence](../Evidence/oasis-xslt10-static-true-descendant-match-patterns-2026-09-13.md).
- 2026-09-13 -- Single-name leading descendant patterns now admit one
  unqualified attribute presence/value predicate or one positive static sibling
  position through the existing typed path evaluator. Three more cases execute
  and all three match exactly; the generic unsupported match-pattern frontier
  falls from 47 to 43 without admitting child-value, relational, dynamic, or
  multi-step predicate forms. Exact results rise from 1,341 to 1,344.
  See [leading descendant predicate evidence](../Evidence/oasis-xslt10-leading-descendant-attribute-and-position-patterns-2026-09-13.md).
- 2026-09-13 -- Leading descendant patterns now extend to the bounded
  `//ancestor/child` and `//ancestor//descendant` forms while retaining typed
  path priority and document-rooted membership. Ten more cases execute and
  four match exactly; six named-whitespace cases remain visibly outside
  ADR-0012. The generic unsupported match-pattern frontier falls from 59 to 47
  and exact results rise from 1,337 to 1,341.
  See [bounded leading descendant path evidence](../Evidence/oasis-xslt10-bounded-leading-descendant-path-patterns-2026-09-13.md).
- 2026-09-09 -- The exact unqualified `match="//name"` abbreviation now
  retains typed path priority and uses bounded document-rooted membership.
  Eight more cases execute, five match exactly, and three downstream
  serialization/whitespace differences remain visible. The generic unsupported
  match-pattern frontier falls from 85 to 59 without admitting descendant
  predicates or multi-step forms. Exact results rise from 1,332 to 1,337.
  See [leading descendant name match evidence](../Evidence/oasis-xslt10-leading-descendant-name-match-patterns-2026-09-09.md).
- 2026-09-09 -- Dynamic sibling/descendant node-set equality and bounded
  nested positional-child comparisons now share the typed path evaluator.
  The matching positional-child forms reuse `MatchPattern::Path`, while
  general attribute-comparison patterns remain unsupported. Exact results rise
  from 1,331 to 1,332; one newly executable `indent="yes"` whitespace
  difference remains visibly mismatched.
  See [dynamic node-set and predicate-match evidence](../Evidence/oasis-xslt10-dynamic-node-set-and-predicate-match-paths-2026-09-09.md).
- 2026-09-09 -- The typed following-sibling predicate leaf now implements
  existential XSLT 1.0 numeric `=`, `!=`, `<`, `<=`, `>`, and `>=` comparisons
  with correct operand reversal and charged sibling visits. Ten unchanged
  cases raise the strict lower bound from 1,321 to 1,331. See
  [following-sibling relational-predicate evidence](../Evidence/oasis-xslt10-following-sibling-relational-predicates-2026-09-09.md).
- 2026-09-09 -- Bounded child-count, attribute string-length, and attribute
  inequality predicate leaves now compose with path execution and effective
  boolean value. Bare-axis condition admission remains narrowed to the
  evidenced `following-sibling::` form after a broader probe exposed a wrong
  result. Two unchanged cases raise the strict lower bound from 1,319 to 1,321.
  See [predicate cardinality and attribute-length evidence](../Evidence/oasis-xslt10-predicate-cardinality-and-attribute-length-2026-09-09.md).
- 2026-09-09 -- The private typed predicate tree now evaluates bounded
  `starts-with(name(.), literal)` and `string-length(name(.)) = integer`
  leaves while preserving lexical QName prefixes and charged XPath work. Two
  unchanged cases raise the strict lower bound from 1,317 to 1,319. See
  [context lexical-name predicate evidence](../Evidence/oasis-xslt10-context-lexical-name-predicates-2026-09-09.md).
- 2026-09-09 -- Typed predicate paths are now reachable from instruction-local
  effective boolean value, and a context-string/literal leaf composes with
  `not`. One unchanged case raises the strict lower bound from 1,316 to 1,317.
  See [context-string predicate evidence](../Evidence/oasis-xslt10-context-string-predicate-condition-2026-09-09.md).
- 2026-09-09 -- A bounded path-predicate leaf now compares the charged
  `following-sibling::*` node-set with a static integer using shared XSLT 1.0
  numeric conversion. Two operand-order cases raise the strict lower bound
  from 1,314 to 1,316. See
  [following-sibling numeric-predicate evidence](../Evidence/oasis-xslt10-following-sibling-numeric-predicate-2026-09-09.md).
- 2026-09-09 -- The private path-predicate tree now composes charged
  `descendant::*` node-set/string equality and inequality with `not`, `and`,
  and `or`. Six unchanged cases raise the strict lower bound from 1,308 to
  1,314 without treating XSLT 1.0 existential `!=` as negated equality. See
  [descendant node-set predicate evidence](../Evidence/oasis-xslt10-descendant-node-set-predicate-comparison-2026-09-09.md).
- 2026-09-09 -- A private typed attribute-predicate tree now preserves `and`/
  `or` precedence, parentheses, short-circuiting, and charged attribute visits.
  Eleven unchanged cases raise the strict lower bound from 1,297 to 1,308.
  Boolean-tree ownership was extracted after the path source crossed the
  ADR-0004 review threshold. See
  [attribute boolean-predicate evidence](../Evidence/oasis-xslt10-attribute-boolean-predicate-tree-2026-09-09.md).
- 2026-09-09 -- Static XSLT 1.0 mixed-type path predicates now share one typed
  path entry point across selection and value instructions. Boolean-to-number
  ordered conversion adds two exact OASIS results and raises the strict lower
  bound from 1,295 to 1,297 without a mismatch or runtime-failure increase. See
  [ordered literal path-predicate evidence](../Evidence/oasis-xslt10-ordered-literal-path-predicates-2026-09-09.md).
- 2026-09-09 -- Four XSLT 1.0 boolean/node-set predicate spellings now
  normalize to a charged following-sibling existence predicate in the shared
  typed path evaluator. Completing shared `@*` and `not(@*)` presence semantics
  brings two more unchanged cases with it. Six exact results raise the strict
  lower bound from 1,289 to 1,295 with no residual mismatch increase. See
  [boolean node-set axis-predicate evidence](../Evidence/oasis-xslt10-boolean-node-set-axis-predicates-2026-09-09.md).
- 2026-09-09 -- Statically decidable mixed-type equality predicates now fold
  during XSLT 1.0 compilation and retain the ordinary typed location-path
  execution. Three unchanged predicate cases raise the strict lower bound from
  1,286 to 1,289 without admitting dynamic node-set comparison. See
  [static mixed-equality predicate evidence](../Evidence/oasis-xslt10-static-mixed-equality-path-predicates-2026-09-09.md).
- 2026-09-09 -- The shared exact-rational evaluator now retains typed path-
  union operands, and XSLT 1.0 `xsl:copy-of` can copy the resulting atomic
  value. The unchanged `math103` case raises the strict lower bound from 1,285
  to 1,286. A broader unary-root experiment was rejected after exposing legacy
  double-formatting differences. See
  [unary numeric path-union evidence](../Evidence/oasis-xslt10-unary-numeric-path-union-copy-2026-09-09.md).
- 2026-09-09 -- XSLT 1.0 source-node variables now compose with the shared
  typed descendant-path evaluator. The unchanged `variable50` case matches
  exactly, raising the strict lower bound from 1,284 to 1,285; constructed
  result-tree fragments remain non-navigable and fail explicitly. See
  [source-variable descendant-path evidence](../Evidence/oasis-xslt10-source-variable-descendant-path-2026-09-09.md).
- 2026-09-09 -- The shared typed path model now retains literal
  `local-name()` equality predicates over path focus. The unchanged namespace-
  bearing `copy46` case matches exactly, raising the strict lower bound from
  1,283 to 1,284 without changing mismatch or runtime-failure counts. See
  [local-name predicate evidence](../Evidence/oasis-xslt10-local-name-path-predicate-2026-09-09.md).
- 2026-09-09 -- Exact standalone `xsl:copy-of select="current()"` now lowers
  to the existing current-item copy plan. One newly initialized case matches
  exactly, raising the strict lower bound from 1,282 to 1,283 without admitting
  composed `current()` semantics. See
  [standalone current-copy evidence](../Evidence/oasis-xslt10-standalone-current-copy-2026-09-09.md).
- 2026-09-09 -- Computed-attribute `xsl:value-of` now reuses the existing
  literal-only `substring()` fold and exact sequence `position()`/`last()`
  attribute operations. Three newly initialized cases all match exactly,
  raising the strict lower bound from 1,279 to 1,282 without a new mismatch or
  execution failure. See
  [static and focus-value evidence](../Evidence/oasis-xslt10-computed-attribute-static-and-focus-values-2026-09-09.md).
- 2026-09-09 -- A computed attribute can now retain exactly one already-
  admitted `xsl:number` instruction and consume its value through the shared
  bounded number evaluator. Source focus, work accounting, cancellation,
  retained-capacity accounting, and semantic inspection remain explicit. Four
  exact results raise the strict lower bound from 1,275 to 1,279; two other
  cases reach later compilation boundaries and remain uncredited. See
  [computed-attribute numbering evidence](../Evidence/oasis-xslt10-computed-attribute-number-2026-09-09.md).
- 2026-09-09 -- A bounded constructor refinement admits one explicit
  `xsl:text` child as computed-attribute character content by reusing the
  ordinary text validator. Six cases move beyond `FXST1033`, including one
  that now reaches HTML serialization, but no exact-result credit is claimed.
  Static `lang` and `case-order` metadata are now discarded only for numeric
  sort keys, where they cannot affect numeric ordering; three unchanged cases
  raise the strict lower bound from 1,272 to 1,275. Text collation and dynamic
  metadata remain explicit unsupported boundaries. See
  [attribute-text and numeric-sort evidence](../Evidence/oasis-xslt10-attribute-text-and-numeric-sort-metadata-2026-09-09.md).
- 2026-09-09 -- Static prefixed computed-attribute QNames now resolve once at
  compilation, with an optional nonempty static namespace override and strict
  `xml`/`xmlns`, malformed, empty, and unbound-prefix failures. Eleven cases
  reach execution and nine exact results raise the strict lower bound from
  1,263 to 1,272; one line-ending mismatch and one comparator boundary remain
  uncredited. Runtime QName parsing and lexical-prefix identity were not added.
  See [prefixed attribute evidence](../Evidence/oasis-xslt10-static-prefixed-computed-attribute-2026-09-09.md).
- 2026-09-09 -- Direct computed attributes now retain a static namespace URI,
  bounded literal text, and a compile-time prefix binding in the owning
  element's ADR-0018 namespace slice. Twenty-six unchanged cases initialize;
  19 exact results raise the strict lower bound from 1,244 to 1,263, three
  doubts/whitespace comparisons and four later runtime frontiers remain
  uncredited. Namespace AVTs, prefixed names, and reserved XMLNS construction
  remain explicit boundaries. See
  [static attribute-namespace evidence](../Evidence/oasis-xslt10-static-computed-attribute-namespace-2026-09-09.md).
- 2026-09-09 -- The generic 129-case unsupported-attribute frontier is now
  classified by instruction and expanded attribute name; its largest families
  are `xsl:attribute/@namespace` (32), `xsl:sort/@lang` (30), and
  `xsl:text/@disable-output-escaping` (16). The exact semantically inert
  `disable-output-escaping="no"` form now reuses ordinary result text, advancing
  Microsoft `78362` to its independent computed-comment frontier. `yes` and
  invalid lexical values remain explicit unsupported/invalid outcomes, and the
  1,244-pass lower bound is unchanged. See
  [unsupported-attribute evidence](../Evidence/oasis-xslt10-unsupported-attribute-frontier-and-doe-no-2026-09-09.md).
- 2026-09-09 -- Exact standalone `xsl:preserve-space elements="*"` now
  compiles to the engine's existing preserve-source-whitespace default without
  adding runtime representation or weakening ADR-0012. Six Microsoft cases
  reach execution and four exact results raise the strict lower bound from
  1,240 to 1,244; two independent indent-serialization mismatches remain
  visible. Selective and mixed strip/preserve policies still fail explicitly.
  See [preserve-all evidence](../Evidence/oasis-xslt10-preserve-all-whitespace-declaration-2026-09-09.md).
- 2026-09-09 -- Template parameter defaults can now retain the existing typed
  XSLT 1.0 binary-numeric plan, including an explicit `number(path)` operand,
  and bind its result as an atomic double. Microsoft `bvt072` advances through
  execution with every numeric result correct, but its doubts-annotated
  expected output drops preserved source whitespace; the mismatch remains
  visible and the 1,240-pass lower bound is unchanged. See
  [binary-numeric parameter evidence](../Evidence/oasis-xslt10-binary-numeric-parameter-default-2026-09-09.md).
- 2026-09-09 -- A compatibility-only template parameter default may now retain
  one text-only `xsl:choose` whose branches use the shared typed path/string
  equality evaluator. The selected value remains an invocation-owned temporary
  text tree. Unchanged Lotus `variable13` raises the strict lower bound from
  1,239 to 1,240 without adding a mismatch or execution failure. See
  [text-choice parameter evidence](../Evidence/oasis-xslt10-text-choice-parameter-default-2026-09-09.md).
- 2026-09-08 -- Typed template parameter defaults now retain source-node paths
  at the caller's exact focus and copy preceding parameter values without
  erasing atomic, source-node, or temporary-tree kind. Parameter names also
  participate in duplicate local-binding validation. Four unchanged XML
  expectations raise the strict lower bound from 1,235 to 1,239; one
  expected-error case advances to its correct runtime forward-reference
  failure without adding a standard-case regression. See
  [template parameter default evidence](../Evidence/oasis-xslt10-template-parameter-defaults-2026-09-08.md).
- 2026-09-08 -- Local variables and global defaults now share a bounded XSLT
  1.0 temporary-tree materializer for exactly
  `for-each(location-path) -> value-of(.)`. Controlled navigation and node
  string-value accounting remain authoritative. Unchanged Lotus `variable15`
  and `variable16` raise the strict lower bound from 1,233 to 1,235 without
  adding a mismatch or execution failure.
- 2026-09-08 -- A bounded content-parameter constructor now permits text-only
  XSLT 1.0 local variables before one typed `xsl:value-of`. The bindings live
  only in a cloned invocation-local frame, remain charged temporary text trees,
  and are included in compiled retained-capacity accounting. Unchanged Lotus
  `variable14` raises the strict lower bound from 1,232 to 1,233 without adding
  a mismatch or execution failure.
- 2026-09-08 -- Integer equality now retains whether its typed plan belongs to
  XSLT 1.0 compatibility. That path converts temporary-tree parameters through
  the shared numeric owner while the modern atomic route remains unchanged.
  Microsoft `84437` and `84047` advance from execution failures to the same
  doubts-annotated whitespace mismatch already exposed by sibling cases; the
  exact-pass lower bound remains 1,232.
- 2026-09-08 -- Content-built template parameters now reuse the ordinary typed
  `xsl:value-of` plan under the caller's exact focus and remain temporary text
  trees. Variable numeric ordering and existing exact-rational arithmetic close
  unchanged Lotus `namedtemplate10`, raising the lower bound from 1,231 to
  1,232. Four additionally admitted cases remain explicitly visible as two
  whitespace mismatches and two local-scope execution frontiers.
- 2026-09-08 -- XSLT 1.0 `string-length($variable)` now reuses the shared
  compatibility string-value owner and composes with a numeric-variable stop
  condition. A content-built `xsl:with-param` containing one admitted
  `xsl:value-of concat(...)` remains an invocation-owned temporary text tree.
  Unchanged Lotus `variable23` raises the strict lower bound from 1,230 to 1,231
  with no new mismatch or execution failure.
- 2026-09-08 -- Computed attributes and literal-result AVTs now share a typed
  XSLT 1.0 `normalize-space(path)` plan and the controlled source normalization
  owner already used by value construction. Unchanged Lotus `whitespace23`
  raises the strict lower bound from 1,229 to 1,230 with no new mismatch or
  execution failure. The computed-attribute value selector was privately
  extracted when the source unit crossed its ADR-0004 review threshold.
- 2026-09-08 -- The exact XSLT 1.0 computed-attribute expression
  `string-length(normalize-space(.))` now streams the controlled source string
  value, applies XML whitespace rules, and counts Unicode scalar values without
  an intermediate normalized string. Unchanged Lotus `string140` raises the
  strict lower bound from 1,228 to 1,229 with no new mismatch or execution
  failure.
- 2026-09-08 -- Typed path unions now share one controlled document-order and
  identity-normalizing evaluator across existing consumers. XSLT 1.0 computed
  attributes can count such unions, and the exact `name((union)[last()])` value
  shape preserves lexical node names. Both unchanged Lotus `position83`
  identities raise the strict lower bound from 1,226 to 1,228 with no new
  mismatch or execution failure.
- 2026-09-08 -- Computed `xsl:attribute` values now reuse typed, controlled
  `count(location-path)` evaluation from source focus. The unchanged Lotus
  `axes131` case raises the strict lower bound from 1,225 to 1,226 without
  adding a mismatch or execution failure; its empty-element spelling is
  handled by the existing infoset comparator rather than serializer tuning.
- 2026-09-08 -- Typed XSLT 1.0 source node-set `=` and `!=` plans now preserve
  independently existential string-value comparison across conditionals,
  value construction, and the existing template-argument evaluator. Lotus
  `boolean70` through `boolean76` and `position31` raise the strict lower bound
  from 1,217 to 1,225 without adding a mismatch or execution failure. The
  complete sweep caught and rejected an initially over-broad recognizer before
  this evidence was retained.
- 2026-09-08 -- Variable-only literal attributes now convert invocation-local
  temporary text trees through the shared string-value owner and preserve
  nested lexical shadowing. Lotus `variable56` advances from execution failure
  to a visible indentation comparison frontier; the strict 1,217-pass lower
  bound is unchanged and no pass credit is inferred.
- 2026-09-08 -- Text-constructed local XSLT 1.0 variables now retain explicit
  invocation-owned temporary-tree identity, bounded variable/path AVTs can use
  their string value, and direct fragment predicates remain distinct from
  explicit numeric position comparisons. Eleven cases advance past
  initialization and eight become exact results, raising the strict lower bound
  from 1,209 to 1,217 without adding a mismatch; three later execution
  frontiers remain visible.
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
- 2026-09-13 -- Whole-pattern variable references and non-literal `key()`
  arguments now fail as invalid `FXST1005` grammar, while valid literal
  `id()`/`key()` patterns remain explicitly unsupported. The generic
  unsupported match frontier fell from 17 to 15 without changing the 1,369
  exact-result lower bound or any observed expected-error disposition.
- 2026-09-13 -- A bounded typed sequential-predicate matcher admitted the seven
  unchanged Xalan `match20` through `match26` cases, raising exact results from
  1,369 to 1,376 without adding a mismatch or execution failure. Predicate
  position and size are recomputed after each filter, source and temporary
  trees share scalar semantics, and general match-pattern XPath remains out of
  scope.
- 2026-09-13 -- Version-sensitive validation now enforces XSLT 1.0's normative
  prohibition on variable references and `current()` in template match
  patterns while leaving later-edition variable patterns available. Two
  generic frontier failures are now invalid diagnostics; the contradictory
  archival Xalan `match14` success expectation remains uncredited, preserving
  the 1,376 exact-result lower bound.
- 2026-09-13 -- The namespace-aware `//n:book/n:chapter[2]/foo` pattern now
  compiles to a bounded expanded-name plan and shares charged source/temporary
  selection semantics. The unchanged Microsoft case reaches its next boundary,
  `namespace::*`, which is now correctly classified as valid but unsupported
  `FXXP1001` rather than invalid `XPST0003`. Aggregate counters and the 1,376
  exact-result lower bound remain unchanged; no namespace-axis support is
  inferred.
  [Evidence](../Evidence/oasis-xslt10-qualified-descendant-position-match-2026-09-13.md)
- 2026-09-13 -- Child-axis-relative position is now explicit for the bounded
  `chapter//footnote[position() != 1]` match plan. Charged source and temporary
  execution independently number same-named siblings under each immediate
  parent, moving unchanged Xalan `match16` from initialization failure to exact
  pass and raising the lower bound from 1,376 to 1,377 without a new mismatch
  or execution failure.
  [Evidence](../Evidence/oasis-xslt10-descendant-child-axis-position-2026-09-13.md)
- 2026-09-13 -- Differently ranked union-pattern alternatives now retain their
  individual default priorities as separate matched rules sharing one compiled
  body. Five Microsoft cases leave the generic union frontier; two become exact
  passes and three expose later independent boundaries, raising the exact lower
  bound from 1,377 to 1,379 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-union-alternative-priority-2026-09-13.md)
- 2026-09-13 -- Computed attributes now reuse the complete context-string
  operation for `xsl:value-of select="."` across source and temporary trees.
  Eleven Microsoft Output cases leave the compiler frontier and expose their
  later HTML-serialization boundary; the exact lower bound remains 1,379 and
  no new mismatch is introduced.
  [Evidence](../Evidence/oasis-xslt10-computed-attribute-context-string-2026-09-13.md)
- 2026-09-14 -- Plain local integer-literal variables now reuse the existing
  typed atomic binding path. Eight cases leave `FXXP1008`; two become exact
  passes, while two later path boundaries, two duplicate-binding diagnostics,
  one alias execution failure, and one whitespace mismatch remain explicitly
  visible. The exact lower bound rises from 1,379 to 1,381.
  [Evidence](../Evidence/oasis-xslt10-local-integer-literal-variables-2026-09-14.md)
- 2026-09-14 -- Local `name()` and `name(.)` bindings now retain a private
  context-derived atomic instruction with source-prefix and work-accounting
  semantics. Three cases leave `FXXP1008`; two expose later language boundaries
  and one exposes an existing whitespace mismatch, so the exact lower bound
  remains 1,381.
  [Evidence](../Evidence/oasis-xslt10-local-context-name-variables-2026-09-14.md)
- 2026-09-14 -- Local `count(location-path)` variables now reuse the controlled
  path evaluator and typed integer frames. All three
  `count(preceding::text())` cases leave `FXXP1008`; one becomes an exact pass
  and two expose a later `$this + 1` classification defect. The exact lower
  bound rises from 1,381 to 1,382.
  [Evidence](../Evidence/oasis-xslt10-local-count-path-variables-2026-09-14.md)
- 2026-09-14 -- Template arguments now reuse the bounded XSLT 1.0 binary
  numeric evaluator, moving the two `$this + 1` cases into execution. Their
  newly visible reverse-axis mismatch was repaired at the existing single-step
  variable-position seam: predicate proximity observes reverse-axis order while
  the general path result remains document ordered. Both unchanged cases become
  exact passes, raising the lower bound from 1,382 to 1,384 without a new
  mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-template-argument-arithmetic-and-reverse-axis-position-2026-09-14.md)
- 2026-09-14 -- Local source-dependent numeric variables now reuse the same
  bounded binary-numeric plan, controlled evaluator, and typed atomic frame as
  other numeric consumers. Unchanged Lotus `variable43` moves from `FXXP1008`
  to exact result, raising the lower bound from 1,384 to 1,385 without a new
  mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-local-path-arithmetic-variable-2026-09-14.md)
- 2026-09-14 -- Local source-node variables now compose through typed relative
  paths while preserving controlled evaluation, document order, duplicate
  elimination, and downstream focus size. The unchanged Microsoft `last()`
  case moves from `FXXP1008` to exact result, raising the lower bound from 1,385
  to 1,386 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-local-variable-rooted-path-2026-09-14.md)
- 2026-09-14 -- Exact `name()` and `name(.)` template arguments now reuse the
  charged source lexical-name operation. The unchanged Lotus recursive
  named-template case retains caller focus and becomes exact, raising the lower
  bound from 1,386 to 1,387 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-template-argument-context-name-2026-09-14.md)
- 2026-09-14 -- Untyped boolean globals, variable numeric conversion,
  focus-position offsets, variable sort keys, and source-variable template
  argument paths now compose through existing typed state. One unchanged
  Microsoft variable-sort case becomes exact and a broader BVT reaches a
  visible indentation mismatch. The exact lower bound rises from 1,387 to
  1,388; the sorted-variable path also replaces one exposed panic assumption
  with typed source-node handling.
  [Evidence](../Evidence/oasis-xslt10-variable-flow-and-sort-2026-09-14.md)
- 2026-09-14 -- Context-node `name()` equality now retains the source lexical
  QName and its comparison mode in a private boolean plan. Unchanged Lotus
  `axes121` reaches its intended line-feed branch. The local XML comparator now
  applies XML source line-ending normalization and ignores post-declaration
  whitespace before tree comparison, moving six executing mismatches to exact
  results and raising the lower bound from 1,388 to 1,394. The engine's DTD
  authority boundary remains unchanged.
  [Evidence](../Evidence/oasis-xslt10-context-name-condition-and-xml-line-endings-2026-09-14.md)
- 2026-09-14 -- Recognized XPath node tests now accept intervening whitespace
  before their parentheses and canonicalize into the existing typed path plan.
  Unchanged Lotus `select18` moves from the generic function-shaped path
  frontier to an exact comment-copy result, raising the lower bound from 1,394
  to 1,395 without admitting arbitrary function steps.
  [Evidence](../Evidence/oasis-xslt10-node-test-whitespace-2026-09-14.md)
- 2026-09-14 -- An exact XSLT 1.0 `current()` expression now lowers to the
  existing context-item path, and the value-expression fallback consistently
  uses the compatibility parser. Eight cases leave the generic function path
  frontier; three become exact and five remain visible mismatches, raising the
  lower bound from 1,395 to 1,398 without admitting composed `current()`
  semantics.
  [Evidence](../Evidence/oasis-xslt10-exact-current-select-2026-09-14.md)
- 2026-09-14 -- The existing XSLT 1.0 path string-function plan now retains a
  literal or typed path second operand for `contains()`. Both paths use
  controlled first-node string conversion and complete retention accounting.
  Four unchanged Lotus cases become exact, raising the lower bound from 1,398
  to 1,402 without broadening the other binary string functions.
  [Evidence](../Evidence/oasis-xslt10-path-to-path-contains-2026-09-14.md)
- 2026-09-14 -- Bounded source-free NaN composition now preserves XPath 1.0
  equality, arithmetic, and integral-function behavior at compilation while
  leaving modern decimal semantics unchanged. Eight unchanged Lotus cases move
  from initialization failure to exact results, raising the lower bound from
  1,402 to 1,410 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-nan-composition-2026-09-14.md)
- 2026-09-14 -- A bounded XSLT 1.0 `string()` wrapper now reuses context-path
  numeric plans. The unchanged Lotus long-decimal case exposed a shared exact-
  rational formatting overflow, repaired by using the minimal terminating-
  decimal scale. One case becomes exact, raising the lower bound from 1,410 to
  1,411 with no added mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-string-number-and-exact-decimal-formatting-2026-09-14.md)
- 2026-09-14 -- The bounded XSLT 1.0 predicate
  `string(number(.)) = 'NaN'` now compiles to a typed context-number test and
  reuses controlled context string-value access. Unchanged Microsoft
  `Number__84431` enters execution and produces the intended numeric branch
  decisions, but remains an upstream doubts-annotated mismatch because its
  expected result omits whitespace emitted by normative built-in template
  processing. The exact lower bound remains 1,411.
  [Evidence](../Evidence/oasis-xslt10-context-number-nan-predicate-2026-09-14.md)
- 2026-09-14 -- Explicit-axis paths accepted by the typed XPath parser now
  participate in ordinary node-set effective-boolean-value evaluation instead
  of being limited to a special following-sibling spelling. Unchanged Lotus
  `axes_axes130` enters execution with all four attribute-context self-axis
  choices correct. Its remaining XML mismatch is implementation-dependent
  indentation emitted under `indent='yes'`; it is retained as comparison-
  harness pressure and the exact lower bound remains 1,411.
  [Evidence](../Evidence/oasis-xslt10-explicit-axis-boolean-paths-2026-09-14.md)
- 2026-09-14 -- XPath 1.0 `contains(number(.), 'NaN')` now reuses the typed
  context-number NaN boolean plan, preserving implicit number-to-string
  conversion without a general nested-function evaluator. Unchanged Lotus
  `math_math104` becomes exact, raising the lower bound from 1,411 to 1,412
  without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-contains-context-number-nan-2026-09-14.md)
- 2026-09-14 -- Bounded `position() mod N = M` instruction predicates now
  compile to a typed dynamic-focus plan with explicit divisor/remainder and
  source location. Unchanged Lotus `numbering45` composes the predicate with
  the existing modulo-aware `xsl:number` count pattern and becomes exact,
  raising the lower bound from 1,412 to 1,413 without a new mismatch or
  execution failure.
  [Evidence](../Evidence/oasis-xslt10-position-modulo-boolean-2026-09-14.md)
- 2026-09-14 -- Literal-QName XSLT 1.0 implementation introspection now folds
  during stylesheet compilation. `system-property()` supplies the required
  XSLT properties, `function-available()` reports the admitted function
  surface, and `element-available()` resolves the expression site's default
  namespace before reporting admitted instructions. Fifteen unchanged cases
  enter execution: thirteen become exact XML results and two vendor-property
  cases retain their upstream manual-comparison disposition. The exact lower
  bound rises from 1,413 to 1,426 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-static-introspection-2026-09-14.md)
- 2026-09-14 -- Static XSLT 1.0 `substring()` calls now preserve XPath 1.0
  NaN and positive/negative infinity behavior when their numeric operands are
  bounded literal divisions. Five unchanged Lotus cases become exact, raising
  the lower bound from 1,426 to 1,431 without broadening the modern fold or
  adding a mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-non-finite-substring-2026-09-14.md)
- 2026-09-14 -- Bounded relative child paths on both sides of a predicate
  equality now use XPath 1.0 node-set general-comparison semantics: any pair
  of selected nodes with equal string values satisfies the predicate. Four
  Microsoft namespace cases become exact; relaxed recognition also admits the
  already-typed spaced `. = 'literal'` spelling in Lotus `output25`. Microsoft
  `Miscellaneous__84427` advances to its later explicit HTML-serialization
  boundary. The exact lower bound rises from 1,431 to 1,436.
  [Evidence](../Evidence/oasis-xslt10-child-node-set-comparison-2026-09-14.md)
- 2026-09-14 -- The bounded source-free composition
  `number(string(static-decimal)) = static-decimal` now folds under XSLT 1.0
  compatibility rules. Unchanged Lotus `math16` becomes exact, raising the
  lower bound from 1,436 to 1,437 while modern and dynamic nested conversions
  remain rejected.
  [Evidence](../Evidence/oasis-xslt10-nested-static-conversion-2026-09-14.md)
- 2026-09-15 -- Typed lexical context-name comparison now serves both path
  predicates and instruction conditions, while context string-value
  inequality reuses the existing equality plan through negation. Unchanged
  Lotus `sort_sort37` and `node_node15` become exact, raising the lower bound
  from 1,437 to 1,439 without adding a mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-lexical-context-comparison-2026-09-15.md)
- 2026-09-15 -- Relative child and child/final-attribute node sets now compare
  with string literals under bounded XPath 1.0 existential equality and
  inequality semantics. Two unchanged result-tree cases become exact,
  including Lotus `select_select51`, raising the lower bound from 1,439 to
  1,441 without adding a mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-child-node-set-literal-comparison-2026-09-15.md)
- 2026-09-15 -- Literal-result AVTs now preserve the context node's retained
  lexical QName for `name()`/`name(.)` and fold one static quoted XPath string
  expression. Unchanged Lotus `lre_lre06` becomes exact, raising the lower
  bound from 1,441 to 1,442; Lotus `axes_axes129` advances only to its later
  namespace-axis frontier and remains excluded from the pass count.
  [Evidence](../Evidence/oasis-xslt10-literal-result-avt-scalars-2026-09-15.md)
- 2026-09-15 -- A bounded typed AVT can now compose ordered static text and
  multiple already-admitted source paths. Microsoft `AVTs__77582` initializes
  and executes with the expected brace-bearing attribute values, but remains a
  visible XML-comparison mismatch because of serializer-added whitespace
  between expected top-level fragment elements. The exact lower bound remains
  1,442; the comparator was not weakened.
  [Evidence](../Evidence/oasis-xslt10-multi-path-avt-2026-09-15.md)
- 2026-09-15 -- The shared typed location-path evaluator now applies XPath 1.0
  node-set effective boolean value to `[text()]` and `child::text()` predicates.
  Unchanged Microsoft `AVTs__77570` becomes exact through the bounded
  multi-path AVT lane, raising the lower bound from 1,442 to 1,443 without a
  new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-text-child-predicate-2026-09-15.md)
- 2026-09-15 -- One bounded multi-part AVT expression may now reuse the shared
  controlled path-union evaluator, preserving document-order normalization,
  duplicate removal, traversal charging, and first-node string conversion.
  Unchanged Microsoft `AVTs__77571` becomes exact, raising the lower bound from
  1,443 to 1,444 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-avt-path-union-2026-09-15.md)
- 2026-09-15 -- Relative child-path comparison now retains a typed terminal
  text-node step, and a single-expression AVT may emit the lexical result of a
  bounded path-to-string equality or inequality. Microsoft `AVTs__77564`
  executes with the expected attribute values but remains a visible mismatch
  because of expected inter-element fragment formatting. The exact lower bound
  remains 1,444.
  [Evidence](../Evidence/oasis-xslt10-text-node-comparison-avt-2026-09-15.md)
- 2026-09-15 -- The private typed AVT scanner now admits one dynamic path when
  accompanied by an escaped literal brace, and permits one nonempty outer pair
  of parentheses around that path. Three unchanged Microsoft cases execute
  with the expected attribute values but remain visible fragment-format
  mismatches. The exact lower bound remains 1,444 and malformed brace forms
  remain rejected.
  [Evidence](../Evidence/oasis-xslt10-parenthesized-path-and-escaped-brace-avt-2026-09-15.md)
- 2026-09-15 -- Bounded multi-part AVTs now retain typed binary numeric parts
  compiled and executed by the shared controlled XSLT 1.0 exact-rational
  machinery. Microsoft `AVTs__77576` produces all expected decimal values but
  remains a visible fragment-format mismatch. The exact lower bound remains
  1,444.
  [Evidence](../Evidence/oasis-xslt10-numeric-composition-avt-2026-09-15.md)
- 2026-09-15 -- Single-level numbering now distinguishes a context node that
  matches the count pattern from a counted ancestor: the former remains
  eligible without a matching `from` ancestor, while the latter still requires
  the boundary. Lotus `numbering_numbering20` becomes exact; doubts-annotated
  Microsoft `Number__84687`, whose suite metadata calls the behavior a gray
  area referred for an erratum, moves to a visible legacy-expectation mismatch.
  The conserved exact lower bound remains 1,444.
  [Evidence](../Evidence/oasis-xslt10-single-number-self-before-from-boundary-2026-09-15.md)
- 2026-09-15 -- Namespace-qualified foreign top-level stylesheet data is now
  ignored without interpretation, retention, execution, or new authority;
  unqualified top-level literals remain invalid. Three unchanged OASIS cases
  become exact, one becomes a visible mismatch, and four advance to later
  explicit boundaries, raising the lower bound from 1,444 to 1,447.
  [Evidence](../Evidence/oasis-xslt10-foreign-top-level-data-2026-09-15.md)
- 2026-09-15 -- One bounded computed-attribute constructor now retains an
  existing controlled path for `xsl:for-each` with exactly one
  `xsl:value-of select="."` body. Execution concatenates selected source-node
  string values while preserving path, string traversal, instruction, budget,
  and cancellation controls. Unchanged Lotus `attribset_attribset25` becomes
  exact, raising the lower bound from 1,447 to 1,448 without a new mismatch or
  execution failure.
  [Evidence](../Evidence/oasis-xslt10-computed-attribute-for-each-2026-09-15.md)
- 2026-09-15 -- The same compile-validated `xsl:for-each` plus
  `xsl:value-of select="."` path constructor now supplies content-valued named
  template arguments. Runtime evaluation materializes one parentless temporary
  text node, preserving XSLT 1.0 result-tree-fragment behavior and all existing
  work controls. Unchanged Lotus `namedtemplate_namedtemplate11` becomes exact,
  raising the lower bound from 1,448 to 1,449.
  [Evidence](../Evidence/oasis-xslt10-template-argument-for-each-2026-09-15.md)
- 2026-09-15 -- Content-valued named-template arguments now admit a bounded
  static mixed text/literal-result-element constructor. The compiler reuses the
  existing private constructed-node plan and each invocation materializes one
  fresh temporary tree, while dynamic instructions and attributes remain
  unsupported. Unchanged Lotus `copy_copy08` becomes exact, raising the lower
  bound from 1,449 to 1,450 without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-static-template-argument-tree-2026-09-15.md)
- 2026-09-15 -- Computed attributes now admit one bounded constructor-local
  source-path variable followed by `count($same-name)`. The private plan retains
  only the controlled path, preserves surrounding lexical shadowing, and does
  not establish a general local-frame contract. Unchanged Microsoft
  `Variables__78409` becomes exact, raising the lower bound from 1,450 to 1,451
  without a new mismatch or execution failure.
  [Evidence](../Evidence/oasis-xslt10-computed-attribute-local-count-2026-09-15.md)
- 2026-09-15 -- A first attribute-set slice resolves uniquely named local sets
  with static text values into literal-result-element plans during compilation.
  Set, literal, and explicit child-attribute precedence is preserved; dynamic
  values and same-name include/import composition remain unsupported rather
  than producing partial results. Seven unchanged Lotus cases become exact,
  raising the lower bound from 1,451 to 1,458 with no new XML mismatch.
  [Evidence](../Evidence/oasis-xslt10-local-static-attribute-sets-2026-09-15.md)
- 2026-09-15 -- The same compile-time local static attribute-set resolver now
  serves statically named `xsl:element`, preserving explicit child-attribute
  override and existing namespace fixup. Three unchanged cases become exact,
  raising the lower bound from 1,458 to 1,461 with no new XML mismatch.
  [Evidence](../Evidence/oasis-xslt10-static-computed-element-attribute-sets-2026-09-15.md)
- 2026-09-15 -- Local static attribute sets now form a compile-validated
  dependency graph. Referenced sets expand before the referring set, final
  consumers retain higher precedence, undefined names report `XTSE0710`, and
  direct or indirect cycles report `XTSE0720`. Seven unchanged cases become
  exact, raising the lower bound from 1,461 to 1,468 without a runtime registry
  or a new XML mismatch.
  [Evidence](../Evidence/oasis-xslt10-inherited-static-attribute-sets-2026-09-15.md)
- 2026-09-16 -- `xsl:copy` now consumes the same immutable compile-time static
  attribute-set expansion, with explicit child attributes overriding set
  values. Document-node copy emits the set constructors as pending attributes
  before its body, preserving XSLT 1.0 root-copy behavior. Six unchanged cases
  become exact, raising the lower bound from 1,468 to 1,474 without a new XML
  mismatch, execution failure, or runtime registry.
  [Evidence](../Evidence/oasis-xslt10-source-copy-attribute-sets-2026-09-16.md)
- 2026-09-16 -- Multiple same-name local `xsl:attribute-set` declarations now
  compose at compilation in document order. Each declaration applies its
  referenced sets before its own values, later declarations override earlier
  values, and the consuming instruction retains final precedence. Complete
  graph validation still diagnoses undefined references and cycles, while
  same-name cross-module composition remains explicit. Eight unchanged Lotus
  cases become exact, raising the lower bound from 1,474 to 1,482 without a new
  XML mismatch, execution failure, runtime registry, or panic.
  [Evidence](../Evidence/oasis-xslt10-composed-local-attribute-sets-2026-09-16.md)
- 2026-09-16 -- Attribute-set computed values may now reference one declared
  global atomic variable or parameter. Compilation records the reference as
  global, so a same-named local binding at the consuming instruction cannot
  capture it. Unchanged Lotus `attribset44` becomes exact, raising the lower
  bound from 1,482 to 1,483 without a new mismatch, execution failure, runtime
  attribute-set registry, or panic.
  [Evidence](../Evidence/oasis-xslt10-attribute-set-global-variable-scope-2026-09-16.md)
- 2026-09-16 -- XSLT 1.0 same-expanded-name result attributes now select the
  last statically compiled value, covering computed-over-literal and repeated
  leading computed attributes. The compatibility choice is made during
  compilation; later versions retain `XTDE0410`, and dynamic late-attribute
  failures are unchanged. Three unchanged cases become exact, raising the lower
  bound from 1,483 to 1,486 without a new mismatch, execution failure, or
  runtime version branch.
  [Evidence](../Evidence/oasis-xslt10-duplicate-result-attribute-recovery-2026-09-16.md)
- 2026-09-16 -- Statically evident XSLT 1.0 `xsl:attribute` constructors after
  result child construction now use the specification's permitted recovery and
  are ignored. Later versions retain `XTDE0410`, and dynamically late pending
  attributes retain the runtime error. Five unchanged standard-operation cases
  become exact, raising the lower bound from 1,486 to 1,491 while expected-error
  accounting remains unchanged and no mismatch, execution failure, or panic is
  added.
  [Evidence](../Evidence/oasis-xslt10-late-result-attribute-recovery-2026-09-16.md)
- 2026-09-16 -- Principal simplified stylesheets now compile their document
  element as the implicit root template through the ordinary literal-result
  compiler. All 30 former blanket `FXST0009` candidates advance to their real
  semantic boundary: 21 remain initialization failures, eight reach an
  execution/serialization failure, and one reaches a visible mismatch. No
  exact pass is claimed, the lower bound remains 1,491, and the six unexpected
  expected-error successes remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-simplified-stylesheet-frontier-2026-09-16.md)
- 2026-09-16 -- An `xsl:version="1.0"` control attribute on a simplified
  stylesheet root now establishes the same typed value-expression compatibility
  context as a conventional stylesheet root. The existing first-node
  conversion plan makes unchanged Microsoft `Namespace__78215` exact, raising
  the lower bound from 1,491 to 1,492; the higher-version forward-compatibility
  companion remains explicit.
  [Evidence](../Evidence/oasis-xslt10-simplified-stylesheet-value-conversion-2026-09-16.md)
- 2026-09-16 -- Static comment and processing-instruction constructors now
  fold explicit `xsl:text` children, ignoring disable-output-escaping only in
  those node-construction contexts where it has no effect. One unchanged
  Microsoft comment case becomes exact, raising the lower bound from 1,492 to
  1,493; one simplified case advances to the existing HTML serializer boundary.
  [Evidence](../Evidence/oasis-xslt10-static-comment-pi-text-2026-09-16.md)
- 2026-09-16 -- `xsl:text` now accepts and validates its legal `xml:space`
  attribute, and static comment/processing-instruction constructors fold
  bounded context-free `xsl:value-of` literal, `concat()`, and `substring()`
  expressions. Unchanged Lotus `output_output54` becomes exact, raising the
  lower bound from 1,493 to 1,494. Two `xml:space` cases advance only to their
  real serializer or ordinary disable-output-escaping boundary; dynamic
  constructor content remains explicit.
  [Evidence](../Evidence/oasis-xslt10-static-node-content-and-text-space-2026-09-16.md)
- 2026-09-16 -- Static comment construction now applies XSLT 1.0's permitted
  add-space recovery when content contains `--` or ends in `-`. The recovery is
  selected during compilation; later versions retain `FXST1037`. All ten
  formerly blocked unchanged cases become exact, raising the lower bound from
  1,494 to 1,504 without a new mismatch, failure, unexpected success, runtime
  version branch, or panic.
  [Evidence](../Evidence/oasis-xslt10-comment-delimiter-recovery-2026-09-16.md)
- 2026-09-16 -- Literal result elements now apply their namespaced
  `xsl:exclude-result-prefixes` control through the existing compiled namespace
  selection path. Fourteen cases leave `FXST1007`: eight become exact, raising
  the lower bound from 1,504 to 1,512; two expose later discretionary
  indentation mismatches after producing the required namespace nodes;
  and four reach the explicit namespace-alias boundary. The control attribute
  is not copied to the result and no runtime policy branch is introduced.
  [Evidence](../Evidence/oasis-xslt10-literal-result-prefix-exclusion-2026-09-16.md)
- 2026-09-17 -- The stylesheet compiler now rejects retained non-whitespace
  top-level text rather than silently skipping non-element children. Unchanged
  Microsoft `Stylesheet_InvalidStylesheetMustThrowException` moves from
  unexpected success to observed initialization failure, reducing that class
  from six to five while the exact-result lower bound remains 1,512.
  [Evidence](../Evidence/oasis-xslt10-top-level-text-rejection-2026-09-17.md)
- 2026-09-17 -- `exclude-result-prefixes` is now validated at each declaration
  site, so a prefix declared only by an included module cannot satisfy an
  unbound token on the principal stylesheet. Unchanged Microsoft
  `Include_Include_ParentExplicitlyExcludesChildNamespace` moves from
  unexpected success to static `XTSE0808`, reducing that class from five to
  four while the exact-result lower bound remains 1,512. All four survivors are
  upstream doubt-marked and remain visible rather than being forced to fail.
  [Evidence](../Evidence/oasis-xslt10-excluded-prefix-validation-2026-09-17.md)
- 2026-09-17 -- Literal result elements now apply XSLT 1.0 handling for
  unknown XSLT-namespace attributes and lexically scoped
  `extension-element-prefixes`. Declarations are validated at their sites,
  extension namespaces are excluded unless a result name requires them, and
  actual extension execution remains explicit unsupported. Three unchanged
  cases become exact, raising the lower bound from 1,512 to 1,515; two more
  advance to later honest boundaries without adding a mismatch, execution
  failure, unexpected success, runtime version branch, or panic.
  [Evidence](../Evidence/oasis-xslt10-literal-result-extension-controls-2026-09-17.md)
- 2026-09-17 -- Foreign namespaced extension attributes on admitted XSLT
  elements are now ignored under XSLT 1.0 compatibility, while unqualified,
  XSLT-namespace, reserved XML, and modern-version attributes retain their
  existing validation. Two unchanged cases become exact, raising the lower
  bound from 1,515 to 1,517 without adding a mismatch, execution failure,
  unexpected success, runtime extension hook, or panic.
  [Evidence](../Evidence/oasis-xslt10-extension-attributes-2026-09-17.md)
- 2026-09-17 -- Ordinary XSLT instruction validation now admits and validates
  `xml:space`, connecting it to the existing inherited stylesheet-text
  preservation path. Two unchanged Microsoft cases advance from static
  unsupported to visible output mismatches; the exact lower bound remains
  1,517. The slice is retained as honest frontier movement and does not claim
  complete preserved-whitespace behavior for every structural content model.
  [Evidence](../Evidence/oasis-xslt10-stylesheet-xml-space-admission-2026-09-17.md)
- 2026-09-17 -- `xsl:value-of disable-output-escaping="no"` now uses the
  ordinary semantic-result path; `yes` remains explicit unsupported and other
  lexicals are invalid. Unchanged Lotus `output_output07` becomes exact,
  raising the lower bound from 1,517 to 1,518 without adding a result-node
  escape flag, mismatch, failure, runtime version branch, or panic.
  [Evidence](../Evidence/oasis-xslt10-value-of-disabled-output-escaping-2026-09-17.md)
- 2026-09-17 -- Static `xsl:number letter-value` is now admitted only where the
  existing ASCII token semantics are equivalent: traditional formatting, or
  alphabetic formatting over Latin alphabetic tokens. Affected unchanged cases
  advance to their real non-ASCII or compound-token boundaries; the exact lower
  bound remains 1,518. Dynamic, locale-sensitive, and non-equivalent semantics
  remain explicit without a runtime branch.
  [Evidence](../Evidence/oasis-xslt10-static-letter-value-admission-2026-09-17.md)
- 2026-09-17 -- An XSLT 1.0 local variable whose complete constructor is one
  admitted `xsl:value-of` path now materializes through the existing charged,
  invocation-owned temporary-tree path. Variable-valued named-template
  arguments preserve that temporary-tree value kind rather than reporting the
  binding as absent. Five unchanged Lotus named-template cases become exact,
  raising the lower bound from 1,518 to 1,523 without admitting general
  sequence constructors, later-version result-tree-fragment behavior,
  cross-invocation sharing, a new mismatch, or a panic.
  [Evidence](../Evidence/oasis-xslt10-local-value-of-temporary-tree-2026-09-17.md)
- 2026-09-17 -- Local and global XSLT 1.0 bindings made entirely from static
  text and `xsl:text` now fold into the existing temporary text-tree
  representation. The exact nested
  `string-length(string($variable)) * positive-integer` shape composes through
  controlled codepoint counting. Four cases leave generic `FXST1015`: three
  reach the retained disable-output-escaping boundary and Microsoft
  `BVTs_bvt085` executes to a visible archival replacement-character mismatch.
  The exact lower bound remains 1,523; no expected bytes are rewritten and no
  general constructor or expression-composition claim is made.
  [Evidence](../Evidence/oasis-xslt10-static-text-tree-sequence-2026-09-17.md)
- 2026-09-17 -- Qualified `//` and `.//` paths now retain the existing typed
  document- or context-descendant origin, while prefixed element and attribute
  namespace wildcards compile to exact namespace tests. Six unchanged cases
  leave the generic path frontier: two become exact, raising the lower bound
  from 1,523 to 1,525; two expose visible later whitespace/serialization
  mismatches; and two reach independent expression or numbering boundaries.
  Prefix validation, charged traversal, cancellation, document order, and
  source identity remain shared with the ordinary path evaluator.
  [Evidence](../Evidence/oasis-xslt10-qualified-descendant-paths-2026-09-17.md)
- 2026-09-17 -- Static `xsl:namespace-alias` declarations now resolve ordinary
  and `#default` prefixes during compilation and rewrite literal result names
  and namespace bindings before execution. The 43-case blanket frontier is
  eliminated: 27 additional cases initialize, 22 execute, and eight become
  exact, raising the lower bound from 1,525 to 1,533. Fourteen discretionary or
  later semantic mismatches and five later execution failures remain visible;
  general include/import alias precedence is not inferred.
  [Evidence](../Evidence/oasis-xslt10-static-namespace-alias-2026-09-17.md)
- 2026-09-17 -- Unnamed `xsl:decimal-format` declarations now compose and
  validate during compilation, then specialize the existing two-argument
  `format-number()` plan without adding runtime stylesheet-policy lookup. The
  first corpus pass exposed and repaired the configured-minus-sign rule for
  identical negative subpictures. Twenty additional cases initialize and
  sixteen become exact, raising the lower bound from 1,533 to 1,549. Named
  formats, non-ASCII digit families, dynamic third-argument lookup, and general
  include/import decimal-format composition remain explicit rather than
  approximated.
  [Evidence](../Evidence/oasis-xslt10-unnamed-decimal-format-2026-09-17.md)
- 2026-09-17 -- Named decimal-format declarations and literal third-argument
  QNames now resolve in their static namespace contexts and specialize the
  same direct formatting plan. Eleven additional cases initialize and all nine
  newly executing ordinary cases compare exactly, raising the lower bound from
  1,549 to 1,558. Undeclared and cross-module format lookup remains explicit;
  computed names and non-ASCII digit families are not approximated.
  [Evidence](../Evidence/oasis-xslt10-static-named-decimal-format-2026-09-17.md)
- 2026-09-17 -- Decimal formatting now applies XSLT 1.0's consecutive-character
  digit-family rule inside numeric output only. Five more cases initialize and
  one becomes exact, raising the lower bound from 1,558 to 1,559; four reach
  later explicit runtime boundaries.
  [Evidence](../Evidence/oasis-xslt10-decimal-digit-family-2026-09-17.md)
- 2026-09-17 -- The shared HTML serializer now admits general semantic result
  trees instead of using corpus-shaped production whitelists, and an absent
  output method selects legacy HTML when the first significant result element
  is an unnamespaced `html`. All 119 former `FXSR1001` execution failures now
  expose their real disposition: five exact matches raise the lower bound from
  1,559 to 1,564, while 72 mismatches, 41 XML-comparator gaps, and one missing
  expected error remain visible. This is result-tree admission and frontier
  discovery, not a claim of complete HTML serialization conformance.
  [Evidence](../Evidence/oasis-xslt10-general-html-result-admission-2026-09-17.md)
- 2026-09-17 -- Literal-result AVTs now validate escaped, quoted, empty,
  unmatched, and nested brace structure before expression capability
  selection. Eight unchanged malformed cases move from generic unsupported to
  static `XTSE0370 / invalid`; valid but unimplemented expressions continue to
  report `FXST1031`. The exact lower bound remains 1,564.
  [Evidence](../Evidence/oasis-xslt10-malformed-avt-classification-2026-09-17.md)
- 2026-09-17 -- HTML serialization now minimizes the XSLT 1.0 boolean-
  attribute set when an unnamespaced value equals its name case-insensitively.
  `Lotus/attribset_attribset17#1` reaches the expected minimized attribute
  spelling, but the XML-semantic comparator cannot parse that legal HTML form;
  the 1,564 exact lower bound therefore remains unchanged and the harness gap
  stays visible.
  [Evidence](../Evidence/oasis-xslt10-html-boolean-attribute-minimization-2026-09-17.md)
- 2026-09-17 -- HTML URI-attribute recognition now uses the standard
  element/attribute pairs instead of treating every `href` as URI-valued. URI
  serialization percent-encodes an embedded double quote as `%22` while
  preserving apostrophes and existing percent escapes. The unchanged
  `Lotus/output_output70#1` case moves from mismatch to exact comparison,
  raising the strict lower bound from 1,564 to 1,565 without changing any
  execution or comparator denominator.
  [Evidence](../Evidence/oasis-xslt10-html-uri-attribute-escaping-2026-09-17.md)
- 2026-09-17 -- Ordinary HTML attributes now use an HTML-specific escaping
  path that preserves angle brackets and XML whitespace characters while
  retaining escaped ampersands, quotes, and C1 controls. The unchanged
  `Microsoft/Output_HtmlOutputWithLessThanInAttribute#1` and
  `Microsoft/Output_EntityRefInAttribHtml#1` outputs reach their expected
  attribute spellings but retain later pretty-print/newline differences. They
  remain visible mismatches, and the strict lower bound remains 1,565.
  [Evidence](../Evidence/oasis-xslt10-html-ordinary-attribute-escaping-2026-09-17.md)
- 2026-09-17 -- The XSLT 1.0 no-character-map HTML path now preserves `&`
  immediately before `{`, and legacy HTML void elements omit their end tags
  even when result-tree content follows their start tag. The unchanged
  `Lotus/output_output37#1` and
  `Microsoft/Output_HtmlOutputWithAmpersandCurlyBracket#1` cases reach the
  relevant expected HTML spellings. Non-XML comparison and independent
  presentation differences remain visible, so the exact lower bound remains
  1,565.
  [Evidence](../Evidence/oasis-xslt10-html-ampersand-curly-and-void-content-2026-09-17.md)
- 2026-09-18 -- Exact `xsl:element name="{name()}"` and `name="{name(.)}"`
  forms now retain the instruction's static namespace context and resolve the
  context node's lexical QName at execution. Seven cases leave the generic
  `FXST1047` frontier; one unchanged expected-error case now reaches and
  reports runtime `XTDE0830` for a source prefix unavailable in that static
  context. The exact lower bound remains 1,565, and arbitrary name/namespace
  AVTs remain unsupported.
  [Evidence](../Evidence/oasis-xslt10-context-name-computed-element-2026-09-18.md)
- 2026-09-18 -- Structurally complete `format-number()` calls now classify
  top-level arity before operand and picture capability. Four unchanged OASIS
  cases with zero, one, or four arguments move from generic `FXXP1009 /
  unsupported` to static `XPST0017 / invalid`; ten valid-arity unsupported
  cases remain explicit. Aggregate lifecycle and comparison counts are
  unchanged, including the 1,565 exact-result lower bound.
  [Evidence](../Evidence/oasis-xslt10-format-number-arity-classification-2026-09-18.md)
- 2026-09-18 -- Exact string-literal AVTs on `xsl:sort` `data-type` and
  `order` now fold into the existing typed sort plan at compilation. Two
  unchanged Microsoft cases advance to a later computed-name boundary, while
  one variable-valued Lotus order is now honestly unsupported instead of
  invalid. The net `FXST1044` frontier falls from 17 to 16; aggregate counts
  and the 1,565 exact lower bound remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-static-sort-control-avts-2026-09-18.md)
- 2026-09-18 -- Static malformed `xsl:element` names now report structured
  `XTDE0820 / invalid` rather than masquerading as dynamic-name capability
  gaps. Ten unchanged OASIS cases leave `FXST1047`, reducing that frontier
  from 23 to 13; all survivors contain genuinely dynamic AVTs. Aggregate
  lifecycle counts and the 1,565 exact-result lower bound remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-static-computed-element-qname-errors-2026-09-18.md)
- 2026-09-18 -- An `xsl:element` name AVT containing exactly one admitted
  location path now lowers to a typed instruction, evaluates through the
  charged XPath owner, applies XSLT 1.0 first-node string conversion, and
  resolves the resulting lexical `QName` against retained static namespaces.
  Eleven cases leave `FXST1047`, reducing that frontier from 13 to 2. Four
  initialize, three execute, one reports the expected runtime `XTDE0820`, and
  `Lotus/lre_lre08#1` raises the strict exact-result lower bound from 1,565 to
  1,566. Variable, composite, predicate/function, and dynamic-namespace forms
  remain explicit.
  [Evidence](../Evidence/oasis-xslt10-path-valued-computed-element-names-2026-09-18.md)
- 2026-09-18 -- Exact static-text-plus-`position()` computed-element names now
  reuse the established sequence focus and the same private runtime `QName`
  resolver as path-valued names. The path and focus-position forms were
  consolidated behind one typed dynamic-name representation after source-unit
  pressure rejected parallel instruction branches. The unchanged Microsoft
  case becomes exact, raising the strict lower bound from 1,566 to 1,567 and
  reducing `FXST1047` from 2 to 1; variable/path and general composite AVTs
  remain unsupported.
  [Evidence](../Evidence/oasis-xslt10-focus-position-computed-element-name-2026-09-18.md)
- 2026-09-18 -- Exact one-path XSLT 1.0 computed-attribute name AVTs now reuse
  the charged location-path evaluator and first-node string conversion, then
  apply attribute-specific runtime `QName` rules. Static namespace overrides
  are retained on the containing result element during compilation. Five cases
  leave `FXST1062`; two become exact, raising the lower bound from 1,567 to
  1,569, while the other three expose independent later boundaries.
  [Evidence](../Evidence/oasis-xslt10-path-valued-computed-attribute-names-2026-09-18.md)
- 2026-09-18 -- The private computed-attribute name plan now also admits exact
  `name()` / `name(.)` and one-expression string-literal AVTs. Two cases leave
  `FXST1062`: one reaches exact runtime `XTDE0855`, while the other reaches the
  independent result-attribute attachment boundary. Neither receives pass
  credit; the lower bound remains 1,569 and five variable-composition cases
  remain explicit.
  [Evidence](../Evidence/oasis-xslt10-context-and-literal-computed-attribute-names-2026-09-18.md)
- 2026-09-18 -- Computed-attribute names made only from static text and
  unqualified variable references now reuse the existing invocation/global
  variable frame and attribute `QName` validator. The final five cases leave
  `FXST1062`, eliminating that frontier. They expose two empty-attribute-set
  errors, two UTF-16 serialization boundaries, and one correctly unbound local
  variable; no new pass is credited and the lower bound remains 1,569.
  [Evidence](../Evidence/oasis-xslt10-variable-composed-computed-attribute-names-2026-09-18.md)
- 2026-09-18 -- Bounded static `xsl:key` declarations now compile into
  immutable stylesheet-owned expanded names, match patterns, and `use` paths,
  compose across admitted module graphs, and participate in exact retained-
  capacity accounting. Variable and recursive-key `use` expressions report
  `XTSE1205`. All 90 cases leave the blanket top-level declaration frontier;
  three become exact, raising the lower bound from 1,569 to 1,572, while 85
  function-shaped lookup cases remain explicit pending invocation-owned index
  semantics.
  [Evidence](../Evidence/oasis-xslt10-static-key-declaration-admission-2026-09-18.md)
- 2026-09-18 -- Literal-name/literal-value `key()` calls in the private
  `xsl:value-of` path now execute through a complete charged source scan. The
  scan reuses compiled key match/`use` semantics, composes same-name
  declarations, preserves document order and node identity, and remains the
  safe oracle for any later index. Thirteen unchanged cases become exact,
  raising the strict lower bound from 1,572 to 1,585; an undeclared key reports
  runtime `XTDE1260`, while 11 dynamic/nonliteral forms remain explicit as
  `FXXP1023`. No index, cross-invocation retention, or new authority is added.
  [Evidence](../Evidence/oasis-xslt10-literal-key-lookup-reference-2026-09-18.md)
- 2026-09-18 -- The typed key-use plan now admits constant strings and
  `number(location-path)`, while finite source-free numeric lookup values reuse
  the exact arithmetic compiler. Two unchanged Lotus cases become exact,
  raising the strict lower bound from 1,585 to 1,587. Initialization reaches
  1,971 cases and execution reaches 1,868; expected-error accounting is
  unchanged. Dynamic values, node sets, unions, context-dependent arithmetic,
  and retained indexes remain outside the slice.
  [Evidence](../Evidence/oasis-xslt10-static-atomic-key-values-2026-09-18.md)
- 2026-09-18 -- The complete charged key scan now has one private node-
  selection owner shared by value conversion, `xsl:for-each`, and
  `xsl:apply-templates`. A focused oracle confirms additive declarations,
  document order, node identity, and identical focus across both node-set
  consumers. Four unchanged cases become exact, raising the strict lower bound
  from 1,587 to 1,591; initialization reaches 1,979 and execution reaches
  1,875. No source-derived index or wider dynamic-key surface is admitted.
  [Evidence](../Evidence/oasis-xslt10-key-node-selection-2026-09-18.md)
- 2026-09-18 -- `xsl:copy-of` now consumes the same private charged key node
  selection and the existing source deep-copy owner. The unchanged Lotus
  `copy30` case becomes exact, raising the lower bound from 1,591 to 1,592;
  initialization reaches 1,980 and execution reaches 1,876. No index,
  alternate copy semantics, or wider key grammar is introduced.
  [Evidence](../Evidence/oasis-xslt10-key-copy-of-2026-09-18.md)
- 2026-09-18 -- The private key selection now applies exact positional
  predicates before an optional path tail: integer position,
  `position()=N`, `last()`, and `last()=position()`. Thirteen unchanged cases
  become exact, raising the lower bound from 1,592 to 1,605; initialization
  reaches 1,993 and execution reaches 1,889. General predicates and dynamic key
  arguments remain explicit.
  [Evidence](../Evidence/oasis-xslt10-positional-key-selection-2026-09-18.md)
- 2026-09-18 -- A literal-name key lookup may now take a variable second
  argument. Atomic and temporary-tree values reuse the existing XSLT 1.0
  conversion owner; source-node-set values contribute every node string value.
  One Microsoft case becomes exact and one reaches an independent HTML
  indentation mismatch, raising the lower bound from 1,605 to 1,606 while two
  cases leave initialization failure. Dynamic key names remain explicit.
  [Evidence](../Evidence/oasis-xslt10-variable-key-values-2026-09-18.md)
- 2026-09-18 -- A literal-name key lookup may now take an existing typed
  context location path as its second argument. Every selected node string
  value participates in lookup. One Lotus case becomes exact and one Microsoft
  case reaches independent runtime `XTDE0410`, raising the lower bound from
  1,606 to 1,607 while two cases leave initialization failure. Arbitrary
  expressions and cross-document key context remain explicit.
  [Evidence](../Evidence/oasis-xslt10-context-path-key-values-2026-09-18.md)
- 2026-09-18 -- The typed XSLT 1.0 `count()` value-expression consumer now
  reuses the same charged key node selector as the other admitted consumers.
  The unchanged Lotus `idkey15` case becomes exact, raising the strict lower
  bound from 1,607 to 1,608; initialization reaches 1,998 and execution reaches
  1,893. Key expressions in AVTs, conditions, sorting, unions, and match
  patterns remain explicit.
  [Evidence](../Evidence/oasis-xslt10-count-key-selection-2026-09-18.md)
- 2026-09-18 -- Literal-result AVTs may now compose optional static text with
  `generate-id(key(...))`, reusing the shared charged key selector and stable
  principal-source node identity. Focused matching and empty-selection cases
  pass. The two motivating Microsoft cases advance to an independent
  zero-argument `generate-id()` AVT boundary, so the lower bound remains 1,608.
  General function-valued AVTs remain explicit.
  [Evidence](../Evidence/oasis-xslt10-generated-key-identity-avt-2026-09-18.md)
- 2026-09-18 -- The ordered key selector now admits one exact literal
  attribute-equality predicate, resolves its QName statically, and charges each
  inspected source attribute. The unchanged composed-module Microsoft case
  becomes exact, raising the strict lower bound from 1,608 to 1,609;
  initialization reaches 1,999 and execution reaches 1,894. General key
  predicates remain explicit.
  [Evidence](../Evidence/oasis-xslt10-key-attribute-predicate-2026-09-18.md)
- 2026-09-18 -- The private apply/for-each selection owner now unions up to
  eight already typed key lookups. Every alternative uses the charged reference
  scan before source document-order normalization and identity deduplication.
  The unchanged Lotus `select55` case becomes exact, raising the strict lower
  bound from 1,609 to 1,610; initialization reaches 2,000 and execution reaches
  1,895. Mixed key/path unions and retained indexes remain explicit.
  [Evidence](../Evidence/oasis-xslt10-key-union-selection-2026-09-18.md)
- 2026-09-18 -- XSLT 1.0 sort expressions may now reuse the shared charged key
  selector from each candidate's context and apply existing first-node string
  conversion before ordinary sort typing. The unchanged Lotus `idkey32` and
  `idkey33` cases become exact, raising the strict lower bound from 1,610 to
  1,612; initialization reaches 2,002 and execution reaches 1,897. No sort-
  specific index or general function evaluator is added.
  [Evidence](../Evidence/oasis-xslt10-key-sort-selection-2026-09-18.md)
- 2026-09-18 -- Structurally complete `key()` calls with any arity other than
  two now report `XPST0017 / invalid` before operand capability selection.
  Three unchanged Microsoft expected-error cases leave the generic unsupported
  frontier and report the precise static failure. They were already counted as
  observed errors, so the 1,612 exact lower bound and other totals remain
  unchanged. Nested and dynamic two-argument forms remain explicit.
  [Evidence](../Evidence/oasis-xslt10-key-arity-classification-2026-09-18.md)
- 2026-09-18 -- A typed key lookup may now obtain its lookup values from
  another complete typed lookup under a compile-time nesting limit of four.
  Inner and outer reference scans remain independently charged and recursive
  plans participate in exact capacity accounting. The unchanged Lotus
  `idkey21` case becomes exact, raising the strict lower bound from 1,612 to
  1,613; initialization reaches 2,003 and execution reaches 1,898. Dynamic key
  names and arbitrary nested expressions remain explicit.
  [Evidence](../Evidence/oasis-xslt10-nested-key-selection-2026-09-18.md)
- 2026-09-18 -- A key name may now come from one unqualified variable
  reference. Its XSLT 1.0 string value is resolved as a lexical QName against
  immutable namespaces captured at the call site; invalid/unbound names report
  `XTDE1260`, resolution is charged, and the owned namespace slice is exactly
  capacity-accounted. The unchanged Lotus `idkey25` case becomes exact, raising
  the strict lower bound from 1,613 to 1,614; initialization reaches 2,004 and
  execution reaches 1,899. Arbitrary dynamic-name expressions remain explicit.
  [Evidence](../Evidence/oasis-xslt10-variable-key-name-2026-09-18.md)
- 2026-09-18 -- The runtime key-name plan now also consumes typed XSLT 1.0
  `concat()` expressions made only from literals and unqualified variable
  references. The unchanged Microsoft `91727` case initializes and executes
  with correct key selection, then exposes an independent HTML-indentation
  mismatch. Initialization reaches 2,005, execution reaches 1,900, and visible
  mismatches reach 215; the strict exact lower bound remains 1,614. General
  context-dependent dynamic-name expressions remain explicit.
  [Evidence](../Evidence/oasis-xslt10-concat-key-name-2026-09-18.md)
- 2026-09-18 -- Exact one-variable AVTs for `xsl:sort` `data-type` and `order`
  now retain typed variable controls and resolve them through charged XSLT 1.0
  string conversion once per sort invocation. The unchanged Lotus `sort32`
  and `sort33` cases become exact, raising the strict lower bound from 1,614 to
  1,616; initialization reaches 2,007 and execution reaches 1,902. General
  dynamic AVTs and language-sensitive collation remain explicit.
  [Evidence](../Evidence/oasis-xslt10-variable-sort-controls-2026-09-18.md)
- 2026-09-18 -- `xsl:for-each` over an invocation-owned source-node variable
  now retains sort keys and routes the node IDs through the ordinary stable,
  charged source sort. The unchanged Lotus `sort40` case becomes exact,
  raising the strict lower bound from 1,616 to 1,617; initialization reaches
  2,008 and execution reaches 1,903. Atomic-sequence sorting and a second
  evaluator are not admitted.
  [Evidence](../Evidence/oasis-xslt10-variable-selection-sort-2026-09-18.md)
- 2026-09-19 -- The typed key lookup now preserves `//tail` as a
  context-relative descendant path from every selected keyed node rather than
  accidentally lowering it as a document-rooted path. The unchanged Lotus
  `idkey34` case moves from mismatch to exact, raising the strict lower bound
  from 1,617 to 1,618 and reducing visible mismatches from 215 to 214. Existing
  charged path evaluation, ordering, deduplication, and key ownership remain
  unchanged.
  [Evidence](../Evidence/oasis-xslt10-key-descendant-tail-2026-09-19.md)
- 2026-09-19 -- `format-number()` variable operands now reuse the established
  charged XSLT 1.0 string conversion instead of seeing only local atomics and
  temporary trees. Seven unchanged Microsoft cases leave misleading unbound-
  variable failures: six reach the honest bounded dynamic-picture frontier and
  one reaches its independent encoding boundary. `FXRT0002` falls from 20 to
  13 while the strict exact lower bound remains 1,618. The repair does not
  expand the bounded formatter or select arbitrary-precision representation.
  [Evidence](../Evidence/oasis-xslt10-format-number-variable-conversion-2026-09-19.md)
- 2026-09-19 -- Literal-result AVTs now reuse the same charged XSLT 1.0
  variable string conversion, including global temporary trees and declared
  empty parameters, while retaining the AVT source location on failure. Five
  unchanged Microsoft cases execute; `AVTs__77536` becomes exact and four
  namespace-alias cases expose later comparison behavior. The strict lower
  bound rises from 1,618 to 1,619, successful execution reaches 1,908, and the
  `FXRT0002` frontier falls from 13 to 8.
  [Evidence](../Evidence/oasis-xslt10-global-tree-variable-avts-2026-09-19.md)
- 2026-09-19 -- A typed XSLT 1.0 `contains($haystack,$needle)` plan now
  converts exactly two variable operands through the shared charged string-
  value owner. The unchanged Lotus `string56` case becomes exact, raising the
  strict lower bound from 1,619 to 1,620; initialization reaches 2,009 and
  execution reaches 1,909. Modern function conversion and broader argument
  shapes remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-variable-contains-2026-09-19.md)
- 2026-09-19 -- A content-built XSLT 1.0 variable containing exactly one
  `xsl:value-of` now retains the shared typed `ValueExpression` rather than a
  path-only subset, evaluates it with the current focus and lexical frame, and
  materializes the result through the existing invocation-owned temporary text
  tree. The unchanged Lotus `namedtemplate08` case becomes exact, raising the
  strict lower bound from 1,620 to 1,621; initialization reaches 2,010 and
  execution reaches 1,910. General sequence constructors and modern semantics
  remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-value-of-tree-expression-2026-09-19.md)
- 2026-09-19 -- `sum($variable)` now requires an existing source-node variable
  and shares the charged node conversion, XPath numeric conversion,
  accumulation, and lexical formatting used by `sum(path)`. The unchanged,
  doubt-annotated Lotus `math84` case becomes exact, raising the strict lower
  bound from 1,621 to 1,622; initialization reaches 2,011 and execution reaches
  1,911. Temporary-tree node-set conversion and modern function conversion are
  not widened.
  [Evidence](../Evidence/oasis-xslt10-variable-sum-2026-09-19.md)
- 2026-09-19 -- The bounded XSLT 1.0 value compiler now composes streaming,
  charged `normalize-space()` over a source-node variable with the established
  Unicode-codepoint `translate()` implementation when both mapping operands
  are literals. The unchanged Lotus `string121` case becomes exact, raising the
  strict lower bound from 1,622 to 1,623; initialization reaches 2,012 and
  execution reaches 1,912. Dynamic mapping operands and general nested calls
  remain explicit.
  [Evidence](../Evidence/oasis-xslt10-normalized-variable-translate-2026-09-19.md)
- 2026-09-19 -- A typed path-translation plan now permits the search or
  replacement map to reuse the existing bounded XSLT 1.0 concat plan while
  retaining the smaller literal-only path. The unchanged Lotus `string138`
  and `string139` cases become exact, raising the strict lower bound from 1,623
  to 1,625; initialization reaches 2,014 and execution reaches 1,914. Dynamic
  function dispatch and non-path input values remain outside the slice.
  [Evidence](../Evidence/oasis-xslt10-concatenated-translate-maps-2026-09-19.md)
- 2026-09-19 -- `string($variable div $variable)` now selects a narrow XSLT
  1.0 double-division plan and emits the XPath lexical values for infinities
  and NaN without weakening the exact-rational evaluator's zero-divisor
  boundary. Two unchanged Microsoft variable cases initialize and execute with
  the required `Infinity`; initialization reaches 2,016 and execution reaches
  1,916. Both remain visibly blocked by the HTML comparator, so the strict
  exact lower bound remains 1,625.
  [Evidence](../Evidence/oasis-xslt10-variable-division-string-2026-09-19.md)
- 2026-09-19 -- Compiled decimal-format declarations now survive admitted
  module composition and specialize the whole program only after include/import
  merging. Dependency declarations bind principal calls and a principal
  declaration shadows a lower-precedence imported definition without runtime
  lookup. The unchanged Lotus `numberformat45` and `numberformat46` cases
  become exact, bringing the current strict lower bound to 1,805, successful
  execution to 1,934, and initialization to 2,033; visible mismatches remain
  54. Same-name declarations contributed by separate includes remain explicit.
  [Evidence](../Evidence/oasis-xslt10-module-decimal-format-composition-2026-09-19.md)
- 2026-09-19 -- Local XSLT 1.0 content variables that exceed the compact
  static/value-only constructors now execute the ordinary instruction sequence
  into an invocation-owned temporary tree. Materialization preserves result
  structure, receives fresh private identity, and charges every retained node
  before retention; modern content-variable semantics remain unchanged. The
  `FXST1015` initialization frontier falls from 30 to 21. Six unchanged cases
  become exact and three advance to honest later runtime frontiers, raising the
  strict lower bound to 1,811, successful execution to 1,940, and
  initialization to 2,042 while mismatches remain 54.
  [Evidence](../Evidence/oasis-xslt10-local-sequence-temporary-tree-2026-09-19.md)
- 2026-09-19 -- An XSLT 1.0 computed attribute containing exactly one
  path-valued `xsl:copy-of` now reuses the ordinary charged location-path
  evaluator. Selected text and attribute nodes contribute their string values;
  selected document, element, comment, and processing-instruction nodes are
  ignored by this bounded compatibility recovery. Three unchanged Lotus cases
  become exact, raising the strict lower bound to 1,814, successful execution
  to 1,943, and initialization to 2,045 while mismatches remain 54. Modern
  computed-attribute sequence construction remains explicitly unsupported.
  [Evidence](../Evidence/oasis-xslt10-copy-of-attribute-content-2026-09-19.md)
- 2026-09-19 -- The exact XSLT 1.0 boolean predicate
  `starts-with(translate(., literal, literal), literal)` now compiles to a
  typed compatibility plan over the current source node. It reuses the shared
  Unicode translation implementation and controlled source string-value path;
  modern nested string-function composition remains unsupported. The unchanged
  Microsoft `Miscellaneous__84430` case becomes exact, raising the strict lower
  bound to 1,815, successful execution to 1,944, and initialization to 2,046
  while mismatches and execution failures remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-context-translate-prefix-2026-09-19.md)
- 2026-09-19 -- XSLT 1.0 variable/focus string composition now admits numeric
  EBV for `position() mod $variable` and `string-length($variable)`, together
  with variable/literal `contains()`, `starts-with()`, `substring-before()`,
  and `substring-after()`. All reuse existing controlled variable conversion
  and retain modern rejection. The unchanged Microsoft `Variables__84438`
  recursive table case becomes exact, raising the strict lower bound to 1,816,
  successful execution to 1,945, and initialization to 2,047 while mismatches
  and execution failures remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-variable-focus-string-composition-2026-09-19.md)
- 2026-09-19 -- The source-free XSLT 1.0 form
  `string(number(decimal))` now performs its required IEEE-double conversion
  at compilation, retains the resulting XPath lexical string, and folds exact
  equality between two such forms. The equivalent modern expression remains
  unsupported, and exponential formatting is not inferred. The unchanged
  Microsoft `XSLTFunctions_RoundTripNumber_UsingStringFn` case becomes exact,
  raising the strict lower bound to 1,817, successful execution to 1,946, and
  initialization to 2,048 while mismatches and execution failures remain
  unchanged.
  [Evidence](../Evidence/oasis-xslt10-static-number-string-round-trip-2026-09-19.md)
- 2026-09-19 -- The exact XSLT 1.0 boolean path
  `child[@attribute=string($variable)]` now reuses shared variable string
  conversion and performs a charged scan of unqualified children and
  attributes. The bounded form accepts an optional `./`; qualified names,
  deeper paths, and modern semantics remain unsupported. The unchanged Lotus
  `namedtemplate07` case becomes exact, raising the strict lower bound to
  1,818, successful execution to 1,947, and initialization to 2,049 while
  mismatches and execution failures remain unchanged.
  [Evidence](../Evidence/oasis-xslt10-child-attribute-variable-predicate-2026-09-19.md)
- 2026-09-19 -- The optional argument of `generate-id()` now normalizes to the
  current context-node selection in both value and identity-comparison plans.
  The zero-argument form reuses existing stable node identity, cardinality,
  diagnostics, and charging rather than introducing ambient context state.
  The unchanged Lotus `idkey07` uniqueness case becomes exact, raising the
  strict lower bound to 1,819, successful execution to 1,948, and
  initialization to 2,050 while mismatches and execution failures remain
  unchanged.
  [Evidence](../Evidence/oasis-xslt10-zero-argument-generate-id-2026-09-19.md)
