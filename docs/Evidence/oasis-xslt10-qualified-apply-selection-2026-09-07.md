# OASIS XSLT 1.0 Qualified Apply Selection

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:for-each` and `xsl:apply-templates` reuse the namespace-aware
qualified child/attribute path owner already used by ordinary value selection?

## Change

The shared apply-selection compiler now resolves simple explicit QName paths
against the instruction's in-scope stylesheet namespaces. Both
`xsl:for-each` and `xsl:apply-templates` already use this compiler, so the
change does not create instruction-specific path semantics or another runtime
evaluator.

The admitted fallback remains deliberately narrow: child or attribute QName
steps separated by `/`. Namespace wildcards, explicit axes, predicates,
functions, and other general qualified XPath syntax remain outside this slice.
Unbound prefixes retain the typed `XPST0081` failure.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,400 | 1,402 | +2 |
| Executed successfully | 1,239 | 1,241 | +2 |
| Expected-result XML matches | 1,125 | 1,127 | +2 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/axes_axes58#1` and
`Lotus/namespace_namespace21#1`. The former selects `foo:doc` through
`xsl:for-each`; the latter selects `em:foo` while retaining the expected
result namespace semantics.

The strict standard-operation lower bound is now
`1,127 / 2,742 = 41.10%`; the deliberately conservative all-catalog ratio is
`1,127 / 3,173 = 35.52%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused lifecycle test exercises the same qualified child selection from
  both `xsl:for-each` and `xsl:apply-templates`, with qualified template
  dispatch and exact serialized output.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
