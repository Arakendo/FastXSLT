# OASIS XSLT 1.0 Literal `key()` Lookup Reference

## Question

Can FastXSLT execute a useful first `key()` slice without prematurely selecting
an index representation, retaining source-derived state in the compiled
stylesheet, or weakening work accounting and cancellation?

## Method

- Compile a literal key name and literal lookup value into a typed private
  expression, with an optional admitted location-path tail.
- Resolve the key name against the stylesheet instruction's static namespace
  context.
- For each invocation, scan the current source document in document order,
  reuse the compiled key match patterns, and evaluate every matching
  declaration's typed `use` path.
- Treat declarations sharing one expanded name additively, remove duplicate
  result identities, and restore document order after an optional tail.
- Charge source traversal, match evaluation, path evaluation, and string-value
  construction through their existing work owners. Do not retain a key index.
- Keep dynamic key names and values explicit as `FXXP1023 / unsupported`, and
  report an undeclared key at execution as `XTDE1260 / invalid`.
- Run the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The exact expected-result lower bound rises from 1,572 to 1,585. Initialized
cases rise from 1,955 to 1,969, and successfully executed cases rise from 1,853
to 1,866. Initialization failures fall from 1,180 to 1,166; execution failures
rise from 102 to 103 because an unchanged undeclared-key case now reaches and
reports runtime `XTDE1260`.

All 3,173 catalog cases retain a disposition. Expected-error observations move
from 397 initialization / 26 execution to 396 initialization / 27 execution;
the total remains 423, with the same five unexpected successes. The general
function-shaped initialization frontier falls from 85 to 74 cases. Eleven
dynamic or otherwise nonliteral `key()` forms remain explicitly unsupported as
`FXXP1023` rather than being approximated.

A focused runtime case proves that two same-name declarations match different
element names additively and that `key('codes', 'b')/@name` returns the first
result in document order. A compiler regression derived from
`Lotus/idkey_idkey25#1` proves that a variable key name remains a structured
unsupported result and cannot panic initialization.

## Ownership conclusion

The complete charged scan is the safe semantic reference path for this first
lookup slice. It stores no source-derived state in the compiled program and
introduces no global, cross-snapshot, or cross-invocation cache. A later lazy
index may be considered only as an invocation-owned or separately reviewed
prepared representation, must remain differential-testable against this scan,
and must justify its retained-memory and preparation costs with measurements.

This is bounded compatibility evidence for literal `key()` use in the current
private `xsl:value-of` path, not general XPath `key()` support or an XSLT 1.0
conformance claim.
