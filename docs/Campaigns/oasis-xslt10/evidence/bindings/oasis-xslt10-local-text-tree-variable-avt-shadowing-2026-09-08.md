# OASIS XSLT 1.0 Local Text-Tree Variable AVT Shadowing

Date: 2026-09-08  
Status: Verified local compatibility evidence; comparison frontier remains  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a variable-only literal attribute convert an invocation-local XSLT 1.0
temporary text tree to its string value while preserving lexical shadowing in a
nested `xsl:for-each` scope?

## Changes

- Variable-only literal attributes now use the shared attribute-variable string
  conversion owner for both atomic values and invocation-local temporary trees.
- Inner text-tree bindings continue to detach through the existing
  invocation-local copy-on-write variable frame; no state is shared across
  invocations or generations.
- Missing or incompatible bindings retain structured `FXRT0002` failure.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,490 | 1,490 | 0 |
| Executed successfully | 1,324 | 1,325 | +1 |
| Expected-result XML matches | 1,217 | 1,217 | 0 |
| XML comparison mismatches | 81 | 82 | +1 |
| Execution failures | 166 | 165 | -1 |

The unchanged Lotus `variable_variable56#1` case now produces the correct outer
and inner attribute values. It receives no pass credit because its
`indent="yes"` serialization contains two indentation spaces absent from the
reference output. That pre-existing serialization-policy difference remains a
visible comparison frontier rather than being normalized away here.

## Boundaries

This tranche does not select a general indentation policy, claim byte-exact
serialization, add cross-generation tree sharing, or admit arbitrary
result-tree-fragment extensions.

## Verification

- A focused runtime test proves outer and inner temporary-tree values remain
  distinct across nested lexical scope.
- The complete local measurement proves the execution failure becomes exactly
  one comparison mismatch with no unrelated counter movement.
- No upstream corpus byte was edited.
