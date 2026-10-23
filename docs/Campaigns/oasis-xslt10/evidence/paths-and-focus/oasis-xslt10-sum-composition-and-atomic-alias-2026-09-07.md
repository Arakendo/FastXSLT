# OASIS XSLT 1.0 Sum Composition and Atomic Alias

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can existing XSLT 1.0 value primitives compose through template invocation,
computed result attributes, and local bindings without creating alternate
uncontrolled evaluators?

## Changes

- `sum(path)` can supply a template argument as a typed double value. Both the
  ordinary value path and argument path reuse the same charged location-path,
  node string-value, numeric-conversion, and sum implementation.
- The bounded XSLT 1.0 `concat()` plan accepts `sum(path)` parts and can provide
  one `xsl:value-of` computed-attribute value. Dynamic evaluation stays in the
  invocation runtime; compiled state retains only the typed expression.
- An unqualified source-attribute AVT such as `{@rank}` uses the existing
  source-attribute value representation and is evaluated against source focus.
- A local variable selecting an earlier atomic variable copies its typed value
  into the current invocation's copy-on-write frame. This does not admit
  sequence, node-set, temporary-tree, cross-invocation, or cross-generation
  aliasing.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,459 | 1,462 | +3 |
| Executed successfully | 1,298 | 1,300 | +2 |
| Expected-result XML matches | 1,184 | 1,186 | +2 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 162 | +1 |

The unchanged exact cases are:

- `Lotus/variable_variable62#1`
- `Lotus/variable_variable44#1`

One additional stylesheet now initializes and reaches a later unbound-value
runtime boundary. It remains visible and uncredited rather than being treated
as a pass.

The strict standard-operation lower bound becomes
`1,186 / 2,742 = 43.25%`; the conservative all-catalog ratio becomes
`1,186 / 3,173 = 37.38%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test composes a source sum through a template argument,
  a sum-bearing computed attribute, and an unqualified source-attribute AVT.
- A focused runtime test proves that a local atomic alias preserves the bound
  value through the invocation-local variable frame.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
