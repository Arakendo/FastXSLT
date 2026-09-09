# OASIS XSLT 1.0 content-built parameter values -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can content-built `xsl:with-param` values reuse the ordinary typed
`xsl:value-of` compiler and runtime, including exact variable arithmetic and
numeric-variable recursion conditions, while retaining XSLT 1.0 temporary-tree
semantics?

## Changes

- A content-built template argument containing exactly one admitted
  `xsl:value-of` now retains the ordinary typed `ValueExpression`; the earlier
  concat-specific argument representation was removed.
- Evaluation uses the caller's source node, position, and size, then wraps the
  produced text in an invocation-owned temporary tree before binding the target
  parameter.
- XSLT 1.0 variable-to-variable `<`, `<=`, `>`, and `>=` conditions use the
  shared variable string conversion and XPath numeric ordering. `NaN` remains
  unordered.
- Exact-rational binary arithmetic, temporary-tree string conversion, work
  accounting, cancellation, and named-template depth limits remain owned by
  their existing implementations.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,504 | 1,509 | +5 |
| Executed successfully | 1,339 | 1,342 | +3 |
| Expected-result XML matches | 1,231 | 1,232 | +1 |
| XML comparison mismatches | 82 | 84 | +2 |
| Execution failures | 165 | 167 | +2 |

The unchanged Lotus `namedtemplate10` case now passes. Its source attribute is
passed through content, recursive `start` and `stop` parameters remain temporary
text trees, `$start + $step` uses the existing exact-rational evaluator, and
`$start < $stop` controls recursion.

The wider typed admission deliberately exposes four additional non-passes:
Microsoft `84436` and `84038` reach whitespace-result mismatches, while `84437`
and `84047` reach the existing unbound-local-variable execution boundary. They
remain visible evidence rather than being approximated or excluded.

The strict standard-operation lower bound becomes
`1,232 / 2,742 = 44.93%`; the conservative all-catalog ratio becomes
`1,232 / 3,173 = 38.83%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit multiple or mixed content instructions, nested
local bindings inside an argument constructor, general XPath comparisons, or
an atomic shortcut for XSLT 1.0 result-tree fragments. No corpus bytes or
expected results were changed.

## Verification

- Focused tests cover numeric recursion and preserve the prior recursive
  Unicode-string parameter behavior.
- The unchanged `namedtemplate10` case passes its expected XML comparison.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
