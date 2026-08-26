# Peer Test-Corpus Review: Monday

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Reviewer | Monday, peer review supplied by the project owner |
| Scope | External standards suites and distinct adversarial/performance corpus purposes |
| Informs | AR-0001, AR-0008, AR-0010, AR-0011, testing strategy, and corpus planning |

## Reconciliation with the repository

The review recommended four external families. FastXSLT already pins two of
them at immutable revisions:

| Suggested source | Current FastXSLT disposition |
| --- | --- |
| W3C XSLT 3.0 test suite | Admitted as immutable `vendor/xslt30-test`; one overlay-selected case executes privately |
| W3C QT3 | Admitted as immutable `vendor/qt3tests`; catalog inventoried, execution harness pending profile selection |
| W3C XML Conformance Test Suite | Useful AR-0008 candidate; official 2013 archive inspected locally but not admitted |
| QT4 | Deferred future-language/regression candidate; no initial XPath/XSLT profile requires proposed 4.0 behavior |

This confirms the existing modern-language corpus direction rather than adding
duplicate sources. The immediate new pressure is XML mechanics, where AR-0008
already calls for a standards-derived well-formedness and namespace subset.

## Useful design feedback

Every external case should retain upstream identity and be classified before
execution. The review's categories map to FastXSLT as:

- selected and applicable to the accepted profile;
- excluded because its standards edition or optional feature is outside the
  profile;
- applicable but currently unsupported by the engine;
- harness unsupported or malformed from the harness's perspective;
- executed pass, semantic/diagnostic mismatch, or expected operation failure.

Selection and engine outcome are separate axes. “Unsupported” is not a pass,
and a harness limitation is not an engine failure.

The review also distinguishes three questions that must not share a misleading
denominator:

1. conformance cases ask whether named standards behavior is correct;
2. adversarial cases ask whether hostile-but-bounded work terminates through the
   promised resource model; and
3. performance workloads ask what correct work costs under a recorded host and
   reuse configuration.

## Disposition

Adopt the purpose separation and classifier vocabulary as testing policy.
Continue using the two existing pinned suites. Inspect and license-review the
W3C XML archive before admission, then select a nonvalidating/no-external-entity
subset only after AR-0001 names the XML and Namespaces editions.

Do not admit QT4 now, turn adversarial cases into conformance claims, or treat a
stress fixture as a stable budget default. First-party adversarial fixtures may
be generated or minimized from observed failures, but their provenance and
relationship to upstream material must remain explicit.
