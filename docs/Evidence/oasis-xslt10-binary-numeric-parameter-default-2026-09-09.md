# OASIS XSLT 1.0 binary-numeric parameter default -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 template parameter retain a source-dependent numeric default
such as `number(path) + integer` without converting it into an untyped string
or adding another arithmetic evaluator?

## Changes

- The existing XSLT 1.0 exact-rational binary-numeric compiler recognizes an
  explicit `number(location-path)` operand inside its bounded arithmetic tree.
- A template parameter may retain that typed plan only under XSLT 1.0 static
  context.
- Runtime reuses the ordinary controlled numeric evaluator and binds its result
  as an atomic double, preserving numeric comparisons by later instructions.
- Value construction and parameter binding now call the same failure-mapping
  owner for cardinality, invalid lexical values, overflow, zero divisors,
  budgets, and cancellation.
- Compiled retained-capacity accounting includes the numeric expression tree.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,518 | 1,519 | +1 |
| Executed successfully | 1,352 | 1,353 | +1 |
| Expected-result XML matches | 1,240 | 1,240 | 0 |
| XML comparison mismatches | 86 | 87 | +1 |
| Execution failures | 166 | 166 | 0 |

Microsoft `bvt072` now executes all of its numeric defaults, including the
fractional result `7.5`. The case is listed in the suite's doubts metadata and
its expected output omits whitespace-only source text reached by an explicit
`xsl:apply-templates`; FastXSLT preserves that text, so the case remains a
visible comparison mismatch and receives no compatibility credit.

The strict standard-operation lower bound remains
`1,240 / 2,742 = 45.22%`; the conservative all-catalog ratio remains
`1,240 / 3,173 = 39.08%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not generalize template defaults to arbitrary value
expressions, suppress source whitespace to fit archival output, alter modern
numeric semantics, or add binary floating-point arithmetic. The bounded
`number()` operand spelling is compatibility-selected and accepts one location
path only. No corpus bytes or expected results were changed.

## Verification

- A focused runtime test proves exact numeric output and a later typed numeric
  comparison, while an otherwise equivalent modern stylesheet stays
  unsupported.
- The unchanged Microsoft `bvt072` case reaches comparison with all numeric
  values correct and its doubts-annotated whitespace difference visible.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
