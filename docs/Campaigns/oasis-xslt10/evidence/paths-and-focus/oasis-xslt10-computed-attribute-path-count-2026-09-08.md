# OASIS XSLT 1.0 computed-attribute path count -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a computed `xsl:attribute` reuse a typed `count(location-path)` expression
without creating a second XPath evaluator or weakening source-focus and work
accounting?

## Changes

- Under XSLT 1.0 compatibility, the computed-attribute compiler accepts
  `count(path)` only when the operand parses completely through the existing
  typed location-path grammar.
- The compiled attribute retains the location path rather than a source count
  or result string.
- Runtime requires the current source focus, charges result construction and
  the count operation, evaluates the path through controlled navigation, and
  materializes the selected node count.
- Missing source focus is a structured `XPDY0002` failure. Atomic and temporary
  focus do not masquerade as source nodes.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,498 | 1,499 | +1 |
| Executed successfully | 1,333 | 1,334 | +1 |
| Expected-result XML matches | 1,225 | 1,226 | +1 |
| XML comparison mismatches | 82 | 82 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `axes131` case now counts both `descendant::*` and
`descendant-or-self::*` from repeated `grandchild` focus across six sibling
shapes. Every computed attribute agrees with the expected XML. The actual
serializer uses paired empty elements while the archival result uses empty
element tags; the existing infoset comparator correctly treats that spelling
difference as semantically irrelevant.

The strict standard-operation lower bound becomes
`1,226 / 2,742 = 44.71%`; the conservative all-catalog ratio becomes
`1,226 / 3,173 = 38.64%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary computed-attribute value expressions,
path unions beyond the existing grammar, modern sequence counting, temporary
tree navigation, dynamic attribute names, or namespace attributes. No corpus
bytes or expected results were changed.

## Verification

- A focused runtime test counts descendant and descendant-or-self paths over
  populated and empty source elements.
- The unchanged `axes131` case passes its expected XML comparison.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
