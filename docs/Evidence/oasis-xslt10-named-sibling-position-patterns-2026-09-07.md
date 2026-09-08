# OASIS XSLT 1.0 Named-Sibling Position Patterns

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can exact and bounded relational positions in a simple named-element match
pattern use source-tree sibling focus even when the caller applies nodes in a
different order?

## Change

The existing named-sibling match-pattern owner now retains exact
`position()=N` and static `position()<N` boundaries. Selection counts only
source siblings matching the pattern's expanded element name. It does not use
the `xsl:apply-templates` delivery position, so sorting the selected nodes does
not change which source-tree pattern each node matches.

This remains narrower than arbitrary positional match predicates. Chained,
dynamic, modulo, attribute-composed, and multi-step forms remain explicitly
unsupported unless their independent pattern semantics are implemented.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,441 | 1,445 | +4 |
| Executed successfully | 1,280 | 1,284 | +4 |
| Expected-result XML matches | 1,165 | 1,169 | +4 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are:

- `Lotus/position_position08#1`;
- `Lotus/position_position93#1`;
- `Microsoft/Template_MatchPatternVariation4#1`;
- `Microsoft/Template_MatchPatternVariation6#1`.

`position93` sorts application order independently from source order and is
the principal conservation case for this boundary.

The strict standard-operation lower bound is now
`1,169 / 2,742 = 42.63%`; the deliberately conservative all-catalog ratio is
`1,169 / 3,173 = 36.84%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test reverses application order with a numeric sort while
  checking exact source-sibling positions, and also checks a static less-than
  boundary.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
