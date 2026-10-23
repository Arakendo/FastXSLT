# OASIS XSLT 1.0 Static Atomic Key Values

## Question

Can the charged `key()` reference path admit static atomic value conversion and
the matching declaration-side expressions without widening into dynamic XPath
evaluation or selecting a retained index?

## Method

- Replace the declaration's path-only `use` field with a typed private choice:
  location path, string literal, or `number(location-path)`.
- For `number(path)`, apply XSLT 1.0 first-node conversion and the existing
  numeric lexical conversion under normal work accounting.
- Compile a literal numeric or source-free exact-rational lookup argument to its
  canonical string value. Keep variables, node sets, and context-dependent
  arithmetic outside the slice.
- Continue executing lookup through the complete charged source scan established
  by the prior reference-path tranche.
- Run the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

`Lotus/idkey_idkey05#1` and `Lotus/idkey_idkey08#1` initialize, execute, and
compare exactly. The strict exact-result lower bound rises from 1,585 to 1,587;
initialized cases rise from 1,969 to 1,971 and successfully executed cases rise
from 1,866 to 1,868. Initialization failures fall from 1,166 to 1,164 while
execution failures and all expected-error totals remain unchanged.

The general function-shaped frontier falls from 74 to 73 cases. The generic
other-shaped path frontier falls from 17 to 16 because the constant-string key
use is now typed. Eleven dynamic or otherwise non-static key lookup forms remain
explicit as `FXXP1023`.

A focused runtime case composes a constant-string key use, a numeric path key
use, a decimal lookup value, and source-free addition. It verifies that
`key('numbers', 1.0)` and `key('numbers', 1+1)` compare against canonical key
values without changing source identity or document order.

## Boundary conclusion

This tranche reuses compile-time exact arithmetic and invocation-time charged
path/string conversion. It does not add general expression evaluation to key
declarations, admit path unions, permit variables or recursive key references
in `use`, retain source-derived index state, or expose a key representation.
Those independent capabilities remain visible in the corpus frontier.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
