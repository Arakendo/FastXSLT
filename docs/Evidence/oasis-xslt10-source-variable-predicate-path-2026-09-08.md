# OASIS XSLT 1.0 Source-Variable Predicate Path

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:apply-templates` evaluate a predicate-bearing relative path from each
node in a typed source-node variable while preserving XPath step focus and
normalizing the combined result?

## Changes

- The apply-selection compiler now distinguishes a predicate-bearing path
  rooted at a variable from the existing simple temporary-tree path shape.
- Runtime evaluation requires that variable to be a typed source-node
  sequence, evaluates the controlled location path once per root, then restores
  source document order and removes duplicate node identity.
- Predicates retain the existing path evaluator's focus rules and node-visit
  charges. A variable of another kind fails explicitly rather than being
  treated as a source tree.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,475 | 1,476 | +1 |
| Executed successfully | 1,312 | 1,313 | +1 |
| Expected-result XML matches | 1,198 | 1,199 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Lotus/select_select80#1` case now passes. Its
`$all/LI[@flag][last()]` selection proves that `last()` is computed among the
flagged `LI` children of each source-variable root before the results are
combined. The strict standard-operation lower bound becomes
`1,199 / 2,742 = 43.73%`; the conservative all-catalog ratio becomes
`1,199 / 3,173 = 37.79%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche admits predicate-bearing source-variable paths through the
existing controlled location-path grammar. It does not add predicates to
temporary trees, cross-document node ordering, arbitrary variable path
operands, or a public node-sequence representation.

## Verification

- A focused runtime test uses two source-variable roots and proves that
  `[@flag][last()]` establishes predicate focus independently for each root,
  producing the document-ordered `BC` result.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
