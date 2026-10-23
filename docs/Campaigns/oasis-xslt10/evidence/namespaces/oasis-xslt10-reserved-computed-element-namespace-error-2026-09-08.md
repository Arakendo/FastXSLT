# OASIS XSLT 1.0 Reserved Computed-Element Namespace Error

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does a statically known `xsl:element` namespace equal to the reserved XMLNS
namespace fail explicitly instead of constructing a misleading result element?

## Change

Static computed-element name compilation now rejects
`http://www.w3.org/2000/xmlns/` with invalid diagnostic `XTDE0835` before a
result name or namespace binding is retained. This is a semantic validation;
it does not broaden dynamic name or namespace AVT support.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Expected errors observed during initialization | 398 | 400 | +2 |
| Expected-error cases with unexpected success | 14 | 12 | -2 |
| Initialized | 1,481 | 1,479 | -2 |
| Executed successfully | 1,318 | 1,316 | -2 |
| Expected-result XML matches | 1,202 | 1,202 | 0 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |

The two unchanged Microsoft element-construction cases that differ only in
attribute order now receive an explicit initialization error. Their former
execution was not a compatibility pass, so the 1,202 exact-result lower bound
does not change.

## Boundaries

This tranche validates only the reserved XMLNS namespace URI on a static
`xsl:element` namespace. Dynamic name and namespace expressions, other reserved
prefix/URI combinations, and general namespace-fixup expansion remain outside
the admitted slice.

## Verification

- A focused compiler test proves the reserved URI produces invalid
  `XTDE0835`.
- The complete local OASIS measurement proves both former unexpected-success
  cases are now observed expected errors and no result match regresses.
- No upstream corpus byte was edited.
