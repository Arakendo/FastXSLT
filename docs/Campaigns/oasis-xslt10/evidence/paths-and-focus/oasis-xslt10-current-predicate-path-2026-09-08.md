# OASIS XSLT 1.0 Current Predicate Path

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the XSLT-specific `current()` function appear by itself as a path predicate
without widening the general XPath parser or losing XSLT 1.0 first-node
conversion?

## Changes

- XSLT 1.0 value compilation recognizes exactly one standalone `[current()]`
  predicate and retains a private compatibility plan.
- Because the outer source focus is necessarily present for this admitted path,
  the standalone predicate has true effective boolean value; the remaining
  controlled path retains its ordinary axis navigation and node-visit charges.
- The compatibility plan adds an explicit XPath-operation charge and applies
  XSLT 1.0 first-node string conversion.
- The bounded parenthesized `(path[current()])[1]` form is normalized to the
  same first-node plan. The general XPath parser remains unchanged.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,476 | 1,477 | +1 |
| Executed successfully | 1,313 | 1,314 | +1 |
| Expected-result XML matches | 1,199 | 1,200 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Lotus/select_select85#1` case now passes both
`following-sibling::ch[current()]` and
`(following-sibling::ch[current()])[1]`, producing `ch1` in each result. The
strict standard-operation lower bound becomes
`1,200 / 2,742 = 43.76%`; the conservative all-catalog ratio becomes
`1,200 / 3,173 = 37.82%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit `current()` with arguments, boolean composition,
arbitrary expression placement, multiple occurrences, or modern XPath
function semantics. A focused negative regression proves that an equivalent
XSLT 3.0 expression does not enter the general XPath path grammar through this
compatibility rule.

## Verification

- A focused runtime test proves the direct and parenthesized first-node forms
  against two following siblings.
- A focused compilation test proves the same predicate remains outside the
  modern/general XPath path parser.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
