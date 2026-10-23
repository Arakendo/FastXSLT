# OASIS XSLT 1.0 static introspection -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can literal-QName XSLT 1.0 implementation introspection be answered from the
compiled stylesheet's static context without adding runtime reflection or
advertising capabilities FastXSLT does not admit?

## Implemented slice

A private compiler module now evaluates `system-property()`,
`function-available()`, and `element-available()` when their sole argument is a
string-literal QName in an XSLT 1.0 stylesheet.

QName validation and namespace lookup occur against the expression element.
In particular, `element-available()` applies the in-scope default namespace,
which is required by the Microsoft cases that place the XSLT namespace directly
on `xsl:value-of`. The function reports only instructions handled by the
current compiler. `function-available()` similarly reports the admitted XPath
1.0 and XSLT function surface. The required XSLT system properties are folded
to immutable strings. No runtime registry, host callback, public inspection
surface, or ambient feature discovery was introduced.

The compiled values reuse existing literal-string and constant-boolean plans,
so repeated transformations perform no introspection work.

## Corpus result

Fifteen unchanged cases leave the generic function-shaped path frontier and
execute. Thirteen have exact XML expected results. The two
`xsl:vendor`/`xsl:vendor-url` cases are upstream manual comparisons and remain
outside the exact XML denominator even though they now execute.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,731 | 1,746 | +15 |
| Initialization failures | 1,404 | 1,389 | -15 |
| Executed successfully | 1,548 | 1,563 | +15 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,413 | 1,426 | +13 |
| XML comparison mismatches | 108 | 108 | 0 |
| Non-XML/manual comparisons reached | 0 | 2 | +2 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,426 / 2,742 = 52.01%` for standard-operation cases and
`1,426 / 3,173 = 44.94%` for the complete catalog.

## Verification

Focused compiler tests cover QName namespace resolution and true/false
capability answers. A full lifecycle test proves the values are compiled once
and serialized through ordinary execution. The complete local sweep establishes
the fifteen-case frontier movement, thirteen exact additions, two retained
manual dispositions, unchanged mismatch/execution-failure counts, and absence
of panics.
