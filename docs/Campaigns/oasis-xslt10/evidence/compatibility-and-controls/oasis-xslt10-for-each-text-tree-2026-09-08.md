# OASIS XSLT 1.0 `for-each` text trees -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can local variables and global parameter defaults share one bounded XSLT 1.0
temporary-tree constructor for the exact sequence
`for-each(location-path) → value-of(.)`?

## Changes

- Compilation recognizes exactly one XSLT 1.0 `xsl:for-each` containing one
  empty `xsl:value-of select="."` and retains its typed location path.
- Local variables and global defaults use distinct lifecycle representations
  but share the same runtime materializer.
- Runtime evaluates the path with the existing charged navigation owner,
  appends each selected node's controlled string value in selection order, and
  binds the result as an invocation-owned temporary text tree.
- Global parameter overrides still bypass the default constructor. Modern
  stylesheets, arbitrary constructor bodies, and general `for-each` execution
  are unchanged.
- Compiled retained-capacity accounting includes both local and global path
  plans.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,510 | 1,512 | +2 |
| Executed successfully | 1,345 | 1,347 | +2 |
| Expected-result XML matches | 1,233 | 1,235 | +2 |
| XML comparison mismatches | 86 | 86 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `variable15` local-variable case and `variable16` global-
parameter-default case both produce the expected `XYZ` temporary-tree string
value. The strict standard-operation lower bound becomes
`1,235 / 2,742 = 45.04%`; the conservative all-catalog ratio becomes
`1,235 / 3,173 = 38.92%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit multiple constructor instructions, arbitrary
`xsl:for-each` bodies, a value expression other than `.`, variables inside the
loop, sorting, separators, or a general temporary-tree instruction executor.
It does not cache constructed trees across invocations. No corpus bytes or
expected results were changed.

## Verification

- A focused runtime test executes the same source through local and global
  constructor lifecycles and checks their exact combined serialization.
- The unchanged Lotus `variable15` and `variable16` cases pass the suite XML
  comparator.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
