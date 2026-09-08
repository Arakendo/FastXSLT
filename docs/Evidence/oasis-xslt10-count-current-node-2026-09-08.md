# OASIS XSLT 1.0 Count Current Node

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the XSLT 1.0 `current()` singleton be counted directly and can that count
retain numeric-predicate semantics without admitting `current()` into the
general XPath grammar?

## Changes

- XSLT 1.0 value compilation now recognizes exact `count(current())` as the
  count of the required outer source-node focus.
- The existing private XSLT current-predicate plan recognizes exact
  `[count(current())]`. Because the admitted current node is a singleton, the
  count is one and the numeric predicate is lowered to the existing checked
  position-one predicate.
- Both paths retain explicit XPath-operation charging and require source-node
  context. The general XPath parser remains unchanged.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,477 | 1,478 | +1 |
| Executed successfully | 1,314 | 1,315 | +1 |
| Expected-result XML matches | 1,200 | 1,201 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Lotus/select_select86#1` case now proves `count(current())`
equals one under both template application and `xsl:for-each`, while
`following-sibling::*[count(current())]` selects the first following sibling.
The strict standard-operation lower bound becomes
`1,201 / 2,742 = 43.80%`; the conservative all-catalog ratio becomes
`1,201 / 3,173 = 37.85%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche admits only exact zero-argument `current()` beneath `count()` and
the exact standalone numeric predicate derived from it. It does not add
general arithmetic or comparison around `current()`, nested current-node paths,
multiple occurrences, or modern XPath semantics.

## Verification

- The focused current-predicate runtime test now proves direct singleton count,
  standalone current-node predicate, parenthesized first-node selection, and
  numeric current-count predicate behavior together.
- The existing negative compilation test continues to prove that this XSLT 1.0
  compatibility family does not widen the modern/general XPath path parser.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
