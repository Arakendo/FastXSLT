# OASIS XSLT 1.0 descendant child-axis position -- 2026-09-13

Date: 2026-09-13  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT evaluate the valid match pattern
`chapter//footnote[position() != 1]` with position relative to each immediate
child axis, rather than incorrectly numbering the combined descendant set?

## Implemented slice

The match compiler now retains a bounded typed plan containing the expanded
ancestor and candidate names plus a non-equality sibling-position boundary.
For each candidate, runtime selection first computes its position among
same-named element children of its immediate parent, then searches that
parent's ancestor chain for the required `chapter`. Every sibling and ancestor
observation is charged through invocation work control.

The same rule is used for source and temporary trees. Focused coverage places
two `footnote` children under each of two `section` parents and proves that the
first child in each group is excluded independently.

## Corpus result

The unchanged Xalan `match16` case now initializes, executes, and matches its
expected XML result.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,678 | 1,679 | +1 |
| Executed successfully | 1,507 | 1,508 | +1 |
| Expected-result XML matches | 1,376 | 1,377 | +1 |
| Initialization failures | 1,457 | 1,456 | -1 |
| Execution failures | 171 | 171 | 0 |
| XML comparison mismatches | 104 | 104 | 0 |

The strict standard-operation lower bound is now
`1,377 / 2,742 = 50.22%`; the conservative all-catalog ratio is
`1,377 / 3,173 = 43.40%`.

## Boundaries

This does not select a general match-pattern evaluator or arbitrary descendant
predicates. The new plan admits one named ancestor, one descendant-or-self
bridge, one named candidate, and the exact positive-integer
`position() != N` predicate shape. It does not infer global descendant-set
positioning; the child-axis focus is the semantic point of this slice.

## Verification

- Focused compiler coverage checks the retained typed plan and template
  priority.
- Focused runtime coverage checks child-axis grouping and source/temporary
  parity.
- The complete 3,173-case local measurement produced the counters above and an
  exact comparison trace for `Lotus/match_match16#1`.
