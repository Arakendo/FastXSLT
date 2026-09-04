# AR-0019: XSLT 1.0 Compatibility Profile on the Modern Core

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-09-04 |
| Last reviewed | 2026-09-04 |
| Scope | Named XSLT 1.0 compatibility, backwards-compatible behavior, and shared modern execution |
| Trigger | A complete local legacy sweep found 366 definite unchanged passes and dominant gaps that largely overlap the XSLT 3.0 roadmap |
| Related ADRs | ADR-0002, ADR-0006, ADR-0007, ADR-0012, ADR-0013, ADR-0014 |
| Related reviews | AR-0001, AR-0004, AR-0008, AR-0011, AR-0014 |
| Related evidence | `docs/Evidence/oasis-xslt10-suite-candidate-review-2026-08-25.md`; `docs/Evidence/oasis-xslt10-initial-compatibility-measurement-2026-09-04.md` |

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
- [ ] Split the dominant XPath and unsupported-instruction frontiers into
  actionable semantic families and compare them with the XSLT30/QT3 roadmap.
- [ ] Resolve or explicitly classify the 20 known semantic mismatches.
- [ ] Define expected-error and discretionary/doubts comparison rules.
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
