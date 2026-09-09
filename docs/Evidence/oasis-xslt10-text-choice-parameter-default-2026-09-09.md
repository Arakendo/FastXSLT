# OASIS XSLT 1.0 text-choice parameter default -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 template parameter build a temporary text tree from a bounded
`xsl:choose` without introducing a second constructor evaluator?

## Changes

- Compilation recognizes exactly one content-built template-parameter default
  containing `xsl:choose`.
- Every `xsl:when` must use the existing typed path/string equality plan, and
  every selected or fallback branch must contain literal text only.
- Runtime reuses the ordinary charged boolean evaluator at the template's
  source context, then materializes the selected text through the existing
  invocation-owned temporary-tree owner.
- Compiled retained-capacity accounting includes branch tests, branch text,
  vector capacity, and fallback text.
- The compatibility constructor is selected only for an XSLT 1.0 stylesheet;
  an otherwise identical modern stylesheet remains explicitly unsupported.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,517 | 1,518 | +1 |
| Executed successfully | 1,351 | 1,352 | +1 |
| Expected-result XML matches | 1,239 | 1,240 | +1 |
| XML comparison mismatches | 86 | 86 | 0 |
| Execution failures | 166 | 166 | 0 |

The unchanged Lotus `variable13` case selects the second branch from
`item='Y'` and produces the expected temporary-tree string value `25`. The
strict standard-operation lower bound becomes
`1,240 / 2,742 = 45.22%`; the conservative all-catalog ratio becomes
`1,240 / 3,173 = 39.08%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary parameter constructors, nested choices,
non-text branch bodies, general boolean expressions, focus position/size,
variable mutation, or result elements. It does not cache temporary trees or
share them across invocations. No corpus bytes or expected results were
changed.

## Verification

- A focused runtime test selects the middle branch, verifies exact output, and
  proves the same constructor remains unsupported for a modern stylesheet.
- The unchanged Lotus `variable13` case passes the suite XML comparator.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
