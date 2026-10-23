# OASIS XSLT 1.0 Path-Equality Template Argument

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 template invocation pass the boolean result of comparing two
source paths without reducing either node-set to one scalar value?

## Changes

- A bounded `path = path` or `path != path` template argument compiles both
  operands as typed location paths.
- Runtime evaluation preserves XPath 1.0 existential node-set comparison:
  `=` succeeds when any pair has equal string values, while `!=` independently
  succeeds when any pair has unequal string values.
- Both path walks, node string-value construction, and candidate-pair
  comparison remain work charged.
- A standalone `xsl:attribute` using one `xsl:value-of` child now reuses the
  existing computed-attribute compiler, including when nested beneath
  `xsl:if`. The earlier `select`-attribute form remains unchanged.

No source nodes or invocation values are retained in compiled state.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,458 | 1,459 | +1 |
| Executed successfully | 1,297 | 1,298 | +1 |
| Expected-result XML matches | 1,183 | 1,184 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/variable_variable60#1`.

The strict standard-operation lower bound becomes
`1,184 / 2,742 = 43.18%`; the conservative all-catalog ratio becomes
`1,184 / 3,173 = 37.31%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- The focused runtime test proves that equality and inequality can both be
  true for different pairs in the same two node-sets.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
